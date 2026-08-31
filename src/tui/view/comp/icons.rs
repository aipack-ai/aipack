use crate::model::{EndState, RunningState};
use crate::tui::style;
use ratatui::text::Span;

pub fn el_running_ico(arg: impl Into<RunningState>) -> Span<'static> {
	let arg = arg.into();

	match arg {
		RunningState::NotScheduled | RunningState::Unknown => Span::styled("·", style::CLR_TXT_WAITING),
		RunningState::Waiting => Span::styled("⏸", style::CLR_TXT_WAITING),
		RunningState::Running => Span::styled("▶", style::CLR_TXT_RUNNING),
		RunningState::Ended(end_state) => match end_state {
			Some(EndState::Ok) => Span::styled("✔", style::CLR_TXT_DONE),
			Some(EndState::Cancel) => Span::styled("■", style::CLR_TXT),
			Some(EndState::Skip) => Span::styled("■", style::CLR_TXT_SKIP),
			Some(EndState::Err) => Span::styled("✘", style::CLR_TXT_RED),
			None => Span::styled("?", style::CLR_TXT),
		},
	}
}

pub fn el_running_ico_with_flow(arg: impl Into<RunningState>, flow_redo_count: Option<i32>) -> Span<'static> {
	let arg = arg.into();

	match arg {
		RunningState::NotScheduled | RunningState::Unknown => Span::styled("·", style::CLR_TXT_WAITING),
		RunningState::Waiting => Span::styled("⏸", style::CLR_TXT_WAITING),
		RunningState::Running => Span::styled("▶", style::CLR_TXT_RUNNING),
		RunningState::Ended(end_state) => match end_state {
			Some(EndState::Ok) => {
				if flow_redo_count.is_some() && flow_redo_count.unwrap_or_default() > 0 {
					Span::styled("✔", style::CLR_TXT_DONE_LOOPED)
				} else {
					Span::styled("✔", style::CLR_TXT_DONE)
				}
			}
			Some(EndState::Cancel) => Span::styled("■", style::CLR_TXT),
			Some(EndState::Skip) => Span::styled("■", style::CLR_TXT_SKIP),
			Some(EndState::Err) => Span::styled("✘", style::CLR_TXT_RED),
			None => Span::styled("?", style::CLR_TXT),
		},
	}
}

pub fn ico_scroll_up() -> Span<'static> {
	Span::styled("▲", style::CLR_TXT_700)
}

pub fn ico_scroll_down() -> Span<'static> {
	Span::styled("▼", style::CLR_TXT_700)
}

pub fn ico_loop() -> Span<'static> {
	let icon = "↻";
	let icon = if Span::raw(icon).width() == 1 { icon } else { "o" };
	Span::styled(icon, style::CLR_TXT_BLUE)
}
