use crate::model::Result;
use rusqlite::Connection;

// Some notes:
// - Currently, the database is in-memory only, but it may become persistent at the session level.
// - All tables have an `id` used for same-db joins, and a `uid` which is a UUID blob,
//   intended for sharing outside of Rust or across databases.
// - `id` uses `AUTOINCREMENT` to ensure IDs are not reused if a row is deleted.
// - `MAIN_TABLES` are the main database tables for all metadata. They are designed to be relatively small and to scale well.
// - `CONTENT_TABLES` are designed to hold larger content and may have different trimming/cleaning strategies.
//    - A future strategy could involve having a set of content tables per run, using the b58 run.uid suffix. This would make it very fast to clean up old ones.
// - References between these two sets of tables are by `uid`, as they may eventually reside in different databases.

pub fn recreate_db(con: &Connection) -> Result<()> {
	create_schema(con)?;
	Ok(())
}

// region:    --- Main Tables

const LOOP_TABLE: (&str, &str) = (
	"loop",
	"
CREATE TABLE IF NOT EXISTS loop (
		id            INTEGER PRIMARY KEY AUTOINCREMENT,
		uid           BLOB NOT NULL,

		ctime         INTEGER NOT NULL,
		mtime         INTEGER NOT NULL,

		first_run_id  INTEGER NOT NULL,
		last_run_id   INTEGER NOT NULL,
		pending       INTEGER NOT NULL,
		total_cost    REAL NOT NULL DEFAULT 0
) STRICT",
);

const RUN_TABLE: (&str, &str) = (
	"run",
	"
CREATE TABLE IF NOT EXISTS run (
		id          INTEGER PRIMARY KEY AUTOINCREMENT,
		uid         BLOB NOT NULL,

		label       TEXT,	-- Only when agent call. aip.task.set_label('some label')

		parent_id   INTEGER,
		loop_id     INTEGER,

		ctime  INTEGER NOT NULL,
		mtime  INTEGER NOT NULL,

		has_prompt_parts INTEGER,
		has_task_stages  INTEGER,

		-- Step Timestamps
		start       INTEGER,
		ba_start    INTEGER,	-- Before All start/end
		ba_end      INTEGER,
		tasks_start INTEGER,  -- All tasks start/end
		tasks_end   INTEGER,
		aa_start    INTEGER,  -- After All start/end
		aa_end      INTEGER,
		end         INTEGER,

		-- End state & Data
		end_state        TEXT,
		end_err_id       INTEGER,
		end_skip_reason  TEXT,

		agent_name  TEXT,
		agent_path  TEXT,

		model       TEXT,
		concurrency INTEGER,

		-- Computed
		total_cost    REAL,
		total_task_ms INTEGER, -- cummulative time
		flow_redo_count INTEGER

) STRICT",
);

const TASK_TABLE: (&str, &str) = (
	"task",
	"
CREATE TABLE IF NOT EXISTS task (
		id     INTEGER PRIMARY KEY AUTOINCREMENT,
		uid    BLOB NOT NULL,

		ctime  INTEGER NOT NULL,
		mtime  INTEGER NOT NULL,

		run_id INTEGER NOT NULL,
		idx    INTEGER, -- Relative to the run (as created by Run)

		-- Step Timestamps
		start          INTEGER,
		data_start     INTEGER,
		data_end       INTEGER,
		ai_start       INTEGER,
		ai_gen_start   INTEGER,
		ai_gen_end     INTEGER,
		ai_end         INTEGER,
		output_start   INTEGER,
		output_end     INTEGER,
		end            INTEGER,

		-- End state & Data
		end_state        TEXT,
		end_err_id       INTEGER,
		end_skip_reason  TEXT,

		-- prompt
		prompt_size      INTEGER, -- in bytes

		-- Model
		model_ov         TEXT,
		model_upstream   TEXT,    -- from te provider

		-- Model Pricing
		pricing_model         TEXT,
		pricing_input         REAL,
		pricing_input_cached  REAL,
		pricing_output        REAL,

		-- Usage Raw
		usage BLOB, -- jsonb, to have raw usage
		-- Usage Values
		tk_prompt_total             INTEGER,
		tk_prompt_cached            INTEGER,
		tk_prompt_cache_creation    INTEGER,
		tk_completion_total         INTEGER,
		tk_completion_reasoning     INTEGER,

		cost                REAL,
		cost_cache_write    REAL,
		cost_cache_saving   REAL,

		label               TEXT,

		input_uid           BLOB,
		input_short         TEXT,
		input_has_display   INTEGER,

		output_uid          BLOB,
		output_short        TEXT,
		output_has_display  INTEGER

) STRICT",
);

const LOG_TABLE: (&str, &str) = (
	"log",
	"
CREATE TABLE IF NOT EXISTS log (
		id      INTEGER PRIMARY KEY AUTOINCREMENT,
		uid     BLOB NOT NULL,

		ctime   INTEGER NOT NULL,
		mtime   INTEGER NOT NULL,

		run_id  INTEGER NOT NULL, -- Should always belong to a run
		task_id INTEGER,          -- Might belong to a task

		kind    TEXT,  -- UserPrint, SysInfo, SysWarn, SysDebug

		stage   TEXT,

		step    TEXT,

		message TEXT
) STRICT",
);

const ERR_TABLE: (&str, &str) = (
	"err",
	"
CREATE TABLE IF NOT EXISTS err (
		id       INTEGER PRIMARY KEY AUTOINCREMENT,
		uid      BLOB NOT NULL,

		ctime    INTEGER NOT NULL,
		mtime    INTEGER NOT NULL,

		run_id  INTEGER, -- for now, allow null, for global errors
		stage   TEXT,
		task_id INTEGER,

		typ      TEXT, -- 'text' | 'json'
		content  TEXT
) STRICT",
);

/// The actions could be like
/// ```js
/// [
///  { content: "Quit", keys: ["q", "y"]},
///  { content: "Cancel", keys: ["esc", "#any"]},
/// ]
/// ```
/// Fields could be
/// ```js
/// [
///   { name: "domain", label:"Domain Name", type: "number", modifier: {min: 0, max: 99} },
///   { name: "first_page", label:"First Page", type: "text", modifier: "single-line" }
/// ]
/// ```
const PROMPT_TABLE: (&str, &str) = (
	"prompt",
	"
CREATE TABLE IF NOT EXISTS prompt (
		id      INTEGER PRIMARY KEY AUTOINCREMENT,
		uid     BLOB NOT NULL,

		ctime   INTEGER NOT NULL,
		mtime   INTEGER NOT NULL,

		kind    TEXT,    -- Sys, Agent (from what part of the system)

		run_id  INTEGER, -- Might belong to a run
		task_id INTEGER, -- Might belong to a task

		title   TEXT,
		message TEXT,
		fields  TEXT, -- will be json
		actions TEXT -- will be json
) STRICT",
);

// Content:
// marker: {type: "Marker", content: { label: "Pin", label_color: "red", content = "15 files uploaded"}}
// aip.task.pin("work-summary", 0, { type = "Marker", content = {label = }} )
const PIN_TABLE: (&str, &str) = (
	"pin",
	"
CREATE TABLE IF NOT EXISTS pin (
		id        INTEGER PRIMARY KEY AUTOINCREMENT,
		uid       BLOB NOT NULL,

		ctime     INTEGER NOT NULL,
		mtime     INTEGER NOT NULL,

		run_id    INTEGER NOT NULL, -- Should always belong to a run
		task_id   INTEGER,          -- Might belong to a task

		iden      TEXT NOT NULL,    -- The identifier of this pin. Unique for this run/task
		priority  REAL,             -- The priority (if none, will be last)
		ucontent_id INTEGER
) STRICT",
);

const UCONTENT_TABLE: (&str, &str) = (
	"ucontent",
	"
CREATE TABLE IF NOT EXISTS ucontent (
		id        INTEGER PRIMARY KEY AUTOINCREMENT,
		uid       BLOB NOT NULL,

		ctime     INTEGER NOT NULL,
		mtime     INTEGER NOT NULL,

		hash      TEXT NOT NULL,
		is_json   INTEGER NOT NULL DEFAULT 0,
		content   TEXT
) STRICT",
);

const WORK_TABLE: (&str, &str) = (
	"work",
	"
CREATE TABLE IF NOT EXISTS work (
		id          INTEGER PRIMARY KEY AUTOINCREMENT,
		uid         BLOB NOT NULL,

		ctime       INTEGER NOT NULL,
		mtime       INTEGER NOT NULL,

		kind        TEXT NOT NULL,

		start       INTEGER,
		end         INTEGER,
		end_state   TEXT,
		end_err_id  INTEGER,

		data        TEXT, -- JSON
		message     TEXT
) STRICT",
);

const ALL_MAIN_TABLES: &[(&str, &str)] = &[
	LOOP_TABLE,
	RUN_TABLE,
	TASK_TABLE,
	ERR_TABLE,
	LOG_TABLE,
	PROMPT_TABLE,
	PIN_TABLE,
	UCONTENT_TABLE,
	WORK_TABLE,
];

// endregion: --- Main Tables

// region:    --- Content Tables

// NOTE: Currently, the idea is that content tables are for "larger content" and should be efficient to delete per run.
//       However, this concept is not fully realized yet. One idea is to have table names with a run_uid_b58 suffix,
//       allowing for very fast deletion. Implementing this will require additional code.
//       Also, at present, we do not have run_id or run_uid in these tables, which is problematic when we want to trim by run.
//       At least the main tables can do that by run.

const INOUT_TABLE: (&str, &str) = (
	"inout",
	"
CREATE TABLE IF NOT EXISTS inout (
		id       INTEGER PRIMARY KEY AUTOINCREMENT,
		uid      BLOB NOT NULL,

		ctime    INTEGER NOT NULL,
		mtime    INTEGER NOT NULL,

		task_uid BLOB NOT NULL, -- might not be needed but used to guarantee on task

		kind     TEXT, -- In, Out

		typ      TEXT, -- 'text' | 'json'
		content  TEXT,
		display  TEXT
) STRICT",
);

const ALL_CONTENT_TABLES: &[(&str, &str)] = &[INOUT_TABLE];

// endregion: --- Content Tables

// region:    --- Support

fn create_schema(con: &Connection) -> Result<()> {
	for tables in [ALL_MAIN_TABLES, ALL_CONTENT_TABLES] {
		for (name, table_sql) in tables {
			con.execute(table_sql, ())?;
			con.execute(
				&format!(
					"
		CREATE INDEX IF NOT EXISTS idx_{name}_uid ON {name}(uid);
		"
				),
				(),
			)?;
		}
	}

	con.execute(
		"
CREATE INDEX IF NOT EXISTS idx_ucontent_hash ON ucontent(hash);
		",
		(),
	)?;

	con.execute(
		"
CREATE INDEX IF NOT EXISTS idx_run_loop_id ON run(loop_id);
		",
		(),
	)?;

	con.execute(
		"
CREATE INDEX IF NOT EXISTS idx_run_loop_id_id ON run(loop_id, id);
		",
		(),
	)?;

	con.execute(
		"
CREATE INDEX IF NOT EXISTS idx_loop_first_run_id ON loop(first_run_id);
		",
		(),
	)?;

	con.execute(
		"
CREATE INDEX IF NOT EXISTS idx_loop_last_run_id ON loop(last_run_id);
		",
		(),
	)?;

	Ok(())
}

// endregion: --- Support
