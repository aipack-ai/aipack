//! Convenient AppState getters and formatters for the View
//!
// region:    --- Imports
use crate::support::text::{self, format_duration_us, format_f64};
use crate::support::time::now_micro;
use crate::tui::{AppState, support};
// endregion: --- Imports

/// Implement Run Related Data extractors
impl AppState {
	pub fn current_run_duration_txt(&self) -> String {
		if let Some(run_item) = self.current_run_item() {
			let run = run_item.run();
			let duration_us = match (run.start, run.end) {
				(Some(start), Some(end)) => end.as_i64() - start.as_i64(),
				(Some(start), None) => now_micro() - start.as_i64(),
				_ => 0,
			};
			format_duration_us(duration_us)
		} else {
			format_duration_us(0)
		}
	}

	pub fn current_run_cost_fmt(&self) -> String {
		let cost = if let Some(run_item) = self.current_run_item() {
			// NOTE: Not very elegant, but works.
			let mut cost = run_item.run().total_cost;
			if run_item.has_children() {
				let children = self.all_run_children(run_item);
				let addl_cost: f64 = children.iter().filter_map(|r| r.run().total_cost).sum();
				if let Some(c) = cost.as_mut() {
					*c += addl_cost;
					cost
				} else if addl_cost != 0. {
					Some(addl_cost)
				} else {
					cost
				}
			} else {
				cost
			}
		} else {
			None
		};

		support::ui_fmt_cost(cost)
	}

	pub fn current_run_concurrency_txt(&self) -> String {
		if let Some(run_item) = self.current_run_item()
			&& let Some(concurrency) = run_item.run().concurrency
		{
			concurrency.to_string()
		} else {
			"-".to_string()
		}
	}

	/// Returns the agent name for the current run or `"no agent"` if none.
	pub fn current_run_agent_name(&self) -> String {
		self.current_run_item()
			.and_then(|r| r.run().agent_name.clone())
			.unwrap_or_else(|| "no agent".to_string())
	}

	/// Returns the model name for the current run or `"no model"` if none.
	pub fn current_run_model_name(&self) -> String {
		self.current_run_item()
			.and_then(|r| r.run().model.clone())
			.unwrap_or_else(|| "-".to_string())
	}

	/// Returns the cumulative duration of all tasks (formatted)
	/// or `None` when there is **one task or fewer**.
	pub fn tasks_cummulative_duration(&self) -> Option<String> {
		let run_item = self.current_run_item()?;
		let run = run_item.run();

		if let Some(concurrency) = run.concurrency
			&& concurrency <= 1
		{
			return None;
		};

		let tasks = self.tasks();
		if tasks.len() <= 1 {
			return None;
		}

		let mut cumul_us: i64 = 0;
		for task in tasks {
			let du = match (task.start, task.end) {
				(Some(start), Some(end)) => end.as_i64() - start.as_i64(),
				(Some(start), None) => now_micro() - start.as_i64(),
				_ => 0,
			};
			cumul_us += du;
		}

		Some(format_duration_us(cumul_us))
	}

	/// Returns a string describing the models used by the tasks of the
	/// current run.
	///
	/// Rules:
	/// - If all tasks share the same model, that model is returned.
	/// - If several different models are present, return the first two and
	///   append `+N` where `N` is the number of additional distinct models.
	/// - If no task has a model (or there are no tasks), fall back to the
	///   current run model (or `"no model"` when absent).
	/// - If is string over `max_width` then, right align (NOTE: Right now only 8bit char support)
	pub fn tasks_cummulative_models(&self, max_width: usize) -> String {
		let tasks = self.tasks();

		// Note: Will be 'no model' if no models
		let run_model = self.current_run_model_name();

		// Collect unique models while preserving encounter order.
		let mut uniques: Vec<String> = Vec::new();
		let mut some_no_ov = false;
		for task in tasks {
			// if the model skipped before AI, then it's model does not count
			// TODO: Probably need to process also when not skipped but no AI Process (e.g., when prompt becomes empty)
			if !task.is_skipped_before_ai() {
				if let Some(model) = &task.model_ov {
					// if the model is unique and not already added, we add it
					if model != &run_model && !uniques.contains(model) {
						uniques.push(model.clone());
					}
				} else {
					some_no_ov = true;
				}
			}
		}

		// NOTE: If tasks.len() == 0, uniques as well, so run_model (which is what we want. )
		let res = match (uniques.len(), some_no_ov) {
			(0, _) => run_model,
			(n, true) => format!("{run_model} +{n}"),
			(1, false) => uniques.into_iter().next().unwrap_or_default(),
			(n, false) => format!("{} +{}", uniques.into_iter().next().unwrap_or_default(), n - 1),
		};

		text::truncate_left_with_ellipsis(&res, max_width, "..").into_owned()
	}
}

/// Implement Task Related Data extractors
impl AppState {
	pub fn current_task_model_name(&self) -> String {
		self.current_task()
			.and_then(|r| r.model_ov.clone())
			.unwrap_or_else(|| self.current_run_model_name())
	}

	pub fn current_task_cost_fmt(&self) -> String {
		if let Some(task) = self.current_task()
			&& let Some(cost) = task.cost
		{
			format!("${}", format_f64(cost))
		} else {
			"$...".to_string()
		}
	}

	/// Returns the cumulative duration of all tasks (formatted)
	/// or `None` when there is **one task or fewer**.
	pub fn current_task_duration_txt(&self) -> String {
		if let Some(task) = self.current_task() {
			let duration = match (task.start, task.end) {
				(Some(start), Some(end)) => end.as_i64() - start.as_i64(),
				(Some(start), None) => now_micro() - start.as_i64(),
				_ => 0,
			};
			format_duration_us(duration)
		} else {
			"...".to_string()
		}
	}

	pub fn current_task_prompt_tokens_fmt(&self) -> String {
		let Some(task) = self.current_task() else {
			return "... tokens".to_string();
		};

		// -- If no prompt_total yet, display pending info (with eventual pricing)
		let Some(tk_prompt) = task.tk_prompt_total else {
			let mut msg = String::new();

			if let Some(prompt_size) = task.prompt_size {
				let size_fmt = simple_fs::pretty_size(prompt_size as u64);
				msg.push_str(&format!("({})", size_fmt.trim()));
			}

			if let Some(price) = task.pricing_input {
				if !msg.is_empty() {
					msg.push(' ');
				}
				msg.push_str(&format!("${price}/MTk"));
			} else {
				msg.push_str("...");
			}

			return msg;
		};

		// -- if some prompt_total show cost
		let mut res = format!("{} tk", text::format_num(tk_prompt));

		if let Some(tk_cached) = task.tk_prompt_cached
			&& tk_cached > 0
		{
			res.push_str(&format!(" ({} cached)", text::format_num(tk_cached)));
		}

		res
	}

	pub fn current_task_cache_info_fmt(&self) -> Option<String> {
		let task = self.current_task()?;

		let mut buf = String::new();

		if let Some(tk_cached) = task.tk_prompt_cached
			&& tk_cached > 0
			&& let Some(cache_saving) = task.cost_cache_saving
		{
			buf.push_str(&format!("-${cache_saving} saving"));
		}

		if let Some(tk_cache_creation) = task.tk_prompt_cache_creation
			&& tk_cache_creation > 0
		{
			let prefix = if buf.is_empty() { "" } else { " | " };
			let t = text::format_num(tk_cache_creation);
			buf.push_str(&format!("{prefix}{} tk write", t));
			if let Some(cost_cache_write) = task.cost_cache_write {
				buf.push_str(&format!(" (+${cost_cache_write} cost)"));
			}
		}

		if buf.is_empty() { None } else { Some(buf) }
	}

	pub fn current_task_completion_tokens_fmt(&self) -> String {
		let Some(task) = self.current_task() else {
			return "... tokens".to_string();
		};
		let Some(tk_completion) = task.tk_completion_total else {
			let mut msg = String::new();

			if let Some(price) = task.pricing_output {
				msg.push_str(&format!("${price}/MTk"));
			} else {
				msg.push_str("...");
			}

			return msg;
		};

		let mut res = format!("{} tk", text::format_num(tk_completion));

		if let Some(reasonning) = task.tk_completion_reasoning {
			res = format!("{res} ({} reason)", text::format_num(reasonning));
		}

		res
	}
}

/// Impl AppState formatters
impl AppState {
	pub fn db_memory_fmt(&self) -> String {
		match self.mm().db_size() {
			Ok(val) => text::format_pretty_size(val as u64, None),
			Err(_) => "_".to_string(),
		}
	}

	pub fn memory_fmt(&self) -> String {
		let mem = self.memory();
		text::format_pretty_size(mem, None)
	}

	#[allow(unused)]
	pub fn cpu_fmt(&self) -> String {
		let cpu = self.cpu();
		format!("{}%", text::format_percentage(cpu))
	}
}
