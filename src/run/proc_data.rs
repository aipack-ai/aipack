use crate::Result;
use crate::agent::{Agent, AgentOptions};
use crate::run::Literals;
use crate::runtime::Runtime;
use crate::script::{AipackCustom, DataResponse, FromValue};
use crate::store::rt_model::RuntimeCtx;
use crate::store::{Id, Stage};
use genai::ModelName;
use serde_json::Value;

// region:    --- Types

pub struct ProcDataResponse {
	pub agent: Agent,
	pub input: Value, // will be Null if it was None
	pub data: Value,
	pub run_model_resolved: ModelName,
	pub skip: bool,
}

impl ProcDataResponse {
	pub fn new_skip(agent: Agent, input: Value, run_model_resolved: ModelName) -> Self {
		Self {
			agent,
			input,
			data: Value::Null,
			run_model_resolved,
			skip: true,
		}
	}
}

// endregion: --- Types

#[allow(clippy::too_many_arguments)]
pub async fn process_data(
	runtime: &Runtime,
	base_rt_ctx: RuntimeCtx,
	run_id: Id,
	task_id: Id,
	agent: Agent,
	literals: &Literals,
	before_all: &Value,
	input: Value,
) -> Result<ProcDataResponse> {
	// -- Extract the run model resolved
	// (for comparison later)
	let run_model_resolved = agent.model_resolved().clone();

	let agent_dir = agent.file_dir()?;

	// -- Execute data
	let DataResponse { input, data, options } = if let Some(data_script) = agent.data_script().as_ref() {
		// -- Build the scope
		// Note: Probably way to optimize the number of lua engine we create
		//       However, nice to be they are fully scoped.
		let lua_engine = runtime.new_lua_engine_with_ctx(literals, base_rt_ctx.with_stage(Stage::Data))?;

		let lua_scope = lua_engine.create_table()?;
		lua_scope.set("input", lua_engine.serde_to_lua_value(input.clone())?)?;
		lua_scope.set("before_all", lua_engine.serde_to_lua_value(before_all.clone())?)?;
		lua_scope.set("options", agent.options_as_ref())?;

		// -- Rt Step - Data Start
		runtime.step_task_data_start(run_id, task_id).await?;

		// -- Exec
		let lua_value = lua_engine.eval(data_script, Some(lua_scope), Some(&[agent_dir.as_str()]))?;
		let data_res = serde_json::to_value(lua_value)?;

		// -- Rt Step - Start Dt Start
		runtime.step_task_data_end(run_id, task_id).await?;

		// -- Post Process
		// skip input if aipack action is sent
		match AipackCustom::from_value(data_res)? {
			// If it is not a AipackCustom the data is the orginal value
			FromValue::OriginalValue(data) => DataResponse {
				data: Some(data),
				input: Some(input),
				..Default::default()
			},

			// If we have a skip, we can skip
			FromValue::AipackCustom(AipackCustom::Skip { reason }) => {
				runtime.rec_skip_task(run_id, task_id, Stage::Data, reason).await?;
				return Ok(ProcDataResponse::new_skip(agent, input, run_model_resolved));
			}

			// We have a `return aip.flow.data_response(...)``
			FromValue::AipackCustom(AipackCustom::DataResponse(DataResponse {
				input: input_ov,
				data,
				options,
			})) => DataResponse {
				input: input_ov.or(Some(input)),
				data,
				options,
			},

			FromValue::AipackCustom(other) => {
				return Err(format!(
					"Aipack Custom '{other_ref}' is not supported at the Data stage",
					other_ref = other.as_ref()
				)
				.into());
			}
		}
	} else {
		DataResponse {
			input: Some(input),
			data: None,
			options: None,
		}
	};

	// -- Normalize the context
	let input = input.unwrap_or(Value::Null);
	let data = data.unwrap_or(Value::Null);
	// here we use cow, not not clone the agent if no options
	let agent = if let Some(options_to_merge) = options {
		let options_to_merge: AgentOptions = serde_json::from_value(options_to_merge)?;
		let options_ov = agent.options_as_ref().merge_new(options_to_merge)?;
		agent.new_merge(options_ov)?
	} else {
		agent
	};

	Ok(ProcDataResponse {
		agent,
		input,
		data,
		run_model_resolved,
		skip: false,
	})
}
