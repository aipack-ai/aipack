use crate::store::rt_model::{RunBmc, TaskBmc};
use crate::tui::AppState;
use crate::tui::core::{Action, MouseEvt, NavDir, RunItemStore};
use crate::tui::support::offset_and_clamp_option_idx_in_len;
use crossterm::event::{KeyCode, MouseEventKind};

pub fn process_app_state(state: &mut AppState) {
	// -- Toggle show sys state
	if let Some(key_event) = state.last_app_event().as_key_event() {
		if key_event.code == KeyCode::Char('M') && key_event.modifiers.contains(crossterm::event::KeyModifiers::SHIFT) {
			state.toggle_show_sys_states();
		}
	}

	// -- Refresh system metrics
	if state.show_sys_states() {
		state.refresh_sys_state();
	}

	// -- Capture the mouse Event
	if let Some(mouse_event) = state.last_app_event().as_mouse_event() {
		let mouse_evt: MouseEvt = mouse_event.into();
		state.core_mut().mouse_evt = Some(mouse_evt);
		// Here we update the persistent mouse
		state.core_mut().last_mouse_evt = Some(mouse_evt);

		// Find the active scroll zone
		let zone_iden = state.core().find_zone_for_pos(mouse_evt);

		// if let Some(zone_iden) = zone_iden {
		// 	tracing::debug!(" {zone_iden:?}");
		// }

		state.core_mut().active_scroll_zone_iden = zone_iden;
	} else {
		state.core_mut().mouse_evt = None;
		// Note: We do not clear the last_mouse_evt as it should remain persistent
	}

	// -- Scroll
	if let Some(mouse_evt) = state.last_app_event().as_mouse_event()
		&& let Some(zone_iden) = state.core().active_scroll_zone_iden
	{
		match mouse_evt.kind {
			MouseEventKind::ScrollUp => {
				state.core_mut().dec_scroll(zone_iden, 1);
			}
			MouseEventKind::ScrollDown => {
				state.core_mut().inc_scroll(zone_iden, 1);
			}
			_ => (),
		};
	}

	// -- Toggle runs list
	if let Some(KeyCode::Char('n')) = state.last_app_event().as_key_code() {
		let show_runs = !state.core().show_runs;
		state.core_mut().show_runs = show_runs;
	}

	// -- Cycle tasks overview mode
	if let Some(KeyCode::Char('t')) = state.last_app_event().as_key_code() {
		state.core_mut().next_overview_tasks_mode();
	}

	// -- Load runs and keep previous idx for later comparison
	let new_runs = RunBmc::list_for_display(state.mm(), None).unwrap_or_default();
	let has_new_runs = new_runs.len() != state.run_items().len();
	let run_item_store = RunItemStore::new(new_runs);
	state.core_mut().run_item_store = run_item_store;

	// only change if we have new runs
	if has_new_runs {
		let prev_run_idx = state.core().run_idx;
		let prev_run_id = state.core().run_id;

		{
			let inner = state.core_mut();

			// When the runs panel is hidden, always pin the latest run (first run index) run.
			if !inner.show_runs {
				inner.set_run_by_idx(0);
			} else {
				// if the prev_run_idx was at 0, then, we keep it at 0
				if prev_run_idx == Some(0) {
					inner.set_run_by_idx(0);
				}
				// otherwise, we preserve the previous id
				else if let Some(prev_run_id) = prev_run_id {
					inner.set_run_by_id(prev_run_id);
				} else {
					inner.set_run_by_idx(0);
				}
			}
		}

		// -- Reset some view state if run selection changed
		// TODO: Need to check if still needed.
		if state.core().run_idx != prev_run_idx {
			let inner = state.core_mut();
			inner.task_idx = None;
		}
	}

	// -- Navigation inside the runs list
	let runs_nav_offset: i32 = if state.core().show_runs
		&& let Some(code) = state.last_app_event().as_key_code()
	{
		match code {
			KeyCode::Char('w') => -1,
			KeyCode::Char('s') => 1,
			_ => 0,
		}
	} else {
		0
	};
	if runs_nav_offset != 0 {
		state.core_mut().offset_run_idx(runs_nav_offset);
	}

	// -- Load tasks for current run
	let current_run_id = state.current_run_item().map(|r| r.id());
	{
		if let Some(run_id) = current_run_id {
			let tasks = TaskBmc::list_for_run(state.mm(), run_id).unwrap_or_default();
			state.core_mut().tasks = tasks;
		} else {
			state.core_mut().tasks.clear(); // Important when no run is selected
		}
	}

	// -- Initialise RunDetailsView if needed
	{
		let need_init = { state.core().task_idx.is_none() };

		if need_init {
			let tasks_empty = state.tasks().is_empty();
			let inner = state.core_mut();
			if !tasks_empty {
				inner.task_idx = Some(0);
			} else {
				inner.task_idx = None;
			}
		}
	}

	// -- Navigation inside the tasks list
	let nav_dir = NavDir::from_up_down_key_code(
		KeyCode::Char('i'),
		KeyCode::Char('k'),
		state.last_app_event().as_key_event(),
	);
	let nav_tasks_offset = nav_dir.map(|n| n.offset()).unwrap_or_default();

	if nav_tasks_offset != 0 {
		let len_tasks = state.tasks().len();
		let inner = state.core();
		let new_task_idx =
			offset_and_clamp_option_idx_in_len(&inner.task_idx, nav_tasks_offset, len_tasks).unwrap_or_default();
		if let Some(task) = state.tasks().get(new_task_idx as usize) {
			state.set_action(Action::GoToTask { task_id: task.id });
			// Note: Little trick to not show the hover when navigating
			state.clear_mouse_evts();
		}
	}

	// -- Tabs navigation (Run view)
	if let Some(code) = state.last_app_event().as_key_code() {
		let current_run_tab = state.run_tab();
		match code {
			KeyCode::Char('j') => state.set_run_tab(current_run_tab.prev()),
			KeyCode::Char('l') => state.set_run_tab(current_run_tab.next()),
			_ => (),
		}
	};

	// -- Arrow key (keyboard & mouse)
	// if let Some(code) = state.last_app_event().as_key_code() {
	// 	let log_scroll = match code {
	// 		KeyCode::Up => state.dec_scroll(iden, dec),
	// 		KeyCode::Down => Some(current_log_scroll.saturating_add(1)),
	// 		KeyCode::Esc => Some(0),
	// 		_ => None,
	// 	};
	// 	if let Some(log_scroll) = log_scroll {
	// 		state.set_log_scroll(log_scroll);
	// 	}
	// }

	// -- Debug color
	let offset: i32 = if let Some(code) = state.last_app_event().as_key_code() {
		match code {
			KeyCode::Char('-') => -1,
			KeyCode::Char('=') => 1,
			_ => 0,
		}
	} else {
		0
	};
	match offset {
		-1 => state.dec_debug_clr(),
		1 => state.inc_debug_clr(),
		_ => (),
	}
}
