# AIPack TUI Specification for LLM

This document provides a concise reference for the AIPack TUI architecture, types, and APIs.

## Enums & Core Types

### Stages & Tabs (src/tui/core/...)

```rust
pub enum AppStage {
    Normal,     // Main run view
    Installing, // Dialog-style installation progress overlay
    Installed,  // Success dialog overlay (auto-dismisses after 4s)
    PromptInstall(Id), // Interactive installation confirmation
    Config(ConfigTab), // Configuration popup (Work in progress / Disabled)
}

pub enum RunTab { Overview, Tasks }
```

### Events (src/tui/core/event/...)

```rust
pub enum AppEvent {
    DoRedraw,
    Term(crossterm::event::Event),
    Action(AppActionEvent),
    Data(DataEvent),
    Hub(HubEvent),
    Tick(i64),
}

pub enum AppActionEvent {
    Quit, 
    Redo, 
    CancelRun,
    Scroll(ScrollDir), ScrollPage(ScrollDir), ScrollToEnd(ScrollDir),
    WorkConfirm(Id),
    WorkCancel(Id),
    Run(RunArgs),
}
```

### Actions (src/tui/core/types/ui_action.rs)

```rust
pub enum UiAction {
    Quit, Redo, CancelRun,
    ToggleRunsNav,
    CycleTasksOverviewMode,
    // Configuration (Work in progress / Disabled)
    ShowConfig,
    CloseConfig,
    SwitchConfigTab(ConfigTab),

    GoToTask { task_id: Id },
    WorkConfirm(Id),
    WorkCancel(Id),
    WorkRun(Id),
    WorkClose(Id),
    ToClipboardCopy(String),
    OpenFile(String), // Opens file at path using auto-editor
    ShowText,
}
```

## Actions vs. Events

AIPack distinguishes between **UI Intents** and **System Commands** to maintain a clean separation between view state and loop execution.

### UiAction (UI Intent)
`UiAction` represents a deferred request from the view layer. It is stored in `AppState` and processed during the `state_processor` cycle. This allows the system to resolve relative intents (like "Copy this task") into absolute data using the full context of the state before execution.

### AppActionEvent (System Command)
`AppActionEvent` represents a discrete, asynchronous instruction sent to the main loop via the `AppTx` channel. It triggers global side-effects (like quitting or interacting with the `Executor`).

### Execution Flow
1. **User Interaction**: A view component sets a `UiAction` in `AppState`.
2. **State Processing**: `src/tui/core/app_state/state_processor.rs` reads the action.
3. **Conversion**: The processor executes immediate logic (e.g., clipboard) or converts the intent into a system-level `AppActionEvent`.
4. **Transport**: The event is sent via the `AppTx` channel.
5. **Handling**: `src/tui/core/app_event_handlers.rs` performs the final side-effect.

## AppState API

`AppState` is the primary interface for views.

### Main Accessors
- `mm()`, `stage()`, `show_runs()`, `run_tab()`, `last_app_event()`.
- `installing_pack_ref()`, `current_work_id()`.
- `config_tab()`, `is_config_stage()`.
- `should_redraw()`, `trigger_redraw()`, `should_be_pinged()`.

### Run & Task Data
- `run_items()`, `current_run_item()`.
- `tasks()`, `current_task()`, `task_idx()`.
- `all_run_children(run_item)`: Returns direct and indirect children.

### Formatting & UI Getters (impl_fmt.rs)
- `current_run_duration_txt()`, `current_run_cost_fmt()`.
- `current_run_concurrency_txt()`.
- `current_run_agent_name()`, `current_run_model_name()`.
- `tasks_cummulative_duration()`: Cumulative time across parallel tasks.
- `tasks_cummulative_models(max_width)`: Summary of models used.
- `current_task_model_name()`, `current_task_cost_fmt()`, `current_task_duration_txt()`.
- `current_task_prompt_tokens_fmt()`, `current_task_completion_tokens_fmt()`.
- `current_task_cache_info_fmt()`.
- `memory_fmt()`, `db_memory_fmt()`: System and database metrics.

### Interaction
- `mouse_evt()`, `last_mouse_evt()`, `is_mouse_up_only()`.
- `set_action(Action)`, `trigger_redraw()`, `should_be_pinged()`.

## Lifecycle & Timeouts

Time-based state transitions and expirations are primarily managed in `src/tui/core/app_state/state_processor.rs`.

- **Auto-dismiss (4s)**: `AppStage::Installed` reverts to `Normal` after 4,000,000 microseconds (managed in `process_stage`).
- **Timed Popups**: `PopupMode::Timed(Duration)` expiration (managed in `process_app_state`).
- **Keep-Alive**: `AppState::should_be_pinged()` (in `app_state_base.rs`) determines if the TUI should continue receiving `AppEvent::Tick` events when idle.
- **Timer Engine**: `src/tui/core/ping_timer.rs` provides the debounced 100ms background ticker that drives these transitions.

### Scrolling
- `ScrollIden`: `RunsNav`, `TasksNav`, `TaskContent`, `OverviewContent`.
- `set_scroll_area(ScrollIden, Rect)`, `set_scroll(ScrollIden, u16)`.
- `clamp_scroll(ScrollIden, line_count) -> u16`.

## Interaction & Components

### LinkZones (src/tui/core/types/link_zone.rs)
Used to handle hover/click on specific text regions.
- `set_current_line(usize)`, `inc_current_line_by(usize)`: Manages relative anchoring for multi-line sections.
- `start_group() -> u32`: For multi-line section-wide hover.
- `push_group_zone(...)`: Registers a zone belonging to a group.
- `is_mouse_over(area, scroll, mouse_evt, spans) -> Option<&mut [Span]>`.

### Popup (src/tui/view/popup_view.rs)
- `PopupView`: `{ content: String, mode: PopupMode, is_err: bool }`.
- `PopupMode`: `Timed(Duration)`, `User` (Dismiss with Esc/'x').

### Shared Components (src/tui/view/comp/...)
- `el_running_ico(state)`: Status icon (▶, ✔, ✘, etc.).
- `ui_for_marker_section_str(...)`: Indented section with marker.
- `ui_for_logs_with_hover(...)`, `ui_for_pins_with_hover(...)`, `ui_for_err_with_hover(...)`.

## Styling & Traits

### Traits
- `UiExt`: `x_bg(Color)`, `x_fg(Color)`, `x_width()`.
- `RectExt`: `x_row(u16)`, `x_top_right(w, h)`, `x_move_down(y)`, `x_h_margin(u16)`.

### Common Styles
- `CLR_BKG_BLACK`, `CLR_TXT_WHITE`, `CLR_TXT_BLUE`.
- `CLR_TXT_HOVER_TO_CLIP`: High-visibility teal for hover/copy areas.
- `STL_FIELD_LBL`, `STL_FIELD_VAL`: For key-value headers.
- `STL_SECTION_MARKER`, `STL_SECTION_MARKER_ERR`, `STL_PIN_MARKER`.
- `STL_TAB_DEFAULT`, `STL_TAB_ACTIVE`.
- `STL_NAV_ITEM_HIGHLIGHT`.
