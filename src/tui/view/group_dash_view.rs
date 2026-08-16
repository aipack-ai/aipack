use crate::tui::core::{GroupDashTab, ScrollIden};
use crate::tui::support::{ui_fmt_cost, ui_fmt_duration_us};
use crate::tui::view::comp;
use crate::tui::view::support::RectExt as _;
use crate::tui::{AppState, style};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Stylize as _;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, StatefulWidget, Widget as _};

pub struct GroupDashView;

impl GroupDashView {
	pub const SCROLL_IDEN: ScrollIden = ScrollIden::GroupDashContent;
}

impl StatefulWidget for GroupDashView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		Block::new().bg(style::CLR_BKG_GRAY_DARKER).render(area, buf);

		let Some(data) = state.group_dash_data().cloned() else {
			let empty_msg = Paragraph::new("No group dashboard data available")
				.style(style::STL_FIELD_VAL)
				.centered();
			empty_msg.render(area, buf);
			return;
		};

		// -- Layout: Header | Space | Tabs | Tabs Line | Tab Content
		let [header_a, _space_1, tabs_a, tabs_line_a, tab_content_a] = Layout::default()
			.direction(Direction::Vertical)
			.constraints(vec![
				Constraint::Length(2), // Summary Header
				Constraint::Max(1),    // Space 1
				Constraint::Length(1), // Tabs
				Constraint::Max(1),    // Tab Line
				Constraint::Fill(1),   // Tab Content
			])
			.areas(area);

		// -- Render Header Summary
		render_header(header_a, buf, &data);

		// -- Render Tabs Bar
		let selected_tab = render_tabs(tabs_a, tabs_line_a, buf, state);

		// -- Render Tab Content
		match selected_tab {
			GroupDashTab::TopRuns => {
				render_top_runs_view(tab_content_a, buf, &data, state);
			}
			GroupDashTab::Agents => {
				render_agents_view(tab_content_a, buf, &data, state);
			}
			GroupDashTab::Models => {
				render_models_view(tab_content_a, buf, &data, state);
			}
		}
	}
}

fn render_tabs(tabs_a: Rect, tabs_line_a: Rect, buf: &mut Buffer, state: &mut AppState) -> GroupDashTab {
	let [_, tab_top_runs_a, _, tab_agents_a, _, tab_models_a] = Layout::default()
		.direction(Direction::Horizontal)
		.constraints(vec![
			Constraint::Length(1),  // Gap 1
			Constraint::Length(12), // tab_top_runs_a
			Constraint::Length(1),  // Gap
			Constraint::Length(10), // tab_agents_a
			Constraint::Length(1),  // Gap
			Constraint::Length(10), // tab_models_a
		])
		.areas(tabs_a);

	// -- Process UI Event for the tab
	process_for_group_dash_tab_state(state, tab_top_runs_a, tab_agents_a, tab_models_a);

	let current_tab = state.group_dash_tab();

	// Render Top Runs Tab
	let tab_1_style = match (current_tab == GroupDashTab::TopRuns, state.is_last_mouse_over(tab_top_runs_a)) {
		(true, true) => style::STL_TAB_ACTIVE_HOVER,
		(true, false) => style::STL_TAB_ACTIVE,
		(false, true) => style::STL_TAB_DEFAULT_HOVER,
		(false, false) => style::STL_TAB_DEFAULT,
	};
	Paragraph::new("Top Runs")
		.centered()
		.style(tab_1_style)
		.render(tab_top_runs_a, buf);

	// Render Agents Tab
	let tab_2_style = match (current_tab == GroupDashTab::Agents, state.is_last_mouse_over(tab_agents_a)) {
		(true, true) => style::STL_TAB_ACTIVE_HOVER,
		(true, false) => style::STL_TAB_ACTIVE,
		(false, true) => style::STL_TAB_DEFAULT_HOVER,
		(false, false) => style::STL_TAB_DEFAULT,
	};
	Paragraph::new("Agents")
		.centered()
		.style(tab_2_style)
		.render(tab_agents_a, buf);

	// Render Models Tab
	let tab_3_style = match (current_tab == GroupDashTab::Models, state.is_last_mouse_over(tab_models_a)) {
		(true, true) => style::STL_TAB_ACTIVE_HOVER,
		(true, false) => style::STL_TAB_ACTIVE,
		(false, true) => style::STL_TAB_DEFAULT_HOVER,
		(false, false) => style::STL_TAB_DEFAULT,
	};
	Paragraph::new("Models")
		.centered()
		.style(tab_3_style)
		.render(tab_models_a, buf);

	// -- Render Line
	let repeated = "▔".repeat(tabs_line_a.width as usize);
	let line = Line::default().spans(vec![Span::raw(repeated)]).fg(style::CLR_BKG_TAB_ACT);
	line.render(tabs_line_a, buf);

	current_tab
}

fn process_for_group_dash_tab_state(state: &mut AppState, top_runs_a: Rect, agents_a: Rect, models_a: Rect) {
	if let Some(mouse_evt) = state.mouse_evt()
		&& mouse_evt.is_up()
	{
		if mouse_evt.is_over(top_runs_a) {
			state.set_group_dash_tab(GroupDashTab::TopRuns);
			state.clear_mouse_evts(true);
		} else if mouse_evt.is_over(agents_a) {
			state.set_group_dash_tab(GroupDashTab::Agents);
			state.clear_mouse_evts(true);
		} else if mouse_evt.is_over(models_a) {
			state.set_group_dash_tab(GroupDashTab::Models);
			state.clear_mouse_evts(true);
		}
	}
}

fn render_header(area: Rect, buf: &mut Buffer, data: &crate::tui::core::GroupDashData) {
	let [lbl_cost, val_cost, lbl_top_runs, val_top_runs, lbl_all_runs, val_all_runs] = Layout::default()
		.direction(Direction::Horizontal)
		.constraints(vec![
			Constraint::Length(13), // "Total Cost:"
			Constraint::Length(12), // "$X.XX"
			Constraint::Length(11), // "Top Runs:"
			Constraint::Length(8),  // "N"
			Constraint::Length(11), // "All Runs:"
			Constraint::Fill(1),    // "M"
		])
		.spacing(1)
		.areas(area);

	// Row 1: Summary KPIs
	Paragraph::new("Total Cost:")
		.style(style::STL_FIELD_LBL)
		.right_aligned()
		.render(lbl_cost.x_row(1), buf);
	Paragraph::new(ui_fmt_cost(Some(data.total_cost)))
		.style(style::STL_FIELD_VAL)
		.render(val_cost.x_row(1), buf);

	Paragraph::new("Top Runs:")
		.style(style::STL_FIELD_LBL)
		.right_aligned()
		.render(lbl_top_runs.x_row(1), buf);
	Paragraph::new(data.top_runs_count.to_string())
		.style(style::STL_FIELD_VAL)
		.render(val_top_runs.x_row(1), buf);

	Paragraph::new("All Runs:")
		.style(style::STL_FIELD_LBL)
		.right_aligned()
		.render(lbl_all_runs.x_row(1), buf);
	Paragraph::new(data.all_runs_count.to_string())
		.style(style::STL_FIELD_VAL)
		.render(val_all_runs.x_row(1), buf);
}

fn render_top_runs_view(area: Rect, buf: &mut Buffer, data: &crate::tui::core::GroupDashData, state: &mut AppState) {
	if data.top_runs.is_empty() {
		let msg = Paragraph::new("No top runs in this group").style(style::STL_FIELD_LBL);
		msg.render(area.x_h_margin(1), buf);
		return;
	}

	const SCROLL_IDEN: ScrollIden = GroupDashView::SCROLL_IDEN;
	state.set_scroll_area(SCROLL_IDEN, area);

	let items_len = data.top_runs.len();
	let scroll = state.clamp_scroll(SCROLL_IDEN, items_len);
	let visible_height = area.height.saturating_sub(1) as usize;
	let start_idx = scroll as usize;
	let end_idx = (start_idx + visible_height).min(items_len);

	let header_label = "Run / Label";
	let max_name_len = data
		.top_runs
		.iter()
		.map(|e| e.label.chars().count())
		.max()
		.unwrap_or(0);
	let first_col_w = (header_label.len().max(max_name_len) + 3) as u16;

	// Column layout definition
	let cols_layout = Layout::default()
		.direction(Direction::Horizontal)
		.constraints([
			Constraint::Length(first_col_w), // Run / Label
			Constraint::Length(10),          // Sub-runs
			Constraint::Length(12),          // Total Cost
			Constraint::Length(14),          // Total Duration
			Constraint::Length(12),          // Top Run Cost
			Constraint::Length(14),          // Top Duration
		])
		.spacing(1);

	// Render Table Header (Row 1)
	let header_area = area.x_row(1).x_h_margin(1);
	let [h_label, h_subruns, h_tot_cost, h_tot_dur, h_top_cost, h_top_dur] = cols_layout.areas(header_area);

	Paragraph::new("Run / Label").style(style::STL_FIELD_LBL).render(h_label, buf);
	Paragraph::new("Sub-runs").style(style::STL_FIELD_LBL).right_aligned().render(h_subruns, buf);
	Paragraph::new("Total Cost").style(style::STL_FIELD_LBL).right_aligned().render(h_tot_cost, buf);
	Paragraph::new("Total Dur").style(style::STL_FIELD_LBL).right_aligned().render(h_tot_dur, buf);
	Paragraph::new("Top Cost").style(style::STL_FIELD_LBL).right_aligned().render(h_top_cost, buf);
	Paragraph::new("Top Dur").style(style::STL_FIELD_LBL).right_aligned().render(h_top_dur, buf);

	// Process mouse click on visible row
	let mut clicked_run_id = None;
	if let Some(mouse_evt) = state.mouse_evt()
		&& mouse_evt.is_up()
		&& mouse_evt.is_over(area)
		&& mouse_evt.y() > area.y
	{
		let clicked_row = (mouse_evt.y() - area.y - 1) as usize;
		let clicked_idx = start_idx + clicked_row;
		if let Some(entry) = data.top_runs.get(clicked_idx) {
			clicked_run_id = Some(entry.run_id);
		}
	}

	// Render visible data rows
	for (visible_row_idx, item_idx) in (start_idx..end_idx).enumerate() {
		let entry = &data.top_runs[item_idx];
		let row_y = 2 + visible_row_idx as u16;
		let row_area = area.x_row(row_y).x_h_margin(1);
		let is_hovered = state.is_last_mouse_over(row_area);

		if is_hovered {
			Block::new().bg(style::CLR_BKG_400).render(row_area, buf);
		}

		let [r_label, r_subruns, r_tot_cost, r_tot_dur, r_top_cost, r_top_dur] = cols_layout.areas(row_area);

		let label_style = if is_hovered {
			style::STL_TXT_ACT.fg(style::CLR_TXT_HOVER)
		} else {
			style::STL_TXT_ACT
		};
		let val_style = if is_hovered {
			style::STL_FIELD_VAL.fg(style::CLR_TXT_400)
		} else {
			style::STL_FIELD_VAL
		};

		Paragraph::new(entry.label.clone()).style(label_style).render(r_label, buf);

		let subruns_str = if entry.child_count > 0 {
			entry.child_count.to_string()
		} else {
			"-".to_string()
		};
		Paragraph::new(subruns_str).style(val_style).right_aligned().render(r_subruns, buf);

		Paragraph::new(ui_fmt_cost(Some(entry.cost)))
			.style(val_style)
			.right_aligned()
			.render(r_tot_cost, buf);
		Paragraph::new(ui_fmt_duration_us(entry.total_duration_us))
			.style(val_style)
			.right_aligned()
			.render(r_tot_dur, buf);
		Paragraph::new(ui_fmt_cost(Some(entry.top_cost)))
			.style(val_style)
			.right_aligned()
			.render(r_top_cost, buf);
		Paragraph::new(ui_fmt_duration_us(entry.top_duration_us))
			.style(val_style)
			.right_aligned()
			.render(r_top_dur, buf);
	}

	// Execute click action after rendering
	if let Some(run_id) = clicked_run_id {
		state.set_run_id(run_id);
		state.clear_mouse_evts(true);
	}

	// Render scroll indicator icons
	let item_count = items_len as u16;
	if item_count.saturating_sub(scroll) > visible_height as u16 {
		let bottom_ico = area.x_bottom_right(1, 1);
		comp::ico_scroll_down().render(bottom_ico, buf);
	}
	if scroll > 0 {
		let top_ico = area.x_top_right(1, 1);
		comp::ico_scroll_up().render(top_ico, buf);
	}
}

fn render_agents_view(area: Rect, buf: &mut Buffer, data: &crate::tui::core::GroupDashData, state: &mut AppState) {
	if data.agents.is_empty() {
		let msg = Paragraph::new("No agents in this group").style(style::STL_FIELD_LBL);
		msg.render(area.x_h_margin(1), buf);
		return;
	}

	const SCROLL_IDEN: ScrollIden = GroupDashView::SCROLL_IDEN;
	state.set_scroll_area(SCROLL_IDEN, area);

	let items_len = data.agents.len();
	let scroll = state.clamp_scroll(SCROLL_IDEN, items_len);
	let visible_height = area.height.saturating_sub(1) as usize;
	let start_idx = scroll as usize;
	let end_idx = (start_idx + visible_height).min(items_len);

	let header_label = "Agent";
	let max_name_len = data
		.agents
		.iter()
		.map(|e| e.name.chars().count())
		.max()
		.unwrap_or(0);
	let first_col_w = (header_label.len().max(max_name_len) + 3) as u16;

	// Column layout definition
	let cols_layout = Layout::default()
		.direction(Direction::Horizontal)
		.constraints([
			Constraint::Length(first_col_w), // Agent
			Constraint::Length(10),          // Runs
			Constraint::Length(12),          // Total Cost
			Constraint::Length(12),          // Avg Cost
			Constraint::Length(14),          // Total Duration
		])
		.spacing(1);

	// Render Table Header (Row 1)
	let header_area = area.x_row(1).x_h_margin(1);
	let [h_agent, h_runs, h_tot_cost, h_avg_cost, h_tot_dur] = cols_layout.areas(header_area);

	Paragraph::new("Agent").style(style::STL_FIELD_LBL).render(h_agent, buf);
	Paragraph::new("Runs").style(style::STL_FIELD_LBL).right_aligned().render(h_runs, buf);
	Paragraph::new("Total Cost").style(style::STL_FIELD_LBL).right_aligned().render(h_tot_cost, buf);
	Paragraph::new("Avg Cost").style(style::STL_FIELD_LBL).right_aligned().render(h_avg_cost, buf);
	Paragraph::new("Total Dur").style(style::STL_FIELD_LBL).right_aligned().render(h_tot_dur, buf);

	// Render visible data rows
	for (visible_row_idx, item_idx) in (start_idx..end_idx).enumerate() {
		let entry = &data.agents[item_idx];
		let row_y = 2 + visible_row_idx as u16;
		let row_area = area.x_row(row_y).x_h_margin(1);
		let is_hovered = state.is_last_mouse_over(row_area);

		if is_hovered {
			Block::new().bg(style::CLR_BKG_400).render(row_area, buf);
		}

		let [r_agent, r_runs, r_tot_cost, r_avg_cost, r_tot_dur] = cols_layout.areas(row_area);

		let label_style = if is_hovered {
			style::STL_TXT_ACT.fg(style::CLR_TXT_HOVER)
		} else {
			style::STL_TXT_ACT
		};
		let val_style = if is_hovered {
			style::STL_FIELD_VAL.fg(style::CLR_TXT_400)
		} else {
			style::STL_FIELD_VAL
		};

		Paragraph::new(entry.name.clone()).style(label_style).render(r_agent, buf);

		Paragraph::new(entry.count.to_string())
			.style(val_style)
			.right_aligned()
			.render(r_runs, buf);

		Paragraph::new(ui_fmt_cost(Some(entry.cost)))
			.style(val_style)
			.right_aligned()
			.render(r_tot_cost, buf);

		let avg_cost = if entry.count > 0 {
			Some(entry.cost / entry.count as f64)
		} else {
			None
		};
		Paragraph::new(ui_fmt_cost(avg_cost))
			.style(val_style)
			.right_aligned()
			.render(r_avg_cost, buf);

		Paragraph::new(ui_fmt_duration_us(entry.total_duration_us))
			.style(val_style)
			.right_aligned()
			.render(r_tot_dur, buf);
	}

	// Render scroll indicator icons
	let item_count = items_len as u16;
	if item_count.saturating_sub(scroll) > visible_height as u16 {
		let bottom_ico = area.x_bottom_right(1, 1);
		comp::ico_scroll_down().render(bottom_ico, buf);
	}
	if scroll > 0 {
		let top_ico = area.x_top_right(1, 1);
		comp::ico_scroll_up().render(top_ico, buf);
	}
}

fn render_models_view(area: Rect, buf: &mut Buffer, data: &crate::tui::core::GroupDashData, state: &mut AppState) {
	if data.models.is_empty() {
		let msg = Paragraph::new("No models in this group").style(style::STL_FIELD_LBL);
		msg.render(area.x_h_margin(1), buf);
		return;
	}

	const SCROLL_IDEN: ScrollIden = GroupDashView::SCROLL_IDEN;
	state.set_scroll_area(SCROLL_IDEN, area);

	let items_len = data.models.len();
	let scroll = state.clamp_scroll(SCROLL_IDEN, items_len);
	let visible_height = area.height.saturating_sub(1) as usize;
	let start_idx = scroll as usize;
	let end_idx = (start_idx + visible_height).min(items_len);

	let header_label = "Model";
	let max_name_len = data
		.models
		.iter()
		.map(|e| e.name.chars().count())
		.max()
		.unwrap_or(0);
	let first_col_w = (header_label.len().max(max_name_len) + 3) as u16;

	// Column layout definition
	let cols_layout = Layout::default()
		.direction(Direction::Horizontal)
		.constraints([
			Constraint::Length(first_col_w), // Model
			Constraint::Length(10),          // Runs
			Constraint::Length(12),          // Total Cost
			Constraint::Length(12),          // Avg Cost
			Constraint::Length(14),          // Total Duration
		])
		.spacing(1);

	// Render Table Header (Row 1)
	let header_area = area.x_row(1).x_h_margin(1);
	let [h_model, h_runs, h_tot_cost, h_avg_cost, h_tot_dur] = cols_layout.areas(header_area);

	Paragraph::new("Model").style(style::STL_FIELD_LBL).render(h_model, buf);
	Paragraph::new("Runs").style(style::STL_FIELD_LBL).right_aligned().render(h_runs, buf);
	Paragraph::new("Total Cost").style(style::STL_FIELD_LBL).right_aligned().render(h_tot_cost, buf);
	Paragraph::new("Avg Cost").style(style::STL_FIELD_LBL).right_aligned().render(h_avg_cost, buf);
	Paragraph::new("Total Dur").style(style::STL_FIELD_LBL).right_aligned().render(h_tot_dur, buf);

	// Render visible data rows
	for (visible_row_idx, item_idx) in (start_idx..end_idx).enumerate() {
		let entry = &data.models[item_idx];
		let row_y = 2 + visible_row_idx as u16;
		let row_area = area.x_row(row_y).x_h_margin(1);
		let is_hovered = state.is_last_mouse_over(row_area);

		if is_hovered {
			Block::new().bg(style::CLR_BKG_400).render(row_area, buf);
		}

		let [r_model, r_runs, r_tot_cost, r_avg_cost, r_tot_dur] = cols_layout.areas(row_area);

		let label_style = if is_hovered {
			style::STL_TXT_ACT.fg(style::CLR_TXT_HOVER)
		} else {
			style::STL_TXT_ACT
		};
		let val_style = if is_hovered {
			style::STL_FIELD_VAL.fg(style::CLR_TXT_400)
		} else {
			style::STL_FIELD_VAL
		};

		Paragraph::new(entry.name.clone()).style(label_style).render(r_model, buf);

		Paragraph::new(entry.count.to_string())
			.style(val_style)
			.right_aligned()
			.render(r_runs, buf);

		Paragraph::new(ui_fmt_cost(Some(entry.cost)))
			.style(val_style)
			.right_aligned()
			.render(r_tot_cost, buf);

		let avg_cost = if entry.count > 0 {
			Some(entry.cost / entry.count as f64)
		} else {
			None
		};
		Paragraph::new(ui_fmt_cost(avg_cost))
			.style(val_style)
			.right_aligned()
			.render(r_avg_cost, buf);

		Paragraph::new(ui_fmt_duration_us(entry.total_duration_us))
			.style(val_style)
			.right_aligned()
			.render(r_tot_dur, buf);
	}

	// Render scroll indicator icons
	let item_count = items_len as u16;
	if item_count.saturating_sub(scroll) > visible_height as u16 {
		let bottom_ico = area.x_bottom_right(1, 1);
		comp::ico_scroll_down().render(bottom_ico, buf);
	}
	if scroll > 0 {
		let top_ico = area.x_top_right(1, 1);
		comp::ico_scroll_up().render(top_ico, buf);
	}
}
