use crate::get_hub;
use crate::model::base::{self, DbBmc};
use crate::model::{
	EntityAction, EntityType, Id, ModelEvent, ModelManager, Result, Run, RunBmc, RunForCreate, RelIds,
};
use crate::support::time::now_micro;
use modql::SqliteFromRow;
use modql::field::{Fields, HasFields, HasSqliteFields};
use modql::filter::ListOptions;
use uuid::Uuid;

// region:    --- Types

#[derive(Debug, Clone, Fields, SqliteFromRow)]
#[allow(dead_code)]
pub struct Loop {
	pub id: Id,
	pub uid: Uuid,

	pub ctime: i64,
	pub mtime: i64,

	pub first_run_id: Id,
	pub last_run_id: Id,
	pub pending: bool,
	pub total_cost: f64,
}

#[derive(Debug, Clone, Fields, SqliteFromRow)]
pub struct LoopForCreate {
	pub first_run_id: Id,
	pub last_run_id: Id,
	pub pending: bool,
	pub total_cost: f64,
}

#[derive(Debug, Default, Clone, Fields, SqliteFromRow)]
#[allow(dead_code)]
pub struct LoopForUpdate {
	pub first_run_id: Option<Id>,
	pub last_run_id: Option<Id>,
	pub pending: Option<bool>,
	pub total_cost: Option<f64>,
}

// endregion: --- Types

// region:    --- Bmc

pub struct LoopBmc;

impl DbBmc for LoopBmc {
	const TABLE: &'static str = "loop";

	// Loop changes use related run events so existing model-event consumers refresh navigation.
	const ENTITY_TYPE: EntityType = EntityType::Run;
}

#[allow(dead_code)]
impl LoopBmc {
	pub fn create(mm: &ModelManager, loop_c: LoopForCreate) -> Result<Id> {
		let fields = loop_c.sqlite_not_none_fields();
		let id = mm.db().exec_in_tx(|tx| base::create_in_tx::<Self>(tx, fields))?;

		publish_event(EntityAction::Created, None);
		Ok(id)
	}

	pub fn update(mm: &ModelManager, id: Id, loop_u: LoopForUpdate) -> Result<usize> {
		let fields = loop_u.sqlite_not_none_fields();
		let count = mm.db().exec_in_tx(|tx| base::update_in_tx::<Self>(tx, id, fields))?;

		publish_event(EntityAction::Updated, None);
		Ok(count)
	}

	pub fn get(mm: &ModelManager, id: Id) -> Result<Loop> {
		base::get::<Self, _>(mm, id)
	}

	pub fn list(mm: &ModelManager, list_options: Option<ListOptions>) -> Result<Vec<Loop>> {
		base::list::<Self, _>(mm, list_options, None)
	}

	pub fn create_for_run(mm: &ModelManager, run_id: Id) -> Result<Id> {
		Self::create_for_first_member(mm, run_id)
	}

	pub fn list_members(mm: &ModelManager, loop_id: Id) -> Result<Vec<Run>> {
		let sql = format!(
			"SELECT {} FROM run WHERE loop_id = ? ORDER BY id",
			Run::sql_columns()
		);
		mm.db().fetch_all(&sql, (loop_id,))
	}

	pub fn create_for_first_member(mm: &ModelManager, run_id: Id) -> Result<Id> {
		let loop_id = mm.db().exec_in_tx(|tx| {
			let loop_fields = LoopForCreate {
				first_run_id: run_id,
				last_run_id: run_id,
				pending: true,
				total_cost: 0.0,
			}
			.sqlite_not_none_fields();
			let loop_id = base::create_in_tx::<Self>(tx, loop_fields)?;

			let count = tx.exec(
				"UPDATE run SET loop_id = ?, mtime = ? WHERE id = ? AND loop_id IS NULL",
				(loop_id, now_micro(), run_id),
			)?;
			if count != 1 {
				return Err(format!("Cannot assign run {} to a new loop", run_id.as_i64()).into());
			}

			recompute_cost_in_tx(tx, loop_id)?;
			Ok(loop_id)
		})?;

		publish_event(EntityAction::Created, None);
		publish_event_for_run(EntityAction::Updated, run_id, loop_id);
		Ok(loop_id)
	}

	pub fn create_member(mm: &ModelManager, loop_id: Id, run_c: RunForCreate) -> Result<Id> {
		let run_id = mm.db().exec_in_tx(|tx| {
			let mut run_fields = run_c.sqlite_not_none_fields();
			run_fields.push(modql::field::SqliteField::new("loop_id", loop_id));

			let run_id = base::create_in_tx::<RunBmc>(tx, run_fields)?;
			let count = tx.exec(
				"UPDATE loop SET last_run_id = ?, mtime = ?, pending = 1 WHERE id = ? AND pending = 1",
				(run_id, now_micro(), loop_id),
			)?;
			if count != 1 {
				return Err(format!("Cannot append run to loop {}", loop_id.as_i64()).into());
			}

			recompute_cost_in_tx(tx, loop_id)?;
			Ok(run_id)
		})?;

		publish_event_for_run(EntityAction::Created, run_id, loop_id);
		publish_event(EntityAction::Updated, None);
		Ok(run_id)
	}

	pub fn reopen_and_create_member(mm: &ModelManager, loop_id: Id, run_c: RunForCreate) -> Result<Id> {
		let run_id = mm.db().exec_in_tx(|tx| {
			let mut run_fields = run_c.sqlite_not_none_fields();
			run_fields.push(modql::field::SqliteField::new("loop_id", loop_id));

			let run_id = base::create_in_tx::<RunBmc>(tx, run_fields)?;
			let count = tx.exec(
				"UPDATE loop SET last_run_id = ?, mtime = ?, pending = 1 WHERE id = ?",
				(run_id, now_micro(), loop_id),
			)?;
			if count != 1 {
				return Err(format!("Cannot reopen loop {}", loop_id.as_i64()).into());
			}

			recompute_cost_in_tx(tx, loop_id)?;
			Ok(run_id)
		})?;

		publish_event_for_run(EntityAction::Created, run_id, loop_id);
		publish_event(EntityAction::Updated, None);
		Ok(run_id)
	}

	pub fn assign_member(mm: &ModelManager, loop_id: Id, run_id: Id) -> Result<()> {
		mm.db().exec_in_tx(|tx| {
			let count = tx.exec(
				"UPDATE run SET loop_id = ?, mtime = ? WHERE id = ? AND loop_id IS NULL",
				(loop_id, now_micro(), run_id),
			)?;
			if count != 1 {
				return Err(format!("Cannot assign run {} to loop {}", run_id.as_i64(), loop_id.as_i64()).into());
			}

			let count = tx.exec(
				"UPDATE loop SET last_run_id = ?, mtime = ? WHERE id = ?",
				(run_id, now_micro(), loop_id),
			)?;
			if count != 1 {
				return Err(format!("Cannot update loop {}", loop_id.as_i64()).into());
			}

			recompute_cost_in_tx(tx, loop_id)?;
			Ok(())
		})?;

		publish_event_for_run(EntityAction::Updated, run_id, loop_id);
		publish_event(EntityAction::Updated, None);
		Ok(())
	}

	pub fn set_pending(mm: &ModelManager, loop_id: Id, pending: bool) -> Result<()> {
		mm.db().exec_in_tx(|tx| {
			let count = tx.exec(
				"UPDATE loop SET pending = ?, mtime = ? WHERE id = ?",
				(pending, now_micro(), loop_id),
			)?;
			if count != 1 {
				return Err(format!("Cannot update loop {}", loop_id.as_i64()).into());
			}
			Ok(())
		})?;

		publish_event(EntityAction::Updated, None);
		Ok(())
	}

	pub fn reopen(mm: &ModelManager, loop_id: Id) -> Result<()> {
		Self::set_pending(mm, loop_id, true)
	}

	pub fn recompute_cost(mm: &ModelManager, loop_id: Id) -> Result<f64> {
		let total_cost = mm.db().exec_in_tx(|tx| recompute_cost_in_tx(tx, loop_id))?;

		publish_event(EntityAction::Updated, None);
		Ok(total_cost)
	}
}

// endregion: --- Bmc

// region:    --- Support

fn recompute_cost_in_tx(tx: &crate::model::db::DbTx<'_>, loop_id: Id) -> Result<f64> {
	let total_cost = tx.exec_returning_as(
		"SELECT COALESCE(SUM(total_cost), 0.0) FROM run WHERE loop_id = ?",
		(loop_id,),
	)?;

	let count = tx.exec(
		"UPDATE loop SET total_cost = ?, mtime = ? WHERE id = ?",
		(total_cost, now_micro(), loop_id),
	)?;
	if count != 1 {
		return Err(format!("Cannot update loop {}", loop_id.as_i64()).into());
	}

	Ok(total_cost)
}

fn publish_event(action: EntityAction, id: Option<Id>) {
	get_hub().publish_sync(ModelEvent {
		entity: EntityType::Run,
		action,
		id,
		rel_ids: RelIds::default(),
	});
}

fn publish_event_for_run(action: EntityAction, run_id: Id, loop_id: Id) {
	get_hub().publish_sync(ModelEvent {
		entity: EntityType::Run,
		action,
		id: Some(run_id),
		rel_ids: RelIds {
			run_id: Some(run_id),
			..Default::default()
		},
	});
	let _ = loop_id;
}

// endregion: --- Support
