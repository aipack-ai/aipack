use crate::Result;
use crate::agent::Agent;
use crate::hub::get_hub;
use crate::runtime::Runtime;
use crate::store::base::DbBmc;
use crate::store::rt_model::{
	LogBmc, LogForCreate, LogKind, RunBmc, RunForCreate, RunForUpdate, TaskBmc, TaskForCreate, TaskForUpdate,
};
use crate::store::{EndState, Id, ModelManager, Stage, TypedContent};
use derive_more::From;
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, From)]
pub struct RtModel<'a> {
	runtime: &'a Runtime,
}

/// Constructor & Core Getters
impl<'a> RtModel<'a> {
	pub(super) fn new(runtime: &'a Runtime) -> Self {
		Self { runtime }
	}

	fn mm(&self) -> &ModelManager {
		self.runtime.mm()
	}
}

/// Run Create/Update model
impl<'a> RtModel<'a> {
	pub async fn create_run(&self, parent_uid: Option<Uuid>, agent: &Agent) -> Result<Id> {
		let hub = get_hub();

		let agent_path = match self.runtime.dir_context().get_display_path(agent.file_path()) {
			Ok(path) => path.to_string(),
			Err(_) => agent.file_path().to_string(),
		};
		let agent_name = agent.name();

		let parent_id = if let Some(uid) = parent_uid {
			Some(RunBmc::get_id_for_uid(self.mm(), uid)?)
		} else {
			None
		};

		// -- Create Run
		let run_id = RunBmc::create(
			self.mm(),
			RunForCreate {
				parent_id,
				agent_name: Some(agent_name.to_string()),
				agent_path: Some(agent_path.to_string()),
				has_task_stages: Some(agent.has_task_stages()),
				has_prompt_parts: Some(agent.has_prompt_parts()),
			},
		)?;

		// -- For V1 terminal
		hub.publish(format!(
			"\n======= RUNNING: {agent_name}\n     Agent path: {agent_path}",
		))
		.await;

		Ok(run_id)
	}

	pub async fn update_run_model_and_concurrency(
		&self,
		run_id: Id,
		model_name: &str,
		concurrency: usize,
	) -> Result<()> {
		let run_u = RunForUpdate {
			model: Some(model_name.to_string()),
			concurrency: Some(concurrency as i32),
			..Default::default()
		};
		RunBmc::update(self.mm(), run_id, run_u)?;

		Ok(())
	}

	pub fn set_run_end_error(&self, run_id: Id, stage: Option<Stage>, err: &crate::Error) -> Result<()> {
		RunBmc::set_end_error(self.mm(), run_id, stage, err)?;
		Ok(())
	}

	/// Note: the rec log already happened (in the current design)
	/// This does not set the end time
	pub fn set_run_end_state_to_skip(&self, run_id: Id) -> Result<()> {
		RunBmc::update(
			self.mm(),
			run_id,
			RunForUpdate {
				end_state: Some(EndState::Skip),
				..Default::default()
			},
		)?;
		Ok(())
	}

	/// NOTE: Probably shoul put the end state as well
	pub async fn rec_skip_run(&self, run_id: Id, stage: Stage, reason: Option<String>) -> Result<()> {
		let mm = self.mm();

		let reason_txt = reason.as_ref().map(|r| format!(" (Reason: {r})")).unwrap_or_default();

		// -- Update the Run end_skip_reason
		RunBmc::update(
			mm,
			run_id,
			RunForUpdate {
				end_skip_reason: reason.clone(),
				..Default::default()
			},
		)?;

		// -- Update the log
		let log_c = LogForCreate {
			run_id,
			task_id: None,
			step: None,
			stage: Some(stage),
			message: reason,
			kind: Some(LogKind::AgentSkip),
		};
		LogBmc::create(mm, log_c)?;

		// -- publish for legacy
		get_hub()
			.publish(format!("Aipack Skip input at {stage} stage: {reason_txt}"))
			.await;

		Ok(())
	}
}

/// Task Create/Update model
impl<'a> RtModel<'a> {
	pub async fn create_task(&self, run_id: Id, idx: usize, input: &Value) -> Result<Id> {
		let input_content = TypedContent::from_value(input);

		let task_c = TaskForCreate {
			run_id,
			idx: idx as i64,
			label: None,
			input_content: Some(input_content),
		};
		let id = TaskBmc::create(self.mm(), task_c)?;
		Ok(id)
	}

	pub async fn create_tasks_batch(&self, run_id: Id, items: Vec<TaskForCreate>) -> Result<Vec<Id>> {
		// Defensive: ensure all items are for the same run to keep the API predictable at call sites.
		debug_assert!(items.iter().all(|t| t.run_id == run_id));

		let ids = TaskBmc::create_batch(self.mm(), items)?;
		Ok(ids)
	}

	pub async fn update_task_model_ov(&self, _run_id: Id, task_id: Id, model_name_ov: &str) -> Result<()> {
		let task_u = TaskForUpdate {
			model_ov: Some(model_name_ov.to_string()),
			..Default::default()
		};
		TaskBmc::update(self.mm(), task_id, task_u)?;

		Ok(())
	}

	pub async fn update_task_usage(&self, _run_id: Id, task_id: Id, usage: &genai::chat::Usage) -> Result<()> {
		let task_u = TaskForUpdate::from_usage(usage);
		TaskBmc::update(self.mm(), task_id, task_u)?;
		Ok(())
	}

	pub async fn update_task_cost(&self, run_id: Id, task_id: Id, cost: f64) -> Result<()> {
		// -- Update Task
		let task_u = TaskForUpdate {
			cost: Some(cost),
			..Default::default()
		};
		TaskBmc::update(self.mm(), task_id, task_u)?;

		// -- Update the run total cost
		// NOTE: Here we recompute the total cost rather than doing a simple add to avoid
		//       any race condition
		let tasks = TaskBmc::list_for_run(self.mm(), run_id)?;
		let total_cost: f64 = tasks.iter().filter_map(|t| t.cost).sum();
		let run_u = RunForUpdate {
			total_cost: Some(total_cost),
			..Default::default()
		};
		RunBmc::update(self.mm(), run_id, run_u)?;

		Ok(())
	}

	/// NOTE: if the .content & .display is None, then, nothing is saved
	pub async fn update_task_output(&self, task_id: Id, output: &Value) -> Result<()> {
		let output_content = TypedContent::from_value(output);
		if output_content.content.is_none() && output_content.display.is_none() {
			return Ok(()); // Nothing to update
		}

		TaskBmc::update_output(self.mm(), task_id, output_content)?;
		Ok(())
	}

	pub fn set_task_end_error(&self, _run_id: Id, task_id: Id, stage: Option<Stage>, err: &crate::Error) -> Result<()> {
		TaskBmc::set_end_error(self.mm(), task_id, stage, err)?;
		Ok(())
	}

	/// Note: the rec log already happened (in the current design)
	/// This does not set the end time
	pub fn set_task_end_state_to_skip(&self, _run_id: Id, task_id: Id) -> Result<()> {
		TaskBmc::update(
			self.mm(),
			task_id,
			TaskForUpdate {
				end_state: Some(EndState::Skip),
				..Default::default()
			},
		)?;
		Ok(())
	}

	pub async fn rec_skip_task(&self, run_id: Id, task_id: Id, stage: Stage, reason: Option<String>) -> Result<()> {
		let mm = self.mm();

		let reason_txt = reason.as_ref().map(|r| format!(" (Reason: {r})")).unwrap_or_default();

		// -- Update the Run end_skip_reason
		TaskBmc::update(
			mm,
			task_id,
			TaskForUpdate {
				end_skip_reason: reason.clone(),
				..Default::default()
			},
		)?;

		// -- Update the log
		let log_c = LogForCreate {
			run_id,
			task_id: Some(task_id),
			step: None,
			stage: Some(stage),
			message: reason,
			kind: Some(LogKind::AgentSkip),
		};
		LogBmc::create(mm, log_c)?;

		// -- publish for legacy
		get_hub()
			.publish(format!("Aipack Skip input at {stage} stage: {reason_txt}"))
			.await;

		Ok(())
	}
}
