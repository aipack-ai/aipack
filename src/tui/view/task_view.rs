use crate::model::{EndState, Log, LogBmc, ModelManager, PinBmc, Run, RunningState, Task, TaskBmc};
use crate::support::text::truncate_with_ellipsis;
use crate::tui::core::{LinkZones, ScrollIden, UiAction};
use crate::tui::view::support::RectExt as _;
use crate::tui::view::{comp, support};
use crate::tui::{AppState, style};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Color;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Scrollbar, ScrollbarState, StatefulWidget, Widget as _};
use std::borrow::Cow;

/// Renders the content of a task. For now, the logs.
pub struct TaskView;

/// Component scroll identifiers
impl TaskView {
	const CONTENT_SCROLL_IDEN: ScrollIden = ScrollIden::TaskContent;

	const SCROLL_IDENS: &[&ScrollIden] = &[&Self::CONTENT_SCROLL_IDEN];

	pub fn clear_scroll_idens(state: &mut AppState) {
		state.clear_scroll_zone_areas(Self::SCROLL_IDENS);
	}
}
enum HeaderMode {
	Full,
	TokensOnly,
	None,
}

impl StatefulWidget for TaskView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		// (run.has_prompt_parts, many_runs)
		let header_mode = match (state.current_run_has_prompt_parts(), state.tasks().len() > 1) {
			// For now, none has_prompt, like we have some
			(None | Some(true), true) => HeaderMode::Full,
			(None | Some(true), false) => HeaderMode::TokensOnly,
			// For sure, No eadher
			(Some(false), _) => HeaderMode::None,
			// (None, _) => HeaderMode::None,
		};
		// let show_model_row = true;

		// -- Layout Header | Logs
		let (header_height, header_spacing) = match header_mode {
			HeaderMode::Full => (2, 1),
			HeaderMode::TokensOnly => (1, 1),
			HeaderMode::None => (0, 0),
		};

		let [header_a, _space_1, logs_a] = Layout::default()
			.direction(Direction::Vertical)
			.constraints(vec![
				Constraint::Length(header_height), // header
				Constraint::Max(header_spacing),   // space_1
				Constraint::Fill(1),               // logs
			])
			.areas(area);

		render_header(header_a, buf, state, header_mode);

		// don't show the steps
		render_body(logs_a, buf, state, false);
	}
}

fn render_header(area: Rect, buf: &mut Buffer, state: &mut AppState, header_mode: HeaderMode) {
	// Do nothing if None
	if matches!(header_mode, HeaderMode::None) {
		return;
	}

	// -- Prepare Data
	const L1_VAL_1_WIDTH: u16 = 26;
	const L1_VAL_2_WIDTH: u16 = 12; // cost value

	const L2_VAL_1_WIDTH: u16 = 26;
	const L2_VAL_2_WIDTH: u16 = 26;

	// -- Columns layout
	// l1_ is for the model, cost, duration
	let [l1_label_1, l1_val_1, l1_label_2, l1_val_2, l1_label_3, l1_val_3] = Layout::default()
		.direction(Direction::Horizontal)
		.constraints(vec![
			Constraint::Length(10),             // Model
			Constraint::Length(L1_VAL_1_WIDTH), //
			Constraint::Length(7),              // Cost
			Constraint::Length(L1_VAL_2_WIDTH), //
			Constraint::Length(13),             // Duration
			Constraint::Fill(1),                //
		])
		.spacing(1)
		.areas(area);
	// l2_ is for the prompt, compl, cache info
	let [l2_label_1, l2_val_1, l2_label_2, l2_val_2, l2_label_3, l2_val_3] = Layout::default()
		.direction(Direction::Horizontal)
		.constraints(vec![
			Constraint::Length(10),             // Prompt
			Constraint::Length(L2_VAL_1_WIDTH), //
			Constraint::Length(7),              // Completion
			Constraint::Length(L2_VAL_2_WIDTH), //
			Constraint::Length(13),             // Cache Info
			Constraint::Fill(1),                //
		])
		.spacing(1)
		.areas(area);

	let mut current_row = 0;

	// -- Render l1 - Model Row (only if multiple tasks)
	let model_name = state.current_task_model_name();
	let cost = state.current_task_cost_fmt();
	let duration = state.current_task_duration_txt();

	if matches!(header_mode, HeaderMode::Full) {
		current_row += 1;

		Paragraph::new("Model:")
			.style(style::STL_FIELD_LBL)
			.right_aligned()
			.render(l1_label_1.x_row(current_row), buf);
		// NOTE: here a little chack to have maximum space for model name
		Paragraph::new(model_name)
			.style(style::STL_FIELD_VAL)
			.render(l1_val_1.x_row(current_row).x_width(26), buf);

		// NOTE: Here we use Span to give a little bit more space to Model
		Paragraph::new(Span::styled("  Cost:", style::STL_FIELD_LBL))
			.right_aligned()
			.render(l1_label_2.x_row(current_row), buf);
		Paragraph::new(cost)
			.style(style::STL_FIELD_VAL)
			.render(l1_val_2.x_row(current_row), buf);

		Paragraph::new(" Duration:")
			.style(style::STL_FIELD_LBL)
			.right_aligned()
			.render(l1_label_3.x_row(current_row), buf);
		Paragraph::new(duration)
			.style(style::STL_FIELD_VAL)
			.render(l1_val_3.x_row(current_row), buf);
	}

	// -- Render l2 - Row for tokens
	let prompt_tk = state.current_task_prompt_tokens_fmt();
	// Here we assume if overflow is because we have a (1235 tk reasoning)
	let prompt_tk = truncate_with_ellipsis(&prompt_tk, L2_VAL_1_WIDTH as usize, ".)");
	let completion_tk = state.current_task_completion_tokens_fmt();
	// Here we assume if overflow is because have a (123 KB)
	let completion_tk = truncate_with_ellipsis(&completion_tk, L2_VAL_2_WIDTH as usize, ".)");
	let cache_info = state.current_task_cache_info_fmt();

	if matches!(header_mode, HeaderMode::Full | HeaderMode::TokensOnly) {
		current_row += 1;
		Paragraph::new("Prompt:")
			.style(style::STL_FIELD_LBL)
			.right_aligned()
			.render(l2_label_1.x_row(current_row), buf);
		Paragraph::new(prompt_tk)
			.style(style::STL_FIELD_VAL)
			.render(l2_val_1.x_row(current_row), buf);

		Paragraph::new("Compl:")
			.style(style::STL_FIELD_LBL)
			.right_aligned()
			.render(l2_label_2.x_row(current_row), buf);
		Paragraph::new(completion_tk)
			.style(style::STL_FIELD_VAL)
			.render(l2_val_2.union(l2_val_3).x_row(current_row), buf);

		//current_task_cache_write_fmt
		if let Some(cache_info) = cache_info {
			Paragraph::new(" Cache Info:")
				.style(style::STL_FIELD_LBL)
				.right_aligned()
				.render(l2_label_3.x_row(current_row), buf);
			Paragraph::new(cache_info)
				.style(style::STL_FIELD_VAL)
				.render(l2_val_3.union(l2_val_3).x_row(current_row), buf);
		}
	}
}

fn render_body(area: Rect, buf: &mut Buffer, state: &mut AppState, show_steps: bool) {
	const SCROLL_IDEN: ScrollIden = TaskView::CONTENT_SCROLL_IDEN;

	// -- init the scroll area
	state.set_scroll_area(SCROLL_IDEN, area);

	// -- Get the current task (return early)
	let Some(run_item) = state.current_run_item() else {
		Line::raw("No Current Run").render(area, buf);
		return;
	};
	let run = run_item.run();

	let Some(task) = state.current_task() else {
		Line::raw("No Current Task").render(area, buf);
		return;
	};

	// -- Load Pins
	let pins = match PinBmc::list_for_task(state.mm(), task.id) {
		Ok(pins) => pins,
		Err(err) => {
			Paragraph::new(format!("PinBmc::list error. {err}")).render(area, buf);
			return;
		}
	};

	// -- Load Logs
	let logs = match LogBmc::list_for_task(state.mm(), task.id) {
		Ok(logs) => logs,
		Err(err) => {
			Paragraph::new(format!("LogBmc::list error. {err}")).render(area, buf);
			return;
		}
	};

	// -- Setup UI Lines
	let mut all_lines: Vec<Line> = Vec::new();
	let max_width = area.width - 3; // for scroll bar

	// -- Link zones accumulator for hover/click over logs
	let mut link_zones = LinkZones::default();

	let path_color = (state.debug_clr() != 0).then(|| Color::Indexed(state.debug_clr()));

	// -- Add the pins
	link_zones.set_current_line(all_lines.len());
	// ui_for_pins add empty line after, so no ned to ad it again
	support::extend_lines(
		&mut all_lines,
		comp::ui_for_pins_with_hover(&pins, max_width, &mut link_zones, path_color),
		false,
	);
	link_zones.set_current_line(all_lines.len());

	// -- Add Input (with hover/click to copy)
	support::extend_lines(
		&mut all_lines,
		ui_for_input(state.mm(), task, max_width, &mut link_zones, path_color),
		false,
	);
	link_zones.set_current_line(all_lines.len());

	// -- Add Before AI Logs Lines (with hover zones)
	link_zones.set_current_line(all_lines.len());
	support::extend_lines(
		&mut all_lines,
		ui_for_before_ai_logs(task, &logs, max_width, show_steps, &mut link_zones, path_color),
		false,
	);

	// -- Add AI Lines
	// if the run has prompt parts or we do not know, we display the line
	if let Some(true) | None = state.current_run_has_prompt_parts() {
		link_zones.set_current_line(all_lines.len());
		support::extend_lines(
			&mut all_lines,
			ui_for_ai(run, task, max_width, &mut link_zones, path_color),
			true,
		);
	}
	link_zones.set_current_line(all_lines.len());

	// -- Add After AI Logs Lines (with hover zones)
	link_zones.set_current_line(all_lines.len());
	support::extend_lines(
		&mut all_lines,
		ui_for_after_ai_logs(task, &logs, max_width, show_steps, &mut link_zones, path_color),
		false,
	);

	// -- Add output if end (with hover/click to copy)
	if task.output_short.is_some() {
		// Ensure zones are anchored to the first output line
		link_zones.set_current_line(all_lines.len());
		support::extend_lines(
			&mut all_lines,
			ui_for_output(state.mm(), task, max_width, &mut link_zones, path_color),
			false,
		);
	}
	link_zones.set_current_line(all_lines.len());

	// -- Add Error if present
	if let Some(err_id) = task.end_err_id {
		support::extend_lines(
			&mut all_lines,
			comp::ui_for_err_with_hover(state.mm(), err_id, max_width, &mut link_zones, path_color),
			true,
		);
	}
	link_zones.set_current_line(all_lines.len());

	// -- Clamp scroll
	let line_count = all_lines.len();
	let scroll = state.clamp_scroll(SCROLL_IDEN, line_count);

	// -- Perform hover/click over link zones
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
					}
				}
			}
		}

		if state.is_mouse_up_only() && state.is_last_mouse_over(area) {
			state.set_action(action);
			state.trigger_redraw();
			state.clear_mouse_evts();
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

fn ui_for_input(
	mm: &ModelManager,
	task: &Task,
	max_width: u16,
	link_zones: &mut LinkZones,
	path_color: Option<Color>,
) -> Vec<Line<'static>> {
	let marker_txt = "Input:";
	let marker_style = style::STL_SECTION_MARKER_INPUT;

	match TaskBmc::get_input_for_display(mm, task) {
		Ok(Some(content)) => {
			let mut out = comp::ui_for_marker_section_str(
				&content,
				(marker_txt, marker_style),
				max_width,
				None,
				Some(link_zones),
				Some(UiAction::ToClipboardCopy(content.clone())),
				path_color,
			);

			// Separator line (no zones)
			out.push(Line::default());
			link_zones.inc_current_line_by(1);

			out
		}
		Ok(None) => Vec::new(),
		Err(err) => {
			// Render error unchanged and keep a trailing separator for layout consistency.
			let mut out = comp::ui_for_marker_section_str(
				&format!("Error getting input. {err}"),
				(marker_txt, marker_style),
				max_width,
				None,
				None,
				None,
				path_color,
			);
			if !out.is_empty() {
				out.push(Line::default());
			}
			out
		}
	}
}

fn ui_for_ai(
	run: &Run,
	task: &Task,
	max_width: u16,
	link_zones: &mut LinkZones,
	path_color: Option<Color>,
) -> Vec<Line<'static>> {
	let marker_txt = "AI:";
	let marker_style_active = style::STL_SECTION_MARKER_AI;
	let marker_stype_inactive = style::STL_SECTION_MARKER;
	let model_name = task
		.model_ov
		.as_ref()
		.or(run.model.as_ref())
		.map(|v| v.as_str())
		.unwrap_or_default();

	let model_names: Cow<str> = if let Some(model_upstream) = task.model_upstream.as_ref()
		&& model_upstream != model_name
	{
		format!("{model_name} ({model_upstream})").into()
	} else {
		model_name.into()
	};

	let (content, style) = match task.ai_running_state() {
		RunningState::Ended(Some(EndState::Cancel)) => (
			Some(format!("■ AI request canceled {model_names}.")),
			marker_style_active,
		),

		RunningState::Running => (
			Some(format!("➜ Sending prompt to AI model {model_names}.")),
			marker_style_active,
		),

		RunningState::Ended(Some(EndState::Ok)) => {
			// let cost = state.current_task_cost_fmt();
			// let compl = state.current_task_completion_tokens_fmt();
			(
				Some(format!("✔ AI model {model_names} responded.")),
				marker_style_active,
			)
		}

		RunningState::Ended(Some(EndState::Err)) => {
			// let cost = state.current_task_cost_fmt();
			// let compl = state.current_task_completion_tokens_fmt();
			(
				Some(format!("✘ AI model {model_names} responded with an error.")),
				marker_style_active,
			)
		}

		RunningState::NotScheduled => (
			Some(". No instruction given. GenAI Skipped.".to_string()),
			marker_stype_inactive,
		),

		// Anything else ignore for now
		_ => (None, marker_stype_inactive),
	};

	if let Some(content) = content {
		comp::ui_for_marker_section_str(
			&content,
			(marker_txt, style),
			max_width,
			None,
			Some(link_zones),
			Some(UiAction::ToClipboardCopy(content.clone())),
			path_color,
		)
	} else {
		Vec::new()
	}
}

fn ui_for_output(
	mm: &ModelManager,
	task: &Task,
	max_width: u16,
	link_zones: &mut LinkZones,
	path_color: Option<Color>,
) -> Vec<Line<'static>> {
	let marker_txt = "Output:";
	let marker_style = style::STL_SECTION_MARKER_OUTPUT;

	match TaskBmc::get_output_for_display(mm, task) {
		Ok(Some(content)) => {
			let mut out = comp::ui_for_marker_section_str(
				&content,
				(marker_txt, marker_style),
				max_width,
				None,
				Some(link_zones),
				Some(UiAction::ToClipboardCopy(content.clone())),
				path_color,
			);

			// Separator line (no zones)
			out.push(Line::default());
			link_zones.inc_current_line_by(1);

			out
		}
		Ok(None) => Vec::new(),
		Err(err) => {
			// Render error unchanged and keep a trailing separator for layout consistency.
			let mut out = comp::ui_for_marker_section_str(
				&format!("Error getting output. {err}"),
				(marker_txt, marker_style),
				max_width,
				None,
				None,
				None,
				path_color,
			);
			if !out.is_empty() {
				out.push(Line::default());
			}
			out
		}
	}
}

fn ui_for_before_ai_logs(
	task: &Task,
	logs: &[Log],
	max_width: u16,
	show_steps: bool,
	link_zones: &mut LinkZones,
	path_color: Option<Color>,
) -> Vec<Line<'static>> {
	let ai_start: i64 = task.ai_start.map(|v| v.as_i64()).unwrap_or(i64::MAX);
	let iter = logs.iter().filter(|v| v.ctime.as_i64() < ai_start);
	comp::ui_for_logs_with_hover(iter, max_width, None, show_steps, link_zones, path_color)
}

fn ui_for_after_ai_logs(
	task: &Task,
	logs: &[Log],
	max_width: u16,
	show_steps: bool,
	link_zones: &mut LinkZones,
	path_color: Option<Color>,
) -> Vec<Line<'static>> {
	let ai_start: i64 = task.ai_start.map(|v| v.as_i64()).unwrap_or(i64::MAX);
	let iter = logs.iter().filter(|v| v.ctime.as_i64() > ai_start);
	comp::ui_for_logs_with_hover(iter, max_width, None, show_steps, link_zones, path_color)
}

#[allow(unused)]
fn first_line_truncate(s: &str, max: usize) -> String {
	s.lines().next().unwrap_or("").chars().take(max).collect()
}

// endregion: --- UI Builders
