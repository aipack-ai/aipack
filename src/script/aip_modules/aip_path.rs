//! Defines the `path` module, used in the lua engine.
//!
//! ---
//!
//! ## Lua documentation
//! The `aip.path` module exposes functions used to interact with file paths.
//!
//! ### Functions
//!
//! - `aip.path.split(path: string): parent: string, filename: string`
//! - `aip.path.resolve(path: string): string`
//! - `aip.path.exists(path: string): boolean`
//! - `aip.path.is_file(path: string): boolean`
//! - `aip.path.is_dir(path: string): boolean`
//! - `aip.path.diff(file_path: string, base_path: string): string`
//! - `aip.path.parent(path: string): string | nil`
//! - `aip.path.join(paths: string | table): string | nil` (default non os normalized)
//! - `aip.path.join_os_normalized(paths: string | table): string | nil` (windows style if start with like C:)
//! - `aip.path.join_os_non_normalized(paths: string | table): string | nil` (default, as user specified)
//!
//! NOTE 1: Currently, `aip.path.join` uses `aip.path.join_os_non_normalized`. This might change in the future.
//!
//! NOTE 2: The reason why normalized is prefixed with `_os_`
//!         is because there is another type of normalization that removes the "../".
//!         There are no functions for this yet, but keeping the future open.

use crate::Result;
use crate::dir_context::PathResolver;
use crate::runtime::Runtime;
use mlua::{Lua, MultiValue, Table};
use mlua::{Value, Variadic};
use simple_fs::SPath;
use std::path::PathBuf;
use std::path::{MAIN_SEPARATOR, Path};

pub fn init_module(lua: &Lua, runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	// -- split
	let path_split_fn = lua.create_function(path_split)?;

	// -- exists
	let rt = runtime.clone();
	let path_exists_fn = lua.create_function(move |_lua, path: String| path_exists(&rt, path))?;

	// -- resolve
	let rt = runtime.clone();
	let path_resolve_fn = lua.create_function(move |_lua, path: String| path_resolve(&rt, path))?;

	// -- is_file
	let rt = runtime.clone();
	let path_is_file_fn = lua.create_function(move |_lua, path: String| path_is_file(&rt, path))?;

	// -- is_dir
	let rt = runtime.clone();
	let path_is_dir_fn = lua.create_function(move |_lua, path: String| path_is_dir(&rt, path))?;

	// -- diff
	let path_diff_fn =
		lua.create_function(move |_lua, (file_path, base_path): (String, String)| path_diff(file_path, base_path))?;

	// -- parent
	let path_parent_fn = lua.create_function(move |_lua, path: String| path_parent(path))?;

	// -- joins
	let path_join_non_os_normalized_fn = lua.create_function(path_join_non_os_normalized)?;
	let path_join_os_normalized_fn = lua.create_function(path_join_os_normalized)?;
	let path_join_fn = lua.create_function(path_join_non_os_normalized)?;

	// -- Add all functions to the module
	table.set("resolve", path_resolve_fn)?;
	table.set("exists", path_exists_fn)?;
	table.set("is_file", path_is_file_fn)?;
	table.set("is_dir", path_is_dir_fn)?;
	table.set("diff", path_diff_fn)?;
	table.set("parent", path_parent_fn)?;
	table.set("join", path_join_fn)?;
	table.set("join_os_non_normalized", path_join_non_os_normalized_fn)?;
	table.set("join_os_normalized", path_join_os_normalized_fn)?;
	table.set("split", path_split_fn)?;

	Ok(table)
}

// region:    --- Lua Functions

/// ## Lua Documentation
///
/// Split path into parent, filename.
///
/// ```lua
/// -- API Signature
/// aip.path.split(path: string): parent: string, filename: string
/// ```
///
/// Splits the given path into its parent directory component and its filename component.
///
/// ### Arguments
///
/// - `path: string`: The path to split.
///
/// ### Returns
///
/// Returns two strings: the parent directory path and the filename. Returns empty strings
/// if the path has no parent or no filename, respectively (e.g., splitting "." returns "", ".").
///
/// ```lua
/// -- Example output
/// local parent, filename = "some/path", "file.txt"
/// ```
///
/// ### Example
///
/// ```lua
/// local parent, filename = aip.path.split("folder/file.txt")
/// print(parent)   -- Output: "folder"
/// print(filename) -- Output: "file.txt"
///
/// local parent, filename = aip.path.split("justafile.md")
/// print(parent)   -- Output: ""
/// print(filename) -- Output: "justafile.md"
/// ```
///
/// ### Error
///
/// This function does not typically error, returning empty strings for components that do not exist.
fn path_split(lua: &Lua, path: String) -> mlua::Result<MultiValue> {
	let path = SPath::from(path);

	let parent = path.parent().map(|p| p.to_string()).unwrap_or_default();
	let file_name = path.file_name().unwrap_or_default().to_string();

	Ok(MultiValue::from_vec(vec![
		mlua::Value::String(lua.create_string(parent)?),
		mlua::Value::String(lua.create_string(file_name)?),
	]))
}

/// ## Lua Documentation
///
/// Resolves and normalizes a path relative to the workspace.
///
/// ```lua
/// -- API Signature
/// aip.path.resolve(path: string): string
/// ```
///
/// Resolves and normalizes the given path string. This handles relative paths (`.`, `..`),
/// absolute paths, and special aipack pack references (`ns@pack/`, `ns@pack$base/`, `ns@pack$workspace/`).
/// The resulting path is normalized (e.g., `some/../path` becomes `some/path`).
///
/// ### Arguments
///
/// - `path: string`: The path string to resolve and normalize. It can be a relative path, an absolute path, or an aipack pack reference.
///
/// ### Returns
///
/// - `string`: The resolved and normalized path as a string. This path is usually absolute after resolution.
///
/// ### Example
///
/// ```lua
/// local resolved_path = aip.path.resolve("./agent-script/../agent-script/agent-hello.aip")
/// print(resolved_path) -- Output: "/path/to/workspace/agent-script/agent-hello.aip" (example)
///
/// local resolved_pack_path = aip.path.resolve("ns@pack/some/file.txt")
/// print(resolved_pack_path) -- Output: "/path/to/aipack-base/packs/ns/pack/some/file.txt" (example)
/// ```
///
/// ### Error
///
/// Returns an error if the path string cannot be resolved (e.g., invalid pack reference, invalid path format).
///
/// ```ts
/// {
///   error: string // Error message
/// }
/// ```
fn path_resolve(runtime: &Runtime, path: String) -> mlua::Result<String> {
	let path = runtime
		.dir_context()
		.resolve_path(runtime.session(), (&path).into(), PathResolver::WksDir)?;
	Ok(path.to_string())
}

/// ## Lua Documentation
///
/// Checks if the specified path exists.
///
/// ```lua
/// -- API Signature
/// aip.path.exists(path: string): boolean
/// ```
///
/// Checks if the file or directory specified by `path` exists. The path is resolved relative to the workspace root.
/// Supports relative paths, absolute paths, and pack references (`ns@pack/...`).
///
/// ### Arguments
///
/// - `path: string`: The path string to check for existence. Can be relative, absolute, or a pack reference.
///
/// ### Returns
///
/// - `boolean`: Returns `true` if a file or directory exists at the specified path, `false` otherwise.
///
/// ### Example
///
/// ```lua
/// if aip.path.exists("README.md") then
///   print("README.md exists.")
/// end
///
/// if aip.path.exists("ns@pack/main.aip") then
///   print("Pack main agent exists.")
/// end
/// ```
///
/// ### Error
///
/// Returns an error if the path string cannot be resolved (e.g., invalid pack reference, invalid path format).
///
/// ```ts
/// {
///   error: string // Error message
/// }
/// ```
fn path_exists(runtime: &Runtime, path: String) -> mlua::Result<bool> {
	let path = runtime
		.dir_context()
		.resolve_path(runtime.session(), (&path).into(), PathResolver::WksDir)?;
	Ok(path.exists())
}

/// ## Lua Documentation
///
/// Checks if the specified path points to a file.
///
/// ```lua
/// -- API Signature
/// aip.path.is_file(path: string): boolean
/// ```
///
/// Checks if the entity at the specified `path` is a file. The path is resolved relative to the workspace root.
/// Supports relative paths, absolute paths, and pack references (`ns@pack/...`).
///
/// ### Arguments
///
/// - `path: string`: The path string to check. Can be relative, absolute, or a pack reference.
///
/// ### Returns
///
/// - `boolean`: Returns `true` if the path points to an existing file, `false` otherwise.
///
/// ### Example
///
/// ```lua
/// if aip.path.is_file("config.toml") then
///   print("config.toml is a file.")
/// end
///
/// if aip.path.is_file("src/") then
///   print("src/ is a file.") -- This will print false
/// end
/// ```
///
/// ### Error
///
/// Returns an error if the path string cannot be resolved (e.g., invalid pack reference, invalid path format).
///
/// ```ts
/// {
///   error: string // Error message
/// }
/// ```
fn path_is_file(runtime: &Runtime, path: String) -> mlua::Result<bool> {
	let path = runtime
		.dir_context()
		.resolve_path(runtime.session(), (&path).into(), PathResolver::WksDir)?;
	Ok(path.is_file())
}

/// ## Lua Documentation
///
/// Computes the relative path from `base_path` to `file_path`.
///
/// ```lua
/// -- API Signature
/// aip.path.diff(file_path: string, base_path: string): string
/// ```
///
/// Calculates the relative path string that navigates from the `base_path` to the `file_path`.
/// Both paths can be absolute or relative.
///
/// ### Arguments
///
/// - `file_path: string`: The target path.
/// - `base_path: string`: The starting path.
///
/// ### Returns
///
/// - `string`: The relative path string from `base_path` to `file_path`. Returns an empty string if the paths are the same or if a relative path cannot be easily computed (e.g., on different drives on Windows).
///
/// ### Example
///
/// ```lua
/// print(aip.path.diff("/a/b/c/file.txt", "/a/b/")) -- Output: "c/file.txt"
/// print(aip.path.diff("/a/b/", "/a/b/c/file.txt")) -- Output: "../.."
/// print(aip.path.diff("/a/b/c", "/a/d/e"))      -- Output: "../../b/c" (example, depends on OS)
/// print(aip.path.diff("folder/file.txt", "folder")) -- Output: "file.txt"
/// print(aip.path.diff("folder/file.txt", "folder/file.txt")) -- Output: ""
/// ```
///
/// ### Error
///
/// Returns an error if the paths are invalid or cannot be processed.
///
/// ```ts
/// {
///   error: string // Error message
/// }
/// ```
fn path_diff(file_path: String, base_path: String) -> mlua::Result<String> {
	let file_path = SPath::from(file_path);
	let base_path = SPath::from(base_path);
	// NOTE: Right now, using unwrap_or_default, as this should not happen
	//       But will update simple-fs to utf8 diff by default
	let diff = file_path.diff(base_path).map(|p| p.to_string()).unwrap_or_default();
	Ok(diff)
}

/// ## Lua Documentation
///
/// Checks if the specified path points to a directory.
///
/// ```lua
/// -- API Signature
/// aip.path.is_dir(path: string): boolean
/// ```
///
/// Checks if the entity at the specified `path` is a directory. The path is resolved relative to the workspace root.
/// Supports relative paths, absolute paths, and pack references (`ns@pack/...`).
///
/// ### Arguments
///
/// - `path: string`: The path string to check. Can be relative, absolute, or a pack reference.
///
/// ### Returns
///
/// - `boolean`: Returns `true` if the path points to an existing directory, `false` otherwise.
///
/// ### Example
///
/// ```lua
/// if aip.path.is_dir("src/") then
///   print("src/ is a directory.")
/// end
///
/// if aip.path.is_dir("config.toml") then
///   print("config.toml is a directory.") -- This will print false
/// end
/// ```
///
/// ### Error
///
/// Returns an error if the path string cannot be resolved (e.g., invalid pack reference, invalid path format).
///
/// ```ts
/// {
///   error: string // Error message
/// }
/// ```
fn path_is_dir(runtime: &Runtime, path: String) -> mlua::Result<bool> {
	let path = runtime
		.dir_context()
		.resolve_path(runtime.session(), (&path).into(), PathResolver::WksDir)?;
	Ok(path.is_dir())
}

/// ## Lua Documentation
///
/// Returns the parent directory path of the specified path.
///
/// ```lua
/// -- API Signature
/// aip.path.parent(path: string): string | nil
/// ```
///
/// Gets the parent directory component of the given path string.
///
/// ### Arguments
///
/// - `path: string`: The path string.
///
/// ### Returns
///
/// - `string | nil`: Returns the parent directory path as a string. Returns `nil` if the path has no parent (e.g., ".", "/", "C:/").
///
/// ### Example
///
/// ```lua
/// print(aip.path.parent("some/path/file.txt")) -- Output: "some/path"
/// print(aip.path.parent("/root/file"))         -- Output: "/root"
/// print(aip.path.parent("."))                  -- Output: nil
/// ```
///
/// ### Error
///
/// This function does not typically error.
fn path_parent(path: String) -> mlua::Result<Option<String>> {
	match Path::new(&path).parent() {
		Some(parent) => match parent.to_str() {
			Some(parent_str) => Ok(Some(parent_str.to_string())),
			None => Ok(None),
		},
		None => Ok(None),
	}
}

/// ## Lua Documentation
///
/// Returns the path, with paths joined without OS normalization.
///
/// ```lua
/// -- API Signature
/// aip.path.join(paths: string | table): string | nil
/// ```
///
/// Joins one or more path components into a single path string. This function does *not* perform OS-specific normalization of separators (e.g., always uses `/` or `\` as provided) or resolve `.` or `..` components. This is the default behavior for `aip.path.join`.
///
/// ### Arguments
///
/// - `paths: string | table`: Can be:
///   - A series of string arguments representing path components.
///   - A Lua table (list) of strings representing path components.
///
/// ### Example
///
/// ```lua
/// -- Using multiple arguments:
/// local full_path1 = aip.path.join("folder\\", "subfolder/", "file.txt")
/// print(full_path1) -- Output: "folder\\/subfolder/file.txt" (example, depends on simple-fs behavior)
///
/// -- Using a table (list):
/// local paths_list = {"data", "raw", "input.csv"}
/// local full_path2 = aip.path.join(paths_list)
/// print(full_path2) -- Output: "data/raw/input.csv" (example, depends on simple-fs behavior)
/// ```
///
/// ### Returns
///
/// - `string | nil`: Returns the joined path as a string. Returns `nil` if no path components are provided.
///
/// ### Error
///
/// Returns an error if arguments are not strings or a table of strings.
///
/// ```ts
/// {
///   error: string // Error message
/// }
/// ```
pub fn path_join_non_os_normalized(lua: &Lua, paths: Variadic<Value>) -> mlua::Result<Value> {
	let mut path_buf = PathBuf::new();
	if paths.is_empty() {
		return Ok(Value::Nil);
	}
	// If the first argument is a table, iterate over its entries.
	if let Some(Value::Table(table)) = paths.first() {
		for pair in table.clone().pairs::<mlua::Integer, String>() {
			let (_, s) = pair?;
			path_buf.push(s);
		}
	} else {
		// Otherwise, iterate over the variadic arguments.
		for arg in paths {
			if let Value::String(s) = arg {
				path_buf.push(s.to_str()?.to_string());
			}
		}
	}
	Ok(Value::String(lua.create_string(path_buf.to_string_lossy().as_ref())?))
}

/// ## Lua Documentation
///
/// Joins path components with OS normalization.
///
/// ```lua
/// -- API Signature
/// aip.path.join_os_normalized(paths: string | table): string | nil
/// ```
///
/// Joins one or more path components into a single path string, using the operating system's preferred path separator (`/` on Unix-like systems, `\` on Windows) and handling leading/trailing separators on components. Does *not* resolve `.` or `..` components.
///
/// ### Arguments
///
/// - `paths: string | table`: Can be:
///   - A series of string arguments representing path components.
///   - A Lua table (list) of strings representing path components.
///
/// ### Example
///
/// ```lua
/// -- Using multiple arguments (Unix-like OS):
/// local full_path1 = aip.path.join_os_normalized("folder\\", "subfolder/", "file.txt")
/// print(full_path1) -- Output: "folder/subfolder/file.txt"
///
/// -- Using a table (list) (Windows OS):
/// local paths_list = {"C:/Users", "Admin", "Documents/file.txt"}
/// local full_path2 = aip.path.join_os_normalized(paths_list)
/// print(full_path2) -- Output: "C:\Users\Admin\Documents\file.txt"
/// ```
///
/// ### Returns
///
/// - `string | nil`: Returns the joined path as a string, normalized for the operating system. Returns `nil` if no path components are provided.
///
/// ### Error
///
/// Returns an error if arguments are not strings or a table of strings.
///
/// ```ts
/// {
///   error: string // Error message
/// }
/// ```
pub fn path_join_os_normalized(lua: &Lua, paths: Variadic<Value>) -> mlua::Result<Value> {
	let mut comps = Vec::new();
	if paths.is_empty() {
		return Ok(Value::Nil);
	}
	if let Some(Value::Table(table)) = paths.first() {
		for pair in table.clone().pairs::<mlua::Integer, String>() {
			let (_, s) = pair?;
			if !s.is_empty() {
				comps.push(s);
			}
		}
	} else {
		for arg in paths {
			if let Value::String(s) = arg {
				let s = s.to_str()?;
				if !s.is_empty() {
					comps.push(s.to_string());
				}
			}
		}
	}
	if comps.is_empty() {
		return Ok(Value::String(lua.create_string("")?));
	}
	let is_windows = is_windows_style(&comps[0]);
	let sep: char = if is_windows { '\\' } else { MAIN_SEPARATOR };
	let mut result = String::new();
	if is_windows {
		// For Windows‑style, trim trailing slashes from the first component and convert any '/' to '\\'.
		let first = comps[0].trim_end_matches(['\\', '/']).replace("/", "\\");
		result.push_str(&first);
		for comp in comps.iter().skip(1) {
			// For subsequent components, trim both leading and trailing slashes and convert '/' to '\\'.
			let part = comp.trim_matches(|c| c == '\\' || c == '/').replace("/", "\\");
			if !part.is_empty() {
				if !result.ends_with(sep) {
					result.push(sep);
				}
				result.push_str(&part);
			}
		}
	} else {
		// For non–Windows style, simply trim extra slashes.
		let first = comps[0].trim_end_matches(['\\', '/']);
		result.push_str(first);
		for comp in comps.iter().skip(1) {
			let part = comp.trim_matches(|c| c == '\\' || c == '/');
			if !part.is_empty() {
				if !result.ends_with(sep) {
					result.push(sep);
				}
				result.push_str(part);
			}
		}
	}
	Ok(Value::String(lua.create_string(&result)?))
}

/// Returns true if the given string looks like a Windows‑style path.
/// That is, if its second character is a colon (e.g., "C:") or it starts with a backslash.
fn is_windows_style(s: &str) -> bool {
	(s.len() >= 2 && s.as_bytes()[1] == b':') || s.starts_with('\\')
}

// endregion: --- Lua Functions

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{eval_lua, setup_lua};
	use crate::script::aip_modules::aip_path;
	use std::path::MAIN_SEPARATOR;

	#[tokio::test]
	async fn test_lua_path_exists_true() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_path::init_module, "path")?;
		let paths = &[
			"./agent-script/agent-hello.aip",
			"agent-script/agent-hello.aip",
			"./sub-dir-a/agent-hello-2.aip",
			"sub-dir-a/agent-hello-2.aip",
			"./sub-dir-a/",
			"sub-dir-a",
			"./sub-dir-a/",
			"./sub-dir-a/../",
			"./sub-dir-a/..",
		];

		// -- Exec & Check
		for path in paths {
			let code = format!(r#"return aip.path.exists("{path}")"#);
			let res = eval_lua(&lua, &code)?;
			assert!(res.as_bool().ok_or("Result should be a bool")?, "'{path}' should exist");
		}

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_path_exists_false() -> Result<()> {
		let lua = setup_lua(aip_path::init_module, "path")?;
		let paths = &["./no file .rs", "some/no-file.md", "./s do/", "no-dir/at/all"];

		for path in paths {
			let code = format!(r#"return aip.path.exists("{path}")"#);
			let res = eval_lua(&lua, &code)?;
			assert!(
				!res.as_bool().ok_or("Result should be a bool")?,
				"'{path}' should NOT exist"
			);
		}

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_path_is_file_true() -> Result<()> {
		let lua = setup_lua(aip_path::init_module, "path")?;
		let paths = &[
			"./agent-script/agent-hello.aip",
			"agent-script/agent-hello.aip",
			"./sub-dir-a/agent-hello-2.aip",
			"sub-dir-a/agent-hello-2.aip",
			"./sub-dir-a/../agent-script/agent-hello.aip",
		];

		for path in paths {
			let code = format!(r#"return aip.path.is_file("{path}")"#);
			let res = eval_lua(&lua, &code)?;
			assert!(
				res.as_bool().ok_or("Result should be a bool")?,
				"'{path}' should be a file"
			);
		}

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_path_is_file_false() -> Result<()> {
		let lua = setup_lua(aip_path::init_module, "path")?;
		let paths = &["./no-file", "no-file.txt", "sub-dir-a/"];

		for path in paths {
			let code = format!(r#"return aip.path.is_file("{path}")"#);
			let res = eval_lua(&lua, &code)?;
			assert!(
				!res.as_bool().ok_or("Result should be a bool")?,
				"'{path}' should NOT be a file"
			);
		}

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_path_is_dir_true() -> Result<()> {
		let lua = setup_lua(aip_path::init_module, "path")?;
		let paths = &["./sub-dir-a", "sub-dir-a", "./sub-dir-a/.."];

		for path in paths {
			let code = format!(r#"return aip.path.is_dir("{path}")"#);
			let res = eval_lua(&lua, &code)?;
			assert!(
				res.as_bool().ok_or("Result should be a bool")?,
				"'{path}' should be a directory"
			);
		}

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_path_is_dir_false() -> Result<()> {
		let lua = setup_lua(aip_path::init_module, "path")?;
		let paths = &[
			"./agent-hello.aipack",
			"agent-hello.aipack",
			"./sub-dir-a/agent-hello-2.aipack",
			"./sub-dir-a/other-path",
			"nofile.txt",
			"./s rc/",
		];

		for path in paths {
			let code = format!(r#"return aip.path.is_dir("{path}")"#);
			let res = eval_lua(&lua, &code)?;
			assert!(
				!res.as_bool().ok_or("Result should be a bool")?,
				"'{path}' should NOT be a directory"
			);
		}

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_path_parent() -> Result<()> {
		let lua = setup_lua(aip_path::init_module, "path")?;
		// Fixtures: (path, expected_parent)
		let paths = &[
			("./agent-hello.aipack", "."),
			("./", ""),
			(".", ""),
			("./sub-dir/file.txt", "./sub-dir"),
			("./sub-dir/file", "./sub-dir"),
			("./sub-dir/", "."),
			("./sub-dir", "."),
		];

		for (path, expected) in paths {
			let code = format!(r#"return aip.path.parent("{path}")"#);
			let res = eval_lua(&lua, &code)?;
			let result = res.as_str().ok_or("Should be a string")?;
			assert_eq!(result, *expected, "Parent mismatch for path: {path}");
		}

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_path_split() -> Result<()> {
		let lua = setup_lua(aip_path::init_module, "path")?;
		let paths = &[
			("some/path/to_file.md", "some/path", "to_file.md"),
			("folder/file.txt", "folder", "file.txt"),
			("justafile.md", "", "justafile.md"),
			("/absolute/path/file.log", "/absolute/path", "file.log"),
			("/file_at_root", "/", "file_at_root"),
			("trailing/slash/", "trailing", "slash"),
		];

		for (path, expected_parent, expected_filename) in paths {
			let code = format!(
				r#"
                    local parent, filename = aip.path.split("{path}")
                    return {{ parent, filename }}
                "#
			);
			let res = eval_lua(&lua, &code)?;
			let res_array = res.as_array().ok_or("Expected an array from Lua function")?;
			let parent = res_array
				.first()
				.and_then(|v| v.as_str())
				.ok_or("First value should be a string")?;
			let filename = res_array
				.get(1)
				.and_then(|v| v.as_str())
				.ok_or("Second value should be a string")?;
			assert_eq!(parent, *expected_parent, "Parent mismatch for path: {path}");
			assert_eq!(filename, *expected_filename, "Filename mismatch for path: {path}");
		}

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_path_join_default() -> Result<()> {
		common_test_lua_path_join_non_os_normalized("join").await?;
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_path_join_os_non_normalized() -> Result<()> {
		common_test_lua_path_join_non_os_normalized("join_os_non_normalized").await?;
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_path_join_os_normalized_lua_engine() -> Result<()> {
		common_test_lua_path_join_os_normalized_lua_engine("join_os_normalized").await?;
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_path_join_os_normalized_reflective() -> Result<()> {
		common_test_lua_path_join_os_normalized_reflective("join_os_normalized").await?;
		Ok(())
	}

	// region:    --- Tests Support

	async fn common_test_lua_path_join_non_os_normalized(join_fn_name: &str) -> Result<()> {
		let lua = setup_lua(super::init_module, "path")?;
		use std::path::PathBuf;
		let mut expected1 = PathBuf::new();
		expected1.push("folder");
		expected1.push("subfolder");
		expected1.push("file.txt");

		let mut expected2 = PathBuf::new();
		expected2.push("folder\\");
		expected2.push("subfolder/");
		expected2.push("file.txt");

		let cases = vec![
			(
				r#"{"folder", "subfolder", "file.txt"}"#,
				expected1.to_string_lossy().to_string(),
			),
			(
				r#"{"folder\\", "subfolder/", "file.txt"}"#,
				expected2.to_string_lossy().to_string(),
			),
		];

		for (input, expected) in cases {
			let code = format!("return aip.path.{}({})", join_fn_name, input);
			let result: String = lua.load(&code).eval()?;
			assert_eq!(result, expected, "Non-normalized failed for input: {}", input);
		}
		Ok(())
	}

	async fn common_test_lua_path_join_os_normalized_lua_engine(join_fn_name: &str) -> Result<()> {
		let lua = setup_lua(super::init_module, "path")?;
		let sep = MAIN_SEPARATOR;
		let cases = vec![
			(
				r#"{"folder", "subfolder", "file.txt"}"#,
				format!("folder{sep}subfolder{sep}file.txt", sep = sep),
			),
			(
				r#"{"leading", "", "trailing"}"#,
				format!("leading{sep}trailing", sep = sep),
			),
			(
				r#"{"folder\\", "subfolder/", "file.txt"}"#,
				format!("folder{sep}subfolder{sep}file.txt", sep = sep),
			),
			(
				r#"{"C:/Users", "Admin", "Documents/file.txt"}"#,
				"C:\\Users\\Admin\\Documents\\file.txt".to_string(),
			),
			(
				r#"{"\\server", "share", "folder", "file.txt"}"#,
				"\\server\\share\\folder\\file.txt".to_string(),
			),
		];

		for (input, expected) in cases {
			let code = format!("return aip.path.{}({})", join_fn_name, input);
			let result: String = lua.load(&code).eval()?;
			assert_eq!(result, expected, "Normalized failed for input: {}", input);
		}
		Ok(())
	}

	async fn common_test_lua_path_join_os_normalized_reflective(join_fn_name: &str) -> Result<()> {
		let lua = setup_lua(super::init_module, "path")?;
		let cases = &[
			(
				r#"{"folder", "subfolder", "file.txt"}"#,
				format!("folder{}subfolder{}file.txt", MAIN_SEPARATOR, MAIN_SEPARATOR),
			),
			(r#"{"single"}"#, "single".to_string()),
			(
				r#"{"leading", "", "trailing"}"#,
				format!("leading{}trailing", MAIN_SEPARATOR),
			),
			(
				r#"{"C:\\Users", "Admin", "Documents\\file.txt"}"#,
				"C:\\Users\\Admin\\Documents\\file.txt".to_string(),
			),
			(
				r#"{"C:/Users", "Admin", "Documents/file.txt"}"#,
				"C:\\Users\\Admin\\Documents\\file.txt".to_string(),
			),
			(r#"{"C:/", "Windows", "System32"}"#, "C:\\Windows\\System32".to_string()),
		];

		for (lua_table, expected_path) in cases {
			let code = format!(r#"return aip.path.{}({})"#, join_fn_name, lua_table);
			let res = eval_lua(&lua, &code)?;
			let result_path = res.as_str().ok_or("Should return a string")?;
			assert_eq!(
				result_path, expected_path,
				"Path mismatch for table input: {}",
				lua_table
			);
		}
		Ok(())
	}

	// endregion: --- Tests Support
}

// endregion: --- Tests
