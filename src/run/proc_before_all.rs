//! The before all processor

use crate::agent::{Agent, AgentOptions};
use crate::model::{Id, LogKind, RuntimeCtx, Stage};
use crate::run::Literals;
use crate::runtime::Runtime;
use crate::script::{AipackCustom, BeforeAllResponse, FromValue};
use crate::{Error, Result};
use serde_json::Value;

// region:    --- Types

#[derive(Debug)]
pub struct ProcBeforeAllResponse {
	pub before_all: Value,
	pub agent: Agent,
	pub inputs: Option<Vec<Value>>,
	pub skip: bool,
	pub redo: bool,
}

impl ProcBeforeAllResponse {
	fn new_skip(agent: Agent, inputs: Option<Vec<Value>>) -> Self {
		ProcBeforeAllResponse {
			before_all: Value::Null,
			agent,
			inputs,
			skip: true,
			redo: false,
		}
	}

	fn new_redo(agent: Agent, inputs: Option<Vec<Value>>) -> Self {
		ProcBeforeAllResponse {
			before_all: Value::Null,
			agent,
			inputs,
			skip: false,
			redo: true,
		}
	}
}

// endregion: --- Types

pub async fn process_before_all(
	runtime: &Runtime,
	base_rt_ctx: RuntimeCtx,
	run_id: Id,
	agent: Agent,
	literals: Literals,
	inputs: Option<Vec<Value>>,
) -> Result<ProcBeforeAllResponse> {
	let rt_log = runtime.rt_log();

	let rt_ctx = base_rt_ctx.with_stage(Stage::BeforeAll);

	// -- Run the before all
	let res = if agent.before_all_script().is_some() {
		// execute the script
		process_before_all_script(runtime, rt_ctx, run_id, agent, literals, inputs).await?
	} else {
		ProcBeforeAllResponse {
			before_all: Value::Null,
			agent,
			// Now return empty array if no inputs
			inputs,
			skip: false,
			redo: false,
		}
	};

	// -- Rt Log - Legacy TUI
	let msg = format!(
		"Model: {} ({}). Input Concurrency: {}",
		res.agent.model_resolved(),
		res.agent.model(),
		res.agent.options().input_concurrency().unwrap_or(1)
	);
	let _ = rt_log.rec_log_run(run_id, msg, Some(LogKind::SysInfo)).await;

	Ok(res)
}

async fn process_before_all_script(
	runtime: &Runtime,
	rt_ctx: RuntimeCtx,
	run_id: Id,
	agent: Agent,
	literals: Literals,
	inputs: Option<Vec<Value>>,
) -> Result<ProcBeforeAllResponse> {
	let rt_model = runtime.rt_model();

	// NOTE if where are here, we should have a script
	let before_all_script = agent.before_all_script();
	let before_all_script = before_all_script.unwrap_or_default();
	// -- Setup the Lua engine
	let lua_engine = runtime.new_lua_engine_with_ctx(&literals, rt_ctx)?;
	let lua_scope = lua_engine.create_table()?;
	let lua_inputs = inputs.clone().map(Value::Array).unwrap_or(Value::Null);
	lua_scope.set("inputs", lua_engine.serde_to_lua_value(lua_inputs)?)?;
	lua_scope.set("options", agent.options_as_ref())?;

	// -- Exec the script
	let lua_value = lua_engine.eval(before_all_script, Some(lua_scope), Some(&[agent.file_dir()?.as_str()]))?;
	let before_all_res = serde_json::to_value(lua_value)?;

	// -- Process before all response
	let before_all_response = match AipackCustom::from_value(before_all_res)? {
		// it is an skip action
		FromValue::AipackCustom(AipackCustom::Skip { reason }) => {
			// -- Rt Rec - Skip Run
			rt_model.rec_skip_run(run_id, Stage::BeforeAll, reason).await?;

			return Ok(ProcBeforeAllResponse::new_skip(agent, inputs));
		}

		// it is a redo action
		FromValue::AipackCustom(AipackCustom::Redo) => {
			return Ok(ProcBeforeAllResponse::new_redo(agent, inputs));
		}

		// it is before_all_response, so, we eventually override the inputs
		FromValue::AipackCustom(AipackCustom::BeforeAllResponse(BeforeAllResponse {
			inputs: inputs_ov,
			before_all,
			options,
		})) => BeforeAllResponse {
			inputs: inputs_ov.or(inputs),
			before_all,
			options,
		},

		// if it is another AipackCustom, we throw error
		// NOTE: for now, we leave this one as is. Shold use Rt Rec
		FromValue::AipackCustom(other) => {
			return Err(Error::custom(format!(
				"Aipack custom '{}' not supported at the Before All stage",
				other.as_ref()
			)));
		}

		// just plain value
		FromValue::OriginalValue(value) => BeforeAllResponse {
			inputs,
			before_all: Some(value),
			options: None,
		},
	};

	let BeforeAllResponse {
		inputs,
		before_all,
		options: options_to_merge,
	} = before_all_response;

	// -- Merge the eventual options from before all
	// Recompute Agent if needed
	let agent: Agent = match options_to_merge {
		Some(options_to_merge) => {
			let options_to_merge: AgentOptions = serde_json::from_value(options_to_merge)?;
			let options_ov = agent.options_as_ref().merge_new(options_to_merge)?;

			agent.new_merge(options_ov)?
		}
		None => agent,
	};

	// -- Get the Inputs and Before All data for the next stage
	// so, if empty, we have one input of value Value::Null
	let before_all = before_all.unwrap_or_default();

	Ok(ProcBeforeAllResponse {
		before_all,
		agent,
		inputs,
		skip: false,
		redo: false,
	})
}
