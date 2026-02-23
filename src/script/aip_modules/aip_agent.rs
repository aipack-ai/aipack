//! Defines the `aip_agent` module, used in the lua engine.
//!
//! This module provides functionalities to execute other agents from within the current Lua script,
//! allowing for complex workflows and modular agent design.
//!
//! ---
//!
//! ## Lua documentation
//!
//! The `aip.agent` module exposes functions to run other agents from within a Lua script.
//!
//! ### Functions
//!
//! - `aip.agent.run(agent_name: string, options?: table): any`
//! - `aip.agent.extract_options(value: any): table | nil`

use crate::event::new_one_shot_channel;
use crate::model::RuntimeCtx;
use crate::run::RunSubAgentParams;
use crate::runtime::Runtime;
use crate::script::LuaValueExt;
use crate::types::{RunAgentOptions, RunAgentResponse};
use crate::{Error, Result};
use mlua::{IntoLua, Lua, Table, Value};
use simple_fs::SPath;

pub fn init_module(lua: &Lua, runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	let rt = runtime.clone();
	let agent_run = lua.create_function(
		move |lua, (agent_name, run_options): (String, Option<RunAgentOptions>)| {
			aip_agent_run(lua, &rt, agent_name, run_options.unwrap_or_default())
		},
	)?;

	let extract_options = lua.create_function(move |lua, value: Value| aip_agent_extract_options(lua, value))?;

	table.set("run", agent_run)?;
	table.set("extract_options", extract_options)?;

	Ok(table)
}

/// ## Lua Documentation
///
/// Runs another agent and returns its response.
///
/// ```lua
/// -- API Signature
/// aip.agent.run(agent_name: string, params?: table): any
/// ```
///
/// Executes the agent specified by `agent_name`. The function waits for the called agent
/// to complete and returns its result. This allows for chaining agents together.
///
/// ### Arguments
///
/// - `agent_name: string`: The name of the agent to run. This can be a relative path
///   (e.g., `"../my-other-agent.aip"`) or a fully qualified pack reference
///   (e.g., `"my-ns@my-pack/feature/my-agent.aip"`). Relative paths are resolved
///   from the directory of the calling agent.
/// - `params?: table`: An optional table containing input data and agent options.
///   - `input?: any`: (since 0.8.15) A single input value of any type.
///   - `inputs?: list`: Must be a list of input. e.g. `inputs = { "one", "two" }` will be two inputs.
///     Note: If both `input` and `inputs` are provided, `input` is prepended to the `inputs` list.
///   - `options?: table`: Agent-specific options. These options are passed directly to the called agent's
///     execution environment and can override settings defined in the called agent's `.aip` file.
///   - `agent_base_dir?: string`: (since 0.8.15) The base directory used to resolve relative agent paths.
///     By default, it is the directory of the caller agent. If provided, it overrides the default (e.g.,
///     using `CTX.WORKSPACE_DIR`). Note that pack references (e.g., `ns@pack/`) are still resolved to
///     their pack path regardless of this base directory.
///
/// #### Input Examples:
///
/// ```lua
/// -- Run an agent with a single input
/// local response = aip.agent.run("agent-name", { inputs = { "hello" } })
///
/// -- Run an agent with multiple string inputs
/// local response = aip.agent.run("agent-name", { inputs = {"input1", "input2"} })
///
/// -- Run an agent with structured inputs (e.g., file records) and with genai options
/// local response = aip.agent.run("agent-name", {
///   inputs = {
///     { path = "file1.txt", content = "..." },
///     { path = "file2.txt", content = "..." }
///   },
///   options = {
///     model = "gpt-5-mini",
///     input_concurrency = 5
///   }
/// })
/// ```
///
/// ### Returns
///
/// Returns a `RunAgentResponse` table containing the outputs from the agent's stages.
///
/// ```ts
/// {
///   outputs: any[],   // List of values returned by each # Output stage for each input.
///   after_all: any    // The value returned by the # After All stage (or nil).
/// }
/// ```
///
/// ### Error
///
/// Returns an error if:
/// - The `agent_name` is invalid or the agent file cannot be located/loaded.
/// - The options table contains invalid parameters.
/// - The execution of the called agent fails.
/// - An internal communication error occurs while waiting for the agent's result.
///
/// ```ts
/// {
///   error: string // Error message
/// }
/// ```
pub fn aip_agent_run(
	lua: &Lua,
	runtime: &Runtime,
	agent_name: String,
	run_options: RunAgentOptions,
) -> mlua::Result<Value> {
	// get the parent agent dir
	let parent_agent_dir = get_agent_dir_from_lua(lua);

	// -- Parse the Lua Options to the the LuaAgentRunOptions with inputs and agent options
	let (tx, rx) = new_one_shot_channel::<Result<RunAgentResponse>>("agent-run");

	let rt_ctx = RuntimeCtx::extract_from_global(lua)?;
	let parent_uid = rt_ctx
		.run_uid()
		.ok_or(Error::custom("Cannot call agent, no parent run uid found"))?;

	let run_agent_params = RunSubAgentParams::new(
		//
		runtime.clone(),
		parent_uid,
		parent_agent_dir,
		agent_name,
		run_options,
		Some(tx),
	)?;

	// NOTE: Needs to use the send_sync_spawn_and_block, otherwise, message not sent as it wait for this CmdRun to complete
	runtime.executor_sender().send_sync_spawn_and_block(run_agent_params.into())?;
	let res = rx.recv_sync();
	let run_agent_response = res??;

	run_agent_response.into_lua(lua)
}

/// ## Lua Documentation
///
/// Extracts relevant agent options from a given Lua value.
///
/// ```lua
/// -- API Signature
/// aip.agent.extract_options(value: any): table | nil
/// ```
///
/// If the input `value` is a Lua table, this function creates a new table and copies
/// the following properties if they exist in the input table:
///
/// - `model`
/// - `model_aliases`
/// - `input_concurrency`
/// - `temperature`
///
/// Other properties are ignored. If the input `value` is `nil` or not a table,
/// the function returns `nil`.
///
/// ### Arguments
///
/// - `value: any`: The Lua value from which to extract options.
///
/// ### Returns
///
/// A new Lua table containing the extracted options, or `nil` if the input
/// was `nil` or not a table.
pub fn aip_agent_extract_options(lua: &Lua, value: Value) -> mlua::Result<Value> {
	match value {
		Value::Table(table) => {
			let result_table = lua.create_table()?;

			// List of keys to copy
			let keys_to_copy = ["model", "model_aliases", "input_concurrency", "temperature", "top_p", "allow_run_on_task_fail"];

			for key in keys_to_copy.iter() {
				if let Some(val) = table.x_get_value(key)
					&& val != Value::Nil
				{
					result_table.set(*key, val)?;
				}
			}

			Ok(Value::Table(result_table))
		}
		_ => Ok(Value::Nil),
	}
}

// region:    --- Support

/// Helper function to get the calling agent's directory from the Lua CTX global.
fn get_agent_dir_from_lua(lua: &Lua) -> Option<SPath> {
	lua.globals()
		.x_get_value("CTX")?
		.x_get_string("AGENT_FILE_DIR")
		.map(|s| s.into())
}

// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{assert_contains, eval_lua, run_reflective_agent, run_test_agent, setup_lua};
	use crate::agent::Agent;
	use crate::runtime::Runtime;
	use value_ext::JsonValueExt;

	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_script_aip_agent_run_simple() -> Result<()> {
		// -- Setup & Fixtures
		let script = r#"
            local result = aip.agent.run("agent-script/agent-hello-world")
            return result
        "#;

		// -- Exec
		let mut res = run_reflective_agent(script, None).await?;

		// -- Check
		let mut outputs = res.x_remove::<serde_json::Value>("outputs")?;
		let len = outputs.as_array().map(|a| a.len()).ok_or("'outputs' should be array")?;
		assert_eq!(len, 1, "'outputs' should have one output");
		let output = outputs.x_remove::<String>("/0")?;
		assert_contains(&output, "Hello Wonderful World");

		Ok(())
	}

	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_script_aip_agent_run_relative() -> Result<()> {
		// -- Setup & Fixtures
		let runtime = Runtime::new_test_runtime_sandbox_01().await?;
		let agent_file = runtime
			.dir_context()
			.wks_dir()
			.ok_or("Should have workspace setup")?
			.join("agent-script/agent-calling-hello.aip");
		let agent = Agent::mock_from_file(agent_file.as_str())?;

		// -- Exec
		let mut res = run_test_agent(&runtime, &agent).await?;

		// -- Check
		let res = res.x_remove::<String>("res")?;
		assert_contains(&res, "Hello 'me' from agent-hello.aip");

		Ok(())
	}

	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_script_aip_agent_run_with_input() -> Result<()> {
		// -- Setup & Fixtures
		// Note: the agent is relative to sandbox-01/ because the run_reflective_agent mock agent is at the base
		let script = r#"
            local result = aip.agent.run("agent-script/agent-hello", { inputs = {"John"} })
            return result
        "#;

		// -- Exec
		let mut res = run_reflective_agent(script, None).await?;

		// -- Check
		let mut outputs = res.x_remove::<serde_json::Value>("outputs")?;
		let len = outputs.as_array().map(|a| a.len()).ok_or("'ouputs' should be array")?;
		assert_eq!(len, 1, "'ouputs' should have one output");
		let output = outputs.x_remove::<String>("/0")?;
		assert_contains(&output, "Hello 'John' from agent-hello.aip");

		Ok(())
	}

	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_script_aip_agent_run_with_options() -> Result<()> {
		// -- Setup & Fixtures
		let script = r#"
            local result = aip.agent.run("agent-script/agent-hello",
               { inputs =  {"John"},
                 options = {model = "super-fast"}
               }
            )
            return result
        "#;

		// -- Exec
		let mut res = run_reflective_agent(script, None).await?;

		// -- Check
		let mut outputs = res.x_remove::<serde_json::Value>("outputs")?;
		let len = outputs.as_array().map(|a| a.len()).ok_or("'ouputs' should be array")?;
		assert_eq!(len, 1, "'ouputs' should have one output");
		let output = outputs.x_remove::<String>("/0")?;
		assert_contains(&output, "Hello 'John' from agent-hello.aip");
		assert_contains(&output, "options.model = super-fast");

		Ok(())
	}

	#[tokio::test]
	async fn test_script_aip_agent_extract_options_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(super::init_module, "agent").await?;
		let script = r#"
local big = {
		model = "gpt-4",
		temperature = 0.8,
		input_comcurrency = 5,
		some_other_prop = "ignore_me",
		model_aliases = { fast = "gpt-3.5" },
		nil_prop = nil -- should not be included
}
return aip.agent.extract_options(big)
        "#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		assert_eq!(res.x_get_str("model")?, "gpt-4");
		assert_eq!(res.x_get_f64("temperature")?, 0.8);
		assert_eq!(res.x_get_i64("input_comcurrency")?, 5);
		assert!(
			res.x_get::<serde_json::Value>("some_other_prop").is_err(),
			"Should not have 'some_other_prop'"
		);

		Ok(())
	}

	#[tokio::test]
	async fn test_script_aip_agent_extract_options_string() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(super::init_module, "agent").await?;
		let script = r#"
return aip.agent.extract_options("Some string")
        "#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		assert!(matches!(res, serde_json::Value::Null), "Should be nil but was: {res:?}");

		Ok(())
	}

	#[tokio::test]
	async fn test_script_aip_agent_extract_options_nil() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(super::init_module, "agent").await?;
		let script = r#"
return aip.agent.extract_options(nil)
        "#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		assert!(matches!(res, serde_json::Value::Null), "Should be nil but was: {res:?}");

		Ok(())
	}
}

// endregion: --- Tests
