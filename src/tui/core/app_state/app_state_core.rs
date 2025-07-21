use crate::store::rt_model::{Run, Task};
use crate::store::{Id, ModelManager};
use crate::tui::core::event::LastAppEvent;
use crate::tui::core::sys_state::SysState;
use crate::tui::core::{MouseEvt, RunTab, ScrollIden, ScrollZone, ScrollZones};
use crate::tui::support::offset_and_clamp_option_idx_in_len;
use ratatui::layout::Position;

/// Inner representation of the application state.
///
/// `pub` fields are fine here because the whole struct is only visible
/// inside `crate::tui::core`.
pub(in crate::tui::core) struct AppStateCore {
	// -- Debug
	// debug color idx
	pub debug_clr: u8,

	// -- Mouse
	/// Hold the current app mouse event wrapper (None if last event was not a mouse event)
	pub mouse_evt: Option<MouseEvt>,
	/// This keep the current/last mouse event
	pub last_mouse_evt: Option<MouseEvt>,

	// -- Scroll Zones
	pub scroll_zones: ScrollZones,
	pub active_scroll_zone_iden: Option<ScrollIden>,

	// -- Main View
	pub show_runs: bool,

	// -- RunsView
	pub run_idx: Option<i32>,
	pub run_id: Option<Id>,

	// -- RunMainView
	pub run_tab: RunTab,

	// -- RunTasksView
	pub task_idx: Option<i32>,
	pub before_all_show: bool,
	pub after_all_show: bool,

	// -- Data
	pub runs: Vec<Run>,
	pub tasks: Vec<Task>,

	// -- System & Event
	pub mm: ModelManager,
	pub do_redraw: bool,
	pub last_app_event: LastAppEvent,

	// -- SysState
	pub sys_state: SysState,
	pub memory: u64,
	pub cpu: f64,
}

impl AppStateCore {
	pub fn set_run_by_idx(&mut self, idx: i32) {
		self.run_idx = Some(idx);
		self.run_id = self.runs.get(idx as usize).map(|r| r.id);
	}

	pub fn set_run_by_id(&mut self, run_id: Id) {
		let run_idx = self.runs.iter().position(|r| r.id == run_id);
		self.run_idx = run_idx.map(|v| v as i32);
		// For now, we set it a None if not found (need to revise strategy, can syncup with the by_idx)
		self.run_id = run_idx.map(|_| run_id);
	}

	pub fn offset_run_idx(&mut self, offset: i32) {
		let runs_len = self.runs.len();
		let new_idx = offset_and_clamp_option_idx_in_len(&self.run_idx, offset, runs_len);
		if let Some(new_idx) = new_idx {
			self.set_run_by_idx(new_idx);
		}
	}
}

/// Scroll Inner impl
impl AppStateCore {
	pub fn find_zone_for_pos(&self, position: impl Into<Position>) -> Option<ScrollIden> {
		self.scroll_zones.find_zone_for_pos(position)
	}
	#[allow(unused)]
	pub fn get_zone(&self, iden: &ScrollIden) -> Option<&ScrollZone> {
		self.scroll_zones.zones.get(iden)
	}

	pub fn get_zone_mut(&mut self, iden: &ScrollIden) -> Option<&mut ScrollZone> {
		self.scroll_zones.zones.get_mut(iden)
	}

	pub fn get_scroll(&self, iden: ScrollIden) -> u16 {
		self.scroll_zones.zones.get(&iden).and_then(|z| z.scroll()).unwrap_or_default()
	}

	/// return the new value
	pub fn inc_scroll(&mut self, iden: ScrollIden, inc: u16) -> u16 {
		let val = self.get_scroll(iden);
		let val = val.saturating_add(inc);
		if let Some(z) = self.get_zone_mut(&iden) {
			z.set_scroll(val);
		}

		val
	}

	pub fn dec_scroll(&mut self, iden: ScrollIden, dec: u16) -> u16 {
		let val = self.get_scroll(iden);
		let val = val.saturating_sub(dec);
		if let Some(z) = self.get_zone_mut(&iden) {
			z.set_scroll(val);
		}

		val
	}
}
