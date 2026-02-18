use crate::tui::AppState;
use crate::tui::core::ConfigTab;
use crate::tui::view::style;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::widgets::{Block, Paragraph, StatefulWidget, Tabs, Widget};

pub struct ConfigView;

impl StatefulWidget for ConfigView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		let current_tab = state.config_tab();

		// -- Layout centered popup
		let popup_area = Rect {
			x: area.x + 5,
			y: area.y + 2,
			width: area.width.saturating_sub(10),
			height: area.height.saturating_sub(4),
		};

		// -- Clear area (black background)
		Block::new().bg(style::CLR_BKG_BLACK).render(popup_area, buf);

		let block = Block::bordered()
			.border_style(style::CLR_TXT_BLUE)
			.title(" CONFIGURATION ")
			.title_style(style::STL_POPUP_TITLE);
		let inner_area = block.inner(popup_area);
		block.render(popup_area, buf);

		let [tabs_a, _gap, content_a] = Layout::default()
			.direction(Direction::Vertical)
			.constraints([
				Constraint::Length(1),
				Constraint::Length(1),
				Constraint::Fill(1),
			])
			.areas(inner_area);

		// -- Tabs
		let titles = vec![" [1] API Keys ", " [2] Model Aliases ", " [3] Help "];
		let selected_idx = match current_tab {
			ConfigTab::ApiKeys => 0,
			ConfigTab::ModelAliases => 1,
			ConfigTab::Help => 2,
		};

		Tabs::new(titles)
			.select(selected_idx)
			.highlight_style(style::STL_TAB_ACTIVE)
			.divider("|")
			.render(tabs_a, buf);

		// -- Content
		match current_tab {
			ConfigTab::ApiKeys => {
				Paragraph::new("API Keys Configuration Placeholder").render(content_a, buf);
			}
			ConfigTab::ModelAliases => {
				Paragraph::new("Model Aliases Configuration Placeholder").render(content_a, buf);
			}
			ConfigTab::Help => {
				Paragraph::new("TUI Help & Shortcuts Placeholder").render(content_a, buf);
			}
		}
	}
}

