# AIPack Model Specification for LLM

This document provides a concise reference for the AIPack `model` module architecture, entity relationships, and the BMC (Base Model Controller) pattern.

## Core Architecture

The `model` module manages a SQLite in-memory database using `rusqlite` and `modql`. It is organized into layers: `db` (engine), `base` (generic CRUD), `entities` (business logic), and `types` (common primitives).

### ModelManager (src/model/model_manager.rs)

- **ModelManager**: Owns the `Db` instance. Passed throughout the system via `Runtime`.
- **OnceModelManager**: A singleton provider used by the `Executor` to ensure a single shared database instance.
- **trim()**: Clears main tables (`run`, `task`, `log`, `work`). Typically called at the start of new top-level runs.

### Database Schema (src/model/db/rt_db_setup.rs)

- **Main Tables**: `run`, `task`, `log`, `err`, `prompt`, `pin`, `work`.
- **Content Tables**: `inout` (stores large task inputs/outputs, linked by `task_uid`).
- **Keys**: All tables use `id` (INTEGER PRIMARY KEY) for internal joins and `uid` (BLOB/UUID) for external/Lua references.

## The BMC Pattern (src/model/base/...)

Every entity has a corresponding BMC struct (e.g., `RunBmc`, `TaskBmc`, `WorkBmc`) that implements `DbBmc`.

### Generic CRUD (crud_fns.rs)
- `create<MC>(mm, fields)`: Adds `uid`, `ctime`, `mtime`.
- `update<MC>(id, fields)`: Updates `mtime`.
- `get<MC, E>(id)`: Fetches a single entity by ID.
- `list<MC, E>(list_options, filter)`: Fetches a collection with ordering and filtering.
- `batch_create<MC>(mm, items)`: High-performance insertion within a single transaction.

## Entity Modules (src/model/entities/...)

### Run (run.rs)
Tracks the execution of an Agent.
- **Key Fields**: `agent_name`, `agent_path`, `model`, `concurrency`, `total_cost`, `total_task_ms`, `ai_used`.
- **Timestamps**: `start`, `ba_start`/`ba_end`, `tasks_start`/`tasks_end`, `aa_start`/`aa_end`, `end`.
- **End State**: `end_state`, `end_err_id`, `end_skip_reason`.
- **AI Usage**: `ai_used` is a run-level flag set only when a genai call has actually returned, across per-task, before-all, after-all, and auxiliary auto calls. The Group Dash Models and Agents subtabs filter on it, while the header totals and Top Runs do not.

### Task (task.rs)
Tracks individual inputs/files within a run.
- **Key Fields**: `run_id`, `idx`, `label`, `model_ov`, `cost`, `tk_prompt_total`, `tk_completion_total`.
- **Content Refs**: `input_uid`, `input_short`, `output_uid`, `output_short`.
- **BMC Helpers**: `get_input_for_display(mm, task)` and `get_output_for_display(mm, task)` handle reading from `inout` if needed.

### Log (log.rs)
Captures execution events. 
- **LogKind**: `RunStep`, `SysInfo`, `SysWarn`, `SysError`, `SysDebug`, `AgentPrint`, `AgentSkip`.
- **Fields**: `run_id`, `task_id`, `kind`, `step`, `stage`, `message`.

### Err (err.rs)
Stores detailed error messages.
- **Fields**: `run_id`, `task_id`, `stage`, `typ`, `content`.

### Pin (pin.rs)
Persistent UI markers created via `aip.task.pin`.
- **Fields**: `run_id`, `task_id`, `iden`, `priority`, `content` (JSON `uc::Marker`).

### Work (work.rs)
Tracks background operations (e.g., `Install`).
- **WorkKind**: `Install`.
- **Fields**: `kind`, `start`, `end`, `end_state`, `end_err_id`, `data` (JSON metadata like `pack_ref`), `message`.

### Inout (inout.rs)
Large content storage for tasks.
- **Fields**: `task_uid`, `kind` (In/Out), `typ` (Text/Json), `content`, `display`.

## Common Types (src/model/types/...)

- **Id / EpochUs**: `ScalarStruct` wrappers for `i64`.
- **Stage**: `BeforeAll`, `Data`, `Ai`, `AiGen`, `Output`, `AfterAll`.
- **RunStep**: Granular markers like `BaStart`, `TaskAiGenStart`, `End`.
- **RunningState**: `Waiting`, `Running`, `Ended(Option<EndState>)`, `NotScheduled`.
- **EndState**: `Ok`, `Err`, `Cancel`, `Skip`.
- **TypedContent**: Handles `Json` or `Text` content.
    - `extract_short()`: Returns truncated content for the `task` table.
    - `SHORT_MAX_CHAR_LENGTH = 64`.
