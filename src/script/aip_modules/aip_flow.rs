//! Defines the `aip_flow` module, used in the lua engine.
//!
//! ---
//!
//! ## Lua Documentation
//!
//! The `aip.flow` module allows controlling the flow of AIPACK agent execution.
//! The flow functions are designed to be returned so that the Agent Executor can act appropriately.
//!
//! ### Functions
//!
//! - `aip.flow.before_all_response(data: BeforeAllData) -> table`
//! - `aip.flow.data_response(data: DataData) -> table`
//! - `aip.flow.skip(reason?: string) -> table`

use crate::Result;
use crate::runtime::Runtime;
use mlua::{Lua, Table, Value};

pub fn init_module(lua: &Lua, _runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	let data_response_fn = lua.create_function(aipack_data_response)?;
	table.set("data_response", data_response_fn)?;

	let before_all_response_fn = lua.create_function(aipack_before_all_response)?;
	table.set("before_all_response", before_all_response_fn)?;

	let redo_fn = lua.create_function(aipack_redo)?;
	table.set("redo", redo_fn)?;

	let skip_fn = lua.create_function(aipack_skip)?;
	table.set("skip", skip_fn)?;

	Ok(table)
}

// region: --- Lua Functions

/// ## Lua Documentation
///
/// Customize the aipack execution flow at the 'Before All' stage.
///
/// This function is typically called within the `before_all` block of an agent script
/// to override the default behavior of passing all initial inputs to the agent.
///
/// ```lua
/// -- API Signature
/// aip.flow.before_all_response(data: BeforeAllData) -> table
/// ```
///
/// ### Arguments
///
/// - `data: table` - A table defining the new inputs and options for the agent execution cycle.
///   ```ts
///   type BeforeAllData = {
///     inputs?:  any[],        // Optional. A list of new inputs to use for the agent run cycle. Overrides initial inputs.
///     options?: AgentOptions, // Optional. Partial AgentOptions to override for this run.
///     before_all?: any,       // Optional. The before_all data that can be access via before_all...
///   } & any // Can also include other arbitrary data fields if needed.
///   ```
///
///
/// ### Example
///
/// ```lua
/// local result = aip.flow.before_all_response({
///   inputs = {"processed_input_1", "processed_input_2"},
///   options = {
///     model = "gemini-2.5-flash",
///     input_concurrency = 3
///   },
///   before_all = {some_data = "hello world" } -- Arbitrary data is allowed
/// })
/// -- The agent executor will process this result table.
/// ```
///
/// ### Error
///
/// This function does not directly return any errors. Errors might occur during the creation of lua table.
///
/// ## Internal (not for Lua doc
///
/// Internaly this returns a Lua table with a specific structure recognized by AIPACK's executor.
/// This return value does not typically need to be captured or used by the script itself;
/// it serves as a directive for the AIPACK execution engine.
///
/// ```ts
/// type AipackFlowResponse = {
///   _aipack_: {
///     kind: "BeforeAllResponse",
///     data: BeforeAllData // The data table passed as argument
///   }
/// }
/// ```
fn aipack_before_all_response(lua: &Lua, data: Value) -> mlua::Result<Value> {
	let inner = lua.create_table()?;
	inner.set("kind", "BeforeAllResponse")?;
	inner.set("data", data)?;
	let outer = lua.create_table()?;
	outer.set("_aipack_", inner)?;

	Ok(Value::Table(outer))
}

/// ## Lua Documentation
///
/// Customize the aipack execution flow at the 'Data' stage for a single input.
///
/// This function is typically called within the `data` block of an agent script.
/// It allows overriding the input and/or options for the current input cycle,
/// or returning additional arbitrary data.
///
/// ```lua
/// -- API Signature
/// aip.flow.data_response(data: DataData) -> table
/// ```
///
/// ### Arguments
///
/// - `data: table` - A table defining the new input, options, and/or other data for the current cycle.
///   ```ts
///   type DataData = {
///     input?: any | nil,     // Optional. The new input to use for this cycle. If nil, the original input is used.
///     data?: any | nil,      // Data that will be available in the next stage. Same as returning a simple data.
///     options?: AgentOptions // Optional. Partial AgentOptions to override for this cycle.
///   } & any // Can also include other arbitrary data fields (e.g., computed values, flags)
///   ```
///
/// ### Example
///
/// ```lua
/// -- Use a transformed input and override the model for this cycle
/// return aip.flow.data_response({
///   data  = data,              -- The data that would have been returned
///   input = transformed_input,
///   options = { model = "gpt-5" },
/// })
/// -- The agent executor will process this result table.
/// ```
///
/// ### Error
///
/// This function does not directly return any errors. Errors might occur during the creation of lua table.
///
/// ## Internal (not for Lua doc)
///
/// Internaly this returns a Lua table with a specific structure recognized by AIPACK's executor.
/// This return value does not typically need to be captured or used by the script itself;
/// it serves as a directive for the AIPACK execution engine.
///
/// ```ts
/// type AipackFlowResponse = {
///   _aipack_: {
///     kind: "DataResponse",
///     data: DataData // The data table passed as argument
///   }
/// }
/// ```
fn aipack_data_response(lua: &Lua, data: Value) -> mlua::Result<Value> {
	let inner = lua.create_table()?;
	inner.set("kind", "DataResponse")?;
	inner.set("data", data)?;
	let outer = lua.create_table()?;
	outer.set("_aipack_", inner)?;

	Ok(Value::Table(outer))
}

/// ## Lua Documentation
///
/// Returns a response indicating a skip action for the current input cycle.
///
/// This function is typically called within the `data` block of an agent script
/// to instruct AIPACK to skip processing the current input value and move to the next one.
///
/// ```lua
/// -- API Signature
/// aip.flow.skip(reason?: string) -> table
/// ```
///
/// ### Arguments
///
/// - `reason: string (optional)`: An optional string providing the reason for skipping the input cycle.
///   This reason might be logged or displayed depending on the AIPACK execution context.
///
/// ### Example
///
/// ```lua
/// -- Skip processing if the input is nil or empty
/// if input == nil or input == "" then
///   return aip.flow.skip("Input is empty")
/// end
/// -- Continue processing the input if not skipped
/// -- ... rest of data block ...
/// ```
///
/// ### Error
///
/// This function does not directly return any errors. Errors might occur during the creation of lua table.
///
/// ## Internal (not for Lua doc)
///
/// Internaly this returns a Lua table with a specific structure recognized by AIPACK's executor.
/// This return value does not typically need to be captured or used by the script itself;
/// it serves as a directive for the AIPACK execution engine.
///
/// ```ts
/// type AipackFlowResponse = {
///   _aipack_: {
///     kind: "Skip",
///     data: {
///       reason: string | nil // The optional reason provided
///     }
///   }
/// }
/// ```
fn aipack_skip(lua: &Lua, reason: Option<String>) -> mlua::Result<Value> {
	let data = lua.create_table()?;
	data.set("reason", reason)?;

	let inner = lua.create_table()?;
	inner.set("kind", "Skip")?;
	inner.set("data", data)?;

	let outer = lua.create_table()?;
	outer.set("_aipack_", inner)?;

	Ok(Value::Table(outer))
}

/// ## Lua Documentation
///
/// Returns a response instructing AIPACK to redo the entire agent execution.
///
/// When this function is called and its result is returned from any stage (Before All, Data, or Output),
/// the current agent execution will finish its current task(s) and then trigger a complete re-execution
/// of the agent using the same initial arguments but with the latest agent file content.
///
/// ```lua
/// -- API Signature
/// aip.flow.redo() -> table
/// ```
///
/// ### Example
///
/// ```lua
/// -- Trigger a redo if some condition is met in the output stage
/// if ai_response.content == "RETRY" then
///   return aip.flow.redo()
/// end
/// ```
///
/// ### Error
///
/// This function does not directly return any errors. Errors might occur during the creation of lua table.
///
/// ## Internal (not for Lua doc)
///
/// Internaly this returns a Lua table with a specific structure recognized by AIPACK's executor.
///
/// ```ts
/// type AipackFlowResponse = {
///   _aipack_: {
///     kind: "Redo"
///   }
/// }
/// ```
fn aipack_redo(lua: &Lua, _: ()) -> mlua::Result<Value> {
	let inner = lua.create_table()?;
	inner.set("kind", "Redo")?;

	let outer = lua.create_table()?;
	outer.set("_aipack_", inner)?;

	Ok(Value::Table(outer))
}

// endregion: --- Lua Functions

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	use crate::_test_support::{eval_lua, setup_lua};
	use crate::script::aip_modules::aip_flow;
	use serde_json::Value;
	use value_ext::JsonValueExt as _;

	#[tokio::test]
	async fn test_script_lua_aip_flow_before_all_response_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_flow::init_module, "flow").await?;
		let script = r#"
			return aip.flow.before_all_response(123)
		"#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		let kind = res.x_get_str("/_aipack_/kind")?;
		assert_eq!(kind, "BeforeAllResponse");

		let data = res.x_get_i64("/_aipack_/data")?;
		assert_eq!(data, 123);
		Ok(())
	}

	#[tokio::test]
	async fn test_script_lua_aip_flow_skip_with_reason() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_flow::init_module, "flow").await?;
		let script = r#"
			return aip.flow.skip("Not applicable")
		"#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		let kind = res.x_get_str("/_aipack_/kind")?;
		assert_eq!(kind, "Skip");

		let reason = res.x_get_str("/_aipack_/data/reason")?;
		assert_eq!(reason, "Not applicable");
		Ok(())
	}

	#[tokio::test]
	async fn test_script_lua_aip_flow_skip_without_reason() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_flow::init_module, "flow").await?;
		let script = r#"
			return aip.flow.skip()
		"#;

		// -- Exec
		let mut res = eval_lua(&lua, script)?;

		// -- Check
		let kind = res.x_get_str("/_aipack_/kind")?;
		assert_eq!(kind, "Skip");

		// NOTE: For now, even if we ask Option<Value>, on /_aipack_/data/reason, we get an error. Should probably be fix in value-ext
		let data = res.x_remove::<Value>("/_aipack_/data")?;
		let reason = data.x_get::<String>("reason").ok();
		assert!(reason.is_none(), "reason should be none");
		Ok(())
	}
}

// endregion: --- Tests
