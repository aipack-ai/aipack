use crate::model::{ErrBmc, InstallData, RunBmc, TaskBmc, WorkBmc};
use crate::support::time::now_micro;
use crate::tui::AppState;
use crate::tui::core::event::{AppActionEvent, ScrollDir};
use crate::tui::core::{AppStage, MouseEvt, NavDir, RunItemStore, RunTab, ScrollIden, UiAction};
use crate::tui::support::offset_and_clamp_option_idx_in_len;
use crate::tui::view::{PopupMode, PopupView};
use crossterm::event::{KeyCode, MouseEventKind};
use simple_fs::SPath;
use std::time::Duration;

const SCROLL_KEY_MAIN_VIEW: bool = true;

pub fn process_app_state(state: &mut AppState) {
	// -- Process tick
	state.core.time = now_micro();

	// -- Process Stage
	process_stage(state);

	// -- Process actions (clipboard, show-text popup, tab switch)
	process_actions(state);

	// -- Expire timed popups
	if let Some(PopupMode::Timed(duration)) = state.popup().map(|p| &p.mode)
		&& let Some(start) = state.core().popup_start_us
		&& state.core().time.saturating_sub(start) >= duration.as_micros() as i64
	{
		state.clear_popup();
	}

	// -- Dismiss user popups on Esc
	if let Some(key_event) = state.last_app_event().as_key_event()
		&& key_event.code == KeyCode::Esc
		&& let Some(popup) = state.popup()
		&& matches!(popup.mode, PopupMode::User)
	{
		state.clear_popup();
	}

	// -- Toggle show sys state
	if let Some(key_event) = state.last_app_event().as_key_event()
		&& key_event.code == KeyCode::Char('M')
		&& key_event.modifiers.contains(crossterm::event::KeyModifiers::SHIFT)
	{
		state.toggle_show_sys_states();
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

		state.core_mut().active_scroll_zone_iden = zone_iden;
	} else {
		state.core_mut().mouse_evt = None;
		// Note: We do not clear the last_mouse_evt as it should remain persistent
	}

	// -- Scroll
	let mut scroll_dir = None;
	let mut is_key_scroll = false;
	let mut scroll_to_end = false;
	let mut is_page = false;

	if let Some(mouse_evt) = state.last_app_event().as_mouse_event() {
		match mouse_evt.kind {
			MouseEventKind::ScrollUp => scroll_dir = Some(ScrollDir::Up),
			MouseEventKind::ScrollDown => scroll_dir = Some(ScrollDir::Down),
			_ => (),
		}
	} else if let Some(action_evt) = state.last_app_event().as_action_event() {
		match action_evt {
			AppActionEvent::Scroll(dir) => {
				scroll_dir = Some(*dir);
				is_key_scroll = true;
			}
			AppActionEvent::ScrollPage(dir) => {
				scroll_dir = Some(*dir);
				is_key_scroll = true;
				is_page = true;
			}
			AppActionEvent::ScrollToEnd(dir) => {
				scroll_dir = Some(*dir);
				is_key_scroll = true;
				scroll_to_end = true;
			}
			_ => (),
		}
	}

	if let Some(dir) = scroll_dir {
		let mut zone_iden = state.core().active_scroll_zone_iden;

		// If it is a key scroll and we have the SCROLL_KEY_MAIN_VIEW set to true
		// then, we override/fallback to the main view scroll zone.
		if is_key_scroll && SCROLL_KEY_MAIN_VIEW {
			zone_iden = match state.run_tab() {
				RunTab::Overview => Some(ScrollIden::OverviewContent),
				RunTab::Tasks => Some(ScrollIden::TaskContent),
			};
		}

		if let Some(zone_iden) = zone_iden {
			if scroll_to_end {
				match dir {
					ScrollDir::Up => {
						state.set_scroll(zone_iden, 0);
					}
					ScrollDir::Down => {
						// Set to a very large value; the view will clamp it appropriately
						state.set_scroll(zone_iden, u16::MAX);
					}
				}
			} else {
				let amount = if is_page { 5 } else { 1 };
				match dir {
					ScrollDir::Up => {
						state.core_mut().dec_scroll(zone_iden, amount);
					}
					ScrollDir::Down => {
						state.core_mut().inc_scroll(zone_iden, amount);
					}
				}
			}
		}
	}

	// -- Toggle runs list
	if let Some(KeyCode::Char('n')) = state.last_app_event().as_key_code() {
		let show_runs = !state.core().show_runs;
		state.core_mut().show_runs = show_runs;
	}

	// -- Show config popup
	if let Some(KeyCode::Char('c')) = state.last_app_event().as_key_code() {
		state.set_action(UiAction::ShowConfig);
	}

	// -- Config specific keys
	if let AppStage::Config(current_tab) = state.stage() {
		if let Some(key_event) = state.last_app_event().as_key_event() {
			match key_event.code {
				KeyCode::Esc | KeyCode::Char('x') => state.set_action(UiAction::CloseConfig),
				KeyCode::Char('1') => state.set_action(UiAction::SwitchConfigTab(ConfigTab::ApiKeys)),
				KeyCode::Char('2') => state.set_action(UiAction::SwitchConfigTab(ConfigTab::ModelAliases)),
				KeyCode::Char('3') => state.set_action(UiAction::SwitchConfigTab(ConfigTab::Help)),
				KeyCode::Tab => state.set_action(UiAction::SwitchConfigTab(current_tab.next())),
				_ => (),
			}
		}
	}

	// -- Cycle tasks overview mode
	if let Some(KeyCode::Char('t')) = state.last_app_event().as_key_code() {
		state.core_mut().next_overview_tasks_mode();
	}

	// -- Load runs and keep previous idx for later comparison
	let new_runs = RunBmc::list_for_display(state.mm(), None).unwrap_or_default();
	let runs_len = new_runs.len();
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

	// -- Fetch System Error
	// NOTE: For now, we will assume that system errors are before the first run
	// TODO: Eventually, this might not be true, as user could break the config.toml.
	if runs_len == 0 {
		// For now, ignore potential infra erro
		if let Ok(sys_err) = ErrBmc::first_system_err(state.mm()) {
			state.core_mut().sys_err = sys_err
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
			let tasks_len = tasks.len();
			state.core_mut().tasks = tasks;
			// Important to avoid the "no current task" where there is ne.
			// Need to reset task_idx to 0 if current task_idx is > that tasks
			if let Some(current_task_idx) = state.core().task_idx
				&& current_task_idx > tasks_len as i32 - 1
			{
				state.set_task_idx(Some(0));
			}
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
			state.set_action(UiAction::GoToTask { task_id: task.id });
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

	// -- Update running tick
	let is_active = state.run_items().iter().any(|r| r.is_running()) || state.stage() == AppStage::Installing;

	// If active and no running start, then, set the running_start
	if is_active && state.core.running_tick_start.is_none() {
		state.core.running_tick_start = Some(now_micro())
	}
	// Make sure to turn it off if not running
	else if !is_active {
		state.core.running_tick_start = None
	}

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

// region:    --- Action Processing

fn process_stage(state: &mut AppState) {
	let current_stage = state.stage();
	let mm = state.mm();

	// -- Check for active "Install" work
	if let Ok(Some(work)) = WorkBmc::get_active_install(mm) {
		// Determine if it needs confirmation or is actively installing
		let mut needs_confirm = false;
		let mut pack_ref_opt: Option<String> = None;

		if let Ok(Some(install_data)) = work.get_data_as::<InstallData>() {
			needs_confirm = install_data.needs_user_confirm;
			pack_ref_opt = Some(install_data.pack_ref);
		}

		{
			let inner = state.core_mut();
			inner.current_work_id = Some(work.id);
			inner.installing_pack_ref = pack_ref_opt;
		}

		if needs_confirm {
			state.set_stage(AppStage::PromptInstall(work.id));
		} else {
			state.set_stage(AppStage::Installing);
			let inner = state.core_mut();
			inner.installed_start_us = None;
		}
	}
	// -- Handle transition from Installing to Installed or clearing Installed
	else {
		match current_stage {
			AppStage::Installing => {
				// If we were installing and now no active install, check if it was successful
				if let Some(work_id) = state.current_work_id()
					&& let Ok(work) = WorkBmc::get(mm, work_id)
					&& matches!(work.end_state, Some(crate::model::EndState::Ok))
				{
					state.set_stage(AppStage::Installed);
					let now = state.core().time;
					state.core_mut().installed_start_us = Some(now);
				} else {
					state.set_stage(AppStage::Normal);
					state.core_mut().current_work_id = None;
					state.core_mut().installing_pack_ref = None;
				}
			}
			AppStage::Installed => {
				// stay in Installed until user Run or Close
			}
			AppStage::Normal => {
				// stay Normal
				state.core_mut().current_work_id = None;
				state.core_mut().installing_pack_ref = None;
			}
			AppStage::PromptInstall(_id) => {
				state.set_stage(AppStage::Normal);
				state.core_mut().current_work_id = None;
				state.core_mut().installing_pack_ref = None;
			}
		}
	}
}

fn process_actions(state: &mut AppState) {
	if let Some(action) = state.action().cloned() {
		match action {
			UiAction::Quit => {
				state.core_mut().to_send_action = Some(AppActionEvent::Quit);
				state.clear_action();
			}
			UiAction::Redo => {
				state.core_mut().to_send_action = Some(AppActionEvent::Redo);
				state.clear_action();
			}
			UiAction::CancelRun => {
				state.core_mut().to_send_action = Some(AppActionEvent::CancelRun);
				state.clear_action();
			}
			UiAction::ToggleRunsNav => {
				let show_runs = !state.core().show_runs;
				state.core_mut().show_runs = show_runs;
				state.clear_action();
			}
			UiAction::ShowConfig => {
				state.set_stage(AppStage::Config(state.config_tab()));
				state.clear_action();
			}
			UiAction::CloseConfig => {
				state.set_stage(AppStage::Normal);
				state.clear_action();
			}
			UiAction::SwitchConfigTab(tab) => {
				state.set_config_tab(tab);
				state.set_stage(AppStage::Config(tab));
				state.clear_action();
			}
			UiAction::CycleTasksOverviewMode => {
				state.core_mut().next_overview_tasks_mode();
				state.clear_action();
			}
			UiAction::ToClipboardCopy(content) => {
				// Ensure we have a clipboard instance
				let ensure_clipboard: Result<(), String> = if state.core().clipboard.is_some() {
					Ok(())
				} else {
					match arboard::Clipboard::new() {
						Ok(cb) => {
							state.core_mut().clipboard = Some(cb);
							Ok(())
						}
						Err(err) => Err(format!("Clipboard init error: {err}")),
					}
				};

				let mut is_err = false;
				let popup_msg = match ensure_clipboard {
					Ok(()) => {
						if let Some(cb) = state.core_mut().clipboard.as_mut() {
							match cb.set_text(content) {
								Ok(()) => "Copied to clipboard".to_string(),
								Err(err) => {
									is_err = true;
									format!("Clipboard error: {err}")
								}
							}
						} else {
							is_err = true;
							"Clipboard unavailable".to_string()
						}
					}
					Err(msg) => {
						is_err = true;
						msg
					}
				};

				state.set_popup(PopupView {
					content: popup_msg,
					mode: PopupMode::Timed(Duration::from_millis(1000)),
					is_err,
				});
				state.clear_action();
			}
			UiAction::ShowText => {
				state.set_popup(PopupView {
					content: "Click on Content".to_string(),
					mode: PopupMode::Timed(Duration::from_millis(1000)),
					is_err: false,
				});
				state.clear_action();
			}
			UiAction::GoToTask { .. } => {
				// Switch to Tasks tab; keep the action so the view can select and clear it.
				state.set_run_tab(RunTab::Tasks);
			}
			UiAction::OpenFile(path) => {
				let spath = SPath::from(&path);
				match crate::support::editor::open_file_auto(&spath) {
					Ok(editor) => {
						state.set_popup(PopupView {
							content: format!("Opening file\n{path}\n(with {})", editor.program()),
							mode: PopupMode::Timed(Duration::from_millis(2000)),
							is_err: false,
						});
					}
					Err(err) => {
						state.set_popup(PopupView {
							content: format!("Failed to open file\n{path}\n(Cause: {err})"),
							mode: PopupMode::Timed(Duration::from_millis(3000)),
							is_err: true,
						});
					}
				}
				state.clear_action();
			}
			UiAction::WorkConfirm(id) => {
				state.core_mut().to_send_action = Some(AppActionEvent::WorkConfirm(id));
				state.trigger_redraw();
				state.clear_action();
			}
			UiAction::WorkCancel(id) => {
				state.core_mut().to_send_action = Some(AppActionEvent::WorkCancel(id));
				state.trigger_redraw();
				state.clear_action();
			}
			UiAction::WorkRun(id) => {
				let mm = state.mm().clone();
				if let Ok(work) = WorkBmc::get(&mm, id)
					&& let Ok(Some(install_data)) = work.get_data_as::<InstallData>()
					&& let Some(run_args_val) = install_data.run_args
					&& let Ok(run_args) = serde_json::from_value::<crate::exec::cli::RunArgs>(run_args_val)
				{
					state.core_mut().to_send_action = Some(AppActionEvent::Run(run_args));
				}
				state.set_stage(AppStage::Normal);
				state.trigger_redraw();
				state.clear_action();
			}
			UiAction::WorkClose(_id) => {
				state.set_stage(AppStage::Normal);
				state.trigger_redraw();
				state.clear_action();
			}
		}
	}
}

// endregion: --- Action Processing
