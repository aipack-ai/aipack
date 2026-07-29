//! Defines the `cmd` module, used in the lua engine.
//!
//! ---
//!
//! ## Lua documentation
//!
//! The `cmd` module exposes functions to execute system commands.
//!
//! ### Functions
//!
//! - `aip.cmd.exec(cmd_name: string, args?: string | list, options?: CmdOptionsExec): CmdResponse`

use crate::Result;
use crate::runtime::Runtime;
use crate::script::support::into_vec_of_strings;
use mlua::{FromLua, Lua, Table, Value};
use simple_fs::SPath;
use std::process::Command;

#[derive(Debug, Default)]
pub struct CmdExecOptions {
	pub cwd: Option<String>,
}

pub fn init_module(lua: &Lua, runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	let runtime = runtime.clone();
	let exec_fn = lua.create_function(move |lua, args| cmd_exec(lua, &runtime, args))?;

	table.set("exec", exec_fn)?;

	Ok(table)
}

impl FromLua for CmdExecOptions {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		if matches!(value, Value::Nil) {
			return Ok(Self::default());
		}

		let options = Table::from_lua(value, lua)?;
		let cwd: Option<String> = options.get("cwd")?;
		let cwd = cwd.filter(|cwd| !cwd.trim().is_empty());

		Ok(Self { cwd })
	}
}

/// ## Lua Documentation
///
/// Execute a system command with optional arguments.
///
/// ```lua
/// -- API Signature
/// aip.cmd.exec(cmd_name: string, args?: string | list, options?: CmdOptionsExec): CmdResponse
/// ```
///
/// An optional third `{cwd?: string}` options table can set the command's working directory.
/// Relative working directories resolve from the workspace, and absolute paths are supported.
///
/// Executes the specified command using the system shell. Arguments can be provided as a single string
/// or a table of strings.
///
/// On Windows, the command will be wrapped with `cmd /C cmd_name args..` to maximize compatibility.
///
/// ### Arguments
///
/// - `cmd_name: string` - The name or path of the command to execute.
/// - `args?: string | list<string>` (optional) - Arguments to pass to the command. Can be a single string
///   (which might be parsed by the shell) or a Lua list of strings.
///
/// ### Return (CmdResponse)
///
/// Returns a table representing the command's output and exit code if the command process itself
/// starts successfully (even if it returns a non-zero exit code from the command).
///
/// ```ts
/// {
///   stdout: string,  // Standard output captured from the command
///   stderr: string,  // Standard error captured from the command
///   exit:   number   // Exit code returned by the command (0 usually indicates success)
/// }
/// ```
///
/// ### Error
///
/// Returns an error if the command process cannot be started (e.g., command not found, permissions issue).
/// Errors from the command itself (non-zero exit code) are captured in the returned `CmdResponse` but
/// can also be raised as Lua errors if the agent's error handling is configured that way.
///
/// ```ts
/// {
///   error: string, // Error message from command execution failure
///   // Fields below might be available depending on the failure point
///   stdout?: string,
///   stderr?: string,
///   exit?: number,
/// }
/// ```
///
/// ### Example
///
/// ```lua
/// -- Single string argument
/// local result = aip.cmd.exec("echo", "hello world")
/// print("stdout:", result.stdout)
/// print("exit:", result.exit)
///
/// -- Table of arguments
/// local result = aip.cmd.exec("ls", {"-l", "-a"})
/// print("stdout:", result.stdout)
/// print("exit:", result.exit)
///
/// -- Run from a workspace-relative directory
/// local result = aip.cmd.exec("git", {"status"}, {cwd = "some/project"})
/// ```
fn cmd_exec(
	lua: &Lua,
	runtime: &Runtime,
	(cmd_name, args, options): (String, Option<Value>, CmdExecOptions),
) -> mlua::Result<Value> {
	let args = args.map(|args| into_vec_of_strings(args, "command args")).transpose()?;

	let mut command = cross_command(&cmd_name, args)?;
	if let Some(cwd) = options.cwd {
		let cwd = runtime.resolve_path_default(SPath::new(cwd), None)?;
		command.current_dir(cwd.path());
	}

	match command.output() {
		Ok(output) => {
			let stdout = String::from_utf8_lossy(&output.stdout).to_string();
			let stderr = String::from_utf8_lossy(&output.stderr).to_string();
			let exit_code = output.status.code().unwrap_or(-1) as i64;

			let res = lua.create_table()?;
			res.set("stdout", stdout.as_str())?;
			res.set("stderr", stderr.as_str())?;
			res.set("exit", exit_code)?;

			// NOTE: We return the table even on non-zero exit codes as this is the
			//       expected behavior of the Lua API. The caller can check the `exit` code.
			//       If the process itself failed to start, that's a different error case.
			Ok(Value::Table(res))
		}
		Err(err) => {
			let cmd = command.get_program().to_str().unwrap_or_default();
			let args = command
				.get_args()
				.map(|a| a.to_str().unwrap_or_default())
				.collect::<Vec<&str>>();
			let args = args.join(" ");
			let cwd_context = command
				.get_current_dir()
				.map(|cwd| format!("\nWorking directory: {}", cwd.to_string_lossy()))
				.unwrap_or_default();
			Err(crate::Error::custom(format!(
				"\
Fail to execute: {cmd} {args}{cwd_context}
Cause:\n{err}"
			))
			.into())
		}
	}
}

// region:    --- Support

/// Create a command, and make it a `cmd /C cmd_name args..` for windows compatibility.
fn cross_command(cmd_name: &str, args: Option<Vec<String>>) -> Result<Command> {
	let command = if cfg!(windows) {
		let full_cmd = if let Some(args) = args {
			// Quote arguments if needed and join
			let joined = args.join(" ");
			format!("{cmd_name} {joined}")
		} else {
			cmd_name.to_string()
		};

		let mut cmd = Command::new("cmd");
		cmd.args(["/C", &full_cmd]);
		cmd
	} else {
		let mut cmd = Command::new(cmd_name);
		if let Some(args) = args {
			cmd.args(args);
		}

		cmd
	};

	Ok(command)
}
// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{assert_contains, eval_lua, setup_lua};
	use crate::script::aip_modules::aip_cmd;
	use std::fs;
	use value_ext::JsonValueExt as _;

	#[tokio::test]
	async fn test_lua_cmd_exec_echo_single_arg() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_cmd::init_module, "cmd").await?;
		let script = r#"
			return aip.cmd.exec("echo", "hello world")
		"#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		assert_eq!(res.x_get_str("stdout")?.trim(), "hello world");
		assert_eq!(res.x_get_str("stderr")?, "");
		assert_eq!(res.x_get_i64("exit")?, 0);

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_cmd_exec_echo_multiple_args() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_cmd::init_module, "cmd").await?;
		let script = r#"
			return aip.cmd.exec("echo", {"hello", "world"})
		"#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		assert_eq!(res.x_get_str("stdout")?.trim(), "hello world");
		assert_eq!(res.x_get_str("stderr")?, "");
		assert_eq!(res.x_get_i64("exit")?, 0);

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_cmd_exec_invalid_command_pcall() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_cmd::init_module, "cmd").await?;
		let script = r#"
			local ok, err = pcall(function()
				aip.cmd.exec("nonexistentcommand")
			end)
			return err -- Return the error object to Rust
		"#;

		// -- Exec & Check
		// We expect eval_lua to return a Lua error in this case
		let Err(err) = eval_lua(&lua, script) else {
			return Err("Should have returned an error".into());
		};

		// -- Check
		let err_str = err.to_string();
		assert_contains(&err_str, "Fail to execute: {nonexistentcommand}");
		assert_contains(&err_str, "Cause:");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_cmd_exec_invalid_command_direct() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_cmd::init_module, "cmd").await?;
		let script = r#"return aip.cmd.exec("nonexistentcommand")"#;

		// -- Exec & Check
		// We expect eval_lua to return a Lua error in this case
		let Err(err) = eval_lua(&lua, script) else {
			return Err("Should have returned an error".into());
		};

		// -- Check
		let err_str = err.to_string();
		assert_contains(&err_str, "Fail to execute: {nonexistentcommand}");
		assert_contains(&err_str, "Cause:");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_cmd_exec_non_zero_exit() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_cmd::init_module, "cmd").await?;
		// Command that typically exits with non-zero status (e.g., grep non-existent file)
		// Using `false` is more portable than `grep non-existent-file`
		let script = r#"
			return aip.cmd.exec("false")
		"#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		// The Lua function returns the response table even on non-zero exit
		assert_eq!(res.x_get_str("stdout")?, "");
		assert_eq!(res.x_get_str("stderr")?, "");
		assert_ne!(res.x_get_i64("exit")?, 0); // Check that exit code is non-zero

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_cmd_exec_with_relative_cwd() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_cmd::init_module, "cmd").await?;
		let cmd_name = if cfg!(windows) { "cd" } else { "pwd" };
		let script = format!(r#"return aip.cmd.exec("{cmd_name}", nil, {{cwd = "."}})"#);
		let expected_cwd = fs::canonicalize(crate::_test_support::SANDBOX_01_WKS_DIR)?;

		// -- Exec
		let res = eval_lua(&lua, &script)?;

		// -- Check
		let actual_cwd = fs::canonicalize(res.x_get_str("stdout")?.trim())?;
		assert_eq!(actual_cwd, expected_cwd);
		assert_eq!(res.x_get_i64("exit")?, 0);

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_cmd_exec_with_absolute_cwd() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_cmd::init_module, "cmd").await?;
		let cmd_name = if cfg!(windows) { "cd" } else { "pwd" };
		let expected_cwd = fs::canonicalize(std::env::current_dir()?)?;
		let cwd = serde_json::to_string(&expected_cwd.to_string_lossy())?;
		let script = format!(r#"return aip.cmd.exec("{cmd_name}", nil, {{cwd = {cwd}}})"#);

		// -- Exec
		let res = eval_lua(&lua, &script)?;

		// -- Check
		let actual_cwd = fs::canonicalize(res.x_get_str("stdout")?.trim())?;
		assert_eq!(actual_cwd, expected_cwd);
		assert_eq!(res.x_get_i64("exit")?, 0);

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_cmd_exec_with_empty_options() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_cmd::init_module, "cmd").await?;
		let script = r#"
			local empty_options = aip.cmd.exec("echo", "hello", {})
			local empty_cwd = aip.cmd.exec("echo", "world", {cwd = ""})
			return {
				empty_options = empty_options,
				empty_cwd = empty_cwd,
			}
		"#;

		// -- Exec
		let res = eval_lua(&lua, script)?;

		// -- Check
		assert_eq!(res.x_get_str("empty_options.stdout")?.trim(), "hello");
		assert_eq!(res.x_get_i64("empty_options.exit")?, 0);
		assert_eq!(res.x_get_str("empty_cwd.stdout")?.trim(), "world");
		assert_eq!(res.x_get_i64("empty_cwd.exit")?, 0);

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_cmd_exec_with_invalid_cwd() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_cmd::init_module, "cmd").await?;
		let script = r#"
			return aip.cmd.exec("echo", "hello", {cwd = ".aipack-cmd-missing-dir"})
		"#;

		// -- Exec & Check
		let Err(err) = eval_lua(&lua, script) else {
			return Err("Should have returned an error".into());
		};

		// -- Check
		let err_str = err.to_string();
		assert_contains(&err_str, "Fail to execute:");
		assert_contains(&err_str, "Working directory:");
		assert_contains(&err_str, "Cause:");

		Ok(())
	}
}

// endregion: --- Tests
