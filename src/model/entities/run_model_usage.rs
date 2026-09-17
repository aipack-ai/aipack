use crate::model::base::{self, DbBmc};
use crate::model::{EntityType, EpochUs, Id, ModelManager, Result};
use crate::support::time::now_micro;
use modql::SqliteFromRow;
use modql::field::{Fields, HasFields, HasSqliteFields};
use rusqlite::params_from_iter;
use uuid::Uuid;

// region:    --- Types

#[derive(Debug, Clone, Fields, SqliteFromRow)]
pub struct RunModelUsage {
	pub id: Id,
	pub uid: Uuid,

	pub ctime: EpochUs,
	pub mtime: EpochUs,

	// Foreign key
	pub run_id: Id,

	pub agent_name: Option<String>,
	pub model_name: String,

	pub call_count: i64,

	pub cost: Option<f64>,
	pub cost_cache_write: Option<f64>,
	pub cost_cache_saving: Option<f64>,
}

#[derive(Debug, Default, Clone, Fields, SqliteFromRow)]
pub struct RunModelUsageFilter {
	pub run_id: Option<Id>,
}

// endregion: --- Types

// region:    --- Bmc

pub struct RunModelUsageBmc;

impl DbBmc for RunModelUsageBmc {
	const TABLE: &'static str = "run_model_usage";
	// Usage rows are run-scoped, and this bmc writes through raw SQL only, so it shares the run entity type.
	const ENTITY_TYPE: EntityType = EntityType::Run;
}

/// Record & Read
impl RunModelUsageBmc {
	/// Record one genai call for a run and model.
	/// The `call_count` increment and the cost sums happen inside SQLite, so concurrent
	/// tasks sharing a run and a model cannot lose an update.
	pub fn record(
		mm: &ModelManager,
		run_id: Id,
		agent_name: Option<&str>,
		model_name: &str,
		cost: Option<f64>,
		cost_cache_write: Option<f64>,
		cost_cache_saving: Option<f64>,
	) -> Result<()> {
		let fields = RunModelUsageForRecord {
			run_id,
			agent_name: agent_name.map(|s| s.to_string()),
			model_name: model_name.to_string(),
			call_count: 1,
			cost,
			cost_cache_write,
			cost_cache_saving,
			uid: Uuid::new_v4(),
			ctime: now_micro().into(),
			mtime: now_micro().into(),
		};
		let fields = fields.sqlite_not_none_fields();

		let sql = format!(
			"
INSERT INTO {table} ({columns}) VALUES ({placeholders})
ON CONFLICT(run_id, model_name) DO UPDATE SET
	call_count = call_count + excluded.call_count,
	agent_name = COALESCE(excluded.agent_name, agent_name),
	cost = CASE WHEN excluded.cost IS NULL THEN cost ELSE COALESCE(cost, 0.0) + excluded.cost END,
	cost_cache_write = CASE WHEN excluded.cost_cache_write IS NULL THEN cost_cache_write ELSE COALESCE(cost_cache_write, 0.0) + excluded.cost_cache_write END,
	cost_cache_saving = CASE WHEN excluded.cost_cache_saving IS NULL THEN cost_cache_saving ELSE COALESCE(cost_cache_saving, 0.0) + excluded.cost_cache_saving END,
	mtime = excluded.mtime",
			table = Self::table_ref(),
			columns = fields.sql_columns(),
			placeholders = fields.sql_placeholders(),
		);

		let values = fields.values_as_dyn_to_sql_vec();
		let db = mm.db();
		db.exec(&sql, &*values)?;

		Ok(())
	}

	#[allow(unused)]
	pub fn list_for_run(mm: &ModelManager, run_id: Id) -> Result<Vec<RunModelUsage>> {
		let filter_fields = RunModelUsageFilter { run_id: Some(run_id) }.sqlite_not_none_fields();
		base::list::<Self, _>(mm, None, Some(filter_fields))
	}

	#[allow(unused)]
	pub fn list_for_runs(mm: &ModelManager, run_ids: &[Id]) -> Result<Vec<RunModelUsage>> {
		if run_ids.is_empty() {
			return Ok(Vec::new());
		}

		let placeholders = std::iter::repeat_n("?", run_ids.len()).collect::<Vec<_>>().join(", ");
		let sql = format!(
			"SELECT {} FROM {} WHERE run_id IN ({placeholders}) ORDER BY id",
			RunModelUsage::sql_columns(),
			Self::table_ref(),
		);

		let db = mm.db();
		let usages: Vec<RunModelUsage> = db.fetch_all(&sql, params_from_iter(run_ids.iter()))?;

		Ok(usages)
	}
}

// endregion: --- Bmc

// region:    --- Support

/// Insert struct for the record upsert, so `uid`/`ctime`/`mtime` and the non-none columns are built like the other creates.
#[derive(Debug, Fields)]
struct RunModelUsageForRecord {
	run_id: Id,
	agent_name: Option<String>,
	model_name: String,
	call_count: i64,
	cost: Option<f64>,
	cost_cache_write: Option<f64>,
	cost_cache_saving: Option<f64>,
	uid: Uuid,
	ctime: EpochUs,
	mtime: EpochUs,
}

// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;
	use crate::model::{RunBmc, RunForCreate};

	#[tokio::test]
	async fn test_model_run_model_usage_bmc_record_accumulates() -> Result<()> {
		// -- Setup & Fixtures
		let mm = ModelManager::new().await?;
		let run_id = create_run(&mm, "run-1")?;

		// -- Exec
		RunModelUsageBmc::record(&mm, run_id, Some("agent-a"), "gpt-x", Some(0.5), Some(0.25), None)?;
		RunModelUsageBmc::record(&mm, run_id, Some("agent-a"), "gpt-x", Some(0.125), Some(0.0), None)?;

		// -- Check
		let usages = RunModelUsageBmc::list_for_run(&mm, run_id)?;
		assert_eq!(usages.len(), 1);
		let usage = usages.first().ok_or("Should have one usage row")?;
		assert_eq!(usage.call_count, 2);
		assert_eq!(usage.cost, Some(0.625));
		assert_eq!(usage.cost_cache_write, Some(0.25));
		assert_eq!(usage.cost_cache_saving, None);

		Ok(())
	}

	#[tokio::test]
	async fn test_model_run_model_usage_bmc_list_for_runs() -> Result<()> {
		// -- Setup & Fixtures
		let mm = ModelManager::new().await?;
		let run_id_a = create_run(&mm, "run-a")?;
		let run_id_b = create_run(&mm, "run-b")?;

		RunModelUsageBmc::record(&mm, run_id_a, Some("agent-a"), "gpt-x", Some(0.5), None, None)?;
		RunModelUsageBmc::record(&mm, run_id_a, Some("agent-a"), "luna", Some(0.1), None, None)?;
		RunModelUsageBmc::record(&mm, run_id_b, Some("agent-b"), "gpt-x", Some(0.2), None, None)?;

		// -- Exec
		let usages = RunModelUsageBmc::list_for_runs(&mm, &[run_id_a, run_id_b])?;

		// -- Check
		assert_eq!(usages.len(), 3);
		let run_ids: Vec<Id> = usages.iter().map(|u| u.run_id).collect();
		assert!(run_ids.contains(&run_id_a));
		assert!(run_ids.contains(&run_id_b));
		let has_luna = usages.iter().any(|u| u.model_name == "luna");
		assert!(has_luna);

		Ok(())
	}

	// -- Test Support

	fn create_run(mm: &ModelManager, label: &str) -> Result<Id> {
		let run_c = RunForCreate {
			parent_id: None,
			agent_name: Some(label.to_string()),
			agent_path: Some(format!("path/{label}")),
			has_task_stages: None,
			has_prompt_parts: None,
		};
		Ok(RunBmc::create(mm, run_c)?)
	}
}

// endregion: --- Tests
