use crate::tui::AppState;
use crate::tui::core::{AppStage, ConfigTab};
use crate::tui::view::style;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Paragraph, StatefulWidget, Tabs, Widget};

pub struct ConfigView;

impl StatefulWidget for ConfigView {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
		let AppStage::Config(current_tab) = state.stage() else {
			return;
		};

		// -- Calculate popup area
		// horizontal margin 5, vertical line 3 to -2
		let popup_area = Rect {
			x: area.x + 5,
			y: area.y + 2,
			width: area.width.saturating_sub(10),
			height: area.height.saturating_sub(4),
		};

		// -- Block
		let block = Block::bordered()
			.bg(style::CLR_BKG_BLACK)
			.border_style(style::STL_POPUP_TITLE)
			.title("  Configuration  ")
			.title_alignment(Alignment::Center);

		let inner_area = block.inner(popup_area);
		block.render(popup_area, buf);

		// -- Layout
		let [tabs_a, _gap, content_a] = Layout::default()
			.direction(Direction::Vertical)
			.constraints(vec![
				Constraint::Length(1), // Tabs
				Constraint::Length(1), // Gap
				Constraint::Fill(1),   // Content
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
			.style(style::STL_TAB_DEFAULT)
			.render(tabs_a, buf);

		// -- Content
		match current_tab {
			ConfigTab::ApiKeys => {
				Paragraph::new("API Keys placeholder content").render(content_a, buf);
			}
			ConfigTab::ModelAliases => {
				Paragraph::new("Model Aliases placeholder content").render(content_a, buf);
			}
			ConfigTab::Help => {
				Paragraph::new("Help placeholder content").render(content_a, buf);
			}
		}
	}
}
