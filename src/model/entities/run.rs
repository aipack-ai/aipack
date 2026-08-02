use crate::model::base::{self, DbBmc};
use crate::model::{EndState, EntityType, EpochUs, Id, ModelManager, Result, RunningState, Stage};
use modql::SqliteFromRow;
use modql::field::{Fields, HasSqliteFields};
use modql::filter::ListOptions;
use uuid::Uuid;

// region:    --- Types

#[derive(Debug, Clone, Fields, SqliteFromRow)]
pub struct Run {
	pub id: Id,
	pub uid: Uuid,

	pub label: Option<String>,

	pub parent_id: Option<Id>,
	pub loop_id: Option<Id>,

	pub ctime: EpochUs,
	pub mtime: EpochUs,

	pub has_task_stages: Option<bool>,
	pub has_prompt_parts: Option<bool>,

	pub start: Option<EpochUs>,
	pub end: Option<EpochUs>,

	// Before All start/end
	pub ba_start: Option<EpochUs>,
	pub ba_end: Option<EpochUs>,

	// All tasks start/end
	pub tasks_start: Option<EpochUs>,
	pub tasks_end: Option<EpochUs>,

	// After All start/end
	pub aa_start: Option<EpochUs>,
	pub aa_end: Option<EpochUs>,

	// -- End state & Data
	pub end_state: Option<EndState>,
	pub end_err_id: Option<Id>,
	pub end_skip_reason: Option<String>,

	pub agent_name: Option<String>,
	pub agent_path: Option<String>,

	pub model: Option<String>,
	pub concurrency: Option<i32>,

	pub total_cost: Option<f64>,
	pub total_task_ms: Option<i64>,
	pub flow_redo_count: Option<i32>,
}

#[derive(Debug, Clone, Fields, SqliteFromRow)]
pub struct RunForIds {
	pub id: Id,
	pub uid: Uuid,
	pub parent_id: Option<Id>,
}

impl Run {
	pub fn is_done(&self) -> bool {
		self.end.is_some()
	}
	// pub fn has_before_all(&self) -> bool {
	// 	self.ba_start.is_some()
	// }
	// pub fn has_after_all(&self) -> bool {
	// 	self.aa_start.is_some()
	// }
}

#[derive(Debug, Clone, Fields, SqliteFromRow)]
pub struct RunForCreate {
	pub parent_id: Option<Id>,

	pub agent_name: Option<String>,
	pub agent_path: Option<String>,
	pub has_prompt_parts: Option<bool>,
	pub has_task_stages: Option<bool>,
}

#[derive(Debug, Default, Clone, Fields, SqliteFromRow)]
pub struct RunForUpdate {
	pub label: Option<String>,

	pub loop_id: Option<Id>,

	pub start: Option<EpochUs>,
	pub end: Option<EpochUs>,

	// States
	pub has_prompt: Option<bool>,

	// Before All start/end
	pub ba_start: Option<EpochUs>,
	pub ba_end: Option<EpochUs>,

	// All tasks start/end
	pub tasks_start: Option<EpochUs>,
	pub tasks_end: Option<EpochUs>,

	// After All start/end
	pub aa_start: Option<EpochUs>,
	pub aa_end: Option<EpochUs>,

	// -- End state & Data
	pub end_state: Option<EndState>,
	pub end_err_id: Option<Id>,
	pub end_skip_reason: Option<String>,

	pub agent_name: Option<String>,
	pub agent_path: Option<String>,

	pub model: Option<String>,
	pub concurrency: Option<i32>,

	pub total_cost: Option<f64>,
	pub total_task_ms: Option<i64>,
	pub flow_redo_count: Option<i32>,
}

// endregion: --- Types

// region:    --- Froms

impl From<&Run> for RunningState {
	fn from(value: &Run) -> Self {
		if value.end.is_some() {
			RunningState::Ended(value.end_state)
		} else if value.start.is_some() {
			RunningState::Running
		} else {
			RunningState::Waiting
		}
	}
}

// endregion: --- Froms

// region:    --- Bmc

pub struct RunBmc;

impl DbBmc for RunBmc {
	const TABLE: &'static str = "run";
	const ENTITY_TYPE: EntityType = EntityType::Run;
}

/// Basic CRUD
impl RunBmc {
	pub fn create(mm: &ModelManager, run_c: RunForCreate) -> Result<Id> {
		let fields = run_c.sqlite_not_none_fields();
		base::create::<Self>(mm, fields)
	}

	#[allow(unused)]
	pub fn update(mm: &ModelManager, id: Id, run_u: RunForUpdate) -> Result<usize> {
		let fields = run_u.sqlite_not_none_fields();
		base::update::<Self>(mm, id, fields)
	}

	#[allow(unused)]
	pub fn get(mm: &ModelManager, id: Id) -> Result<Run> {
		base::get::<Self, _>(mm, id)
	}

	pub fn list(mm: &ModelManager, list_options: Option<ListOptions>) -> Result<Vec<Run>> {
		base::list::<Self, _>(mm, list_options, None)
	}
}

pub struct RunForUids {
	#[allow(unused)]
	pub id: Id,
	pub uid: Uuid,
	pub parent_uid: Option<Uuid>,
}

/// RunBmc specifics
impl RunBmc {
	pub fn get_uids(mm: &ModelManager, id: Id) -> Result<RunForUids> {
		let RunForIds { id, uid, parent_id } = base::get::<Self, RunForIds>(mm, id)?;
		let parent_uid = if let Some(parent_id) = parent_id {
			Some(RunBmc::get_uid(mm, parent_id)?)
		} else {
			None
		};
		Ok(RunForUids { id, uid, parent_uid })
	}

	pub fn list_for_display(mm: &ModelManager, limit: Option<i64>) -> Result<Vec<Run>> {
		let mut options = ListOptions::from_order_bys("!id");
		if let Some(limit) = limit {
			options.limit = Some(limit);
		}
		Self::list(mm, Some(options))
	}

	/// Create the ErrRec and assign it to this run, and set the end state
	/// NOTE:
	///   - This does not set the end time (just the end_state)
	pub fn set_end_error(mm: &ModelManager, run_id: Id, stage: Option<Stage>, error: &crate::Error) -> Result<()> {
		use crate::model::{ContentTyp, ErrBmc, ErrForCreate};

		// -- Create the err rec
		let err_c = ErrForCreate {
			stage,
			run_id: Some(run_id),
			task_id: None,
			typ: Some(ContentTyp::Text),
			content: Some(error.to_string()),
		};
		let err_id = ErrBmc::create(mm, err_c)?;

		// -- Update the run
		let run_u = RunForUpdate {
			end_state: Some(EndState::Err),
			end_err_id: Some(err_id),
			..Default::default()
		};
		Self::update(mm, run_id, run_u)?;

		Ok(())
	}
}

// endregion: --- Bmc

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;
	use crate::support::time::now_micro;
	use modql::filter::OrderBy;

	#[tokio::test]
	async fn test_model_run_bmc_create() -> Result<()> {
		// -- Fixture
		let mm = ModelManager::new().await?;
		let run_c = RunForCreate {
			agent_name: Some("Test Run".to_string()),
			agent_path: Some("test/path".to_string()),
			parent_id: None,
			has_task_stages: None,
			has_prompt_parts: None,
		};

		// -- Exec
		let id = RunBmc::create(&mm, run_c)?;

		// -- Check
		assert_eq!(id.as_i64(), 1);

		Ok(())
	}

	#[tokio::test]
	async fn test_model_run_bmc_update() -> Result<()> {
		// -- Fixture
		let mm = ModelManager::new().await?;
		let run_c = RunForCreate {
			parent_id: None,
			agent_name: Some("Test Run".to_string()),
			agent_path: Some("test/path".to_string()),
			has_task_stages: None,
			has_prompt_parts: None,
		};
		let id = RunBmc::create(&mm, run_c)?;

		// -- Exec
		let run_u = RunForUpdate {
			start: Some(now_micro().into()),
			..Default::default()
		};
		RunBmc::update(&mm, id, run_u)?;

		// -- Check
		let run = RunBmc::get(&mm, id)?;
		assert!(run.start.is_some());

		Ok(())
	}

	#[tokio::test]
	async fn test_model_run_bmc_list_simple() -> Result<()> {
		// -- Fixture
		let mm = ModelManager::new().await?;
		for i in 0..3 {
			let run_c = RunForCreate {
				parent_id: None,
				agent_name: Some(format!("label-{i}")),
				agent_path: Some(format!("path/label-{i}")),
				has_task_stages: None,
				has_prompt_parts: None,
			};
			RunBmc::create(&mm, run_c)?;
		}

		// -- Exec
		let runs: Vec<Run> = RunBmc::list(&mm, Some(ListOptions::default()))?;
		assert_eq!(runs.len(), 3);
		let run = runs.first().ok_or("Should have first item")?;
		assert_eq!(run.id, 1.into());
		assert_eq!(run.label, Some("label-0".to_string()));
		let run = runs.get(2).ok_or("Should have 3 items")?;
		assert_eq!(run.id, 3.into());
		assert_eq!(run.label, Some("label-2".to_string()));

		Ok(())
	}

	#[tokio::test]
	async fn test_model_run_bmc_list_from_seed() -> Result<()> {
		// -- Fixture
		let mm = ModelManager::new().await?;
		mm.mock_rt_seed().await?;

		// -- Exec
		let runs: Vec<Run> = RunBmc::list(&mm, Some(ListOptions::default()))?;
		assert_eq!(runs.len(), 10);
		let run = runs.first().ok_or("Should have first item")?;
		assert_eq!(run.id, 1.into());
		assert_eq!(run.label, Some("label-0".to_string()));
		let run = runs.get(2).ok_or("Should have third item")?;
		assert_eq!(run.id, 3.into());
		assert_eq!(run.label, Some("label-2".to_string()));

		Ok(())
	}

	#[tokio::test]
	async fn test_model_run_bmc_list_order_by() -> Result<()> {
		// -- Fixture
		let mm = ModelManager::new().await?;
		for i in 0..3 {
			let run_c = RunForCreate {
				parent_id: None,
				agent_name: Some(format!("label-{i}")),
				agent_path: Some(format!("path/label-{i}")),
				has_task_stages: None,
				has_prompt_parts: None,
			};
			RunBmc::create(&mm, run_c)?;
		}

		let order_bys = OrderBy::from("!id");
		let list_options = ListOptions::from(order_bys);

		// -- Exec
		let runs: Vec<Run> = RunBmc::list(&mm, Some(list_options))?;
		assert_eq!(runs.len(), 3);
		let run = runs.first().ok_or("Should have first item")?;
		assert_eq!(run.id, 3.into());
		assert_eq!(run.label, Some("label-2".to_string()));
		let run = runs.get(2).ok_or("Should have third item")?;
		assert_eq!(run.id, 1.into());
		assert_eq!(run.label, Some("label-0".to_string()));

		Ok(())
	}
}

// endregion: --- Tests
