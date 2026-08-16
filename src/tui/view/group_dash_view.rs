use crate::tui::support::ui_fmt_cost;
use crate::tui::view::support::RectExt as _;
use crate::tui::{AppState, style};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Stylize as _;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, StatefulWidget, Widget as _};

pub struct GroupDashView;

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

		// -- Layout: Header | Separator | Content Columns
		let [header_a, _sep_a, content_a] = Layout::default()
			.direction(Direction::Vertical)
			.constraints(vec![
				Constraint::Length(2), // Summary Header
				Constraint::Length(1), // Gap / Divider
				Constraint::Fill(1),   // 3-Column Metrics
			])
			.areas(area);

		// -- Render Header Summary
		render_header(header_a, buf, &data);

		// -- Render 3 Columns
		render_columns(content_a, buf, &data);
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

fn render_columns(area: Rect, buf: &mut Buffer, data: &crate::tui::core::GroupDashData) {
	let [col_runs_a, col_agents_a, col_models_a] = Layout::default()
		.direction(Direction::Horizontal)
		.constraints(vec![
			Constraint::Ratio(1, 3),
			Constraint::Ratio(1, 3),
			Constraint::Ratio(1, 3),
		])
		.spacing(2)
		.areas(area);

	render_top_runs_column(col_runs_a, buf, data);
	render_agents_column(col_agents_a, buf, data);
	render_models_column(col_models_a, buf, data);
}

fn render_top_runs_column(area: Rect, buf: &mut Buffer, data: &crate::tui::core::GroupDashData) {
	let mut lines: Vec<Line<'static>> = Vec::new();

	// Section Title
	lines.push(Line::from(vec![
		Span::styled(" Top Runs ", style::STL_SECTION_MARKER),
		Span::styled(format!(" ({})", data.top_runs.len()), style::STL_FIELD_VAL),
	]));
	lines.push(Line::default());

	for entry in &data.top_runs {
		let cost_str = ui_fmt_cost(Some(entry.cost));
		let child_str = if entry.child_count > 0 {
			format!(" ({} sub-runs)", entry.child_count)
		} else {
			String::new()
		};

		let label_span = Span::styled(entry.label.clone(), style::STL_TXT_ACT);
		let details_span = Span::styled(format!(" {child_str} - {cost_str}"), style::STL_FIELD_VAL);

		lines.push(Line::from(vec![label_span, details_span]));
	}

	if data.top_runs.is_empty() {
		lines.push(Line::from(Span::styled("No runs", style::STL_FIELD_LBL)));
	}

	Paragraph::new(lines).render(area, buf);
}

fn render_agents_column(area: Rect, buf: &mut Buffer, data: &crate::tui::core::GroupDashData) {
	let mut lines: Vec<Line<'static>> = Vec::new();

	// Section Title
	lines.push(Line::from(vec![
		Span::styled(" Agents ", style::STL_SECTION_MARKER),
		Span::styled(format!(" ({})", data.agents.len()), style::STL_FIELD_VAL),
	]));
	lines.push(Line::default());

	for entry in &data.agents {
		let cost_str = ui_fmt_cost(Some(entry.cost));
		let count_str = if entry.count > 1 {
			format!(" (x{})", entry.count)
		} else {
			String::new()
		};

		let name_span = Span::styled(entry.name.clone(), style::STL_TXT_ACT);
		let details_span = Span::styled(format!("{count_str} - {cost_str}"), style::STL_FIELD_VAL);

		lines.push(Line::from(vec![name_span, Span::raw(" "), details_span]));
	}

	if data.agents.is_empty() {
		lines.push(Line::from(Span::styled("No agents", style::STL_FIELD_LBL)));
	}

	Paragraph::new(lines).render(area, buf);
}

fn render_models_column(area: Rect, buf: &mut Buffer, data: &crate::tui::core::GroupDashData) {
	let mut lines: Vec<Line<'static>> = Vec::new();

	// Section Title
	lines.push(Line::from(vec![
		Span::styled(" Models ", style::STL_SECTION_MARKER),
		Span::styled(format!(" ({})", data.models.len()), style::STL_FIELD_VAL),
	]));
	lines.push(Line::default());

	for entry in &data.models {
		let cost_str = ui_fmt_cost(Some(entry.cost));
		let count_str = if entry.count > 1 {
			format!(" (x{})", entry.count)
		} else {
			String::new()
		};

		let name_span = Span::styled(entry.name.clone(), style::STL_TXT_ACT);
		let details_span = Span::styled(format!("{count_str} - {cost_str}"), style::STL_FIELD_VAL);

		lines.push(Line::from(vec![name_span, Span::raw(" "), details_span]));
	}

	if data.models.is_empty() {
		lines.push(Line::from(Span::styled("No models", style::STL_FIELD_LBL)));
	}

	Paragraph::new(lines).render(area, buf);
}
