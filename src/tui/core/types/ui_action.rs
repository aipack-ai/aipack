use crate::model::Id;
use crate::tui::core::ConfigTab;

/// Represents a **UI Intent** stored in `AppState`.
/// It is stateful and represents a request that might need further context
/// (for example, "Copy this task's output") before being executed.
///
/// See `dev/spec-code/spec-code-tui.md` for the architectural rationale and flow.
#[derive(Debug, Clone)]
pub enum UiAction {
	// -- Global Actions
	Quit,
	Redo,
	CancelRun,
	ToggleRunsNav,
	CycleTasksOverviewMode,

	// Configuration
	ShowConfig,
	CloseConfig,
	SwitchConfigTab(ConfigTab),

	// Go to the tasks tab and select this task_id
	GoToTask {
		task_id: Id,
	},

	// Confirm a pending work (e.g., installation)
	WorkConfirm(Id),
	// Cancel a pending work
	WorkCancel(Id),
	// Run the command associated with a work (e.g., after installation)
	WorkRun(Id),
	// Close/Dismiss the work view
	WorkClose(Id),

	#[allow(unused)]
	ShowText,

	// Copy the provided text into the clipboard
	ToClipboardCopy(String),

	// Open the file at the given path
	OpenFile(String),
}
