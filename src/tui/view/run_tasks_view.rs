use crate::tui::core::{LinkZones, ScrollIden, UiAction};
use crate::tui::support::{UiExt as _, clamp_idx_in_len};
use crate::tui::view::comp::{self, ui_for_marker_section_str};
use crate::tui::view::support::RectExt as _;
use crate::tui::{AppState, TaskView, style};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, HighlightSpacing, List, ListItem, ListState, Paragraph, StatefulWidget, Widget as _};

/// Renders the *Tasks* tab (tasks list and content).
pub struct RunTasksView;

/// Component scroll identifiers
impl RunTasksView {
	const TASKS_NAV_SCROLL_IDEN: ScrollIden = ScrollIden::TasksNav;

	const SCROLL_IDENS: &[&ScrollIden] = &[&Self::TASKS_NAV_SCROLL_IDEN];

	pub fn clear_scroll_idens(state: &mut AppState) {
		state.clear_scroll_zone_areas(Self::SCROLL_IDENS);
		TaskView::clear_scroll_idens(state);
	}
}

impl StatefulWidget for RunTasksView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		// -- If not tasks, render no tasks ui
		if state.tasks().is_empty() {
			render_no_tasks(area, buf, state);
			return;
		}

		// -- Render Task(s)
		let show_tasks_nav = state.tasks().len() > 1;

		let tasks_nav_width = if show_tasks_nav { 20 } else { 0 };
		let [nav_a, content_a] = Layout::default()
			.direction(Direction::Horizontal)
			.constraints([Constraint::Max(tasks_nav_width), Constraint::Min(0)])
			.spacing(1)
			.areas(area);

		// -- Process the go to task
		let mut selection_in_view = false;
		if let Some(UiAction::GoToTask { task_id }) = state.action() {
			if let Some(task_idx) = state.tasks().iter().position(|t| t.id == *task_id) {
				state.set_task_idx(Some(task_idx));
				selection_in_view = true;
			}
			// Make sure we clear the action even if task_idx was not found
			state.clear_action();
		}

		// -- Render tasks nav
		// IMPORTANT: Need to display nav first,
		//            because it will process the mouse event for task selection.
		if show_tasks_nav {
			render_tasks_nav(nav_a, buf, selection_in_view, state);
		} else {
			state.clear_scroll_zone_area(&RunTasksView::TASKS_NAV_SCROLL_IDEN);
		}

		// -- Render task content
		TaskView.render(content_a, buf, state);
	}
}

fn render_no_tasks(area: Rect, buf: &mut Buffer, state: &mut AppState) {
	let area = area.x_h_margin(1);
	// -- Render the Error if there is one
	if let Some(err_id) = state.current_run_item().and_then(|r| r.run().end_err_id) {
		let mut link_zones = LinkZones::default();
		link_zones.set_current_line(0);

		let path_color = (state.debug_clr() != 0).then(|| ratatui::style::Color::Indexed(state.debug_clr()));

		let mut lines =
			super::comp::ui_for_err_with_hover(state.mm(), err_id, area.width.min(120), &mut link_zones, path_color);

		// -- Simple hover/click processing (no scroll in this view)
		let scroll: u16 = 0;
		let zones = link_zones.into_zones();

		// Detect hovered zone
		// Note: We look for the most specific zone (the one with the minimum span_count)
		let mut hovered_idx: Option<usize> = None;
		let mut min_span_count = usize::MAX;

		for (i, zone) in zones.iter().enumerate() {
			if let Some(line) = lines.get_mut(zone.line_idx)
				&& zone
					.is_mouse_over(area, scroll, state.last_mouse_evt(), &mut line.spans)
					.is_some() && zone.span_count < min_span_count
			{
				min_span_count = zone.span_count;
				hovered_idx = Some(i);
			}
		}

		// Apply hover style and process click
		if let Some(i) = hovered_idx {
			let action = zones[i].action.clone();
			let group_id = zones[i].group_id;

			match group_id {
				Some(gid) => {
					for z in zones.iter().filter(|z| z.group_id == Some(gid)) {
						if let Some(line) = lines.get_mut(z.line_idx)
							&& let Some(hover_spans) = z.spans_slice_mut(&mut line.spans)
						{
							for span in hover_spans {
								span.style.fg = Some(style::CLR_TXT_HOVER_TO_CLIP);
							}
						}
					}
				}
				None => {
					if let Some(line) = lines.get_mut(zones[i].line_idx)
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
				state.clear_mouse_evts();
			}
		}

		Paragraph::new(lines).render(area, buf);
	}
	// -- Else, check if there is a skip
	else if let Some(run_skip_reason) = state.current_run_item().and_then(|r| r.run().end_skip_reason.as_ref()) {
		let marker = ("■ Skip:", style::STL_SECTION_MARKER_SKIP);
		let path_color = (state.debug_clr() != 0).then(|| ratatui::style::Color::Indexed(state.debug_clr()));
		let line = ui_for_marker_section_str(run_skip_reason, marker, area.width, None, None, None, path_color);
		Paragraph::new(line).render(area, buf);
	} else {
		Paragraph::new("").render(area, buf);
	}
}

fn render_tasks_nav(area: Rect, buf: &mut Buffer, selection_in_view: bool, state: &mut AppState) {
	const SCROLL_IDEN: ScrollIden = RunTasksView::TASKS_NAV_SCROLL_IDEN;

	// -- Render background
	Block::new().bg(style::CLR_BKG_GRAY_DARKER).render(area, buf);

	// -- Layout before_all | Logs
	let [tasks_label_a, tasks_list_a] = Layout::default()
		.direction(Direction::Vertical)
		.constraints(vec![
			Constraint::Length(1), // tasks label
			Constraint::Fill(1),   // tasks list
		])
		.areas(area);

	// region:    --- Render Tasks Label
	let before_line = Line::default().spans(vec![
		// ..
		Span::raw(" Tasks:"),
	]);
	before_line.style(style::STL_FIELD_LBL).render(tasks_label_a, buf);
	// endregion: --- Render Tasks Label

	// -- Scroll & Select logic
	state.set_scroll_area(SCROLL_IDEN, tasks_list_a);
	let tasks_len = state.tasks().len();
	let mut scroll = state.clamp_scroll(SCROLL_IDEN, tasks_len);

	// -- Process UI Event
	// NOTE: Mouse processing (task selection) must occur before building the tasks UI to ensure the selection is up-to-date.
	//       This avoids unnecessary redraws, since we know the number of lines.
	// NOTE: In the future, we may want to trigger a redraw (similar to the runs nav) for consistency and to prevent
	//       inconsistent states. For now, tasks nav is rendered first, so this approach is acceptable.
	process_mouse_for_task_nav(state, tasks_list_a, scroll);

	// -- Build Tasks UI
	let tasks = state.tasks();
	let task_sel_idx = state.task_idx().unwrap_or_default();
	let is_mouse_in_nav = state.is_last_mouse_over(tasks_list_a);
	let items: Vec<ListItem> = tasks
		.iter()
		.enumerate()
		.map(|(idx, task)| {
			let mut line = Line::from(task.ui_label(Some(" "), area.width, tasks_len));
			if task_sel_idx == idx {
				line = line.style(style::STL_NAV_ITEM_HIGHLIGHT);
				line = line.x_fg(style::CLR_TXT_BLACK);
			} else if is_mouse_in_nav && state.is_last_mouse_over(tasks_list_a.x_row((idx + 1) as u16 - scroll)) {
				line = line.fg(style::CLR_TXT_HOVER);
			}
			ListItem::new(line)
		})
		.collect();
	let item_count = items.len() as u16;

	// -- Render with widget
	let list_w = List::new(items)
		// .highlight_style(styles::STL_NAV_ITEM_HIGHLIGHT)
		.highlight_spacing(HighlightSpacing::Always);

	if selection_in_view {
		let visible_top_idx = scroll as usize;
		let area_height = tasks_list_a.height as usize;

		let visible_end_idx = visible_top_idx + area_height.saturating_sub(1);

		if task_sel_idx >= visible_top_idx && task_sel_idx <= visible_end_idx {
			// nothing to do
		} else if task_sel_idx < visible_top_idx {
			scroll = task_sel_idx as u16;
			state.set_scroll(SCROLL_IDEN, scroll);
		} else if task_sel_idx > visible_end_idx {
			scroll = (task_sel_idx - area_height + 1) as u16;
			state.set_scroll(SCROLL_IDEN, scroll);
		}
	}

	let mut list_s = ListState::default().with_offset(scroll as usize);
	// list_s.select(state.task_idx());

	StatefulWidget::render(list_w, tasks_list_a, buf, &mut list_s);

	// -- Render scroll icons
	if item_count - scroll > tasks_list_a.height {
		let bottom_ico = tasks_list_a.x_bottom_right(1, 1);
		comp::ico_scroll_down().render(bottom_ico, buf);
	}
	if scroll > 0 && item_count > tasks_list_a.height - scroll {
		let top_ico = tasks_list_a.x_top_right(1, 1);
		comp::ico_scroll_up().render(top_ico, buf);
	}
}

// region:    --- Mouse Processing

fn process_mouse_for_task_nav(state: &mut AppState, nav_a: Rect, scroll: u16) {
	if let Some(mouse_evt) = state.mouse_evt()
		&& mouse_evt.is_up()
		&& mouse_evt.is_over(nav_a)
	{
		let new_idx = mouse_evt.y() - nav_a.y + scroll;
		let new_idx = clamp_idx_in_len(new_idx as usize, state.tasks().len());

		state.set_task_idx(Some(new_idx));
	}
}

// endregion: --- Mouse Processing
