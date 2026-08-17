#![allow(unused)]

use crate::model::{EpochUs, Id};

// region:    --- Types

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GroupDashTab {
	#[default]
	TopRuns,
	Agents,
	Models,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroupDashTarget {
	Loops(Vec<Id>),
	Runs(Vec<Id>),
	Mixed { loop_ids: Vec<Id>, run_ids: Vec<Id> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct GroupDashCostEntry {
	pub name: String,
	pub cost: f64,
	pub count: usize,
	pub total_duration_us: Option<i64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GroupDashRunEntry {
	pub run_id: Id,
	pub label: String,
	pub is_running: bool,
	pub cost: f64,
	pub top_cost: f64,
	pub total_duration_us: Option<i64>,
	pub top_duration_us: Option<i64>,
	pub child_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GroupDashData {
	pub target: GroupDashTarget,
	pub mtime: EpochUs,
	pub total_cost: f64,
	pub total_duration_us: Option<i64>,
	pub cumul_task_duration_us: Option<i64>,
	pub top_runs_count: usize,
	pub all_runs_count: usize,
	pub has_active_runs: bool,
	pub top_runs: Vec<GroupDashRunEntry>,
	pub agents: Vec<GroupDashCostEntry>,
	pub models: Vec<GroupDashCostEntry>,
}

// endregion: --- Types

// region:    --- GroupDashTab Impl

impl GroupDashTab {
	pub fn next(self) -> Self {
		match self {
			Self::TopRuns => Self::Agents,
			Self::Agents => Self::Models,
			Self::Models => Self::TopRuns,
		}
	}

	pub fn prev(self) -> Self {
		match self {
			Self::TopRuns => Self::Models,
			Self::Agents => Self::TopRuns,
			Self::Models => Self::Agents,
		}
	}
}

// endregion: --- GroupDashTab Impl

// region:    --- GroupDashTarget Impl

/// Constructors
impl GroupDashTarget {
	pub fn from_loop(loop_id: Id) -> Self {
		Self::Loops(vec![loop_id])
	}

	pub fn from_run(run_id: Id) -> Self {
		Self::Runs(vec![run_id])
	}

	pub fn from_loops(loop_ids: Vec<Id>) -> Self {
		Self::Loops(loop_ids)
	}

	pub fn from_runs(run_ids: Vec<Id>) -> Self {
		Self::Runs(run_ids)
	}

	pub fn from_mixed(loop_ids: Vec<Id>, run_ids: Vec<Id>) -> Self {
		Self::Mixed { loop_ids, run_ids }
	}
}

/// Accessors
impl GroupDashTarget {
	pub fn loop_ids(&self) -> &[Id] {
		match self {
			Self::Loops(ids) => ids,
			Self::Mixed { loop_ids, .. } => loop_ids,
			Self::Runs(_) => &[],
		}
	}

	pub fn run_ids(&self) -> &[Id] {
		match self {
			Self::Runs(ids) => ids,
			Self::Mixed { run_ids, .. } => run_ids,
			Self::Loops(_) => &[],
		}
	}

	pub fn is_empty(&self) -> bool {
		match self {
			Self::Loops(ids) => ids.is_empty(),
			Self::Runs(ids) => ids.is_empty(),
			Self::Mixed { loop_ids, run_ids } => loop_ids.is_empty() && run_ids.is_empty(),
		}
	}
}

// endregion: --- GroupDashTarget Impl

// region:    --- GroupDashCostEntry Impl

/// Constructors
impl GroupDashCostEntry {
	pub fn new(name: impl Into<String>, cost: f64, count: usize, total_duration_us: Option<i64>) -> Self {
		Self {
			name: name.into(),
			cost,
			count,
			total_duration_us,
		}
	}
}

// endregion: --- GroupDashCostEntry Impl

// region:    --- GroupDashRunEntry Impl

/// Constructors
impl GroupDashRunEntry {
	#[allow(clippy::too_many_arguments)]
	pub fn new(
		run_id: Id,
		label: impl Into<String>,
		is_running: bool,
		cost: f64,
		top_cost: f64,
		total_duration_us: Option<i64>,
		top_duration_us: Option<i64>,
		child_count: usize,
	) -> Self {
		Self {
			run_id,
			label: label.into(),
			is_running,
			cost,
			top_cost,
			total_duration_us,
			top_duration_us,
			child_count,
		}
	}
}

// endregion: --- GroupDashRunEntry Impl

// region:    --- GroupDashData Impl

/// Constructors
impl GroupDashData {
	#[allow(clippy::too_many_arguments)]
	pub fn new(
		target: GroupDashTarget,
		mtime: EpochUs,
		total_cost: f64,
		total_duration_us: Option<i64>,
		cumul_task_duration_us: Option<i64>,
		top_runs_count: usize,
		all_runs_count: usize,
		has_active_runs: bool,
		top_runs: Vec<GroupDashRunEntry>,
		agents: Vec<GroupDashCostEntry>,
		models: Vec<GroupDashCostEntry>,
	) -> Self {
		Self {
			target,
			mtime,
			total_cost,
			total_duration_us,
			cumul_task_duration_us,
			top_runs_count,
			all_runs_count,
			has_active_runs,
			top_runs,
			agents,
			models,
		}
	}
}

// endregion: --- GroupDashData Impl

// region:    --- Tests

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_group_dash_tab_cycling() {
		let tab = GroupDashTab::TopRuns;
		assert_eq!(tab.next(), GroupDashTab::Agents);
		assert_eq!(tab.next().next(), GroupDashTab::Models);
		assert_eq!(tab.next().next().next(), GroupDashTab::TopRuns);

		assert_eq!(tab.prev(), GroupDashTab::Models);
		assert_eq!(tab.prev().prev(), GroupDashTab::Agents);
		assert_eq!(tab.prev().prev().prev(), GroupDashTab::TopRuns);
	}

	#[test]
	fn test_group_dash_target_accessors() {
		let loop_id_1: Id = 10.into();
		let loop_id_2: Id = 11.into();
		let run_id_1: Id = 100.into();

		let loop_target = GroupDashTarget::from_loop(loop_id_1);
		assert_eq!(loop_target.loop_ids(), &[loop_id_1]);
		assert_eq!(loop_target.run_ids(), &[]);
		assert!(!loop_target.is_empty());

		let mixed_target =
			GroupDashTarget::from_mixed(vec![loop_id_1, loop_id_2], vec![run_id_1]);
		assert_eq!(mixed_target.loop_ids(), &[loop_id_1, loop_id_2]);
		assert_eq!(mixed_target.run_ids(), &[run_id_1]);
		assert!(!mixed_target.is_empty());

		let empty_target = GroupDashTarget::from_loops(Vec::new());
		assert!(empty_target.is_empty());
	}

	#[test]
	fn test_group_dash_data_constructor() {
		let target = GroupDashTarget::from_loop(1.into());
		let mtime: EpochUs = 1000.into();
		let top_run = GroupDashRunEntry::new(1.into(), "run-1", false, 0.05, 0.03, Some(500), Some(300), 2);
		let agent = GroupDashCostEntry::new("agent-a", 0.03, 1, Some(300));
		let model = GroupDashCostEntry::new("gpt-4o", 0.05, 2, Some(500));

		let data = GroupDashData::new(
			target.clone(),
			mtime,
			0.05,
			Some(500),
			Some(400),
			1,
			3,
			false,
			vec![top_run],
			vec![agent],
			vec![model],
		);

		assert_eq!(data.target, target);
		assert_eq!(data.mtime, mtime);
		assert_eq!(data.total_cost, 0.05);
		assert_eq!(data.total_duration_us, Some(500));
		assert_eq!(data.cumul_task_duration_us, Some(400));
		assert_eq!(data.top_runs_count, 1);
		assert_eq!(data.all_runs_count, 3);
		assert!(!data.has_active_runs);
		assert_eq!(data.top_runs.len(), 1);
		assert_eq!(data.agents.len(), 1);
		assert_eq!(data.models.len(), 1);
	}
}

// endregion: --- Tests
