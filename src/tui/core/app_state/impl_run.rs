use crate::model::Id;
use crate::support::time::tick_count;
use crate::tui::core::{AppState, GroupDashData, GroupDashTab, GroupDashTarget, RunItem, RunNavRow, RunTab};
use crate::tui::support::offset_and_clamp_option_idx_in_len;

/// RunsView
impl AppState {
	pub fn running_tick_count(&self) -> Option<i64> {
		let running_start = self.core().running_tick_start?;

		let duration_micro = (self.core().time - running_start).max(0);
		let ticks = tick_count(duration_micro, 0.2);

		Some(ticks)
	}

	/// Running tick flag (true/false) when running
	pub fn running_tick_flag(&self) -> Option<bool> {
		let ticks = self.running_tick_count()?;

		Some((ticks / 3) % 2 == 0)
	}

	pub fn set_run_id(&mut self, run_id: Id) {
		self.core.set_run_by_id(run_id);
	}

	pub fn selected_loop_id(&self) -> Option<Id> {
		self.core.selected_loop_id
	}

	#[allow(unused)]
	pub fn is_selected_on_top_loop(&self) -> bool {
		if let Some(selected_loop_id) = self.selected_loop_id()
			&& let Some(top_loop_id) = self.core.run_item_store.top_loop_id()
		{
			selected_loop_id == top_loop_id
		} else {
			false
		}
	}

	#[allow(unused)]
	pub fn is_selected_on_top_run(&self) -> bool {
		if let Some(current_run) = self.current_run_item()
			&& let Some(top_run_id) = self.core.run_item_store.top_run_id()
		{
			current_run.id() == top_run_id
		} else {
			false
		}
	}

	#[allow(unused)]
	pub fn is_selected_on_top_run_of_top_loop(&self) -> bool {
		if let Some(current_run) = self.current_run_item()
			&& let Some(top_loop_run_id) = self.core.run_item_store.top_run_id_of_top_loop()
		{
			current_run.id() == top_loop_run_id
		} else {
			false
		}
	}

	pub fn set_loop_id(&mut self, loop_id: Id) {
		self.core.set_loop_by_id(loop_id);
	}

	pub fn run_items(&self) -> &[RunItem] {
		self.core.run_item_store.items()
	}

	pub fn current_run_item(&self) -> Option<&RunItem> {
		if let Some(idx) = self.core.run_idx {
			self.core.run_item_store.items().get(idx as usize)
		} else {
			None
		}
	}

	pub fn current_root_run_id(&self) -> Option<Id> {
		let run_item = self.current_run_item()?;

		if run_item.is_root() {
			Some(run_item.id())
		} else {
			run_item.ancestors().first().copied()
		}
	}

	/// Returns true when the current run belongs to a nested run tree.
	pub fn current_run_is_in_nested_run_tree(&self) -> bool {
		self.current_run_item()
			.map(|run_item| run_item.has_parent() || run_item.has_children())
			.unwrap_or_default()
	}

	pub fn visible_run_nav_rows(&self) -> Vec<&RunNavRow> {
		self.core.run_item_store.visible_nav_rows(self.current_root_run_id())
	}

	#[allow(unused)]
	pub fn visible_run_items_for_nav(&self) -> Vec<&RunItem> {
		self.visible_run_nav_rows().iter().filter_map(|row| row.run_item()).collect()
	}

	/// Move the run selection by `offset` within the currently visible nav list.
	/// This keeps keyboard navigation aligned with the visible rows so collapsed
	/// sub-run branches are skipped.
	pub fn offset_run_idx_in_visible_nav(&mut self, offset: i32) {
		let visible_rows = self.visible_run_nav_rows();
		let len = visible_rows.len();
		if len == 0 {
			return;
		}

		let current_visible_idx: Option<i32> = if let Some(selected_loop_id) = self.selected_loop_id() {
			visible_rows
				.iter()
				.position(|row| matches!(row, RunNavRow::LoopHeader { loop_info } if loop_info.id == selected_loop_id))
				.map(|i| i as i32)
		} else if let Some(current_run_id) = self.current_run_item().map(|r| r.id()) {
			visible_rows
				.iter()
				.position(|row| matches!(row, RunNavRow::Run { item, .. } if item.id() == current_run_id))
				.map(|i| i as i32)
		} else {
			None
		};

		let new_idx = offset_and_clamp_option_idx_in_len(&current_visible_idx, offset, len);
		if let Some(new_idx) = new_idx
			&& let Some(target_row) = visible_rows.get(new_idx as usize)
		{
			match target_row {
				RunNavRow::LoopHeader { loop_info } => self.set_loop_id(loop_info.id),
				RunNavRow::Run { item, .. } => self.set_run_id(item.id()),
			}
		}
	}

	#[allow(unused)]
	pub fn all_run_children<'a>(&'a self, run_item: &RunItem) -> Vec<&'a RunItem> {
		self.core.run_item_store.all_children(run_item)
	}

	#[allow(unused)]
	pub fn is_root_run(&self, run_item: &RunItem) -> bool {
		run_item.is_root()
	}

	pub fn run_tab(&self) -> RunTab {
		self.core.run_tab
	}

	pub fn set_run_tab(&mut self, run_tab: RunTab) {
		self.core.run_tab = run_tab;
	}

	#[allow(unused)]
	pub fn group_dash_tab(&self) -> GroupDashTab {
		self.core.group_dash_tab
	}

	#[allow(unused)]
	pub fn set_group_dash_tab(&mut self, tab: GroupDashTab) {
		self.core.group_dash_tab = tab;
	}

	#[allow(unused)]
	pub fn group_dash_data(&self) -> Option<&GroupDashData> {
		self.core.group_dash_data.as_ref()
	}

	#[allow(unused)]
	pub fn set_group_dash_data(&mut self, data: Option<GroupDashData>) {
		self.core.group_dash_data = data;
	}

	#[allow(unused)]
	pub fn clear_group_dash_data(&mut self) {
		self.core.group_dash_data = None;
	}

	#[allow(unused)]
	pub fn get_or_compute_group_dash_data(&mut self, target: &GroupDashTarget) -> Option<&GroupDashData> {
		let latest_mtime = self.core.run_item_store.latest_mtime_for_target(target);
		let needs_recompute = match &self.core.group_dash_data {
			Some(cached) => cached.target != *target || cached.mtime < latest_mtime,
			None => true,
		};
		if needs_recompute {
			self.core.group_dash_data = self.core.run_item_store.compute_group_dash_data(target);
		}
		self.core.group_dash_data.as_ref()
	}
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	use super::*;
	use crate::model::{EpochUs, LoopBmc, ModelManager, RunBmc, RunForCreate, RunForUpdate};
	use crate::tui::core::{RunItemStore, RunNavGroup};
	use crate::tui::core::event::LastAppEvent;

	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	#[tokio::test]
	async fn test_app_state_group_dash_caching_and_invalidation() -> Result<()> {
		// -- Setup & Fixtures
		let mm = ModelManager::new().await?;
		let first_run_id = RunBmc::create(&mm, run_for_test("first"))?;
		let loop_id = LoopBmc::create_for_first_member(&mm, first_run_id)?;
		let member_run_id = LoopBmc::create_member(&mm, loop_id, run_for_test("member"))?;
		LoopBmc::set_pending(&mm, loop_id, false)?;

		RunBmc::update(
			&mm,
			member_run_id,
			RunForUpdate {
				total_cost: Some(0.50),
				model: Some("gpt-4o".to_string()),
				agent_name: Some("test-agent".to_string()),
				..Default::default()
			},
		)?;

		let loop_info = LoopBmc::get(&mm, loop_id)?;
		let runs = vec![RunBmc::get(&mm, member_run_id)?, RunBmc::get(&mm, first_run_id)?];
		let store = RunItemStore::new_with_loops(
			runs,
			vec![RunNavGroup {
				loop_info,
				member_ids: vec![member_run_id, first_run_id],
			}],
		);

		let mut state = AppState::new(mm.clone(), LastAppEvent::default())?;
		state.core_mut().run_item_store = store;

		let target = GroupDashTarget::from_loop(loop_id);

		// -- Exec & Check 1: Initial computation and caching
		assert!(state.group_dash_data().is_none());
		let computed = state
			.get_or_compute_group_dash_data(&target)
			.ok_or("Should compute dash data")?
			.clone();
		assert_eq!(computed.top_runs_count, 2);
		assert!((computed.total_cost - 0.50).abs() < 1e-6);
		assert!(state.group_dash_data().is_some());

		// -- Exec & Check 2: Cache hit
		let cached = state
			.get_or_compute_group_dash_data(&target)
			.ok_or("Should return cached dash data")?;
		assert_eq!(cached.mtime, computed.mtime);

		// -- Exec & Check 3: Invalidation on mtime increase
		RunBmc::update(
			&mm,
			member_run_id,
			RunForUpdate {
				total_cost: Some(0.75),
				start: Some(EpochUs::from(2000i64)),
				..Default::default()
			},
		)?;
		let updated_runs = vec![RunBmc::get(&mm, member_run_id)?, RunBmc::get(&mm, first_run_id)?];
		let updated_store = RunItemStore::new_with_loops(
			updated_runs,
			vec![RunNavGroup {
				loop_info: LoopBmc::get(&mm, loop_id)?,
				member_ids: vec![member_run_id, first_run_id],
			}],
		);
		state.core_mut().run_item_store = updated_store;

		let refreshed = state
			.get_or_compute_group_dash_data(&target)
			.ok_or("Should recompute dash data")?;
		assert!((refreshed.total_cost - 0.75).abs() < 1e-6);

		// -- Exec & Check 4: Navigating to standard run clears cache
		state.set_run_id(first_run_id);
		assert!(state.group_dash_data().is_none());
		assert!(state.selected_loop_id().is_none());

		Ok(())
	}

	#[tokio::test]
	async fn test_app_state_navigation_offset_with_loops() -> Result<()> {
		// -- Setup & Fixtures
		let mm = ModelManager::new().await?;
		let first_run_id = RunBmc::create(&mm, run_for_test("first"))?;
		let loop_id = LoopBmc::create_for_first_member(&mm, first_run_id)?;
		let member_run_id = LoopBmc::create_member(&mm, loop_id, run_for_test("member"))?;
		LoopBmc::set_pending(&mm, loop_id, false)?;

		let loop_info = LoopBmc::get(&mm, loop_id)?;
		let runs = vec![RunBmc::get(&mm, member_run_id)?, RunBmc::get(&mm, first_run_id)?];
		let store = RunItemStore::new_with_loops(
			runs,
			vec![RunNavGroup {
				loop_info,
				member_ids: vec![member_run_id, first_run_id],
			}],
		);

		let mut state = AppState::new(mm, LastAppEvent::default())?;
		state.core_mut().run_item_store = store;

		// Start with loop header selected
		state.set_loop_id(loop_id);
		assert_eq!(state.selected_loop_id(), Some(loop_id));
		assert!(state.current_run_item().is_none());

		// -- Exec: Move forward to first member run
		state.offset_run_idx_in_visible_nav(1);
		assert_eq!(state.selected_loop_id(), None);
		assert_eq!(state.current_run_item().map(|r| r.id()), Some(member_run_id));

		// -- Exec: Move back up to loop header
		state.offset_run_idx_in_visible_nav(-1);
		assert_eq!(state.selected_loop_id(), Some(loop_id));
		assert!(state.current_run_item().is_none());

		Ok(())
	}

	#[tokio::test]
	async fn test_app_state_selection_position_helpers() -> Result<()> {
		// -- Setup & Fixtures
		let mm = ModelManager::new().await?;
		let first_run_id = RunBmc::create(&mm, run_for_test("first"))?;
		let loop_id = LoopBmc::create_for_first_member(&mm, first_run_id)?;
		let member_run_id = LoopBmc::create_member(&mm, loop_id, run_for_test("member"))?;
		LoopBmc::set_pending(&mm, loop_id, false)?;

		let loop_info = LoopBmc::get(&mm, loop_id)?;
		let runs = vec![RunBmc::get(&mm, member_run_id)?, RunBmc::get(&mm, first_run_id)?];
		let store = RunItemStore::new_with_loops(
			runs,
			vec![RunNavGroup {
				loop_info,
				member_ids: vec![member_run_id, first_run_id],
			}],
		);

		let mut state = AppState::new(mm, LastAppEvent::default())?;
		state.core_mut().run_item_store = store;

		// -- Check loop header selection
		state.set_loop_id(loop_id);
		assert!(state.is_selected_on_top_loop());
		assert!(!state.is_selected_on_top_run());
		assert!(!state.is_selected_on_top_run_of_top_loop());

		// -- Check top run of top loop selection
		state.set_run_id(member_run_id);
		assert!(!state.is_selected_on_top_loop());
		assert!(state.is_selected_on_top_run());
		assert!(state.is_selected_on_top_run_of_top_loop());

		// -- Check older run selection
		state.set_run_id(first_run_id);
		assert!(!state.is_selected_on_top_loop());
		assert!(!state.is_selected_on_top_run());
		assert!(!state.is_selected_on_top_run_of_top_loop());

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

	// endregion: --- Support
}

// endregion: --- Tests
