use crate::agent::{Agent, AgentRef};
use crate::hub::get_hub;
use crate::model::{Id, LogKind, RuntimeCtx, Stage, TaskForCreate};
use crate::run::RunBaseOptions;
use crate::run::literals::Literals;
use crate::run::proc_after_all::{ProcAfterAllResponse, process_after_all};
use crate::run::proc_before_all::{ProcBeforeAllResponse, process_before_all};
use crate::run::run_agent_task::run_agent_task_outer;
use crate::runtime::Runtime;
use crate::script::{AipackCustom, FromValue};
use crate::types::RunAgentResponse;
use crate::{Error, Result};
use serde_json::Value;
use tokio::task::{JoinError, JoinSet};
use uuid::Uuid;
use value_ext::JsonValueExt;

const DEFAULT_CONCURRENCY: usize = 1;

pub async fn run_agent(
	runtime: &Runtime,
	parent_uid: Option<Uuid>,
	agent: Agent,
	inputs: Option<Vec<Value>>,
	run_base_options: &RunBaseOptions,
	return_output_values: bool,
) -> Result<RunAgentResponse> {
	let rt_step = runtime.rt_step();
	let rt_model = runtime.rt_model();

	// -- Trim the runtime db
	// runtime.rec_trim().await?;
	// display relative agent path if possible
	// -- Rt Create - New run
	let run_id = rt_model.create_run(parent_uid, &agent).await?;

	// -- Rt Step - Start Run
	let run_id = rt_step.step_run_start(run_id).await?;

	let cancel_rx_opt = runtime.cancel_rx().cloned();

	let run_future = run_agent_inner(runtime, run_id, agent, inputs, run_base_options, return_output_values);
	tokio::pin!(run_future);

	let (run_agent_res, canceled) = if let Some(cancel_rx) = cancel_rx_opt {
		let cancel_fut = cancel_rx.cancelled();
		tokio::pin!(cancel_fut);

		tokio::select! {
			res = &mut run_future => (res, false),
			_ = &mut cancel_fut => (Ok(RunAgentResponse{ outputs: None, after_all: None, redo_requested: false }), true)
		}
	} else {
		(run_future.await, false)
	};

	match run_agent_res.as_ref() {
		// NOTE: Eventually we want to store the after all response as well
		Ok(_ok_res) => {
			// -- Rt Step - End
			if canceled {
				rt_step.step_run_end_canceled(run_id).await?;
			} else {
				rt_step.step_run_end_ok(run_id).await?;
			}
		}
		Err(err) => {
			// -- Rt end with err
			// NOTE: If the run error is already set, it won't reset it.
			rt_step.step_run_end_err(run_id, err).await?;
		}
	}

	run_agent_res
}

async fn run_agent_inner(
	runtime: &Runtime,
	run_id: Id,
	agent: Agent,
	inputs: Option<Vec<Value>>,
	run_base_options: &RunBaseOptions,
	return_output_values: bool,
) -> Result<RunAgentResponse> {
	let hub = get_hub();

	let rt_step = runtime.rt_step();
	let rt_model = runtime.rt_model();

	let literals = Literals::from_runtime_and_agent_path(runtime, &agent)?;
	let base_rt_ctx = RuntimeCtx::from_run_id(runtime, run_id)?;

	// -- Process Before All
	// Rt Step - Start Before All
	rt_step.step_ba_start(run_id).await?;
	// process
	let res = process_before_all(
		runtime,
		base_rt_ctx.clone(),
		run_id,
		agent.clone(),
		literals.clone(),
		inputs.clone(),
	)
	.await;
	// Capture error if anyw
	if let Err(err) = res.as_ref() {
		rt_model.set_run_end_error(run_id, Some(Stage::BeforeAll), err)?;
	}
	// -- Rt Step - End Before All
	rt_step.step_ba_end(run_id).await?;

	let ProcBeforeAllResponse {
		before_all,
		agent,
		inputs,
		skip,
		redo: redo_ba,
	} = res?;
	// skip
	if skip {
		rt_model.set_run_end_state_to_skip(run_id)?;
		return Ok(RunAgentResponse::default());
	}
	// redo
	if redo_ba {
		return Ok(RunAgentResponse {
			redo_requested: true,
			..Default::default()
		});
	}

	// -- Print the run info
	print_run_info(runtime, run_id, &agent).await?;

	// -- Run Tasks
	let (inputs, outputs) = if inputs.as_ref().is_some_and(|v| !v.is_empty()) || agent.has_task_stages() {
		// IMPORTANT - if if input is None or empty, we create a array of one nil, so that we can one task since we have some task stage
		let inputs = match inputs {
			Some(mut inputs) => {
				if inputs.is_empty() {
					vec![Value::Null]
				} else {
					// -- Add the eventual _display for _type FileInfo or FileRecord
					for input in inputs.iter_mut() {
						let is_file_item = matches!(input.x_get_str("_type"), Ok("FileRecord") | Ok("FileInfo"));
						if is_file_item
							&& input.get("_display").is_none()
							&& let Ok(path) = input.x_get_str("path").map(|v| v.to_string())
						{
							let _ = input.x_insert("_display", path);
						}
					}
					inputs
				}
			}
			None => vec![Value::Null],
		};

		// Rt Step - Tasks Start
		rt_step.step_tasks_start(run_id).await?;

		let (captured_outputs, redo_tasks) = run_tasks(
			runtime,
			run_id,
			&agent,
			&literals,
			run_base_options,
			&before_all,
			&inputs,
			return_output_values,
		)
		.await?;

		// Rt Step - Tasks End
		rt_step.step_tasks_end(run_id).await?;

		// -- Post-process outputs
		let outputs = if let Some(mut captured_outputs) = captured_outputs {
			captured_outputs.sort_by_key(|(idx, _)| *idx);
			Some(captured_outputs.into_iter().map(|(_, v)| v).collect::<Vec<_>>())
		} else {
			None
		};

		if redo_tasks {
			return Ok(RunAgentResponse {
				outputs,
				redo_requested: true,
				..Default::default()
			});
		}

		(Some(inputs), outputs)
	} else {
		(inputs, None)
	};

	let redo_requested = redo_ba;

	// -- Process After All
	// Rt Step - Start After All
	rt_step.step_aa_start(run_id).await?;
	let res = process_after_all(
		runtime,
		base_rt_ctx,
		run_id,
		&agent,
		literals,
		before_all,
		inputs,
		outputs,
	)
	.await;
	// Capture error if any
	if let Err(err) = res.as_ref() {
		rt_model.set_run_end_error(run_id, Some(Stage::AfterAll), err)?;
	}
	// Rt Step - End After All
	rt_step.step_aa_end(run_id).await?;
	let ProcAfterAllResponse { after_all, outputs } = res?;

	// -- Aggregate Redo from After All
	let mut redo_requested = redo_requested;
	if let Some(after_all) = after_all.as_ref()
		&& let Ok(FromValue::AipackCustom(AipackCustom::Redo)) = AipackCustom::from_value(after_all.clone())
	{
		redo_requested = true;
	}

	// -- For legacy tui
	hub.publish(format!("\n======= COMPLETED: {}", agent.name())).await;

	Ok(RunAgentResponse {
		after_all,
		outputs,
		redo_requested,
	})
}

async fn print_run_info(runtime: &Runtime, run_id: Id, agent: &Agent) -> Result<()> {
	let rt_log = runtime.rt_log();

	let genai_info = get_genai_info(agent);
	// display relative agent path if possible
	let agent_path = match runtime.dir_context().get_display_path(agent.file_path()) {
		Ok(path) => path.to_string(),
		Err(_) => agent.file_path().to_string(),
	};

	// Show the message
	let model_str: &str = agent.model();
	let model_resolved_str: &str = agent.model_resolved();
	let model_info = if model_str != model_resolved_str {
		format!("{model_str} ({model_resolved_str})")
	} else {
		model_resolved_str.to_string()
	};
	let agent_name = agent.name();

	let mut agent_info: Option<String> = None;
	if let AgentRef::PackRef(pack_ref) = agent.agent_ref() {
		let kind_pretty = pack_ref.repo_kind.to_pretty_lower();
		let pack_ref = pack_ref.to_string();
		agent_info = Some(format!(" ({pack_ref} from {kind_pretty})"))
	}
	let agent_info = agent_info.as_deref().unwrap_or_default();
	// TODO: might simplify message
	let msg = format!(
		"Running agent command: {agent_name}{agent_info}\n                 from: {agent_path}\n   with default model: {model_info}{genai_info}"
	);

	// -- Rt Rec - Message
	rt_log.rec_log_run(run_id, msg, Some(LogKind::SysInfo)).await?;

	Ok(())
}

/// Return the captured output if asked
#[allow(clippy::too_many_arguments)]
async fn run_tasks(
	runtime: &Runtime,
	run_id: Id,
	agent: &Agent,
	literals: &Literals,
	run_base_options: &RunBaseOptions,
	before_all: &Value,
	inputs: &[Value],
	return_output_values: bool,
) -> Result<(Option<Vec<(usize, Value)>>, bool)> {
	let rt_model = runtime.rt_model();

	// -- Initialize outputs for capture
	let mut captured_outputs: Option<Vec<(usize, Value)>> =
		if agent.after_all_script().is_some() || return_output_values {
			Some(Vec::new())
		} else {
			None
		};

	// extract concurrency
	let concurrency = agent.options().input_concurrency().unwrap_or(DEFAULT_CONCURRENCY);

	// -- Rt Update - model name & concurrency
	let _ = rt_model
		.update_run_model_and_concurrency(run_id, agent.model_resolved(), concurrency)
		.await;

	// -- Run the Tasks
	let mut join_set = JoinSet::new();
	let mut in_progress = 0;
	let mut redo_requested = false;

	// -- Rt Create all tasks (with their input)
	// Build tasks-for-create for batch insertion to reduce events and improve performance.
	let tasks_for_create: Vec<TaskForCreate> = inputs
		.iter()
		.enumerate()
		.map(|(idx, input)| TaskForCreate::new_with_input(run_id, idx as i64, None, input))
		.collect();

	// Create all tasks in one operation; ids are returned in input order.
	let task_ids: Vec<Id> = rt_model.create_tasks_batch(run_id, tasks_for_create).await?;

	// Pair each original input with its corresponding index and created task id.
	let input_idx_task_id_list: Vec<(Value, usize, Id)> = inputs
		.iter()
		.cloned()
		.zip(task_ids.into_iter())
		.enumerate()
		.map(|(idx, (input, task_id))| (input, idx, task_id))
		.collect();

	// -- Iterate and run each task (concurrency as setup)
	for (input, task_idx, task_id) in input_idx_task_id_list {
		if redo_requested {
			break;
		}

		let runtime_clone = runtime.clone();
		let agent_clone = agent.clone();
		let before_all_clone = before_all.clone();
		let literals = literals.clone();

		let base_run_config_clone = run_base_options.clone();

		// -- Spawn tasks up to the concurrency limit
		let rt = runtime.clone();
		join_set.spawn(async move {
			let rt_step = rt.rt_step();

			// -- Rt Step - Task Start
			let _ = rt_step.step_task_start(run_id, task_id).await;

			// Execute the command agent (this will perform do Data, Instruction, and Output stages)
			let res = run_agent_task_outer(
				run_id,
				task_id,
				task_idx,
				&runtime_clone,
				&agent_clone,
				before_all_clone,
				input,
				&literals,
				&base_run_config_clone,
			)
			.await;

			// -- Rt Step - Task End
			match res {
				Ok((task_idx, output)) => {
					rt_step.step_task_end_ok(run_id, task_id).await?;
					Ok((task_idx, output))
				}
				Err(err) => {
					//
					rt_step.step_task_end_err(run_id, task_id, &err).await?;
					Err(err)
				}
			}
		});

		in_progress += 1;

		// If we've reached the concurrency limit, wait for one task to complete
		if in_progress >= concurrency
			&& let Some(res) = join_set.join_next().await
			&& process_join_set_res(res, &mut in_progress, &mut captured_outputs).await?
		{
			redo_requested = true;
		}
	}

	// Wait for the remaining tasks to complete
	while in_progress > 0 {
		if let Some(res) = join_set.join_next().await
			&& process_join_set_res(res, &mut in_progress, &mut captured_outputs).await?
		{
			redo_requested = true;
		}
	}

	Ok((captured_outputs, redo_requested))
}

type JoinSetResult = core::result::Result<Result<(usize, Value)>, JoinError>;
async fn process_join_set_res(
	res: JoinSetResult,
	in_progress: &mut usize,
	outputs_vec: &mut Option<Vec<(usize, Value)>>,
) -> Result<bool> {
	*in_progress -= 1;
	match res {
		Ok(Ok((task_idx, output))) => {
			// Check for redo
			let redo = matches!(
				AipackCustom::from_value(output.clone()),
				Ok(FromValue::AipackCustom(AipackCustom::Redo))
			);

			if let Some(outputs_vec) = outputs_vec.as_mut() {
				outputs_vec.push((task_idx, output));
			}
			Ok(redo)
		}
		Ok(Err(e)) => Err(e),
		Err(e) => Err(Error::custom(format!("Error while running input. Cause {e}"))),
	}
}

/// Workaround to expose the run_command_agent_input only for test.
#[allow(unused)]
#[cfg(test)]
pub async fn run_command_agent_input_for_test(
	input_idx: usize,
	runtime: &Runtime,
	agent: &Agent,
	before_all: Value,
	input: impl serde::Serialize,
	run_base_options: &RunBaseOptions,
) -> Result<Option<Value>> {
	use crate::run::run_agent_task::run_agent_task_outer;

	let literals = Literals::from_runtime_and_agent_path(runtime, agent)?;

	//NOTE: Need to reactive.
	let (idx, output) = run_agent_task_outer(
		0.into(), // run_id,
		0.into(), // task_id,
		input_idx,
		runtime,
		agent,
		before_all,
		input,
		&literals,
		run_base_options,
	)
	.await?;

	Ok(Some(output))
}

// region:    --- Support

/// For the run commands info (before each input run)
fn get_genai_info(agent: &Agent) -> String {
	let mut genai_infos: Vec<String> = vec![];

	if let Some(temp) = agent.options().temperature() {
		genai_infos.push(format!("temperature: {temp}"));
	}

	if let Some(top_p) = agent.options().top_p() {
		genai_infos.push(format!("top_p: {top_p}"));
	}

	if genai_infos.is_empty() {
		"".to_string()
	} else {
		format!(" ({})", genai_infos.join(", "))
	}
}
// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
#[path = "../_tests/tests_run_agent_llm.rs"]
mod tests_run_agent_llm;

#[cfg(test)]
#[path = "../_tests/tests_run_agent_script.rs"]
mod tests_run_agent_script;

// endregion: --- Tests
