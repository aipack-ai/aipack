use crate::model::{EndState, Log, LogBmc, PinBmc, RunningState, Stage, Task};
use crate::tui::AppState;
use crate::tui::core::{LinkZones, ScrollIden, UiAction};
use crate::tui::support::UiExt as _;
use crate::tui::view::support::{self, RectExt as _};
use crate::tui::view::{comp, style};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Color;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Scrollbar, ScrollbarState, StatefulWidget, Widget as _};

/// Placeholder view for *Before All* tab.
pub struct RunOverviewView;

/// Component scroll identifiers
impl RunOverviewView {
	const BODY_SCROLL_IDEN: ScrollIden = ScrollIden::OverviewContent;

	const SCROLL_IDENS: &[&ScrollIden] = &[&Self::BODY_SCROLL_IDEN];

	pub fn clear_scroll_idens(state: &mut AppState) {
		state.clear_scroll_zone_areas(Self::SCROLL_IDENS);
	}
}

impl StatefulWidget for RunOverviewView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		let area = area.x_h_margin(1);

		// -- Render Body
		render_body(area, buf, state);
	}
}

fn render_body(area: Rect, buf: &mut Buffer, state: &mut AppState) {
	const SCROLL_IDEN: ScrollIden = RunOverviewView::BODY_SCROLL_IDEN;

	// -- Init the scroll area
	state.set_scroll_area(SCROLL_IDEN, area);

	// -- Determine tasks mode
	let tasks_len = state.tasks().len();

	let is_grid = state.overview_tasks_mode().is_grid(tasks_len);

	// -- Prep
	let Some(run_id) = state.current_run_item().map(|r| r.id()) else {
		Paragraph::new("No current run").render(area, buf);
		return;
	};

	// -- Load Pins
	let pins = match PinBmc::list_for_run(state.mm(), run_id) {
		Ok(pins) => pins,
		Err(err) => {
			Paragraph::new(format!("PinBmc::list error. {err}")).render(area, buf);
			return;
		}
	};

	// -- Load logs
	let logs = match LogBmc::list_for_run_only(state.mm(), run_id) {
		Ok(logs) => logs,
		Err(err) => {
			Paragraph::new(format!("Error fetch log for run. {err}")).render(area, buf);
			return;
		}
	};

	// -- Setup lines
	let max_width = area.width - 3; // for scroll

	let mut link_zones = LinkZones::default();
	let mut all_lines: Vec<Line> = Vec::new();

	let path_color = (state.debug_clr() != 0).then(|| Color::Indexed(state.debug_clr()));

	// -- Add the pins
	link_zones.set_current_line(all_lines.len());
	// ui_for_pins add empty line after, so no ned to ad it again
	support::extend_lines(
		&mut all_lines,
		comp::ui_for_pins_with_hover(&pins, max_width, &mut link_zones, path_color),
		false,
	);

	// -- Add before all
	// Here false for add empty end line because logs add it for each log section
	link_zones.set_current_line(all_lines.len());
	support::extend_lines(
		&mut all_lines,
		ui_for_before_all(&logs, max_width, false, &mut link_zones, path_color),
		false,
	);
	link_zones.set_current_line(all_lines.len());

	let task_section_start = all_lines.len();
	let tasks_section_line_count = if is_grid {
		task_grid_line_count(state.tasks(), max_width)
	} else {
		task_list_line_count(state.tasks())
	};
	let after_task_section_start = task_section_start + tasks_section_line_count;

	let mut after_task_lines: Vec<Line> = Vec::new();

	// -- Add after all
	// Here false for add empty end line because logs add it for each log section
	link_zones.set_current_line(after_task_section_start + after_task_lines.len());
	support::extend_lines(
		&mut after_task_lines,
		ui_for_after_all(&logs, max_width, false, &mut link_zones, path_color),
		false,
	);
	link_zones.set_current_line(after_task_section_start + after_task_lines.len());

	// -- Add Error if present
	if let Some(err_id) = state.current_run_item().and_then(|r| r.run().end_err_id) {
		support::extend_lines(
			&mut after_task_lines,
			comp::ui_for_err_with_hover(state.mm(), err_id, max_width, &mut link_zones, path_color),
			true,
		);
	}

	link_zones.set_current_line(task_section_start);

	// -- Clamp scroll
	// TODO: Needs to have it's own scroll state.
	let line_count = task_section_start + tasks_section_line_count + after_task_lines.len();
	let scroll = state.clamp_scroll(SCROLL_IDEN, line_count);

	// -- Add the tasks ui
	let task_list_lines = if is_grid {
		let top_padding = scroll.saturating_sub(task_section_start as u16) as usize;
		let mut lines = if top_padding > 0 {
			vec![Line::from(vec![Span::raw("")]); top_padding]
		} else {
			Vec::new()
		};
		lines.extend(ui_for_task_grid_viewport(
			state.tasks(),
			max_width,
			task_section_start,
			scroll as usize,
			area.height as usize,
			&mut link_zones,
		));
		lines
	} else {
		let top_padding = scroll.saturating_sub(task_section_start as u16) as usize;
		let mut lines = if top_padding > 0 {
			vec![Line::from(vec![Span::raw("")]); top_padding]
		} else {
			Vec::new()
		};
		lines.extend(ui_for_task_list_viewport(
			state.tasks(),
			max_width,
			task_section_start,
			scroll as usize,
			area.height as usize,
			&mut link_zones,
		));
		lines
	};
	support::extend_lines(&mut all_lines, task_list_lines, true);

	link_zones.set_current_line(all_lines.len());

	// -- Add after-task content
	all_lines.extend(after_task_lines);

	// -- Perform the Click on a link zone
	let zones = link_zones.into_zones();

	// First pass: detect which zone (if any) is hovered.
	// Note: We look for the most specific zone (the one with the minimum span_count)
	let mut hovered_idx: Option<usize> = None;
	let mut min_span_count = usize::MAX;

	for (i, zone) in zones.iter().enumerate() {
		if let Some(line) = all_lines.get_mut(zone.line_idx)
			&& zone
				.is_mouse_over(area, scroll, state.last_mouse_evt(), &mut line.spans)
				.is_some()
			&& zone.span_count < min_span_count
		{
			min_span_count = zone.span_count;
			hovered_idx = Some(i);
		}
	}

	// Second pass: apply hover style to the hovered zone or the whole section group.
	if let Some(i) = hovered_idx {
		let action = zones[i].action.clone();
		let group_id = zones[i].group_id;

		match group_id {
			Some(gid) => {
				for z in zones.iter().filter(|z| z.group_id == Some(gid)) {
					if let Some(line) = all_lines.get_mut(z.line_idx)
						&& let Some(hover_spans) = z.spans_slice_mut(&mut line.spans)
					{
						for span in hover_spans {
							span.style.fg = Some(style::CLR_TXT_HOVER_TO_CLIP);
							if is_grid {
								span.style.bg = Some(style::CLR_BKG_BLACK);
							}
							// span.style = span.style.add_modifier(Modifier::BOLD);
						}
					}
				}
			}
			None => {
				if let Some(line) = all_lines.get_mut(zones[i].line_idx)
					&& let Some(hover_spans) = zones[i].spans_slice_mut(&mut line.spans)
				{
					for span in hover_spans {
						span.style = style::style_text_path(true, None);
						if is_grid {
							span.style.bg = Some(style::CLR_BKG_BLACK);
						}
					}
				}
			}
		}

		if state.is_mouse_up_only() {
			state.set_action(action);
			// Note: Little trick to not show hover on the next tasks tab screen
			state.clear_mouse_evts(true);
		}
	}

	// -- Render All Content
	let p = Paragraph::new(all_lines).scroll((scroll, 0));
	p.render(area, buf);

	// -- Render Scrollbar
	let content_size = line_count.saturating_sub(area.height as usize);
	let mut scrollbar_state = ScrollbarState::new(content_size).position(scroll as usize);

	let scrollbar = Scrollbar::default()
		.orientation(ratatui::widgets::ScrollbarOrientation::VerticalRight)
		.begin_symbol(Some("▲"))
		.end_symbol(Some("▼"));
	scrollbar.render(area, buf, &mut scrollbar_state);
}

// region:    --- UI Builders

fn ui_for_before_all(
	logs: &[Log],
	max_width: u16,
	show_steps: bool,
	link_zones: &mut LinkZones,
	path_color: Option<Color>,
) -> Vec<Line<'static>> {
	comp::ui_for_logs_with_hover(
		logs.iter(),
		max_width,
		Some(Stage::BeforeAll),
		show_steps,
		link_zones,
		path_color,
	)
}

fn ui_for_after_all(
	logs: &[Log],
	max_width: u16,
	show_steps: bool,
	link_zones: &mut LinkZones,
	path_color: Option<Color>,
) -> Vec<Line<'static>> {
	comp::ui_for_logs_with_hover(
		logs.iter(),
		max_width,
		Some(Stage::AfterAll),
		show_steps,
		link_zones,
		path_color,
	)
}

fn ui_for_task_list(tasks: &[Task], max_width: u16, link_zones: &mut LinkZones) -> Vec<Line<'static>> {
	if tasks.is_empty() {
		return Vec::new();
	}

	// -- Prep
	let tasks_len = tasks.len();

	// let mut line: u16 = 0;
	let (marker, marker_spacer) = tasks_marker();
	let marker_width = marker.x_width();
	let marker_spacer_width = marker_spacer.x_width();
	let _marker_and_spacer_width = marker_width + marker_spacer_width;
	let marker_prefix_spans_len = marker.len() + marker_spacer.len();

	let content_width = max_width.saturating_sub(marker_spacer_width + marker_width);
	let gap_span = Span::raw("  ");
	let gap_width = gap_span.width() as u16;

	let mut all_lines: Vec<Vec<Span<'static>>> = Vec::new();

	// Not used in this case
	// link_zones.inc_current_line_by(all_lines.len());

	// -- Layout
	let [label_a, _, input_a, _, _ai_a, _, output_a] = Layout::default()
		.direction(Direction::Horizontal)
		.constraints(vec![
			Constraint::Length(12),        // label_a
			Constraint::Length(gap_width), // gap
			Constraint::Fill(3),           // input_a
			Constraint::Length(gap_width), // gap
			Constraint::Length(6),         // ai_a (hardcode in task.ui_ai())
			Constraint::Length(gap_width), // gap
			Constraint::Fill(5),           // output_a
		])
		.areas(Rect::new(0, 0, content_width, 1));

	// --  Build the UI lines
	for (idx, task) in tasks.iter().enumerate() {
		let mut task_line = task.ui_label(None, label_a.width, tasks_len);
		let task_id = task.id;

		// -- Link Zone
		// +2 for the space + ico (from the ui_label), 2 to take space and label text
		// NOTE: This should probably be part of the task facade (should not make those assumption here)
		link_zones.push_link_zone(idx, marker_prefix_spans_len + 2, 2, UiAction::GoToTask { task_id });

		// -- Gap
		task_line.push(gap_span.clone());

		// -- Add Input
		let input_spans = task.ui_input(input_a.width);
		task_line.extend(input_spans);

		// -- Gap
		task_line.push(gap_span.clone());

		// -- Add AI
		let ai_spans = task.ui_ai();
		task_line.extend(ai_spans);

		// -- Gap
		task_line.push(gap_span.clone());

		// -- Add Output or skip
		if task.has_skip() {
			let skip_spans = task.ui_skip(output_a.width);
			task_line.extend(skip_spans);
		} else {
			let output_spans = task.ui_output(output_a.width);
			task_line.extend(output_spans);
		}

		all_lines.push(task_line);

		// line += 1;
	}

	// -- render legend (on bottom)
	// build legend_line
	all_lines.push(Vec::new());
	all_lines.push(ui_for_legend(tasks));

	// -- Build the marker component
	comp::ui_for_marker_section(marker, marker_spacer, all_lines)
}

fn ui_for_task_list_viewport(
	tasks: &[Task],
	max_width: u16,
	task_section_start: usize,
	scroll: usize,
	viewport_height: usize,
	link_zones: &mut LinkZones,
) -> Vec<Line<'static>> {
	if tasks.is_empty() {
		return Vec::new();
	}

	let section_line_count = task_list_line_count(tasks);
	let section_end = task_section_start + section_line_count;
	let viewport_end = scroll.saturating_add(viewport_height);

	if viewport_end <= task_section_start || scroll >= section_end {
		return Vec::new();
	}

	let local_start = scroll.saturating_sub(task_section_start);
	let local_end = viewport_end.saturating_sub(task_section_start).min(section_line_count);

	if local_start >= local_end {
		return Vec::new();
	}

	let full_lines = ui_for_task_list(tasks, max_width, link_zones);
	// Keep only the visible logical task-section lines and let the caller insert
	// the top padding corresponding to the current scroll offset.
	full_lines.into_iter().skip(local_start).take(local_end - local_start).collect()
}

// endregion: --- UI Builders

// region:    --- Support

#[derive(Debug, Clone, Copy)]
struct OverviewTaskGridLayout {
	items_per_row: usize,
	task_row_count: usize,
	logical_line_count: usize,
	marker_width: u16,
	marker_spacer_width: u16,
	marker_prefix_spans_len: usize,
}

fn tasks_marker() -> (Vec<Span<'static>>, Vec<Span<'static>>) {
	let marker = vec![comp::new_marker("Tasks:", style::STL_SECTION_MARKER)];
	let marker_spacer = vec![Span::raw(" ")];
	(marker, marker_spacer)
}

fn task_list_line_count(tasks: &[Task]) -> usize {
	if tasks.is_empty() { 0 } else { tasks.len() + 2 }
}

fn task_grid_line_count(tasks: &[Task], max_width: u16) -> usize {
	if tasks.is_empty() {
		return 0;
	}

	task_grid_layout(tasks, max_width).logical_line_count
}

fn task_grid_layout(tasks: &[Task], max_width: u16) -> OverviewTaskGridLayout {
	let (marker, marker_spacer) = tasks_marker();
	let marker_width = marker.x_width();
	let marker_spacer_width = marker_spacer.x_width();
	let marker_prefix_spans_len = marker.len() + marker_spacer.len();
	let content_width = max_width.saturating_sub(marker_spacer_width + marker_width);

	let sample_block_width = tasks
		.first()
		.map(|task| task.ui_short_block(tasks.len()).x_width())
		.unwrap_or_default();

	// avoid div by zero and ensure at least 1 item
	let items_per_row = (content_width / sample_block_width.max(1)).max(1) as usize;

	let task_row_count = tasks.len().div_ceil(items_per_row);
	let logical_line_count = task_row_count + 2;

	OverviewTaskGridLayout {
		items_per_row,
		task_row_count,
		logical_line_count,
		marker_width,
		marker_spacer_width,
		marker_prefix_spans_len,
	}
}

fn ui_for_task_grid_viewport(
	tasks: &[Task],
	max_width: u16,
	task_section_start: usize,
	scroll: usize,
	viewport_height: usize,
	link_zones: &mut LinkZones,
) -> Vec<Line<'static>> {
	if tasks.is_empty() {
		return Vec::new();
	}

	let layout = task_grid_layout(tasks, max_width);
	let section_end = task_section_start + layout.logical_line_count;
	let viewport_end = scroll.saturating_add(viewport_height);

	if viewport_end <= task_section_start || scroll >= section_end {
		return Vec::new();
	}

	let local_start = scroll.saturating_sub(task_section_start);
	let local_end = viewport_end.saturating_sub(task_section_start).min(layout.logical_line_count);

	if local_start >= local_end {
		return Vec::new();
	}

	let mut lines: Vec<Line<'static>> = Vec::new();
	let rendered_row_offset = local_start;

	for local_row_idx in local_start..local_end {
		if local_row_idx < layout.task_row_count {
			let task_start = local_row_idx * layout.items_per_row;
			let task_end = (task_start + layout.items_per_row).min(tasks.len());
			let mut row_spans: Vec<Span<'static>> = Vec::new();

			for task in &tasks[task_start..task_end] {
				let task_block = task.ui_short_block(tasks.len());
				let zone_span_start = row_spans.len();
				let zone_span_count = task_block.len();
				let task_id = task.id;

				row_spans.extend(task_block);

				link_zones.push_link_zone(
					rendered_row_offset + lines.len(),
					layout.marker_prefix_spans_len + zone_span_start,
					zone_span_count,
					UiAction::GoToTask { task_id },
				);
			}

			let prefix = if local_row_idx == 0 {
				vec![comp::new_marker("Tasks:", style::STL_SECTION_MARKER)]
			} else {
				vec![Span::raw(" ".repeat(layout.marker_width as usize))]
			};
			let mut spans = prefix;
			spans.push(Span::raw(" ".repeat(layout.marker_spacer_width as usize)));
			spans.extend(row_spans);
			lines.push(Line::from(spans));
		} else if local_row_idx == layout.task_row_count {
			let prefix = if layout.task_row_count == 0 {
				vec![comp::new_marker("Tasks:", style::STL_SECTION_MARKER)]
			} else {
				vec![Span::raw(" ".repeat(layout.marker_width as usize))]
			};
			let mut spans = prefix;
			spans.push(Span::raw(" ".repeat(layout.marker_spacer_width as usize)));
			lines.push(Line::from(spans));
		} else if local_row_idx == layout.task_row_count + 1 {
			let prefix = if layout.task_row_count == 0 {
				vec![comp::new_marker("Tasks:", style::STL_SECTION_MARKER)]
			} else {
				vec![Span::raw(" ".repeat(layout.marker_width as usize))]
			};
			let mut spans = prefix;
			spans.push(Span::raw(" ".repeat(layout.marker_spacer_width as usize)));
			spans.extend(ui_for_legend(tasks));
			lines.push(Line::from(spans));
		}
	}

	lines
}

fn ui_for_legend(tasks: &[Task]) -> Vec<Span<'static>> {
	let mut count_done = 0;
	let mut count_waiting = 0;
	let mut count_skip = 0;
	let mut count_err = 0;
	let mut count_ai = 0;

	// -- Process counts
	for task in tasks {
		match RunningState::from(task) {
			RunningState::NotScheduled | RunningState::Unknown => (), // this is for other state, ignore for now.
			RunningState::Waiting => count_waiting += 1,
			RunningState::Running => {
				if task.is_ai_running() {
					count_ai += 1;
				}
			}
			RunningState::Ended(end_state) => match end_state {
				Some(EndState::Ok) => count_done += 1,
				Some(EndState::Err) => count_err += 1,
				Some(EndState::Skip) => count_skip += 1,
				Some(EndState::Cancel) => (), // TODO: handle cancel
				None => (),
			},
		}
	}

	// -- Build the UI
	let num_width = 4;
	let mut legend_line = vec![
		// The Done
		Span::styled("Done:", style::CLR_BKG_RUNNING_DONE),
		Span::raw(format!(" {count_done:<num_width$} ")),
	];
	if count_ai > 0 {
		legend_line.push(Span::styled("AI:", style::CLR_BKG_RUNNING_AI));
		legend_line.push(Span::raw(format!(" {count_ai:<num_width$} ")));
	}
	if count_skip > 0 {
		legend_line.push(Span::styled("Skip:", style::CLR_BKG_RUNNING_SKIP));
		legend_line.push(Span::raw(format!(" {count_skip:<num_width$} ")));
	}
	if count_waiting > 0 {
		legend_line.push(Span::styled("Queue:", style::CLR_TXT_650));
		legend_line.push(Span::raw(format!(" {count_waiting:<num_width$} ")));
	}
	if count_err > 0 {
		legend_line.push(Span::styled("Error:", style::CLR_BKG_RUNNING_ERR));
		legend_line.push(Span::raw(format!(" {count_err:<num_width$} ")));
	}

	legend_line
}

// endregion: --- Support

// region:    --- UI Event Processing

// NOTE: Probably need a area_offset
// #[allow(unused)]
// fn process_mouse_for_task_list(state: &mut AppState, task_list_zones: LinkZones, x_offset: u16, y_offset: u16) {
// 	if let Some(mouse_evt) = state.mouse_evt()
// 		&& mouse_evt.is_click()
// 	{
// 		let data_ref = task_list_zones.find_data_key(mouse_evt.position(), x_offset, y_offset);
// 		// NOTE: now select the right data_ref
// 		if let Some(_data_ref) = data_ref {
// 			// tracing::debug!("data_ref: {data_ref:?}");
// 		}
// 		// TODO: ...
// 	}
// }

// endregion: --- UI Event Processing
