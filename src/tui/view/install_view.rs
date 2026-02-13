use crate::model::{LogBmc, WorkBmc};
use crate::tui::core::AppStage;
use crate::tui::view::comp;
use crate::tui::{AppState, style};
use crossterm::event::KeyCode;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::Stylize as _;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, Padding, Paragraph, StatefulWidget, Widget as _};

pub struct InstallView;

impl StatefulWidget for InstallView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		match state.stage() {
			AppStage::Installing => render_installing(area, buf, state),
			AppStage::Installed => render_installed(area, buf, state),
			AppStage::PromptInstall(work_id) => render_prompt_install(area, buf, work_id, state),
			_ => (),
		}
	}
}

fn render_prompt_install(area: Rect, buf: &mut Buffer, work_id: crate::model::Id, state: &mut AppState) {
	let pack_ref = state.installing_pack_ref().unwrap_or("Unknown pack");

	// Dialog layout
	let dialog_width = 60;
	let dialog_height = 10;

	let [_, mid_v, _] = Layout::default()
		.direction(Direction::Vertical)
		.constraints(vec![
			Constraint::Fill(1),
			Constraint::Length(dialog_height),
			Constraint::Length(4),
		])
		.areas(area);

	let [_, content_a, _] = Layout::default()
		.direction(Direction::Horizontal)
		.constraints(vec![
			Constraint::Fill(1),
			Constraint::Length(dialog_width),
			Constraint::Fill(1),
		])
		.areas(mid_v);

	// Clear and Background
	Clear.render(content_a, buf);

	let block = Block::bordered()
		.border_type(BorderType::Rounded)
		.border_style(style::CLR_TXT_WHITE)
		.bg(style::CLR_BKG_BLACK)
		.padding(Padding::new(2, 2, 1, 1))
		.title(Line::from("  Pack Missing  ").alignment(Alignment::Center));

	let inner_area = block.inner(content_a);
	block.render(content_a, buf);

	// Content layout
	let [msg_a, _gap, actions_a] = Layout::default()
		.direction(Direction::Vertical)
		.constraints(vec![Constraint::Fill(1), Constraint::Length(1), Constraint::Length(1)])
		.areas(inner_area);

	let lines = vec![
		Line::from(vec![
			Span::raw("Agent pack '"),
			Span::styled(pack_ref, style::STL_FIELD_VAL),
			Span::raw("' is not installed."),
		])
		.alignment(Alignment::Center),
		Line::default(),
		Line::from("Do you want to install it now?").alignment(Alignment::Center),
	];
	Paragraph::new(lines).render(msg_a, buf);

	// Render Action Bar
	let btn_a = comp::ActionBarBtn {
		label: "Cancel".to_string(),
		shortcut: KeyCode::Esc,
		action: crate::tui::core::UiAction::WorkCancel(work_id),
		extra_keys: vec![KeyCode::Char('c')],
	};
	let btn_b = comp::ActionBarBtn {
		label: "Install".to_string(),
		shortcut: KeyCode::Enter,
		action: crate::tui::core::UiAction::WorkConfirm(work_id),
		extra_keys: vec![KeyCode::Char('i')],
	};
	comp::ui_action_bar_ab(actions_a, buf, state, btn_a, btn_b);
}

fn render_installing(area: Rect, buf: &mut Buffer, state: &mut AppState) {
	let pack_ref = state.installing_pack_ref().unwrap_or("Unknown pack").to_string();
	let work_id = state.current_work_id();

	let mut detail_msg = String::new();
	if let Some(work_id) = work_id
		&& let Ok(work) = WorkBmc::get(state.mm(), work_id)
		&& let Some(msg) = work.message
	{
		detail_msg = msg;
	}

	let mut log_msg = String::new();
	if let Ok(logs) = LogBmc::list_for_run_only(state.mm(), 0.into())
		&& let Some(last_log) = logs.last()
		&& let Some(msg) = &last_log.message
	{
		log_msg = msg.clone();
	}

	// Indicator animation
	let dots = match state.running_tick_count().map(|c| c % 4) {
		Some(1) => ".  ",
		Some(2) => ".. ",
		Some(3) => "...",
		_ => "   ",
	};

	let mut lines = vec![
		Line::from(format!("Installing pack {dots}"))
			.alignment(Alignment::Center)
			.style(style::STL_POPUP_TITLE),
		Line::default(),
		Line::from(pack_ref).alignment(Alignment::Center).style(style::STL_FIELD_VAL),
		Line::default(),
	];

	// Detail or Log message
	if !detail_msg.is_empty() {
		lines.push(Line::from(detail_msg).alignment(Alignment::Center).style(style::STL_TXT));
	} else if !log_msg.is_empty() {
		lines.push(Line::from(log_msg).alignment(Alignment::Center).style(style::CLR_TXT_800));
	}

	render_dialog_base(area, buf, state, lines, None);
}

fn render_installed(area: Rect, buf: &mut Buffer, state: &mut AppState) {
	let work_id = state.current_work_id();
	let mut pack_info = "Unknown pack".to_string();

	if let Some(work_id) = work_id
		&& let Ok(work) = WorkBmc::get(state.mm(), work_id)
		&& let Some(msg) = work.message
	{
		pack_info = msg;
	}

	let lines = vec![
		Line::from("✔ Installed")
			.alignment(Alignment::Center)
			.style(style::CLR_TXT_DONE),
		Line::default(),
		Line::from(pack_info).alignment(Alignment::Center).style(style::STL_FIELD_VAL),
		Line::default(),
		Line::from("Pack successfully installed.").alignment(Alignment::Center),
	];

	render_dialog_base(area, buf, state, lines, work_id);
}

// region:    --- Support

fn render_dialog_base(
	area: Rect,
	buf: &mut Buffer,
	state: &mut AppState,
	lines: Vec<Line>,
	work_id: Option<crate::model::Id>,
) {
	// Dialog layout
	let dialog_width = 60;
	let dialog_height = 10;

	let [_, mid_v, _] = Layout::default()
		.direction(Direction::Vertical)
		.constraints(vec![
			Constraint::Fill(1),
			Constraint::Length(dialog_height),
			Constraint::Length(4), // Push it slightly higher
		])
		.areas(area);

	let [_, content_a, _] = Layout::default()
		.direction(Direction::Horizontal)
		.constraints(vec![
			Constraint::Fill(1),
			Constraint::Length(dialog_width),
			Constraint::Fill(1),
		])
		.areas(mid_v);

	// Clear and Background
	Clear.render(content_a, buf);

	let block = Block::bordered()
		.border_type(BorderType::Rounded)
		.border_style(style::CLR_TXT_WHITE)
		.bg(style::CLR_BKG_BLACK)
		.padding(Padding::new(2, 2, 1, 1));

	let inner_area = block.inner(content_a);
	block.render(content_a, buf);

	// Content layout (split between msg and buttons)
	let [msg_a, _gap, actions_a] = Layout::default()
		.direction(Direction::Vertical)
		.constraints(vec![Constraint::Fill(1), Constraint::Length(1), Constraint::Length(1)])
		.areas(inner_area);

	Paragraph::new(lines).render(msg_a, buf);

	// If we have a work_id and are in Installed stage, show buttons
	if let Some(work_id) = work_id
		&& state.stage() == AppStage::Installed
	{
		// Render Action Bar
		let btn_a = comp::ActionBarBtn {
			label: "Close".to_string(),
			shortcut: KeyCode::Esc,
			action: crate::tui::core::UiAction::WorkClose(work_id),
			extra_keys: vec![KeyCode::Char('x')],
		};
		let btn_b = comp::ActionBarBtn {
			label: "Run Agent".to_string(),
			shortcut: KeyCode::Enter,
			action: crate::tui::core::UiAction::WorkRun(work_id),
			extra_keys: vec![KeyCode::Char('r')],
		};
		comp::ui_action_bar_ab(actions_a, buf, state, btn_a, btn_b);
	}
}

// endregion: --- Support
