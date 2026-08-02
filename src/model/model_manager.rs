use crate::model::Result;
use crate::model::db::Db;

#[derive(Debug, Clone)]
pub struct ModelManager {
	db: Db,
}

/// Constructors
impl ModelManager {
	pub async fn new() -> Result<Self> {
		let db = Db::new()?;
		db.recreate()?;
		Ok(Self { db })
	}
}

/// Getters
impl ModelManager {
	pub fn db(&self) -> &Db {
		&self.db
	}
}

/// Management
impl ModelManager {
	/// NOTE: This is to make sure the db does not become too big in memory
	///      For now, very agressive, just delete everything.
	/// Should be called at the start of each run
	pub fn trim(&self) -> Result<usize> {
		let db = self.db();
		let run_count = db.exec("DELETE FROM run", [])?;
		let task_count = db.exec("DELETE FROM task", [])?;
		let log_count = db.exec("DELETE FROM log", [])?;
		let work_count = db.exec("DELETE FROM work", [])?;
		let loop_count = db.exec(
			"DELETE FROM loop WHERE NOT EXISTS (SELECT 1 FROM run WHERE run.loop_id = loop.id)",
			[],
		)?;

		Ok(run_count + task_count + log_count + work_count + loop_count)
	}

	pub fn db_size(&self) -> Result<i64> {
		let db = self.db();
		let sql = r#"
SELECT page_count * page_size as size_bytes
FROM pragma_page_count(), pragma_page_size();		
		"#;

		let res = db.exec_returning_num(sql, ())?;
		Ok(res)
	}
}

// region:    --- OnceModelManager

use tokio::sync::OnceCell;

#[derive(Clone, Copy)]
pub struct OnceModelManager;

impl OnceModelManager {
	/// Returns a reference to the singleton `ModelManager`, creating it on first call.
	pub async fn get(&self) -> Result<ModelManager> {
		static INSTANCE: OnceCell<ModelManager> = OnceCell::const_new();
		let val = INSTANCE.get_or_try_init(|| async { ModelManager::new().await }).await?;
		Ok(val.clone())
	}
}

// endregion: --- OnceModelManager

// region:    --- Mock Seed

impl ModelManager {
	pub async fn mock_rt_seed(&self) -> Result<()> {
		use crate::model::{RunBmc, RunForCreate};

		for i in 0..10 {
			let run_c = RunForCreate {
				parent_id: None,
				agent_name: Some(format!("agent_name-{i}")),
				agent_path: Some(format!("agent_path-{i}")),
				has_task_stages: None,
				has_prompt_parts: None,
			};
			let _id = RunBmc::create(self, run_c)?;
		}

		Ok(())
	}
}

// endregion: --- Mock Seed
