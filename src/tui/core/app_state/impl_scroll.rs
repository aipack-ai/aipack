use crate::tui::core::{AppState, ScrollIden};
use ratatui::layout::Rect;

/// Scroll
impl AppState {
	pub fn set_scroll_area(&mut self, iden: ScrollIden, area: Rect) {
		if let Some(zone) = self.core.get_zone_mut(&iden) {
			zone.set_area(area);
		}
	}

	pub fn clear_scroll_zone_area(&mut self, iden: &ScrollIden) {
		if let Some(zone) = self.core.get_zone_mut(iden) {
			zone.clear_area();
		}
	}

	pub fn clear_scroll_zone_areas(&mut self, idens: &[&ScrollIden]) {
		for iden in idens {
			self.clear_scroll_zone_area(iden);
		}
	}

	pub fn set_scroll(&mut self, iden: ScrollIden, scroll: u16) {
		if let Some(zone) = self.core.get_zone_mut(&iden) {
			zone.set_scroll(scroll);
		}
	}

	/// Note: will return 0 if no scroll was set yet
	#[allow(unused)]
	pub fn get_scroll(&self, iden: ScrollIden) -> u16 {
		self.core.get_scroll(iden)
	}

	pub fn clamp_scroll(&mut self, iden: ScrollIden, line_count: usize) -> u16 {
		let Some(scroll_zone) = self.core.get_zone_mut(&iden) else {
			return 0;
		};
		let area_height = scroll_zone.area().map(|a| a.height).unwrap_or_default();
		let max_scroll = line_count.saturating_sub(area_height as usize).min(u16::MAX as usize) as u16;
		let scroll = scroll_zone.scroll().unwrap_or_default();
		if scroll > max_scroll {
			scroll_zone.set_scroll(max_scroll);
			max_scroll
		} else {
			scroll
		}
	}
}
