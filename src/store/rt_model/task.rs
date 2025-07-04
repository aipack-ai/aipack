use crate::store::base::{self, DbBmc};
use crate::store::{Id, ModelManager, Result, UnixTimeUs};
use modql::SqliteFromRow;
use modql::field::{Fields, HasSqliteFields};
use modql::filter::ListOptions;
use uuid::Uuid;

// region:    --- Types

#[derive(Debug, Clone, Fields, SqliteFromRow)]
pub struct Task {
	pub id: Id,
	pub uid: Uuid,

	pub ctime: UnixTimeUs,
	pub mtime: UnixTimeUs,

	// Foreign key
	pub run_id: Id,

	pub num: Option<i64>,

	pub start: Option<UnixTimeUs>,
	pub end: Option<UnixTimeUs>,

	pub label: Option<String>,
}

impl Task {
	#[allow(unused)]
	pub fn is_done(&self) -> bool {
		self.end.is_some()
	}
}

#[derive(Debug, Clone, Fields, SqliteFromRow)]
pub struct TaskForCreate {
	pub run_id: Id,
	pub num: Option<i64>,
	pub label: Option<String>,
}

#[derive(Debug, Default, Clone, Fields, SqliteFromRow)]
pub struct TaskForUpdate {
	pub start: Option<UnixTimeUs>,
	pub end: Option<UnixTimeUs>,
	pub label: Option<String>,
}

// endregion: --- Types

// region:    --- Bmc

pub struct TaskBmc;

impl DbBmc for TaskBmc {
	const TABLE: &'static str = "task";
}

impl TaskBmc {
	#[allow(unused)]
	pub fn create(mm: &ModelManager, task_c: TaskForCreate) -> Result<Id> {
		let fields = task_c.sqlite_not_none_fields();
		base::create::<Self>(mm, fields)
	}

	#[allow(unused)]
	pub fn update(mm: &ModelManager, id: Id, task_u: TaskForUpdate) -> Result<usize> {
		let fields = task_u.sqlite_not_none_fields();
		base::update::<Self>(mm, id, fields)
	}

	#[allow(unused)]
	pub fn get(mm: &ModelManager, id: Id) -> Result<Task> {
		base::get::<Self, _>(mm, id)
	}

	#[allow(unused)]
	pub fn list(mm: &ModelManager, list_options: Option<ListOptions>) -> Result<Vec<Task>> {
		base::list::<Self, _>(mm, list_options)
	}
}

// endregion: --- Bmc

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;
	use crate::store::rt_model::{RunBmc, RunForCreate};
	use crate::support::time::now_unix_time_us;
	use modql::filter::OrderBy;

	// region:    --- Support
	async fn create_run(mm: &ModelManager, label: &str) -> Result<Id> {
		let run_c = RunForCreate {
			agent_name: Some(label.to_string()),
			agent_path: Some(format!("path/{label}")),
			start: None,
		};
		Ok(RunBmc::create(mm, run_c)?)
	}
	// endregion: --- Support

	#[tokio::test]
	async fn test_model_task_bmc_create() -> Result<()> {
		// -- Fixture
		let mm = ModelManager::new().await?;
		let run_id = create_run(&mm, "run-1").await?;
		let task_c = TaskForCreate {
			run_id,
			num: Some(1),
			label: Some("Test Task".to_string()),
		};

		// -- Exec
		let id = TaskBmc::create(&mm, task_c)?;

		// -- Check
		assert_eq!(id.as_i64(), 1);

		Ok(())
	}

	#[tokio::test]
	async fn test_model_task_bmc_update() -> Result<()> {
		// -- Fixture
		let mm = ModelManager::new().await?;
		let run_id = create_run(&mm, "run-1").await?;
		let task_c = TaskForCreate {
			run_id,
			num: Some(1),
			label: Some("Test Task".to_string()),
		};
		let id = TaskBmc::create(&mm, task_c)?;

		// -- Exec
		let task_u = TaskForUpdate {
			start: Some(now_unix_time_us().into()),
			..Default::default()
		};
		TaskBmc::update(&mm, id, task_u)?;

		// -- Check
		let task = TaskBmc::get(&mm, id)?;
		assert!(task.start.is_some());

		Ok(())
	}

	#[tokio::test]
	async fn test_model_task_bmc_list_simple() -> Result<()> {
		// -- Fixture
		let mm = ModelManager::new().await?;
		let run_id = create_run(&mm, "run-1").await?;
		for i in 0..3 {
			let task_c = TaskForCreate {
				run_id,
				num: Some(i + 1),
				label: Some(format!("label-{i}")),
			};
			TaskBmc::create(&mm, task_c)?;
		}

		// -- Exec
		let tasks: Vec<Task> = TaskBmc::list(&mm, Some(ListOptions::default()))?;
		assert_eq!(tasks.len(), 3);
		let task = tasks.first().ok_or("Should have first item")?;
		assert_eq!(task.id, 1.into());
		assert_eq!(task.label, Some("label-0".to_string()));
		let task = tasks.get(2).ok_or("Should have 3 items")?;
		assert_eq!(task.id, 3.into());
		assert_eq!(task.label, Some("label-2".to_string()));

		Ok(())
	}

	#[tokio::test]
	async fn test_model_task_bmc_list_from_seed() -> Result<()> {
		// -- Fixture
		let mm = ModelManager::new().await?;
		let run_id = create_run(&mm, "run-seed").await?;
		for i in 0..10 {
			let task_c = TaskForCreate {
				run_id,
				num: Some(i + 1),
				label: Some(format!("label-{i}")),
			};
			TaskBmc::create(&mm, task_c)?;
		}

		// -- Exec
		let tasks: Vec<Task> = TaskBmc::list(&mm, Some(ListOptions::default()))?;
		assert_eq!(tasks.len(), 10);
		let task = tasks.first().ok_or("Should have first item")?;
		assert_eq!(task.id, 1.into());
		assert_eq!(task.label, Some("label-0".to_string()));
		let task = tasks.get(2).ok_or("Should have 3 items")?;
		assert_eq!(task.id, 3.into());
		assert_eq!(task.label, Some("label-2".to_string()));

		Ok(())
	}

	#[tokio::test]
	async fn test_model_task_bmc_list_order_by() -> Result<()> {
		// -- Fixture
		let mm = ModelManager::new().await?;
		let run_id = create_run(&mm, "run-1").await?;
		for i in 0..3 {
			let task_c = TaskForCreate {
				run_id,
				num: Some(i + 1),
				label: Some(format!("label-{i}")),
			};
			TaskBmc::create(&mm, task_c)?;
		}

		let order_bys = OrderBy::from("!id");
		let list_options = ListOptions::from(order_bys);

		// -- Exec
		let tasks: Vec<Task> = TaskBmc::list(&mm, Some(list_options))?;
		assert_eq!(tasks.len(), 3);
		let task = tasks.first().ok_or("Should have first item")?;
		assert_eq!(task.id, 3.into());
		assert_eq!(task.label, Some("label-2".to_string()));
		let task = tasks.get(2).ok_or("Should have third item")?;
		assert_eq!(task.id, 1.into());
		assert_eq!(task.label, Some("label-0".to_string()));

		Ok(())
	}
}

// endregion: --- Tests
