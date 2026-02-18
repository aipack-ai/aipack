use super::{AppStateCore, SysState};
use crate::Result;
use crate::model::{Id, ModelManager, Task};
use crate::support::time::now_micro;
use crate::tui::core::event::{AppActionEvent, LastAppEvent};
use crate::tui::core::{AppStage, ConfigTab, OverviewTasksMode, RunItemStore, RunTab, ScrollZones};
use crate::tui::view::PopupView;

/// Public wrapper around AppStateCore.
///
/// Visible only to the `tui` module so it does not leak to the whole crate.
pub struct AppState {
	pub(in crate::tui::core) core: AppStateCore,
}

/// Constructors
impl AppState {
	pub fn new(mm: ModelManager, last_app_event: LastAppEvent) -> Result<Self> {
		let sys_state = SysState::new()?;

		let inner = AppStateCore {
			// -- Debug
			debug_clr: 0,

			// -- Mouse
			mouse_evt: None,
			last_mouse_evt: None,

			// -- ScrollZones
			scroll_zones: ScrollZones::default(),
			active_scroll_zone_iden: None,

			// -- MainView
			show_runs: true,

			// -- RunsView
			run_idx: None,
			run_id: None,

			running_tick_start: None,

			// -- RunMainView
			run_tab: RunTab::Tasks, // Tasks tab by default

			// -- RunOverview
			overview_tasks_mode: OverviewTasksMode::Auto,

			// -- RunTasksView
			task_idx: None,

			// -- Data
			run_item_store: RunItemStore::default(),
			tasks: Vec::new(),

			// -- Stage & Work
			stage: AppStage::Normal,
			config_tab: ConfigTab::ApiKeys,
			installing_pack_ref: None,
			current_work_id: None,

			// -- System & Event
			mm,
			last_app_event,

			// -- Action
			do_redraw: false,
			do_action: None,
			to_send_action: None,

			// -- SysState
			time: now_micro(), // the current time
			sys_err: None,
			show_sys_states: false,
			sys_state,
			memory: 0,
			cpu: 0.,

			// -- Clipboard
			clipboard: None,

			// -- Popup
			popup: None,
			popup_start_us: None,

			installed_start_us: None,
		};

		Ok(Self { core: inner })
	}

	// -- Inner accessors

	/// Immutable access to the inner state (core-exclusive).
	pub(in crate::tui::core) fn core(&self) -> &AppStateCore {
		&self.core
	}

	/// Mutable access to the inner state (core-exclusive).
	pub(in crate::tui::core) fn core_mut(&mut self) -> &mut AppStateCore {
		&mut self.core
	}
}

/// Debug
impl AppState {
	pub fn debug_clr(&self) -> u8 {
		self.core.debug_clr
	}

	pub(in crate::tui::core) fn inc_debug_clr(&mut self) {
		self.core.debug_clr = self.core.debug_clr.wrapping_add(1);
	}

	pub(in crate::tui::core) fn dec_debug_clr(&mut self) {
		self.core.debug_clr = self.core.debug_clr.wrapping_sub(1);
	}
}

/// MainView
impl AppState {
	pub fn stage(&self) -> AppStage {
		self.core.stage
	}

	pub fn set_stage(&mut self, stage: AppStage) {
		self.core.stage = stage;
	}

	pub fn config_tab(&self) -> ConfigTab {
		self.core.config_tab
	}

	pub fn set_config_tab(&mut self, config_tab: ConfigTab) {
		self.core.config_tab = config_tab;
	}

	#[allow(unused)]
	pub fn is_config_stage(&self) -> bool {
		matches!(self.stage(), AppStage::Config(_))
	}

	pub fn installing_pack_ref(&self) -> Option<&str> {
		self.core.installing_pack_ref.as_deref()
	}

	pub fn current_work_id(&self) -> Option<Id> {
		self.core.current_work_id
	}

	pub fn show_runs(&self) -> bool {
		self.core.show_runs
	}
}

/// OverviewView
impl AppState {
	pub fn overview_tasks_mode(&self) -> OverviewTasksMode {
		self.core.overview_tasks_mode
	}
}

/// RunTasksView
impl AppState {
	pub fn task_idx(&self) -> Option<usize> {
		self.core.task_idx.map(|idx| idx as usize)
	}

	pub fn set_task_idx(&mut self, idx: Option<usize>) {
		self.core.task_idx = idx.map(|i| i as i32);
	}

	pub fn tasks(&self) -> &[Task] {
		&self.core.tasks
	}

	pub fn current_task(&self) -> Option<&Task> {
		if let Some(idx) = self.core.task_idx {
			self.core.tasks.get(idx as usize)
		} else {
			None
		}
	}
}

/// System & Event
impl AppState {
	pub fn mm(&self) -> &ModelManager {
		&self.core.mm
	}

	pub fn last_app_event(&self) -> &LastAppEvent {
		&self.core.last_app_event
	}
}

/// Others
impl AppState {
	pub fn take_action_event_to_send(&mut self) -> Option<AppActionEvent> {
		self.core.to_send_action.take()
	}

	pub fn should_redraw(&self) -> bool {
		self.core.do_redraw
	}

	pub fn trigger_redraw(&mut self) {
		self.core.do_redraw = true;
	}

	pub fn should_be_pinged(&self) -> bool {
		self.running_tick_count().is_some()
			|| self.popup().is_some_and(|p| p.is_timed())
			|| matches!(self.stage(), AppStage::Installing | AppStage::Installed)
	}
}

/// Popup
impl AppState {
	pub fn popup(&self) -> Option<&PopupView> {
		self.core.popup.as_ref()
	}

	pub fn set_popup(&mut self, popup: PopupView) {
		self.core.popup_start_us = Some(self.core.time);
		self.core.popup = Some(popup);
	}

	pub fn clear_popup(&mut self) {
		self.core.popup = None;
		self.core.popup_start_us = None;
	}
}
