# AIPack Runtime Specification for LLM

This document provides a concise reference for the AIPack `runtime` module, orchestrating the lifecycle of agent execution and linking the Scripting, AI, and Model layers.

## Core Architecture

The `Runtime` is an `Arc` wrapper around `RuntimeInner`. It is the central state object cloned and passed to every execution task.

### Runtime (src/runtime/runtime_impl.rs)

Contains references to:
- **DirContext**: Path resolution logic and environment structure.
- **ModelManager**: Database access.
- **genai::Client**: AI connector.
- **ExecutorTx**: Channel to send new actions back to the executor.
- **Session**: A unique UUID for the current application lifecycle.
- **CancelTrx**: Watcher for user-initiated cancellation.

## Runtime Components

The `Runtime` provides specialized helpers (sub-handles) to perform specific side effects.

### RtStep (src/runtime/rt_step.rs)

Orchestrates execution milestones. Every `step_...` function:
1. Updates the corresponding timestamp in the `run` or `task` table.
2. Records a `LogKind::RunStep` log entry.
3. Potentially updates aggregate states (e.g., `total_task_ms`).

### RtModel (src/runtime/rt_model.rs)

High-level helpers for entity lifecycle:
- `create_run`: Normalizes agent metadata and initializes the DB record.
- `create_task` / `create_tasks_batch`: Initializes tasks with `TypedContent` handling.
- `update_task_cost`: Updates task cost and recomputes the total run cost.
- `rec_skip_run` / `rec_skip_task`: Handles skip logic, updating status and logging reasons.
- `mark_run_ai_used`: Sets the run-level `ai_used` flag to `true`. It is idempotent and called from every real genai call site, including non-task stages (before-all, after-all, and auxiliary auto calls). It carries no cost change, so no loop cost recompute is done. The Group Dash Models and Agents subtabs filter on this flag, while the header totals and Top Runs do not.
- **Work Tracking**: While not having a dedicated `RtWork` handle yet, the runtime layer uses the `WorkBmc` to track the lifecycle of background operations (like pack installation) to provide status updates to the TUI.

### RtLog (src/runtime/rt_log.rs)

Unified logging interface:
- Writes to the `log` table in the database.
- Publishes messages to the `Hub` for real-time TUI/CLI display.
- Supports `LogKind` levels (Info, Warn, Debug) and links logs to specific `Stage` values.

## Runtime Context (src/model/runtime_ctx.rs)

The `RuntimeCtx` is a lightweight structure (UIDs only) injected into Lua as a global `CTX` table. It allows Lua module calls to know:
- Which `RUN_UID` and `TASK_UID` they are operating under.
- The current execution `STAGE`.
- The current redo-chain count through `CTX.RUN_FLOW_REDO_COUNT` when the run is part of a redo rerun.
- It is extracted using `RuntimeCtx::extract_from_global(lua)`.

## Execution Lifecycle

1. **Initialization**: `Runtime::new` is called with a `DirContext` and `ModelManager`.
2. **Run Start**: `rt_step().step_run_start(run_id)` is called.
3. **Phases**: `BeforeAll`, `Data`, `Ai`, `Output`, `AfterAll` are executed.
4. **Task Iteration**: Tasks are created in `Data` and processed through `Ai` and `Output`.
5. **Completion**: `rt_step().step_run_end_ok(run_id)` (or `err`/`canceled`) finalizes the database records.

## Redo Interaction with Runtime

- The runtime and run pipeline are responsible for detecting redo directives returned by agent stages and surfacing redo intent in the run result.
- The runtime does not directly schedule the next redo action.
- The runtime carries the current redo count in run context for the active execution.
- Instead, redo scheduling is delegated to the executor, which:
  - stores the latest redo context
  - increments the redo count for the next rerun
  - waits `500ms`
  - enqueues the next executor `Redo` action when the completed run requested redo

This separation keeps runtime concerns focused on execution, state transitions, and result production, while the executor owns lifecycle scheduling for follow-up runs.

## Redo Count Flow

- A new top-level run starts with redo count `0`.
- Each accepted redo transition advances the count by `1` for the next rerun in the same chain.
- The count is exposed to Lua through `CTX.RUN_FLOW_REDO_COUNT`.
- `CTX.RUN_FLOW_REDO_COUNT` is not present for the initial non-redo run.
- Invalid redo attempts do not advance the count because no rerun is scheduled.
