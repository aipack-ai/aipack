use crate::support::text::format_f64;

pub fn ui_fmt_cost(cost: Option<f64>) -> String {
	if let Some(cost) = cost {
		format!("${}", format_f64(cost))
	} else {
		"-".to_string()
	}
}

pub fn ui_fmt_duration_us(duration_us: Option<i64>) -> String {
	let Some(us) = duration_us else {
		return "-".to_string();
	};
	if us < 0 {
		return "-".to_string();
	}
	let ms = us / 1000;
	if ms < 1000 {
		format!("{ms}ms")
	} else if ms < 60_000 {
		let sec = ms as f64 / 1000.0;
		format!("{sec:.1}s")
	} else {
		let total_sec = ms / 1000;
		let mins = total_sec / 60;
		let rem_sec = total_sec % 60;
		if mins < 60 {
			format!("{mins}m {rem_sec}s")
		} else {
			let hours = mins / 60;
			let rem_mins = mins % 60;
			format!("{hours}h {rem_mins}m")
		}
	}
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;

	#[test]
	fn test_formatters_ui_fmt_cost_simple() -> Result<()> {
		// -- Setup & Fixtures
		let val_none: Option<f64> = None;
		let val_some = Some(1.234);

		// -- Exec & Check
		assert_eq!(ui_fmt_cost(val_none), "-");
		assert_eq!(ui_fmt_cost(val_some), "$1.23");

		Ok(())
	}

	#[test]
	fn test_formatters_ui_fmt_duration_us_various() -> Result<()> {
		// -- Setup & Fixtures
		let dur_none: Option<i64> = None;
		let dur_ms = Some(450_000);
		let dur_sec = Some(2_500_000);
		let dur_min = Some(125_000_000);
		let dur_hour = Some(3_665_000_000);

		// -- Exec & Check
		assert_eq!(ui_fmt_duration_us(dur_none), "-");
		assert_eq!(ui_fmt_duration_us(dur_ms), "450ms");
		assert_eq!(ui_fmt_duration_us(dur_sec), "2.5s");
		assert_eq!(ui_fmt_duration_us(dur_min), "2m 5s");
		assert_eq!(ui_fmt_duration_us(dur_hour), "1h 1m");

		Ok(())
	}
}

// endregion: --- Tests
