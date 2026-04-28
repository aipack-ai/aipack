// region:    --- Modules

use crate::hub::get_hub;
use crate::model::base::{self, DbBmc};
use crate::model::{EntityAction, EntityType, EpochUs, Id, ModelEvent, ModelManager, RelIds, Result, UcontentBmc};
use modql::SqliteFromRow;
use modql::field::{Fields, HasSqliteFields, SqliteField, SqliteFields};
use modql::filter::{ListOptions, OrderBys};
use uuid::Uuid;

// endregion: --- Modules

// region:    --- Public Types

#[derive(Debug, Clone, Fields, SqliteFromRow)]
pub struct Pin {
	pub id: Id,
	pub uid: Uuid,

	pub ctime: EpochUs,
	pub mtime: EpochUs,

	pub run_id: Id,
	pub task_id: Option<Id>,

	pub iden: Option<String>,
	pub priority: Option<f64>,
	pub content: Option<String>,
}

/// Used by aip_.. to do the save
/// Not to be send to the data save layer
#[derive(Debug, Clone)]
pub struct PinForRunSave {
	pub run_id: Id,
	pub iden: String,

	pub priority: Option<f64>,
	pub content: Option<String>,
}

/// Used by aip_.. to do the save
/// Not to be send to the data save layer
#[derive(Debug, Clone)]
pub struct PinForTaskSave {
	pub run_id: Id,
	pub task_id: Id,
	pub iden: String,

	pub priority: Option<f64>,
	pub content: Option<String>,
}

// endregion: --- Public Types

// region:    --- Private Types

/// This is private to this module
#[derive(Debug, Clone, Fields, SqliteFromRow)]
struct PinForCreate {
	pub run_id: Id,
	pub task_id: Option<Id>,

	pub iden: String,
	pub priority: Option<f64>,
	pub ucontent_id: Option<Id>,
}

/// This is private to thie module
#[derive(Debug, Clone, Fields, SqliteFromRow, Default)]
#[allow(unused)]
struct PinForUpdate {
	pub priority: Option<f64>,
	pub ucontent_id: Option<Id>,
}

// endregion: --- Private Types

// region:    --- Bmc

pub struct PinBmc;

impl DbBmc for PinBmc {
	const TABLE: &'static str = "pin";
	const ENTITY_TYPE: EntityType = EntityType::Pin;
}

/// Public Bmcs
impl PinBmc {
	/// NOTE: Here we use a double-check approach to avoid using a transaction and to work around
	///       the limitations of SQLite's RETURNING id constraints with CTE and conflict handling.
	/// - First, we check if a pin already exists and update it if found.
	/// - If not, we attempt to create one, but ensure we do not create a duplicate if the iden for this run already exists.
	///   - If creation did not occur (i.e., it was created concurrently), we assume it was created concurrently and try to update.
	///   - NOTE: There is an argument for not updating if it was created concurrently.
	pub fn save_run_pin(mm: &ModelManager, pin_s: PinForRunSave) -> Result<Id> {
		let run_id = pin_s.run_id;
		let pin_id = Self::get_run_pin_by_iden(mm, pin_s.run_id, &pin_s.iden)?;
		let pin_u = Self::resolve_run_pin_fields(mm, &pin_s)?;

		let (id, action) = if let Some(pin_id) = pin_id {
			Self::update(mm, pin_id, pin_u.clone())?;
			(pin_id, EntityAction::Updated)
		} else {
			let pin_c = pin_u.clone();

			// -- Attempt to create
			let fields = pin_c.sqlite_not_none_fields();
			let where_not_exists_fields = SqliteFields::new(vec![
				SqliteField::new("run_id", pin_s.run_id),
				SqliteField::new("iden", pin_s.iden.clone()),
			]);

			let may_id =
				base::create_where_not_exists::<Self>(mm, fields, where_not_exists_fields, Some("task_id IS NULL"))?;

			if let Some(id) = may_id {
				(id, EntityAction::Created)
			} else {
				let pin_id = Self::get_run_pin_by_iden(mm, pin_s.run_id, &pin_s.iden)?;
				let pin_id = pin_id.ok_or(format!("Should have returned a pin id for pin: {}", pin_s.iden))?;
				// NOTE: we might not want to update here. This was was perhaps late?
				Self::update(mm, pin_id, pin_u)?;

				(pin_id, EntityAction::Updated)
			}
		};

		get_hub().publish_sync(ModelEvent {
			entity: EntityType::Pin,
			action,
			id: Some(id),
			rel_ids: RelIds {
				run_id: Some(run_id),
				pin_id: Some(id),
				..Default::default()
			},
		});

		Ok(id)
	}

	pub fn save_task_pin(mm: &ModelManager, pin_s: PinForTaskSave) -> Result<Id> {
		let run_id = pin_s.run_id;
		let task_id = pin_s.task_id;
		let pin_id = Self::get_task_pin_by_iden(mm, pin_s.task_id, &pin_s.iden)?;
		let pin_u = Self::resolve_task_pin_fields(mm, &pin_s)?;

		let (id, action) = if let Some(pin_id) = pin_id {
			Self::update(mm, pin_id, pin_u)?;
			(pin_id, EntityAction::Updated)
		} else {
			(Self::create(mm, pin_u)?, EntityAction::Created)
		};

		get_hub().publish_sync(ModelEvent {
			entity: EntityType::Pin,
			action,
			id: Some(id),
			rel_ids: RelIds {
				run_id: Some(run_id),
				task_id: Some(task_id),
				pin_id: Some(id),
				..Default::default()
			},
		});

		Ok(id)
	}

	// -- Read helpers (unchanged)

	#[allow(unused)]
	pub fn get(mm: &ModelManager, id: Id) -> Result<Pin> {
		let sql = format!(
			"SELECT {} FROM {} p LEFT JOIN ucontent uc ON p.ucontent_id = uc.id WHERE p.id = ? LIMIT 1",
			Self::pin_select_columns(),
			Self::table_ref()
		);

		let db = mm.db();
		let entity: Pin = db
			.fetch_first(&sql, [(&id)])?
			.ok_or_else(|| format!("Cannot get entity '{}'", Self::TABLE))?;

		Ok(entity)
	}

	/// List for run
	/// NOTE: Manual since we have the IS NULL
	pub fn list_for_run(mm: &ModelManager, run_id: Id) -> Result<Vec<Pin>> {
		// Sort by priority (nulls last), then by creation time.
		let order_bys = OrderBys::from(vec!["priority IS NULL", "priority", "ctime"]);
		let list_options = ListOptions::from(order_bys);

		let order_by = list_options
			.order_bys
			.map(|ob| ob.join_for_sql())
			.unwrap_or_else(|| "id".to_string());

		let sql = format!(
			"SELECT {} FROM {} p LEFT JOIN ucontent uc ON p.ucontent_id = uc.id WHERE run_id = ? AND task_id IS NULL ORDER BY {order_by}",
			Self::pin_select_columns(),
			PinBmc::table_ref()
		);

		let db = mm.db();
		let entities: Vec<Pin> = db.fetch_all(&sql, (run_id,))?;

		Ok(entities)
	}

	/// List for task
	pub fn list_for_task(mm: &ModelManager, task_id: Id) -> Result<Vec<Pin>> {
		// Sort by priority (nulls last), then by creation time.
		let order_bys = OrderBys::from(vec!["priority IS NULL", "priority", "ctime"]);
		let list_options = ListOptions::from(order_bys);
		let order_by = list_options
			.order_bys
			.map(|ob| ob.join_for_sql())
			.unwrap_or_else(|| "id".to_string());

		let sql = format!(
			"SELECT {} FROM {} p LEFT JOIN ucontent uc ON p.ucontent_id = uc.id WHERE task_id = ? ORDER BY {order_by}",
			Self::pin_select_columns(),
			Self::table_ref()
		);

		let db = mm.db();
		let entities: Vec<Pin> = db.fetch_all(&sql, (task_id,))?;

		Ok(entities)
	}
}

/// Private
impl PinBmc {
	fn create(mm: &ModelManager, pin_c: PinForCreate) -> Result<Id> {
		let fields = pin_c.sqlite_not_none_fields();
		base::create::<Self>(mm, fields)
	}

	fn update(mm: &ModelManager, id: Id, pin_c: PinForCreate) -> Result<usize> {
		let fields = pin_c.sqlite_not_none_fields();
		base::update::<Self>(mm, id, fields)
	}

	fn resolve_run_pin_fields(mm: &ModelManager, pin_s: &PinForRunSave) -> Result<PinForCreate> {
		let ucontent_id = Self::resolve_ucontent_id(mm, pin_s.content.as_deref())?;

		Ok(PinForCreate {
			run_id: pin_s.run_id,
			task_id: None,
			iden: pin_s.iden.clone(),
			priority: pin_s.priority,
			ucontent_id,
		})
	}

	fn resolve_task_pin_fields(mm: &ModelManager, pin_s: &PinForTaskSave) -> Result<PinForCreate> {
		let ucontent_id = Self::resolve_ucontent_id(mm, pin_s.content.as_deref())?;

		Ok(PinForCreate {
			run_id: pin_s.run_id,
			task_id: Some(pin_s.task_id),
			iden: pin_s.iden.clone(),
			priority: pin_s.priority,
			ucontent_id,
		})
	}

	fn resolve_ucontent_id(mm: &ModelManager, content: Option<&str>) -> Result<Option<Id>> {
		content
			.map(|content| UcontentBmc::get_or_create_for_text(mm, content, true))
			.transpose()
	}

	fn pin_select_columns() -> &'static str {
		r#"p.id AS "id", p.uid AS "uid", p.ctime AS "ctime", p.mtime AS "mtime", p.run_id AS "run_id", p.task_id AS "task_id", p.iden AS "iden", p.priority AS "priority", uc.content AS "content""#
	}

	// return the pin ID
	pub fn get_run_pin_by_iden(mm: &ModelManager, run_id: Id, iden: &str) -> Result<Option<Id>> {
		let sql = "SELECT id FROM pin WHERE run_id = ? AND iden = ? AND task_id IS NULL";
		let id = mm.db().exec_returning_as_optional::<i64>(sql, (run_id, iden))?;

		Ok(id.map(|id| id.into()))
	}

	// return the pin ID
	pub fn get_task_pin_by_iden(mm: &ModelManager, task_id: Id, iden: &str) -> Result<Option<Id>> {
		let sql = "SELECT id FROM pin WHERE task_id = ? AND iden = ?";
		let id = mm.db().exec_returning_as_optional::<i64>(sql, (task_id, iden))?;

		Ok(id.map(|id| id.into()))
	}
}

// endregion: --- Bmc

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;
	use crate::_test_support;

	#[tokio::test]
	async fn test_model_pin_bmc_save_run_pin() -> Result<()> {
		// -- Setup & Fixtures
		let mm = ModelManager::new().await?;
		let run_id = _test_support::create_run(&mm, "run-1")?;

		// -- Exec
		let pin_c = PinForRunSave {
			run_id,
			iden: "work-summary".to_string(),
			priority: Some(0.5),
			content: Some("content 01".to_string()),
		};
		let id = PinBmc::save_run_pin(&mm, pin_c)?;

		// -- Check
		assert_eq!(id.as_i64(), 1);
		let pin: Pin = PinBmc::get(&mm, id)?;
		assert_eq!(pin.run_id, run_id);
		assert_eq!(pin.priority, Some(0.5));
		assert_eq!(pin.content.as_deref(), Some("content 01"));

		Ok(())
	}

	#[tokio::test]
	async fn test_model_pin_bmc_save_task_pin() -> Result<()> {
		// -- Setup & Fixtures
		let mm = ModelManager::new().await?;
		let run_id = _test_support::create_run(&mm, "run-1")?;
		let task_id = _test_support::create_task(&mm, run_id, 1)?;

		// -- Exec
		let pin_c = PinForTaskSave {
			run_id,
			task_id,
			iden: "work-summary".to_string(),
			priority: Some(0.5),
			content: Some(r#"{"type":"Marker","content":"First Pin"}"#.to_string()),
		};
		let id = PinBmc::save_task_pin(&mm, pin_c)?;

		// -- Check
		assert_eq!(id.as_i64(), 1);
		let pin: Pin = PinBmc::get(&mm, id)?;
		assert_eq!(pin.task_id, Some(task_id));
		assert_eq!(pin.priority, Some(0.5));

		Ok(())
	}

	#[tokio::test]
	async fn test_model_pin_bmc_list_for_run() -> Result<()> {
		// -- Setup & Fixtures
		let mm = ModelManager::new().await?;
		let run_1_id = _test_support::create_run(&mm, "run-1")?;
		let run_2_id = _test_support::create_run(&mm, "run-2")?;

		// Pins for run 1 - create out of order to test sorting
		PinBmc::save_run_pin(
			&mm,
			PinForRunSave {
				run_id: run_1_id,
				iden: "pin-1.2".to_string(),
				priority: Some(2.0),
				content: None,
			},
		)?;
		// twice the same ot make sure only update
		PinBmc::save_run_pin(
			&mm,
			PinForRunSave {
				run_id: run_1_id,
				iden: "pin-1.2".to_string(),
				priority: Some(2.0),
				content: None,
			},
		)?;
		PinBmc::save_run_pin(
			&mm,
			PinForRunSave {
				run_id: run_1_id,
				iden: "pin-1.1".to_string(),
				priority: Some(1.0),
				content: None,
			},
		)?;
		PinBmc::save_run_pin(
			&mm,
			PinForRunSave {
				run_id: run_1_id,
				iden: "pin-1.none".to_string(),
				priority: None,
				content: None,
			},
		)?;

		// Pin for run 2
		PinBmc::save_run_pin(
			&mm,
			PinForRunSave {
				run_id: run_2_id,
				iden: "pin-2.1".to_string(),
				priority: Some(1.0),
				content: None,
			},
		)?;

		// -- Exec
		let pins = PinBmc::list_for_run(&mm, run_1_id)?;

		// -- Check
		assert_eq!(pins.len(), 3);
		// Check run_id for all
		for pin in &pins {
			assert_eq!(pin.run_id, run_1_id);
		}

		// Check order and priorities
		let pin0 = pins.first().ok_or("Should have pin 0")?;
		assert_eq!(pin0.iden.as_deref(), Some("pin-1.1"));
		assert_eq!(pin0.priority, Some(1.0));

		let pin1 = pins.get(1).ok_or("Should have pin 1")?;
		assert_eq!(pin1.iden.as_deref(), Some("pin-1.2"));
		assert_eq!(pin1.priority, Some(2.0));

		let pin2 = pins.get(2).ok_or("Should have pin 2")?;
		assert_eq!(pin2.iden.as_deref(), Some("pin-1.none"));
		assert_eq!(pin2.priority, None);

		Ok(())
	}
}

// endregion: --- Tests
