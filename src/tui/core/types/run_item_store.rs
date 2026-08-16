use crate::model::{EpochUs, Id, Loop, Run};
use crate::tui::core::{GroupDashCostEntry, GroupDashData, GroupDashRunEntry, GroupDashTarget, RunItem, RunNavGroup, RunNavRow};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default, Clone)]
pub struct RunItemStore {
	items: Vec<RunItem>,
	items_by_id: HashMap<Id, RunItem>,
	nav_rows: Vec<RunNavRow>,
}

impl RunItemStore {
	/// Returns the flat list of `RunItem`s.
	pub fn items(&self) -> &[RunItem] {
		&self.items
	}

	#[allow(unused)]
	pub fn nav_rows(&self) -> &[RunNavRow] {
		&self.nav_rows
	}

	#[allow(unused)]
	pub fn top_loop_id(&self) -> Option<Id> {
		self.nav_rows.iter().find_map(|row| row.loop_info().map(|l| l.id))
	}

	#[allow(unused)]
	pub fn top_run_id(&self) -> Option<Id> {
		self.nav_rows.iter().find_map(|row| row.run_id())
	}

	#[allow(unused)]
	pub fn top_run_id_of_top_loop(&self) -> Option<Id> {
		let top_loop_id = self.top_loop_id()?;
		self.nav_rows.iter().find_map(|row| match row {
			RunNavRow::Run { item, loop_id: Some(lid) } if *lid == top_loop_id => Some(item.id()),
			_ => None,
		})
	}

	pub fn visible_nav_rows(&self, root_id: Option<Id>) -> Vec<&RunNavRow> {
		self.nav_rows
			.iter()
			.filter(|row| match row {
				RunNavRow::LoopHeader { .. } => true,
				RunNavRow::Run { item, .. } => {
					item.is_root()
						|| root_id.is_some_and(|root_id| item.belongs_to_root_branch(root_id))
				}
			})
			.collect()
	}

	#[allow(unused)]
	pub fn visible_items_for_root_branch(&self, root_id: Option<Id>) -> Vec<&RunItem> {
		self.items
			.iter()
			.filter(|item| item.is_root() || root_id.is_some_and(|root_id| item.belongs_to_root_branch(root_id)))
			.collect()
	}

	#[allow(unused)]
	pub fn get(&self, id: Id) -> Option<&RunItem> {
		self.items_by_id.get(&id)
	}

	/// Returns a list of direct children for a given `RunItem`.
	/// The children are ordered by their creation time (oldest first).
	#[allow(unused)]
	pub fn direct_children<'a>(&'a self, parent_item: &RunItem) -> Vec<&'a RunItem> {
		self.items
			.iter()
			.filter(|item| item.parent_id() == Some(parent_item.id()))
			.collect()
	}

	/// Returns all children (direct and indirect) for a given `RunItem`.
	pub fn all_children<'a>(&'a self, parent_item: &RunItem) -> Vec<&'a RunItem> {
		let children_ids: HashSet<Id> = parent_item.all_children_ids().iter().copied().collect();
		if children_ids.is_empty() {
			return Vec::new();
		}

		self.items.iter().filter(|item| children_ids.contains(&item.id())).collect()
	}

	#[allow(unused)]
	pub fn target_root_items(&self, target: &GroupDashTarget) -> Vec<&RunItem> {
		let loop_ids = target.loop_ids();
		let run_ids = target.run_ids();

		let mut roots = Vec::new();
		let mut seen_ids = HashSet::new();

		for row in &self.nav_rows {
			if let RunNavRow::Run { item, loop_id } = row
				&& item.is_root()
			{
				let matches_loop = loop_id.is_some_and(|lid| loop_ids.contains(&lid))
					|| item.run().loop_id.is_some_and(|lid| loop_ids.contains(&lid));
				let matches_run = run_ids.contains(&item.id());

				if (matches_loop || matches_run) && seen_ids.insert(item.id()) {
					roots.push(item);
				}
			}
		}

		for &run_id in run_ids {
			if let Some(item) = self.items_by_id.get(&run_id)
				&& seen_ids.insert(item.id())
			{
				roots.push(item);
			}
		}

		roots
	}

	#[allow(unused)]
	pub fn latest_mtime_for_target(&self, target: &GroupDashTarget) -> EpochUs {
		let roots = self.target_root_items(target);
		let mut max_mtime = EpochUs::from(0i64);

		for root in roots {
			if root.run().mtime > max_mtime {
				max_mtime = root.run().mtime;
			}
			for child_id in root.all_children_ids() {
				if let Some(child) = self.items_by_id.get(child_id)
					&& child.run().mtime > max_mtime
				{
					max_mtime = child.run().mtime;
				}
			}
		}

		max_mtime
	}

	#[allow(unused)]
	pub fn compute_group_dash_data(&self, target: &GroupDashTarget) -> Option<GroupDashData> {
		let roots = self.target_root_items(target);
		if roots.is_empty() && target.is_empty() {
			return None;
		}

		let mut all_unique_run_ids = HashSet::new();
		let mut top_runs = Vec::new();
		let mut max_mtime = EpochUs::from(0i64);
		let mut total_duration_us: Option<i64> = None;
		let mut cumul_task_duration_us: Option<i64> = None;

		for root in &roots {
			all_unique_run_ids.insert(root.id());
			if root.run().mtime > max_mtime {
				max_mtime = root.run().mtime;
			}

			if let Some(dur) = root.run().duration_us() {
				total_duration_us = Some(total_duration_us.unwrap_or(0) + dur);
			}

			let mut subtree_cost = root.run().total_cost.unwrap_or(0.0);
			let top_cost = subtree_cost;
			let mut child_count = 0;
			let top_duration_us = root.run().duration_us();
			let mut root_total_duration_us = top_duration_us;

			for child_id in root.all_children_ids() {
				if let Some(child) = self.items_by_id.get(child_id) {
					all_unique_run_ids.insert(child.id());
					subtree_cost += child.run().total_cost.unwrap_or(0.0);
					child_count += 1;
					if let Some(child_dur) = child.run().duration_us() {
						root_total_duration_us = Some(root_total_duration_us.unwrap_or(0) + child_dur);
					}
					if child.run().mtime > max_mtime {
						max_mtime = child.run().mtime;
					}
				}
			}

			let label = root
				.run()
				.label
				.clone()
				.or_else(|| root.run().agent_name.clone())
				.unwrap_or_else(|| format!("Run {}", root.id()));

			top_runs.push(GroupDashRunEntry::new(
				root.id(),
				label,
				subtree_cost,
				top_cost,
				root_total_duration_us,
				top_duration_us,
				child_count,
			));
		}

		let mut total_cost = 0.0;
		let mut agent_map: HashMap<String, (f64, usize, Option<i64>)> = HashMap::new();
		let mut model_map: HashMap<String, (f64, usize, Option<i64>)> = HashMap::new();

		for run_id in &all_unique_run_ids {
			if let Some(item) = self.items_by_id.get(run_id) {
				let run = item.run();
				let cost = run.total_cost.unwrap_or(0.0);
				let duration = run.duration_us();
				total_cost += cost;

				if let Some(task_ms) = run.total_task_ms {
					cumul_task_duration_us = Some(cumul_task_duration_us.unwrap_or(0) + task_ms * 1000);
				}

				let agent_name = run.agent_name.clone().unwrap_or_else(|| "Unknown".to_string());
				let agent_entry = agent_map.entry(agent_name).or_insert((0.0, 0, None));
				agent_entry.0 += cost;
				agent_entry.1 += 1;
				if let Some(dur) = duration {
					agent_entry.2 = Some(agent_entry.2.unwrap_or(0) + dur);
				}

				let model_name = run.model.clone().unwrap_or_else(|| "Unknown".to_string());
				let model_entry = model_map.entry(model_name).or_insert((0.0, 0, None));
				model_entry.0 += cost;
				model_entry.1 += 1;
				if let Some(dur) = duration {
					model_entry.2 = Some(model_entry.2.unwrap_or(0) + dur);
				}
			}
		}

		let mut agents: Vec<GroupDashCostEntry> = agent_map
			.into_iter()
			.map(|(name, (cost, count, dur))| GroupDashCostEntry::new(name, cost, count, dur))
			.collect();
		agents.sort_by(|a, b| {
			b.cost
				.partial_cmp(&a.cost)
				.unwrap_or(std::cmp::Ordering::Equal)
				.then_with(|| a.name.cmp(&b.name))
		});

		let mut models: Vec<GroupDashCostEntry> = model_map
			.into_iter()
			.map(|(name, (cost, count, dur))| GroupDashCostEntry::new(name, cost, count, dur))
			.collect();
		models.sort_by(|a, b| {
			b.cost
				.partial_cmp(&a.cost)
				.unwrap_or(std::cmp::Ordering::Equal)
				.then_with(|| a.name.cmp(&b.name))
		});

		let top_runs_count = top_runs.len();
		let all_runs_count = all_unique_run_ids.len();

		Some(GroupDashData::new(
			target.clone(),
			max_mtime,
			total_cost,
			total_duration_us,
			cumul_task_duration_us,
			top_runs_count,
			all_runs_count,
			top_runs,
			agents,
			models,
		))
	}
}

/// Contrustor
impl RunItemStore {
	#[allow(unused)]
	pub fn new(runs: Vec<Run>) -> Self {
		Self::new_with_loops(runs, Vec::new())
	}

	pub fn new_with_loops(runs: Vec<Run>, loop_groups: Vec<RunNavGroup>) -> Self {
		// -- Early Exit
		if runs.is_empty() {
			return RunItemStore::default();
		}

		// -- Build Roots & Children Map
		let mut children_map: HashMap<Id, Vec<Run>> = HashMap::new();
		let mut root_runs: Vec<Run> = Vec::new();

		for run in runs {
			if let Some(parent_id) = run.parent_id {
				children_map.entry(parent_id).or_default().push(run);
			} else {
				root_runs.push(run); // Keep original (most-recent-first) order.
			}
		}

		// -- Recursively Flatten
		fn push_with_children(
			out: &mut Vec<RunItem>,
			children_map: &mut HashMap<Id, Vec<Run>>,
			run: Run,
			indent: u32,
			ancestors: &[Id],
		) {
			let id = run.id;
			// This is the item for the current run
			out.push(RunItem::new(run, indent, ancestors.to_vec()));

			if let Some(mut kids) = children_map.remove(&id) {
				// Oldest → Newest
				kids.sort_by_key(|r| r.id);

				// The ancestors for all the direct children of this run.
				let mut child_ancestors = ancestors.to_vec();
				child_ancestors.push(id);

				for child in kids {
					push_with_children(out, children_map, child, indent + 1, &child_ancestors);
				}
			}
		}

		let mut flat: Vec<RunItem> = Vec::new();

		for run in root_runs {
			push_with_children(&mut flat, &mut children_map, run, 0, &[]);
		}

		// -- Orphan Handling (if any)
		if !children_map.is_empty() {
			let mut remaining: Vec<Run> = children_map.into_values().flatten().collect();
			remaining.sort_by_key(|r| r.id);
			for run in remaining {
				// Note: orphans will have an empty ancestor list (besides themselve)
				push_with_children(&mut flat, &mut HashMap::new(), run, 0, &[]);
			}
		}

		// -- Populate `all_children_ids` for each `RunItem`
		//    Iterate in reverse order (from children to parents) to build the `all_children_ids` map.
		let mut all_children_ids_by_id: HashMap<Id, Vec<Id>> = HashMap::new();

		// Create a map of items by parent_id for efficient lookup of direct children.
		let mut direct_children_by_parent_id: HashMap<Id, Vec<Id>> = HashMap::new();
		for item in &flat {
			if let Some(parent_id) = item.parent_id() {
				direct_children_by_parent_id.entry(parent_id).or_default().push(item.id());
			}
		}

		for item in flat.iter().rev() {
			let mut all_children_ids = Vec::new();

			// Look up direct children.
			if let Some(mut direct_children) = direct_children_by_parent_id.get(&item.id()).cloned() {
				direct_children.sort(); // Sort to be consistent with original logic
				for child_id in &direct_children {
					all_children_ids.push(*child_id);
					// Add grandchildren from the map we are building.
					if let Some(grand_children_ids) = all_children_ids_by_id.get(child_id) {
						all_children_ids.extend_from_slice(grand_children_ids);
					}
				}
			}
			all_children_ids_by_id.insert(item.id(), all_children_ids);
		}

		// Now, update the `all_children_ids` for each item in the `flat` vec.
		for item in &mut flat {
			if let Some(ids) = all_children_ids_by_id.get(&item.id()) {
				item.all_children_ids = ids.clone();
			}
		}

		let items_by_id = flat.iter().map(|item| (item.id(), item.clone())).collect();
		let nav_rows = build_nav_rows(&flat, &items_by_id, loop_groups);

		RunItemStore {
			items: flat,
			items_by_id,
			nav_rows,
		}
	}
}

// region:    --- Support

#[derive(Debug)]
enum RootNavEntry {
	Loop { loop_info: Loop, member_ids: Vec<Id> },
	Run { run_id: Id },
}

impl RootNavEntry {
	fn newest_id(&self) -> Id {
		match self {
			Self::Loop { loop_info, .. } => loop_info.last_run_id,
			Self::Run { run_id } => *run_id,
		}
	}
}

fn build_nav_rows(
	flat: &[RunItem],
	items_by_id: &HashMap<Id, RunItem>,
	loop_groups: Vec<RunNavGroup>,
) -> Vec<RunNavRow> {
	let mut grouped_member_ids: HashSet<Id> = HashSet::new();
	let mut root_entries: Vec<RootNavEntry> = Vec::new();

	for loop_group in loop_groups {
		let mut member_ids: Vec<Id> = loop_group
			.member_ids
			.into_iter()
			.filter(|id| items_by_id.get(id).is_some_and(|item| item.is_root()))
			.collect();
		member_ids.sort_by_key(|id| std::cmp::Reverse(*id));
		member_ids.dedup();

		if member_ids.is_empty() {
			continue;
		}

		grouped_member_ids.extend(member_ids.iter().copied());
		root_entries.push(RootNavEntry::Loop {
			loop_info: loop_group.loop_info,
			member_ids,
		});
	}

	for item in flat
		.iter()
		.filter(|item| item.is_root() && !grouped_member_ids.contains(&item.id()))
	{
		root_entries.push(RootNavEntry::Run { run_id: item.id() });
	}

	root_entries.sort_by_key(|entry| std::cmp::Reverse(entry.newest_id()));

	let mut rows = Vec::new();
	for entry in root_entries {
		match entry {
			RootNavEntry::Loop { loop_info, member_ids } => {
				let loop_id = loop_info.id;
				rows.push(RunNavRow::LoopHeader { loop_info });

				for member_id in member_ids {
					if let Some(item) = items_by_id.get(&member_id) {
						push_run_nav_item(&mut rows, item, Some(loop_id), items_by_id);
					}
				}
			}
			RootNavEntry::Run { run_id } => {
				if let Some(item) = items_by_id.get(&run_id) {
					push_run_nav_item(&mut rows, item, None, items_by_id);
				}
			}
		}
	}

	rows
}

fn push_run_nav_item(
	rows: &mut Vec<RunNavRow>,
	item: &RunItem,
	loop_id: Option<Id>,
	items_by_id: &HashMap<Id, RunItem>,
) {
	rows.push(RunNavRow::Run {
		item: item.clone(),
		loop_id,
	});

	for child_id in item.all_children_ids() {
		if let Some(child) = items_by_id.get(child_id) {
			rows.push(RunNavRow::Run {
				item: child.clone(),
				loop_id,
			});
		}
	}
}

// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	use super::*;
	use crate::model::{LoopBmc, ModelManager, RunBmc, RunForCreate};

	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	#[tokio::test]
	async fn test_tui_core_types_run_item_store_new_with_loops_keeps_membership() -> Result<()> {
		// -- Setup & Fixtures
		let mm = ModelManager::new().await?;
		let standalone_before_id = RunBmc::create(&mm, run_for_test("standalone-before"))?;
		let first_run_id = RunBmc::create(&mm, run_for_test("first"))?;
		let loop_id = LoopBmc::create_for_first_member(&mm, first_run_id)?;
		let loop_member_id = LoopBmc::create_member(&mm, loop_id, run_for_test("member"))?;
		let standalone_between_id = RunBmc::create(&mm, run_for_test("standalone-between"))?;
		let loop_member_child_id = RunBmc::create(&mm, child_run_for_test(loop_member_id, "member-child"))?;
		let standalone_after_id = RunBmc::create(&mm, run_for_test("standalone-after"))?;
		LoopBmc::set_pending(&mm, loop_id, false)?;

		let loop_info = LoopBmc::get(&mm, loop_id)?;
		let runs = vec![
			RunBmc::get(&mm, standalone_after_id)?,
			RunBmc::get(&mm, standalone_between_id)?,
			RunBmc::get(&mm, loop_member_child_id)?,
			RunBmc::get(&mm, loop_member_id)?,
			RunBmc::get(&mm, standalone_before_id)?,
			RunBmc::get(&mm, first_run_id)?,
		];

		// -- Exec
		let store = RunItemStore::new_with_loops(
			runs,
			vec![RunNavGroup {
				loop_info,
				member_ids: vec![loop_member_id, first_run_id, loop_member_id],
			}],
		);
		let nav_rows = store.nav_rows();

		// -- Check
		assert_eq!(nav_rows.len(), 7);
		let loop_header_count = nav_rows.iter().filter(|row| row.loop_info().is_some()).count();
		assert_eq!(loop_header_count, 1);

		let first_row = nav_rows.first().ok_or("Should have a first navigation row")?;
		assert_eq!(first_row.run_id(), Some(standalone_after_id));

		let standalone_between = nav_rows.get(1).ok_or("Should have a middle standalone run")?;
		assert_eq!(standalone_between.run_id(), Some(standalone_between_id));

		let loop_header = nav_rows.get(2).ok_or("Should have a loop header")?;
		let loop_info = loop_header.loop_info().ok_or("Row should be a loop header")?;
		assert_eq!(loop_info.id, loop_id);
		assert!(!loop_info.pending);
		assert_eq!(loop_header.click_run_id(), Some(loop_member_id));

		let loop_member_row = nav_rows.get(3).ok_or("Should have a loop member")?;
		let loop_member_item = loop_member_row.run_item().ok_or("Row should be a run")?;
		assert_eq!(loop_member_item.id(), loop_member_id);
		assert_eq!(loop_member_item.indent(), 0);

		let child_row = nav_rows.get(4).ok_or("Should have a loop member child")?;
		let child_item = child_row.run_item().ok_or("Row should be a child run")?;
		assert_eq!(child_item.id(), loop_member_child_id);
		assert_eq!(child_item.parent_id(), Some(loop_member_id));
		assert_eq!(child_item.indent(), loop_member_item.indent() + 1);

		let grouped_ids = nav_rows
			.iter()
			.filter(|row| row.loop_id() == Some(loop_id))
			.filter_map(|row| row.run_id())
			.collect::<Vec<_>>();
		assert_eq!(grouped_ids, vec![loop_member_id, loop_member_child_id, first_run_id]);

		let standalone_ids = nav_rows
			.iter()
			.filter(|row| row.loop_id().is_none())
			.filter_map(|row| row.run_id())
			.collect::<Vec<_>>();
		assert_eq!(
			standalone_ids,
			vec![standalone_after_id, standalone_between_id, standalone_before_id]
		);

		Ok(())
	}

	#[tokio::test]
	async fn test_tui_core_types_run_item_store_visible_nav_rows_expands_selected_branch() -> Result<()> {
		// -- Setup & Fixtures
		let mm = ModelManager::new().await?;
		let standalone_root_id = RunBmc::create(&mm, run_for_test("standalone-root"))?;
		let standalone_child_id =
			RunBmc::create(&mm, child_run_for_test(standalone_root_id, "standalone-child"))?;
		let first_run_id = RunBmc::create(&mm, run_for_test("first"))?;
		let loop_id = LoopBmc::create_for_first_member(&mm, first_run_id)?;
		let loop_member_id = LoopBmc::create_member(&mm, loop_id, run_for_test("member"))?;
		let first_child_id = RunBmc::create(&mm, child_run_for_test(first_run_id, "first-child"))?;
		let loop_member_child_id = RunBmc::create(&mm, child_run_for_test(loop_member_id, "member-child"))?;
		LoopBmc::set_pending(&mm, loop_id, false)?;

		let loop_info = LoopBmc::get(&mm, loop_id)?;
		let runs = vec![
			RunBmc::get(&mm, loop_member_child_id)?,
			RunBmc::get(&mm, first_child_id)?,
			RunBmc::get(&mm, standalone_child_id)?,
			RunBmc::get(&mm, loop_member_id)?,
			RunBmc::get(&mm, first_run_id)?,
			RunBmc::get(&mm, standalone_root_id)?,
		];

		// -- Exec
		let store = RunItemStore::new_with_loops(
			runs,
			vec![RunNavGroup {
				loop_info,
				member_ids: vec![loop_member_id, first_run_id],
			}],
		);
		let visible_loop_member_rows = store.visible_nav_rows(Some(loop_member_id));
		let visible_first_rows = store.visible_nav_rows(Some(first_run_id));
		let visible_standalone_rows = store.visible_nav_rows(Some(standalone_root_id));
		let visible_loop_member_ids = run_ids(&visible_loop_member_rows);
		let visible_first_ids = run_ids(&visible_first_rows);
		let visible_standalone_ids = run_ids(&visible_standalone_rows);

		// -- Check
		assert_eq!(
			visible_loop_member_rows
				.iter()
				.filter(|row| row.loop_info().is_some())
				.count(),
			1
		);
		assert_eq!(visible_loop_member_ids.len(), 4);
		assert!(visible_loop_member_ids.contains(&loop_member_id));
		assert!(visible_loop_member_ids.contains(&first_run_id));
		assert!(visible_loop_member_ids.contains(&standalone_root_id));
		assert!(visible_loop_member_ids.contains(&loop_member_child_id));
		assert!(!visible_loop_member_ids.contains(&first_child_id));
		assert!(!visible_loop_member_ids.contains(&standalone_child_id));

		assert_eq!(visible_first_ids.len(), 4);
		assert!(visible_first_ids.contains(&loop_member_id));
		assert!(visible_first_ids.contains(&first_run_id));
		assert!(visible_first_ids.contains(&standalone_root_id));
		assert!(visible_first_ids.contains(&first_child_id));
		assert!(!visible_first_ids.contains(&loop_member_child_id));
		assert!(!visible_first_ids.contains(&standalone_child_id));

		assert_eq!(visible_standalone_ids.len(), 4);
		assert!(visible_standalone_ids.contains(&loop_member_id));
		assert!(visible_standalone_ids.contains(&first_run_id));
		assert!(visible_standalone_ids.contains(&standalone_root_id));
		assert!(visible_standalone_ids.contains(&standalone_child_id));
		assert!(!visible_standalone_ids.contains(&loop_member_child_id));
		assert!(!visible_standalone_ids.contains(&first_child_id));

		Ok(())
	}

	#[tokio::test]
	async fn test_tui_core_types_run_item_store_compute_group_dash_data() -> Result<()> {
		// -- Setup & Fixtures
		let mm = ModelManager::new().await?;
		let first_run_id = RunBmc::create(&mm, run_for_test("first"))?;
		let loop_id = LoopBmc::create_for_first_member(&mm, first_run_id)?;
		let loop_member_id = LoopBmc::create_member(&mm, loop_id, run_for_test("member"))?;
		let loop_member_child_id = RunBmc::create(&mm, child_run_for_test(loop_member_id, "member-child"))?;
		LoopBmc::set_pending(&mm, loop_id, false)?;

		// Set cost and model on member run
		crate::model::RunBmc::update(
			&mm,
			loop_member_id,
			crate::model::RunForUpdate {
				total_cost: Some(0.12),
				model: Some("gpt-4o".to_string()),
				agent_name: Some("agent-alpha".to_string()),
				start: Some(0.into()),
				end: Some(1_000_000.into()),
				total_task_ms: Some(800),
				..Default::default()
			},
		)?;

		// Set cost and model on child run
		crate::model::RunBmc::update(
			&mm,
			loop_member_child_id,
			crate::model::RunForUpdate {
				total_cost: Some(0.08),
				model: Some("gpt-4o-mini".to_string()),
				agent_name: Some("agent-beta".to_string()),
				start: Some(0.into()),
				end: Some(500_000.into()),
				total_task_ms: Some(400),
				..Default::default()
			},
		)?;

		let loop_info = LoopBmc::get(&mm, loop_id)?;
		let runs = vec![
			RunBmc::get(&mm, loop_member_child_id)?,
			RunBmc::get(&mm, loop_member_id)?,
			RunBmc::get(&mm, first_run_id)?,
		];

		let store = RunItemStore::new_with_loops(
			runs,
			vec![RunNavGroup {
				loop_info,
				member_ids: vec![loop_member_id, first_run_id],
			}],
		);

		// -- Exec
		let target = GroupDashTarget::from_loop(loop_id);
		let dash_data = store.compute_group_dash_data(&target).ok_or("Should compute dash data")?;

		// -- Check
		assert_eq!(dash_data.top_runs_count, 2);
		assert_eq!(dash_data.all_runs_count, 3);
		assert!((dash_data.total_cost - 0.20).abs() < 1e-6);
		assert_eq!(dash_data.total_duration_us, Some(1_000_000));
		assert_eq!(dash_data.cumul_task_duration_us, Some(1_200_000));
		assert_eq!(dash_data.top_runs.len(), 2);

		let member_top = dash_data.top_runs.iter().find(|r| r.run_id == loop_member_id).ok_or("Member top run missing")?;
		assert_eq!(member_top.child_count, 1);
		assert!((member_top.cost - 0.20).abs() < 1e-6);
		assert!((member_top.top_cost - 0.12).abs() < 1e-6);
		assert_eq!(member_top.top_duration_us, Some(1_000_000));
		assert_eq!(member_top.total_duration_us, Some(1_500_000));

		assert_eq!(dash_data.agents.len(), 2);
		let alpha_agent = dash_data.agents.iter().find(|a| a.name == "agent-alpha").ok_or("Alpha agent missing")?;
		assert_eq!(alpha_agent.total_duration_us, Some(1_000_000));
		assert_eq!(alpha_agent.count, 1);

		assert_eq!(dash_data.models.len(), 2);
		let gpt4o_model = dash_data.models.iter().find(|m| m.name == "gpt-4o").ok_or("GPT-4o model missing")?;
		assert_eq!(gpt4o_model.total_duration_us, Some(1_000_000));

		Ok(())
	}

	#[tokio::test]
	async fn test_tui_core_types_run_item_store_compute_group_dash_data_without_task_ms() -> Result<()> {
		// -- Setup & Fixtures
		let mm = ModelManager::new().await?;
		let standalone_id = RunBmc::create(&mm, run_for_test("standalone"))?;

		crate::model::RunBmc::update(
			&mm,
			standalone_id,
			crate::model::RunForUpdate {
				total_cost: Some(0.05),
				start: Some(100.into()),
				end: Some(500_100.into()),
				total_task_ms: None,
				..Default::default()
			},
		)?;

		let runs = vec![RunBmc::get(&mm, standalone_id)?];
		let store = RunItemStore::new(runs);

		// -- Exec
		let target = GroupDashTarget::from_run(standalone_id);
		let dash_data = store.compute_group_dash_data(&target).ok_or("Should compute dash data")?;

		// -- Check
		assert_eq!(dash_data.top_runs_count, 1);
		assert_eq!(dash_data.all_runs_count, 1);
		assert_eq!(dash_data.total_duration_us, Some(500_000));
		assert_eq!(dash_data.cumul_task_duration_us, None);

		Ok(())
	}

	#[tokio::test]
	async fn test_tui_core_types_run_item_store_top_positions() -> Result<()> {
		// -- Setup & Fixtures
		let mm = ModelManager::new().await?;
		let standalone_id = RunBmc::create(&mm, run_for_test("standalone"))?;
		let first_run_id = RunBmc::create(&mm, run_for_test("first"))?;
		let loop_id = LoopBmc::create_for_first_member(&mm, first_run_id)?;
		let member_run_id = LoopBmc::create_member(&mm, loop_id, run_for_test("member"))?;
		LoopBmc::set_pending(&mm, loop_id, false)?;

		let loop_info = LoopBmc::get(&mm, loop_id)?;
		let runs = vec![
			RunBmc::get(&mm, member_run_id)?,
			RunBmc::get(&mm, first_run_id)?,
			RunBmc::get(&mm, standalone_id)?,
		];

		let store = RunItemStore::new_with_loops(
			runs,
			vec![RunNavGroup {
				loop_info,
				member_ids: vec![member_run_id, first_run_id],
			}],
		);

		// -- Exec & Check
		assert_eq!(store.top_loop_id(), Some(loop_id));
		assert_eq!(store.top_run_id(), Some(member_run_id));
		assert_eq!(store.top_run_id_of_top_loop(), Some(member_run_id));

		Ok(())
	}

	// region:    --- Support

	fn run_for_test(label: &str) -> RunForCreate {
		RunForCreate {
			parent_id: None,
			agent_name: Some(label.to_string()),
			agent_path: Some(format!("path/{label}")),
			has_task_stages: None,
			has_prompt_parts: None,
		}
	}

	fn child_run_for_test(parent_id: Id, label: &str) -> RunForCreate {
		RunForCreate {
			parent_id: Some(parent_id),
			agent_name: Some(label.to_string()),
			agent_path: Some(format!("path/{label}")),
			has_task_stages: None,
			has_prompt_parts: None,
		}
	}

	fn run_ids(rows: &[&RunNavRow]) -> Vec<Id> {
		rows.iter().filter_map(|row| row.run_id()).collect()
	}

	// endregion: --- Support
}

// endregion: --- Tests
