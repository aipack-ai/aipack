use ratatui::layout::{Position, Rect};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum ScrollIden {
	RunsNav,
	TasksNav,
	TaskContent,
	OverviewContent,
	GroupDashContent,
}

#[derive(Debug, Default)]
pub struct ScrollZone {
	area: Option<Rect>,
	// When set by the user
	scroll: Option<u16>,

	#[allow(unused)]
	is_bottom: bool,
}

/// Getters
impl ScrollZone {
	pub fn area(&self) -> Option<Rect> {
		self.area
	}

	pub fn scroll(&self) -> Option<u16> {
		self.scroll
	}

	#[allow(unused)]
	pub fn is_bottom(&self) -> bool {
		self.is_bottom
	}
}

/// Setters
impl ScrollZone {
	pub fn set_area(&mut self, area: Rect) {
		self.area = Some(area);
	}
	pub fn clear_area(&mut self) {
		self.area = None;
	}

	pub fn set_scroll(&mut self, scroll: u16) {
		self.scroll = Some(scroll);
	}
	#[allow(unused)]
	pub fn clear_scroll(&mut self) {
		self.scroll = None;
	}

	#[allow(unused)]
	pub fn set_is_bottom(&mut self, is_bottom: bool) {
		self.is_bottom = is_bottom;
	}
}

// region:    --- ScrollZones

#[derive(Debug)]
pub(in crate::tui::core) struct ScrollZones {
	pub zones: HashMap<ScrollIden, ScrollZone>,
}

impl Default for ScrollZones {
	fn default() -> Self {
		let mut zones = HashMap::new();
		zones.insert(ScrollIden::RunsNav, ScrollZone::default());
		zones.insert(ScrollIden::TasksNav, ScrollZone::default());
		zones.insert(ScrollIden::TaskContent, ScrollZone::default());
		zones.insert(ScrollIden::OverviewContent, ScrollZone::default());
		zones.insert(ScrollIden::GroupDashContent, ScrollZone::default());

		Self { zones }
	}
}

/// Immutable Getters & Finders
impl ScrollZones {
	pub fn find_zone_for_pos(&self, position: impl Into<Position>) -> Option<ScrollIden> {
		let position = position.into();
		self.zones
			.iter()
			.find(|(_, zone)| zone.area().is_some_and(|area| area.contains(position)))
			.map(|(iden, _)| *iden)
	}
}

// endregion: --- ScrollZones
