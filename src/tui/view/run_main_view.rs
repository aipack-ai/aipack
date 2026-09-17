use crate::tui::core::RunTab;
use crate::tui::view::support::RectExt as _;
use crate::tui::view::{GroupDashView, RunOverviewView, RunTasksView, comp};
use crate::tui::{AppState, style};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Stylize as _;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, StatefulWidget, Widget as _};

pub struct RunMainView;

impl RunMainView {
	pub fn clear_scroll_idens(state: &mut AppState) {
		RunTasksView::clear_scroll_idens(state);
		RunOverviewView::clear_scroll_idens(state);
		// The run main content pane is swapped with the group dashboard in the same region,
		// so clear the group dashboard zone here as well (both MainView branches call this).
		GroupDashView::clear_scroll_idens(state);
	}
}

impl StatefulWidget for RunMainView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		Block::new().bg(style::CLR_BKG_GRAY_DARKER).render(area, buf);

		// -- Layout Header | Tabs | Tab Content
		let [header_a, _space_1, tabs_a, tabs_line, tab_content_a] = Layout::default()
			.direction(Direction::Vertical)
			.constraints(vec![
				Constraint::Length(2), // header
				Constraint::Max(1),    // space_1
				Constraint::Length(1), // tabs
				Constraint::Max(1),    // tab_line
				Constraint::Fill(1),   // tab_content
			])
			.areas(area);

		// -- render header
		render_header(header_a, buf, state);

		// -- Render tabs with line
		let selected_tab = render_tabs(tabs_a, tabs_line, buf, state);

		// -- Render the selected tab
		match selected_tab {
			RunTab::Overview => {
				RunTasksView::clear_scroll_idens(state);
				RunOverviewView.render(tab_content_a, buf, state);
			}
			RunTab::Tasks => {
				RunOverviewView::clear_scroll_idens(state);
				RunTasksView.render(tab_content_a, buf, state);
			}
		}
	}
}

fn render_header(area: Rect, buf: &mut Buffer, state: &mut AppState) {
	const VAL_1_WIDTH: u16 = 25; // for the agent and model column

	// -- Prepare Data
	let agent_name = state.current_run_agent_name();
	let model_name = state.tasks_cummulative_models(VAL_1_WIDTH as usize);
	let cost_txt = state.current_run_cost_fmt();
	let concurrency_txt = state.current_run_concurrency_txt();

	// Tasks progress and optional cumulative duration.

	let (items_label, items_txt) = if let Some(run_item) = state.current_run_item()
		&& run_item.has_children()
	{
		let children_runs = state.all_run_children(run_item);
		let total_items = children_runs.len();
		// let done_items = children_runs.into_iter().filter(|&ri| ri.run().is_done()).count();
		let items_txt = if total_items == 0 {
			"-".to_string()
		} else if run_item.run().is_done() {
			format!("✔ {total_items}")
		} else {
			format!("▶ {total_items}")
		};

		("Agents:", items_txt)
	} else {
		let total_items = state.tasks().len();
		let done_items = state.tasks().iter().filter(|t| t.is_ended()).count();
		let items_txt = if total_items == 0 {
			"-".to_string()
		} else {
			format!("{done_items}/{total_items}")
		};
		("Tasks:", items_txt)
	};

	let mut duration_txt = state.current_run_duration_txt();
	if let Some(cumul_txt) = state.tasks_cummulative_duration() {
		duration_txt = format!("{duration_txt} ({cumul_txt})");
	}

	// -- Layout Helpers
	let [lbl_1, val_1, lbl_2, val_2, lbl_3, val_3] = Layout::default()
		.direction(Direction::Horizontal)
		.constraints(vec![
			Constraint::Length(11),          // Agent/Model
			Constraint::Length(VAL_1_WIDTH), //
			Constraint::Length(8),           // Tasks/Cost
			Constraint::Length(9),           //
			Constraint::Length(13),          // Concurrency/Duration
			Constraint::Fill(1),             //
		])
		.spacing(1)
		.areas(area);

	// -- Render Row 1
	// Agent label with marker
	let mut line_1 = Line::default();
	// Note: Make sure this run_item is the one running, otherwise, running_tick_flag is global
	if let Some(run_item) = state.current_run_item() {
		let is_running = run_item.is_running();
		if !is_running || matches!(state.running_tick_flag(), Some(true) | None) {
			line_1.push_span(comp::el_running_ico_with_flow(run_item, run_item.run().flow_redo_count));
		}
	}

	line_1.push_span(" Agent:");
	Paragraph::new(line_1)
		.style(style::STL_FIELD_LBL)
		.right_aligned()
		.render(lbl_1.x_row(1), buf);
	// Agent value
	Paragraph::new(agent_name)
		.style(style::STL_FIELD_VAL)
		.render(val_1.x_row(1), buf);

	Paragraph::new(items_label)
		.style(style::STL_FIELD_LBL)
		.right_aligned()
		.render(lbl_2.x_row(1), buf);
	Paragraph::new(items_txt)
		.style(style::STL_FIELD_VAL)
		.render(val_2.x_row(1), buf);

	Paragraph::new("Concurrency:")
		.style(style::STL_FIELD_LBL)
		.right_aligned()
		.render(lbl_3.x_row(1), buf);
	Paragraph::new(concurrency_txt)
		.style(style::STL_FIELD_VAL)
		.render(val_3.x_row(1), buf);

	// -- Render Row 2
	Paragraph::new("Model:")
		.style(style::STL_FIELD_LBL)
		.right_aligned()
		.render(lbl_1.x_row(2), buf);
	Paragraph::new(model_name)
		.style(style::STL_FIELD_VAL)
		.render(val_1.x_row(2), buf);

	Paragraph::new("Cost:")
		.style(style::STL_FIELD_LBL)
		.right_aligned()
		.render(lbl_2.x_row(2), buf);
	Paragraph::new(cost_txt).style(style::STL_FIELD_VAL).render(val_2.x_row(2), buf);

	Paragraph::new("Duration:")
		.style(style::STL_FIELD_LBL)
		.right_aligned()
		.render(lbl_3.x_row(2), buf);
	Paragraph::new(duration_txt)
		.style(style::STL_FIELD_VAL)
		.render(val_3.x_row(2), buf);
}

fn render_tabs(tabs_a: Rect, tabs_line_a: Rect, buf: &mut Buffer, state: &mut AppState) -> RunTab {
	// -- Layout Header | Tabs | Tab Content
	let [_, tab_overview_a, _, tab_tasks_a] = Layout::default()
		.direction(Direction::Horizontal)
		.constraints(vec![
			Constraint::Length(1),  // gap 1
			Constraint::Length(12), // tab_overview_a
			Constraint::Length(1),  // gap
			Constraint::Length(11), // tab_tasks_a
		])
		.areas(tabs_a);

	// -- Process UI Event for the tab
	// NOTE: There would be an argument to say that this could be in the process_app_state(..)
	//       But then, it will requires to have perhaps too much inner knowledge
	process_for_run_tab_state(state, tab_overview_a, tab_tasks_a);

	let run_tab = state.run_tab();

	// -- Render Overview Tab
	let tab_1_label = "Overview";
	let tab_1_style = match (run_tab == RunTab::Overview, state.is_last_mouse_over(tab_overview_a)) {
		// (active, hover)
		(true, true) => style::STL_TAB_ACTIVE_HOVER,
		(true, false) => style::STL_TAB_ACTIVE,
		(false, true) => style::STL_TAB_DEFAULT_HOVER,
		(false, false) => style::STL_TAB_DEFAULT,
	};

	Paragraph::new(tab_1_label)
		.centered()
		.style(tab_1_style)
		.render(tab_overview_a, buf);

	// -- Render Task (only if at least 1)
	if !state.tasks().is_empty() {
		let tab_2_label = if state.tasks().len() > 1 { "Tasks" } else { "Task" };
		let tab_2_style = match (run_tab == RunTab::Tasks, state.is_last_mouse_over(tab_tasks_a)) {
			// (active, hover)
			(true, true) => style::STL_TAB_ACTIVE_HOVER,
			(true, false) => style::STL_TAB_ACTIVE,
			(false, true) => style::STL_TAB_DEFAULT_HOVER,
			(false, false) => style::STL_TAB_DEFAULT,
		};
		Paragraph::new(tab_2_label)
			.centered()
			.style(tab_2_style)
			.render(tab_tasks_a, buf);
	}

	// -- Render Line
	// Trick to have a single line of tab active bkg color
	let repeated = "▔".repeat(tabs_line_a.width as usize);
	let line = Line::default().spans(vec![Span::raw(repeated)]).fg(style::CLR_BKG_TAB_ACT);
	line.render(tabs_line_a, buf);

	// -- Return tab selected
	run_tab
}

// region:    --- UI Event Processing

fn process_for_run_tab_state(state: &mut AppState, overview_a: Rect, tasks_a: Rect) {
	// -- Set the tab to Overview if not tasks
	// NOTE: here we are conservative.
	if let Some(false) = state.current_run_has_task_stages()
		&& state.tasks().is_empty()
	{
		state.set_run_tab(RunTab::Overview);
		return;
	} else if state.current_run_has_skip() && state.tasks().is_empty() {
		state.set_run_tab(RunTab::Overview);
		return;
	}

	// -- Otherwise process the mouse
	if let Some(mouse_evt) = state.mouse_evt()
		&& mouse_evt.is_up()
	{
		if mouse_evt.is_over(overview_a) {
			state.set_run_tab(RunTab::Overview);
			state.clear_mouse_evts(true);
		} else if mouse_evt.is_over(tasks_a) {
			state.set_run_tab(RunTab::Tasks);
			state.clear_mouse_evts(true);
		}
	}
}

// endregion: --- UI Event Processing
