use super::RunItem;
use crate::model::{Id, Loop};

// region:    --- Types

#[derive(Debug, Clone)]
pub struct RunNavGroup {
	pub loop_info: Loop,
	pub member_ids: Vec<Id>,
}

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum RunNavRow {
	LoopHeader { loop_info: Loop },
	Run { item: RunItem, loop_id: Option<Id> },
}

// endregion: --- Types

impl RunNavRow {
	#[allow(unused)]
	pub fn loop_info(&self) -> Option<&Loop> {
		match self {
			Self::LoopHeader { loop_info } => Some(loop_info),
			Self::Run { .. } => None,
		}
	}

	pub fn run_item(&self) -> Option<&RunItem> {
		match self {
			Self::LoopHeader { .. } => None,
			Self::Run { item, .. } => Some(item),
		}
	}

	#[allow(unused)]
	/// Returns the selectable real run ID, excluding non-selectable loop headers.
	pub fn run_id(&self) -> Option<Id> {
		self.run_item().map(RunItem::id)
	}

	#[allow(unused)]
	/// Returns the real run selected when this row is clicked.
	pub fn click_run_id(&self) -> Option<Id> {
		match self {
			Self::LoopHeader { loop_info } => Some(loop_info.last_run_id),
			Self::Run { item, .. } => Some(item.id()),
		}
	}

	#[allow(unused)]
	pub fn loop_id(&self) -> Option<Id> {
		match self {
			Self::LoopHeader { loop_info } => Some(loop_info.id),
			Self::Run { loop_id, .. } => *loop_id,
		}
	}
}
