//! Defines the `run` and `task` pin helpers for Lua scripts.
//!
//! ---
//!
//! ## Lua documentation
//!
//! This module adds two helper functions for recording pins at runtime.
//!
//! ### Functions
//!
//! - `aip.run.pin(iden?: string, priority?: number, content?: any): integer`  
//!   Creates a pin attached to the current **run** (requires `CTX.RUN_UID` to be set).
//!
//! - `aip.task.pin(iden?: string, priority?: number, content?: any): integer`  
//!   Creates a pin attached to the current **task** (requires both `CTX.RUN_UID` and `CTX.TASK_UID`).
//!
//! The functions return the numeric database identifier of the created pin.

use crate::Result;
use crate::runtime::Runtime;
use crate::script::LuaValueExt;
use crate::store::rt_model::{PinBmc, PinForRunSave, PinForTaskSave, RuntimeCtx};
use mlua::{Lua, Table, Value, Variadic};
use serde_json;

/// Registers the `run.pin` and `task.pin` helpers in Lua.
pub fn init_module(lua: &Lua, runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	// -- run.pin
	{
		let rt = runtime.clone();
		let run_fn = lua.create_function(move |lua, args: Variadic<Value>| {
			create_pin(lua, &rt, /*for_task*/ false, args).map_err(mlua::Error::external)
		})?;
		let run_tbl = lua.create_table()?;
		run_tbl.set("pin", run_fn)?;
		table.set("run", run_tbl)?;
	}

	// -- task.pin
	{
		let rt = runtime.clone();
		let task_fn = lua.create_function(move |lua, args: Variadic<Value>| {
			create_pin(lua, &rt, /*for_task*/ true, args).map_err(mlua::Error::external)
		})?;
		let task_tbl = lua.create_table()?;
		task_tbl.set("pin", task_fn)?;
		table.set("task", task_tbl)?;
	}

	Ok(table)
}

// region:    --- Support

// -- PinCommand
// Captures the parsed arguments provided to the Lua `...pin(..)` helpers.
struct PinCommand {
	iden: String,
	priority: Option<f64>,
	content: String,
}

impl PinCommand {
	/// Parses the variadic Lua arguments for the two supported signatures:
	///
	/// 1. `pin(iden, priority, content)`
	/// 2. `pin(iden, content)`
	///
	/// Returns an informative error if the arguments do not match either form.
	fn from_lua_variadic(args: Variadic<Value>) -> Result<Self> {
		match args.len() {
			2 => {
				let iden = args
					.first()
					.and_then(Value::x_as_lua_str)
					.ok_or("aip...pin(iden, content) – expected <string> for parameter `iden`.")?;

				let content = args.get(1).ok_or("aip...pin(iden, content) – expected content.")?;
				let content = Self::value_to_string(content)?;

				Ok(Self {
					iden: iden.to_string(),
					priority: None,
					content,
				})
			}
			3 => {
				let iden = args
					.first()
					.and_then(Value::x_as_lua_str)
					.ok_or("aip...pin(iden, priority, content) – expected <string> for parameter `iden`.")?;

				let priority = args
					.get(1)
					.and_then(Value::x_as_f64)
					.ok_or("aip...pin(iden, priority, content) – expected <number> for parameter `priority`.")?;

				let content = args.get(2).ok_or("aip...pin(iden, priority, content) – expected content.")?;
				let content = Self::value_to_string(content)?;

				Ok(Self {
					iden: iden.to_string(),
					priority: Some(priority),
					content,
				})
			}
			_ => Err(crate::Error::custom(
				"aip...pin(...) – expected 2 or 3 parameters: (iden, content) or (iden, priority, content).",
			)),
		}
	}

	/// Converts a Lua `Value` into a `String` representation, serialising to JSON
	/// when the value is not already a string.
	fn value_to_string(val: &Value) -> Result<String> {
		match val {
			Value::String(s) => Ok(s.to_str()?.to_string()),
			Value::Nil => Err(crate::Error::custom(
				"aip...pin – `content` cannot be nil. Provide a string or any serialisable value.",
			)),
			other => {
				let json_val = serde_json::to_value(other)
					.map_err(|e| crate::Error::custom(format!("Cannot serialise content: {e}")))?;
				Ok(json_val.to_string())
			}
		}
	}
}

/// Shared implementation for both `run.pin` and `task.pin`.
fn create_pin(lua: &Lua, runtime: &Runtime, for_task: bool, args: Variadic<Value>) -> Result<i64> {
	let cmd = PinCommand::from_lua_variadic(args)?;

	let ctx = RuntimeCtx::extract_from_global(lua)?;

	let mm = runtime.mm();
	let (run_id, task_id) = {
		let run_id = ctx.get_run_id(mm)?.ok_or("Cannot create pin – no RUN context available")?;
		let task_id = if for_task { ctx.get_task_id(mm)? } else { None };
		(run_id, task_id)
	};

	let id = if for_task {
		let task_id = task_id.ok_or(
			"Cannot call 'aip.task.pin(...)' in a before all or after all code block.\nCall `aip.run.pin(..)`'",
		)?;
		let pin_c = PinForTaskSave {
			run_id,
			task_id,
			iden: cmd.iden,
			priority: cmd.priority,
			content: Some(cmd.content),
		};

		PinBmc::save_task_pin(mm, pin_c)?
	} else {
		let pin_c = PinForRunSave {
			run_id,
			iden: cmd.iden,
			priority: cmd.priority,
			content: Some(cmd.content),
		};

		PinBmc::save_run_pin(mm, pin_c)?
	};

	Ok(id.as_i64())
}

// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	use crate::_test_support::run_reflective_agent_with_runtime;
	use crate::runtime::Runtime;
	use crate::store::rt_model::PinBmc;

	#[tokio::test(flavor = "multi_thread")]
	async fn test_lua_run_pin_simple() -> Result<()> {
		// -- Setup & Fixtures
		let runtime = Runtime::new_test_runtime_sandbox_01().await?;
		let fx_code = r#"
aip.run.pin("some-iden", "Some pin content")		
return "OK"
			"#;

		// -- Exec
		let res = run_reflective_agent_with_runtime(fx_code, None, runtime.clone()).await?;

		// -- Check
		assert_eq!(res.as_str().unwrap_or_default(), "OK");
		// check pins
		let pins = PinBmc::list_for_run(runtime.mm(), 0.into())?;
		assert_eq!(pins.len(), 1);
		assert_eq!(pins[0].content.as_deref().unwrap_or_default(), "\"Some pin content\"");

		Ok(())
	}

	#[tokio::test(flavor = "multi_thread")]
	async fn test_lua_run_pin_with_priority() -> Result<()> {
		// -- Setup & Fixtures
		let runtime = Runtime::new_test_runtime_sandbox_01().await?;
		let fx_code = r#"
aip.run.pin("some-iden", 0.7, "Other content")
return "OK"
		"#;

		// -- Exec
		let res = run_reflective_agent_with_runtime(fx_code, None, runtime.clone()).await?;

		// -- Check
		assert_eq!(res.as_str().unwrap_or_default(), "OK");
		let pins = PinBmc::list_for_run(runtime.mm(), 0.into())?;
		assert_eq!(pins.len(), 1);
		assert_eq!(pins[0].priority, Some(0.7));

		Ok(())
	}
}

// endregion: --- Tests
