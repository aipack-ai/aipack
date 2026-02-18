use super::{ActionView, ConfigView, InstallView, RunsView, SumView};
use crate::model::ErrRec;
use crate::tui::AppState;
use crate::tui::core::AppStage;
use crate::tui::view::{PopupOverlay, RunMainView, style};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Padding, Paragraph, StatefulWidget, Widget};

pub struct MainView;

impl StatefulWidget for MainView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		// -- Add background
		Block::new().bg(style::CLR_BKG_BLACK).render(area, buf);

		if let Some(err_rec) = state.sys_err_rec() {
			render_err(err_rec, buf, area);
			return;
		}

		// -- Layout
		let [header_a, _gap_a, content_a, action_a] = Layout::default()
			.direction(Direction::Vertical)
			.constraints(vec![
				Constraint::Length(1), // Header line
				Constraint::Length(1), // Gap
				Constraint::Fill(1),   // content
				Constraint::Length(1), // Action bar
			])
			.areas(area);

		// -- Render header
		SumView.render(header_a, buf, state);

		// -- Render main
		match state.stage() {
			AppStage::Normal | AppStage::Installing | AppStage::Installed | AppStage::PromptInstall(_) | AppStage::Config(_) => {
				if state.show_runs() {
					RunMainView::clear_scroll_idens(state);
					RunsView.render(content_a, buf, state);
				} else {
					RunsView::clear_scroll_idens(state);
					RunMainView.render(content_a, buf, state);
				}

				// Render Install/Installed/Prompt overlay (excluding Config)
				if matches!(state.stage(), AppStage::Installing | AppStage::Installed | AppStage::PromptInstall(_)) {
					InstallView.render(content_a, buf, state);
				}

				// Render Config overlay
				if let AppStage::Config(_) = state.stage() {
					ConfigView.render(content_a, buf, state);
				}
			}
		}

		// -- Render action
		ActionView.render(action_a, buf, state);

		// -- Render popup overlay last (on top)
		PopupOverlay.render(area, buf, state);
	}
}

fn render_err(err_rec: &ErrRec, buf: &mut Buffer, area: Rect) {
	let [content_a, _gap] = Layout::default()
		.direction(Direction::Vertical)
		.constraints(vec![
			Constraint::Fill(1), // content
			Constraint::Max(1),  // gap
		])
		.areas(area);

	let err_msg = err_rec
		.content
		.as_deref()
		.unwrap_or("Some unknown error happened. Quit and restart");
	Paragraph::new(err_msg)
		.block(
			Block::bordered()
				.border_style(style::CLR_TXT_RED)
				.padding(Padding::new(1, 1, 1, 1))
				.title("  ERROR  ")
				.title_alignment(Alignment::Center),
		)
		.centered()
		.render(content_a, buf);

	let line: Vec<Span> = vec![
		Span::raw("Press ["),
		Span::styled("q", style::CLR_BKG_BLUE),
		Span::raw("] to quit and restart"),
	];

	let [_, content_a, _] = Layout::default()
		.direction(Direction::Vertical)
		.constraints(vec![
			Constraint::Fill(1),   //
			Constraint::Length(1), // content
			Constraint::Length(1),
		])
		.areas(content_a);

	Paragraph::new(Line::from(line)).centered().render(content_a, buf);
}
