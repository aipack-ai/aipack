use crate::agent::Agent;
use crate::hub::{HubEvent, get_hub};
use crate::model::{Id, RuntimeCtx, Stage};
use crate::run::literals::Literals;
use crate::run::proc_ai::{ProcAiResponse, build_chat_messages, process_ai};
use crate::run::proc_data::{ProcDataResponse, process_data};
use crate::run::proc_output::process_output;
use crate::run::{AiResponse, DryMode, RunBaseOptions};
use crate::runtime::Runtime;
use crate::script::{AipackCustom, FromValue};
use crate::{Error, Result};
use serde::Serialize;
use serde_json::Value;
use value_ext::JsonValueExt as _;

// region:    --- Run Task Outer

/// Run the command agent input for the run_command_agent_inputs
/// Not public by design, should be only used in the context of run_command_agent_inputs
#[allow(clippy::too_many_arguments)]
pub async fn run_agent_task_outer(
	run_id: Id,
	task_id: Id,
	input_idx: usize,
	runtime: &Runtime,
	agent: &Agent,
	before_all: Value,
	input: impl Serialize,
	literals: &Literals,
	run_base_options: &RunBaseOptions,
) -> Result<(usize, Value)> {
	let hub = get_hub();

	// -- prepare the scope_input
	let input = serde_json::to_value(input)?;

	// get the eventual "._label" property of the input
	// try to get the path, name
	let label = get_input_label(&input).unwrap_or_else(|| format!("{input_idx}"));
	hub.publish(format!("\n==== Running input: {label}")).await;

	let run_response = run_agent_task(
		runtime,
		run_id,
		task_id,
		agent,
		before_all,
		&label,
		input,
		literals,
		run_base_options,
	)
	.await?;

	// if the response value is a String, then, print it
	if let Some(response_txt) = run_response.as_ref().and_then(|r| r.as_str()) {
		// let short_text = truncate_with_ellipsis(response_txt, 72);
		hub.publish(format!("-> Agent Output:\n\n{response_txt}\n")).await;
	}

	hub.publish(format!("==== DONE (input: {label})")).await;

	let output = process_agent_response_to_output(runtime, task_id, run_response).await?;

	Ok((input_idx, output))
}

async fn process_agent_response_to_output(
	runtime: &Runtime,
	task_id: Id,
	run_task_response: Option<RunAgentInputResponse>,
) -> Result<Value> {
	let hub = get_hub();

	let rt_model = runtime.rt_model();

	// Process the output
	let run_input_value = run_task_response.map(|v| v.into_value()).unwrap_or_default();
	let output = match AipackCustom::from_value(run_input_value)? {
		// if it is a skip, we skip
		FromValue::AipackCustom(AipackCustom::Skip { reason }) => {
			let reason_msg = reason.map(|reason| format!(" (Reason: {reason})")).unwrap_or_default();
			hub.publish(HubEvent::info_short(format!(
				"Aipack Skip input at Output stage{reason_msg}"
			)))
			.await;
			Value::Null
		}

		// Any other AipackCustom is not supported at output stage
		FromValue::AipackCustom(other) => match other {
			AipackCustom::Redo => Value::Null,
			_ => {
				return Err(Error::custom(format!(
					"Aipack custom '{}' not supported at the Output stage",
					other.as_ref()
				)));
			}
		},

		// Plain value passthrough
		FromValue::OriginalValue(value) => value,
	};

	// -- Rt Rec - Update the task output
	rt_model.update_task_output(task_id, &output).await?;
	Ok(output)
}

fn get_input_label(input: &Value) -> Option<String> {
	const LABEL_KEYS: &[&str] = &["path", "name", "label", "_label"];
	for &key in LABEL_KEYS {
		if let Ok(value) = input.x_get::<String>(key) {
			return Some(value);
		}
	}
	None
}

// endregion: --- Run Task Outer

// region:    --- RunAgentInputResponse

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum RunAgentInputResponse {
	AiReponse(AiResponse),
	OutputResponse(Value),
	Redo,
}

impl RunAgentInputResponse {
	pub fn as_str(&self) -> Option<&str> {
		match self {
			RunAgentInputResponse::AiReponse(ai_response) => ai_response.content.as_deref(),
			RunAgentInputResponse::OutputResponse(value) => value.as_str(),
			RunAgentInputResponse::Redo => None,
		}
	}

	/// Note: for now, we do like this. Might want to change that.
	/// Note: There is something to do about AI being able to structured output and manage it her
	/// - If AiResposne take the String as value or Null
	/// - If OutputResponse, then, the value is result
	pub fn into_value(self) -> Value {
		match self {
			RunAgentInputResponse::AiReponse(ai_response) => ai_response.content.into(),
			RunAgentInputResponse::OutputResponse(value) => value,
			RunAgentInputResponse::Redo => serde_json::json!({
				"_aipack_": {
					"kind": "Redo"
				}
			}),
		}
	}
}

// endregion: --- RunAgentInputResponse

/// Run the agent for one input
/// - Build the scope
/// - Execute Data
/// - Render the prompt sections
/// - Send the AI
/// - Execute Output
///
/// Note 1: For now, this will create a new Lua engine.
///         This is likely to stay as it creates a strong segregation between input execution
#[allow(clippy::too_many_arguments)]
pub async fn run_agent_task(
	runtime: &Runtime,
	run_id: Id,
	task_id: Id,
	agent: &Agent,
	before_all_result: Value,
	_label: &str,
	input: Value,
	literals: &Literals,
	run_base_options: &RunBaseOptions,
) -> Result<Option<RunAgentInputResponse>> {
	let rt_step = runtime.rt_step();
	let rt_model = runtime.rt_model();

	let client = runtime.genai_client();

	// -- Build Base Rt Context
	let base_rt_ctx = RuntimeCtx::from_run_task_ids(runtime, Some(run_id), Some(task_id))?;

	// -- Process Data Stage
	// Rt Step - Start Data stage
	rt_step.step_task_data_start(run_id, task_id).await?;
	let res = process_data(
		runtime,
		base_rt_ctx.clone(),
		run_id,
		task_id,
		agent.clone(),
		literals,
		&before_all_result,
		input,
	)
	.await;
	// Capture error if any
	if let Err(err) = res.as_ref() {
		rt_model.set_task_end_error(run_id, task_id, Some(Stage::Data), err)?;
	}
	// Rt Step - End Data stage
	rt_step.step_task_data_end(run_id, task_id).await?;

	let ProcDataResponse {
		agent,
		input,
		data,
		attachments,
		run_model_resolved,
		skip,
		redo: redo_data,
	} = res?;
	if skip {
		rt_model.set_task_end_state_to_skip(run_id, task_id)?;
		return Ok(None);
	}
	if redo_data {
		return Ok(Some(RunAgentInputResponse::Redo));
	}

	// -- Execute genai if we have an instruction

	// Rt Step - Start AI stage
	rt_step.step_task_ai_start(run_id, task_id).await?;

	let chat_messages = build_chat_messages(runtime, &agent, &before_all_result, &input, &data, &attachments)?;
	let res = process_ai(
		runtime,
		client,
		run_base_options,
		&run_model_resolved,
		run_id,
		task_id,
		agent.clone(),
		chat_messages,
	)
	.await;

	// Capture error if any
	if let Err(err) = res.as_ref() {
		rt_model.set_task_end_error(run_id, task_id, Some(Stage::Ai), err)?;
	}
	let ProcAiResponse { ai_response } = res?;
	// Rt Step - End AI stage
	rt_step.step_task_ai_end(run_id, task_id).await?;

	// -- if dry_mode res, we stop
	if matches!(run_base_options.dry_mode(), DryMode::Res) {
		return Ok(None);
	}

	// -- Exec output
	// -- Rt Step - start output
	rt_step.step_task_output_start(run_id, task_id).await?;
	let res = process_output(
		runtime,
		&base_rt_ctx,
		agent,
		literals,
		data,
		before_all_result,
		input,
		ai_response,
	)
	.await;
	// Capture error if any
	if let Err(err) = res.as_ref() {
		rt_model.set_task_end_error(run_id, task_id, Some(Stage::Output), err)?;
	}
	// -- Rt Step - end output
	rt_step.step_task_output_end(run_id, task_id).await?;

	res
}
