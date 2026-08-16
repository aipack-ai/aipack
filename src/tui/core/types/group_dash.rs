#![allow(unused)]

use crate::model::{EpochUs, Id};

// region:    --- Types

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
}

#[derive(Debug, Clone, PartialEq)]
pub struct GroupDashRunEntry {
	pub run_id: Id,
	pub label: String,
	pub cost: f64,
	pub child_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GroupDashData {
	pub target: GroupDashTarget,
	pub mtime: EpochUs,
	pub total_cost: f64,
	pub top_runs_count: usize,
	pub all_runs_count: usize,
	pub top_runs: Vec<GroupDashRunEntry>,
	pub agents: Vec<GroupDashCostEntry>,
	pub models: Vec<GroupDashCostEntry>,
}

// endregion: --- Types

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
	pub fn new(name: impl Into<String>, cost: f64, count: usize) -> Self {
		Self {
			name: name.into(),
			cost,
			count,
		}
	}
}

// endregion: --- GroupDashCostEntry Impl

// region:    --- GroupDashRunEntry Impl

/// Constructors
impl GroupDashRunEntry {
	pub fn new(run_id: Id, label: impl Into<String>, cost: f64, child_count: usize) -> Self {
		Self {
			run_id,
			label: label.into(),
			cost,
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
		top_runs_count: usize,
		all_runs_count: usize,
		top_runs: Vec<GroupDashRunEntry>,
		agents: Vec<GroupDashCostEntry>,
		models: Vec<GroupDashCostEntry>,
	) -> Self {
		Self {
			target,
			mtime,
			total_cost,
			top_runs_count,
			all_runs_count,
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
		let top_run = GroupDashRunEntry::new(1.into(), "run-1", 0.05, 2);
		let agent = GroupDashCostEntry::new("agent-a", 0.03, 1);
		let model = GroupDashCostEntry::new("gpt-4o", 0.05, 2);

		let data = GroupDashData::new(
			target.clone(),
			mtime,
			0.05,
			1,
			3,
			vec![top_run],
			vec![agent],
			vec![model],
		);

		assert_eq!(data.target, target);
		assert_eq!(data.mtime, mtime);
		assert_eq!(data.total_cost, 0.05);
		assert_eq!(data.top_runs_count, 1);
		assert_eq!(data.all_runs_count, 3);
		assert_eq!(data.top_runs.len(), 1);
		assert_eq!(data.agents.len(), 1);
		assert_eq!(data.models.len(), 1);
	}
}

// endregion: --- Tests
