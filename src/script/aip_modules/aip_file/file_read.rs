//! Defines the `stats`, `load`, `exists`, `info`, `list`, `list_load`, and `first` functions for the `aip.file` Lua module.
//!
//! ---
//!
//! ## Lua documentation for `aip.file` read functions
//!
//! ### Functions
//!
//! - `aip.file.stats(include_globs: string | string[] | nil, options?: {base_dir?: string, absolute?: boolean}): FileStats | nil`
//! - `aip.file.load(rel_path: string, options?: {base_dir: string}): FileRecord`
//! - `aip.file.exists(path: string): boolean`
//! - `aip.file.info(path: string): FileInfo | nil`
//! - `aip.file.list(include_globs: string | string[], options?: {base_dir?: string, absolute?: boolean, with_meta?: boolean}): FileInfo[]`
//! - `aip.file.list_load(include_globs: string | string[], options?: {base_dir?: string, absolute?: boolean}): FileRecord[]`
//! - `aip.file.first(include_globs: string | string[], options?: {base_dir?: string, absolute?: boolean}): FileInfo | nil`

use crate::dir_context::PathResolver;
use crate::runtime::Runtime;
use crate::script::LuaValueExt;
use crate::script::aip_modules::support::{
	base_dir_and_globs, compute_base_dir, create_file_records, list_files_with_options,
};
use crate::script::support::into_option_string;
use crate::support::AsStrsExt;
use crate::types::{FileInfo, FileRecord, FileStats};
use mlua::{IntoLua, Lua, Value};
use simple_fs::{SMeta, SPath, iter_files};

/// ## Lua Documentation
///
/// Calculates aggregate statistics for a set of files matching glob patterns.
///
/// ```lua
/// -- API Signature
/// aip.file.stats(
///   include_globs: string | list<string> | nil,
///   options?: {
///     base_dir?: string,
///     absolute?: boolean
///   }
/// ): FileStats | nil
/// ```
///
/// Finds files matching the `include_globs` patterns within the specified `base_dir` (or workspace root)
/// and returns aggregate statistics about these files in a `FileStats` object.
/// If `include_globs` is `nil` or no files match the patterns, returns `nil`.
///
/// ### Arguments
///
/// - `include_globs: string | list<string> | nil` - A single glob pattern string, a Lua list (table) of glob pattern strings, or `nil`.
///   If `nil`, the function returns `nil`.
///   Globs can include standard wildcards (`*`, `?`, `**`, `[]`). Pack references (e.g., `ns@pack/**/*.md`) are supported.
///   Note: Common build/dependency folders (e.g., `target/`, `node_modules/`, `.build/`, `__pycache__/`)
///
///   are excluded by default unless explicitly matched by the globs.
/// - `options?: table` (optional) - A table containing options:
///   - `base_dir?: string` (optional): The directory relative to which the `include_globs` are applied.
///     Defaults to the workspace root. Pack references (e.g., `ns@pack/`) are supported.
///   - `absolute?: boolean` (optional): Affects how files are resolved internally, but the statistics remain the same regardless.
///
/// ### Returns
///
/// - `FileStats`: A `FileStats` object containing aggregate statistics about the matching files.
/// - `nil` if `include_globs` is `nil`
///
/// If no files if ound a FileStats will all 0 will be returned.
///
/// ### Example
///
/// ```lua
/// -- Get statistics for all Markdown files in the 'docs' directory
/// local stats = aip.file.stats("*.md", { base_dir = "docs" })
/// if stats then
///   print("Number of files:", stats.number_of_files)
///   print("Total size:", stats.total_size)
///   print("First created:", stats.ctime_first)
///   print("Last modified:", stats.mtime_last)
/// end
///
/// -- Get statistics for all '.aip' files in a specific pack
/// local agent_stats = aip.file.stats("**/*.aip", { base_dir = "ns@pack/" })
/// if agent_stats then
///   print("Total agent files:", agent_stats.number_of_files)
/// end
///
/// -- Nil globs return nil
/// local nil_stats = aip.file.stats(nil)
/// print(nil_stats) -- Output: nil
/// ```
///
/// ### Error
///
/// Returns an error if:
/// - `include_globs` is not a string, a list of strings, or `nil`.
/// - `base_dir` cannot be resolved (e.g., invalid pack reference).
/// - An error occurs during file system traversal or glob matching.
///
pub(super) fn file_stats(
	lua: &Lua,
	runtime: &Runtime,
	include_globs: Value,
	options: Option<Value>,
) -> mlua::Result<Value> {
	// Handle nil globs
	if include_globs.is_nil() {
		return Ok(Value::Nil);
	}

	let (base_path, include_globs) = base_dir_and_globs(runtime, include_globs, options.as_ref())?;
	let absolute = options.x_get_bool("absolute").unwrap_or(false);

	let file_refs = list_files_with_options(runtime, base_path.as_ref(), &include_globs.x_as_strs(), absolute, false)?;

	if file_refs.is_empty() {
		return FileStats::default().into_lua(lua);
	}

	// We need metadata to compute stats
	let smetas: Vec<&SMeta> = file_refs.iter().filter_map(|f_ref| f_ref.meta()).collect();

	// Compute aggregate statistics
	let mut total_size: u64 = 0;
	let mut number_of_files: u64 = 0;
	let mut ctime_first: Option<i64> = None;
	let mut ctime_last: Option<i64> = None;
	let mut mtime_first: Option<i64> = None;
	let mut mtime_last: Option<i64> = None;

	for smeta in smetas {
		number_of_files += 1;
		total_size += smeta.size;

		let ctime = smeta.created_epoch_us;
		let mtime = smeta.modified_epoch_us;

		ctime_first = Some(match ctime_first {
			Some(v) => v.min(ctime),
			None => ctime,
		});

		ctime_last = Some(match ctime_last {
			Some(v) => v.max(ctime),
			None => ctime,
		});

		mtime_first = Some(match mtime_first {
			Some(v) => v.min(mtime),
			None => mtime,
		});

		mtime_last = Some(match mtime_last {
			Some(v) => v.max(mtime),
			None => mtime,
		});
	}

	let file_stats = FileStats {
		total_size,
		number_of_files,
		ctime_first: ctime_first.unwrap_or(0),
		ctime_last: ctime_last.unwrap_or(0),
		mtime_first: mtime_first.unwrap_or(0),
		mtime_last: mtime_last.unwrap_or(0),
	};

	let res = file_stats.into_lua(lua)?;

	Ok(res)
}

/// ## Lua Documentation
///
/// Loads a [`FileRecord`] with its content.
///
/// ```lua
/// -- API Signature
/// aip.file.load(rel_path: string, options?: {base_dir: string}): FileRecord
/// ```
///
/// Loads the file specified by `rel_path` and returns a `FileRecord` object containing
/// the file's metadata and its content.
///
/// ### Arguments
///
/// - `rel_path: string` - The path to the file, relative to the `base_dir` or workspace root.
/// - `options?: table` - An optional table containing:
///   - `base_dir: string` (optional): The base directory from which `rel_path` is resolved. Defaults to the workspace root. Pack references (e.g., `ns@pack/`) can be used.
///
/// ### Returns
///
/// - `FileRecord`: A [`FileRecord`] object.
///
/// ### Example
///
/// ```lua
/// local readme = aip.file.load("doc/README.md")
/// print(readme.path)    -- Output: "doc/README.md"
/// print(readme.name)    -- Output: "README.md"
/// print(#readme.content) -- Output: <length of content>
///
/// local agent_file = aip.file.load("agent.aip", { base_dir = "ns@pack/" })
/// print(agent_file.path) -- Output: "agent.aip" (relative to the resolved base_dir)
/// ```
///
/// ### Error
///
/// Returns an error if:
/// - The `base_dir` cannot be resolved (e.g., invalid pack reference).
/// - The final file path cannot be resolved.
/// - The file does not exist or cannot be read.
/// - Metadata cannot be retrieved.
///
/// ```ts
/// {
///   error: string // Error message
/// }
/// ```
pub(super) fn file_load(
	lua: &Lua,
	runtime: &Runtime,
	rel_path: String,
	options: Option<Value>,
) -> mlua::Result<mlua::Value> {
	let dir_context = runtime.dir_context();
	let base_path = compute_base_dir(runtime, options.as_ref())?;
	let full_path = dir_context.resolve_path(
		runtime.session(),
		(&rel_path).into(),
		PathResolver::WksDir,
		base_path.as_ref(),
	)?;
	let full_path = match (base_path, full_path.is_absolute()) {
		(Some(base_path), false) => base_path.join(full_path),
		_ => full_path,
	};

	let rel_path = SPath::new(rel_path);

	let file_record = FileRecord::load_from_full_path(runtime.dir_context(), &full_path, rel_path)?;
	let res = file_record.into_lua(lua)?;

	Ok(res)
}

/// ## Lua Documentation
///
/// Checks if a file or directory exists at the given path.
///
/// ```lua
/// -- API Signature
/// aip.file.exists(path: string): boolean
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
/// if aip.file.exists("README.md") then
///   print("README.md exists.")
/// end
///
/// if aip.file.exists("ns@pack/main.aip") then
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
pub(super) fn file_exists(_lua: &Lua, runtime: &Runtime, path: String) -> mlua::Result<bool> {
	Ok(crate::script::support::path_exits(runtime, &path))
}

/// ## Lua Documentation
///
/// Retrieves file metadata ([`FileInfo`]) for the specified path.
///
/// ```lua
/// -- API Signature
/// aip.file.info(path: string): FileInfo | nil
/// ```
///
/// If the given `path` exists, this function returns a [`FileInfo`] object
/// containing the file metadata (no content).
/// If the path cannot be resolved or the file does not exist, it returns `nil`.
///
/// ### Arguments
///
/// - `path: string` – The file or directory path. Can be relative, absolute,
///   or use pack references (`ns@pack/...`, `ns@pack$workspace/...`, etc.).
///
/// ### Returns
///
/// - `FileInfo | nil`: Metadata for the file, or `nil` when not found.
///
/// ### Example
///
/// ```lua
/// local meta = aip.file.info("README.md")
/// if meta then
///   print("Size:", meta.size)
/// end
/// ```
///
/// ### Error
///
/// Returns an error only if the path cannot be resolved (invalid pack
/// reference, invalid format, …). If the path resolves successfully but the
/// file does not exist, the function simply returns `nil`.
pub(super) fn file_info(lua: &Lua, runtime: &Runtime, path: Value) -> mlua::Result<Value> {
	let Some(path) = into_option_string(path, "aip.text.replace_markers")? else {
		return Ok(Value::Nil);
	};

	// Empty string ‑> nil directly.
	if path.trim().is_empty() {
		return Ok(Value::Nil);
	}

	let rel_path = SPath::new(path);
	let full_path =
		runtime
			.dir_context()
			.resolve_path(runtime.session(), rel_path.clone(), PathResolver::WksDir, None)?;

	if !full_path.is_file() {
		return Ok(Value::Nil);
	}

	// TODO: Might want to put it ~ in case absolute and home based
	let file_info = FileInfo::new(runtime.dir_context(), rel_path, &full_path);
	file_info.into_lua(lua)
}

/// ## Lua Documentation
///
/// Lists file metadata ([`FileInfo`]) matching glob patterns.
///
/// ```lua
/// -- API Signature
/// aip.file.list(
///   include_globs: string | list<string>,
///   options?: {
///     base_dir?: string,
///     absolute?: boolean,
///     with_meta?: boolean
///   }
/// ): list<FileInfo>
/// ```
///
/// Finds files matching the `include_globs` patterns within the specified `base_dir` (or workspace root)
/// and returns a list of `FileInfo` objects containing information about each file (path, name, timestamps, size, etc.),
/// but *not* the file content.
///
/// ### Arguments
///
/// - `include_globs: string | list<string>` - A single glob pattern string or a Lua list (table) of glob pattern strings.
///   Globs can include standard wildcards (`*`, `?`, `**`, `[]`). Pack references (e.g., `ns@pack/**/*.md`) are supported.
///   Note: Common build/dependency folders (e.g., `target/`, `node_modules/`, `.build/`, `__pycache__/`)
///
///   are excluded by default unless explicitly matched by the globs.
/// - `options?: table` (optional) - A table containing options:
///   - `base_dir?: string` (optional): The directory relative to which the `include_globs` are applied.
///     Defaults to the workspace root. Pack references (e.g., `ns@pack/`) are supported.
///   - `absolute?: boolean` (optional): If `true`, the `path` in the returned `FileInfo` objects will be absolute.
///     If `false` (default), the `path` will be relative to the `base_dir`. If a path resolves outside the `base_dir`
///     (e.g., using `../` in globs), it will be returned as an absolute path even if `absolute` is false.
///   - `with_meta?: boolean` (optional): If `false`, the function will skip fetching detailed metadata
///     (`ctime`, `mtime`, `size`) for each file, potentially improving performance
///     if only the path information is needed. Defaults to `true`.
///
/// ### Returns
///
/// - `list<FileInfo>`: A list of [`FileInfo`] objects. Returns an empty list if no files match.
///
/// ### Example
///
/// ```lua
/// -- List all Markdown files in the 'docs' directory (relative paths)
/// local doc_files = aip.file.list("*.md", { base_dir = "docs" })
/// for _, file in ipairs(doc_files) do
///   print(file.path) -- e.g., "guide.md", "api.md"
/// end
///
/// -- List all '.aip' files in a specific pack (absolute paths, no detailed meta)
/// local agent_files = aip.file.list("**/*.aip", {
///   base_dir = "ns@pack/",
///   absolute = true,
///   with_meta = false
/// })
/// for _, file in ipairs(agent_files) do
///   print(file.path) -- e.g., "/path/to/workspace/.aipack/ns/pack/agent1.aip"
/// end
///
/// -- List text and config files from the workspace root
/// local config_files = aip.file.list({"*.txt", "*.config"})
/// for _, file in ipairs(config_files) do
///   print(file.path, file.size) -- e.g., "notes.txt", 1024
/// end
/// ```
///
/// ### Error
///
/// Returns an error if:
/// - `include_globs` is not a string or a list of strings.
/// - `base_dir` cannot be resolved (e.g., invalid pack reference).
/// - An error occurs during file system traversal or glob matching.
/// - Metadata cannot be retrieved (and `with_meta` is true).
///
/// ```ts
/// {
///   error: string // Error message
/// }
/// ```
pub(super) fn file_list(
	lua: &Lua,
	runtime: &Runtime,
	include_globs: Value,
	options: Option<Value>,
) -> mlua::Result<Value> {
	let (base_path, include_globs) = base_dir_and_globs(runtime, include_globs, options.as_ref())?;
	let absolute = options.x_get_bool("absolute").unwrap_or(false);

	// NOTE: For now, not `with_meta` flag always true. Might add it to `list_files_with_options` later.
	// Default is true, as we want convenient APIs, and offer user way to optimize it
	// let with_meta = options.x_get_bool("with_meta").unwrap_or(true);

	let spaths = list_files_with_options(runtime, base_path.as_ref(), &include_globs.x_as_strs(), absolute, true)?;

	let file_infos: Vec<FileInfo> = spaths
		.into_iter()
		.map(|f_ref| FileInfo::from_file_ref(runtime.dir_context(), f_ref))
		.collect();

	let res = file_infos.into_lua(lua)?;

	Ok(res)
}

/// ## Lua Documentation
///
/// Lists and loads files ([`FileRecord`]) matching glob patterns.
///
/// ```lua
/// -- API Signature
/// aip.file.list_load(
///   include_globs: string | list<string>,
///   options?: {
///     base_dir?: string,
///     absolute?: boolean
///   }
/// ): list<FileRecord>
/// ```
///
/// Finds files matching the `include_globs` patterns within the specified `base_dir` (or workspace root),
/// loads the content of each matching file, and returns a list of `FileRecord` objects.
/// Each `FileRecord` contains both metadata and the file content.
///
/// ### Arguments
///
/// - `include_globs: string | list<string>` - A single glob pattern string or a Lua list (table) of glob pattern strings.
///   Globs can include standard wildcards (`*`, `?`, `**`, `[]`). Pack references (e.g., `ns@pack/**/*.md`) are supported.
///   Note: Common build/dependency folders (e.g., `target/`, `node_modules/`, `.build/`, `__pycache__/`)
///   are excluded by default unless explicitly matched by the globs.
///
/// - `options?: table` (optional) - A table containing options:
///   - `base_dir?: string` (optional): The directory relative to which the `include_globs` are applied.
///     Defaults to the workspace root. Pack references (e.g., `ns@pack/`) are supported.
///   - `absolute?: boolean` (optional): If `true`, the paths used internally and potentially the `path` in the returned `FileRecord`
///     objects will be absolute. If `false` (default), paths will generally be relative to the `base_dir`.
///     Note: The exact path stored in `FileRecord.path` depends on internal resolution logic, especially if paths resolve outside `base_dir`.
///
/// ### Returns
///
/// - `list<FileRecord>`: A list of [`FileRecord`] objects, each with content.
///   Returns an empty list if no files match.
///
/// ### Example
///
/// ```lua
/// -- Load all Markdown files in the 'docs' directory
/// local doc_files = aip.file.list_load("*.md", { base_dir = "docs" })
/// for _, file in ipairs(doc_files) do
///   print("--- File:", file.path, "---")
///   print(file.content)
/// end
///
/// -- Load all '.aip' files in a specific pack
/// local agent_files = aip.file.list_load("**/*.aip", { base_dir = "ns@pack/" })
/// for _, file in ipairs(agent_files) do
///   print("Agent Name:", file.stem)
/// end
/// ```
///
/// ### Error
///
/// Returns an error if:
/// - `include_globs` is not a string or a list of strings.
/// - `base_dir` cannot be resolved (e.g., invalid pack reference).
/// - An error occurs during file system traversal or glob matching.
/// - Any matching file cannot be read or its metadata retrieved.
///
/// ```ts
/// {
///   error: string // Error message
/// }
/// ```
pub(super) fn file_list_load(
	lua: &Lua,
	runtime: &Runtime,
	include_globs: Value,
	options: Option<Value>,
) -> mlua::Result<Value> {
	let (base_path, include_globs) = base_dir_and_globs(runtime, include_globs, options.as_ref())?;

	let absolute = options.x_get_bool("absolute").unwrap_or(false);

	let file_refs = list_files_with_options(runtime, base_path.as_ref(), &include_globs.x_as_strs(), absolute, true)?;

	let file_records = create_file_records(runtime, file_refs, base_path.as_ref(), absolute)?;

	let res = file_records.into_lua(lua)?;

	Ok(res)
}

/// ## Lua Documentation
///
/// Finds the first file matching glob patterns and returns its [`FileInfo`].
///
/// ```lua
/// -- API Signature
/// aip.file.first(
///   include_globs: string | list<string>,
///   options?: {
///     base_dir?: string,
///     absolute?: boolean
///   }
/// ): FileInfo | nil
/// ```
///
/// Searches for files matching the `include_globs` patterns within the specified `base_dir` (or workspace root).
/// It stops searching as soon as the first matching file is found and returns its `FileInfo` object (metadata only, no content).
/// If no matching file is found, it returns `nil`.
///
/// ### Arguments
///
/// - `include_globs: string | list<string>` - A single glob pattern string or a Lua list (table) of glob pattern strings.
///   Globs can include standard wildcards (`*`, `?`, `**`, `[]`). Pack references (e.g., `ns@pack/**/*.md`) are supported.
///   Note: Common build/dependency folders (e.g., `target/`, `node_modules/`, `.build/`, `__pycache__/`)
///   are excluded by default unless explicitly matched by the globs.
///
/// - `options?: table` (optional) - A table containing options:
///   - `base_dir?: string` (optional): The directory relative to which the `include_globs` are applied.
///     Defaults to the workspace root. Pack references (e.g., `ns@pack/`) are supported.
///   - `absolute?: boolean` (optional): If `true`, the `path` in the returned `FileInfo` object (if found) will be absolute.
///     If `false` (default), the `path` will be relative to the `base_dir`. Similar to `aip.file.list`, paths outside `base_dir` become absolute.
///
/// ### Returns
///
/// - `FileInfo | nil`: A [`FileInfo`] object for the first matching file, or `nil` if no match is found.
///
/// ### Example
///
/// ```lua
/// -- Find the first '.aip' file in a pack
/// local agent_meta = aip.file.first("*.aip", { base_dir = "ns@pack/" })
/// if agent_meta then
///   print("Found agent:", agent_meta.path)
///   -- To load its content:
///   -- local agent_file = aip.file.load(agent_meta.path, { base_dir = "ns@pack/" })
///   -- print(agent_file.content)
/// else
///   print("No agent file found in pack.")
/// end
///
/// -- Find any config file in the root
/// local config_meta = aip.file.first({"*.toml", "*.yaml", "*.json"}, { base_dir = "." })
/// if config_meta then
///   print("Config file:", config_meta.name)
/// end
/// ```
///
/// ### Error
///
/// Returns an error if:
/// - `include_globs` is not a string or a list of strings.
/// - `base_dir` cannot be resolved (e.g., invalid pack reference).
/// - An error occurs during file system traversal or glob matching *before* the first file is found.
/// - Metadata cannot be retrieved for the first found file.
///
/// ```ts
/// {
///   error: string // Error message
/// }
/// ```
pub(super) fn file_first(
	lua: &Lua,
	runtime: &Runtime,
	include_globs: Value,
	options: Option<Value>,
) -> mlua::Result<Value> {
	let (base_path, include_globs) = base_dir_and_globs(runtime, include_globs, options.as_ref())?;

	let absolute = options.x_get_bool("absolute").unwrap_or(false);

	let base_path = match base_path {
		Some(base_path) => base_path.clone(),
		None => runtime
			.dir_context()
			.wks_dir()
			.ok_or(crate::Error::custom("Cannot create file records, no workspace"))?
			.clone(),
	};

	let mut sfiles = iter_files(
		&base_path,
		Some(&include_globs.iter().map(|s| s.as_str()).collect::<Vec<&str>>()),
		Some(simple_fs::ListOptions::from_relative_glob(!absolute)),
	)
	.map_err(crate::Error::from)?;

	let Some(sfile) = sfiles.next() else {
		return Ok(Value::Nil);
	};

	let absolute_path = SPath::from(&sfile);

	let spath = if absolute {
		sfile
	} else {
		sfile
			.try_diff(&base_path)
			.map_err(|err| crate::Error::cc("Cannot diff with base_path", err))?
	};

	let res = FileInfo::new(runtime.dir_context(), spath, &absolute_path).into_lua(lua)?;

	Ok(res)
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{assert_contains, eval_lua, run_reflective_agent, setup_lua};
	use crate::script::aip_modules::aip_file;
	use serde_json::Value;
	use simple_fs::SPath;
	use std::collections::HashMap;
	use value_ext::JsonValueExt as _;

	#[tokio::test]
	async fn test_lua_file_load_simple_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_path = "./agent-script/agent-hello.aip";

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.load("{fx_path}")"#), None).await?;

		// -- Check
		assert_contains(res.x_get_str("content")?, "from agent-hello.aip");
		assert_eq!(res.x_get_str("path")?, fx_path);
		assert_eq!(res.x_get_str("name")?, "agent-hello.aip");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_load_pack_ref_simple() -> Result<()> {
		// -- Setup & Fixtures
		let fx_path = "ns_b@pack_b_2/main.aip";

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.load("{fx_path}")"#), None).await?;

		// -- Check
		assert_contains(res.x_get_str("content")?, "custom ns_b@pack_b_2 main.aip");
		assert_contains(res.x_get_str("path")?, "pack_b_2/main.aip");
		assert_eq!(res.x_get_str("name")?, "main.aip");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_load_pack_ref_base_support() -> Result<()> {
		// -- Setup & Fixtures
		let fx_path = "ns_b@pack_b_2$base/extra/test.txt";

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.load("{fx_path}")"#), None).await?;

		// -- Check
		assert_contains(
			res.x_get_str("content")?,
			"Some support content - ..@..$base/extra/test.txt",
		);
		// NOTE: right now it gives back the path given (the `ns_b@pack_b_2$base/...`)
		// Will be resolve that this: ".aipack-base/support/pack/ns_b/pack_b_2/extra/test.txt",
		assert_contains(res.x_get_str("path")?, fx_path);

		assert_eq!(res.x_get_str("name")?, "test.txt");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_load_pack_ref_workspace_support() -> Result<()> {
		// -- Setup & Fixtures
		let fx_path = "ns_a@pack_a_1$workspace/extra/test.txt";

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.load("{fx_path}")"#), None).await?;

		// -- Check
		assert_contains(
			res.x_get_str("content")?,
			"Some support content - ..@..$workspace/extra/test.txt",
		);
		// NOTE: right now it gives back the path given (the `ns_a@pack_a_1$workspace/...`)
		// ".aipack/support/pack/ns_a/pack_a_1/extra/test.txt",
		assert_contains(res.x_get_str("path")?, fx_path);
		assert_eq!(res.x_get_str("name")?, "test.txt");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_exists_true() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_file::init_module, "file").await?;
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
			// Pack references
			"ns_b@pack_b_2/main.aip",
			"ns_a@pack_a_1$workspace/extra/test.txt",
		];

		// -- Exec & Check
		for path in paths {
			let code = format!(r#"return aip.file.exists("{path}")"#);
			let res = eval_lua(&lua, &code)?;
			assert!(res.as_bool().ok_or("Result should be a bool")?, "Should exist: {path}");
		}

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_info_ok() -> Result<()> {
		// -- Exec
		let fx_path = "./agent-script/agent-hello.aip";
		let res = run_reflective_agent(&format!(r#"return aip.file.info("{fx_path}")"#), None).await?;

		// -- Check
		assert_eq!(res.x_get_str("name")?, "agent-hello.aip");
		assert_eq!(res.x_get_str("path")?, fx_path);

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_info_not_found() -> Result<()> {
		// -- Exec
		let res = run_reflective_agent(r#"return aip.file.info("not/a/file.txt")"#, None).await?;

		// -- Check
		assert_eq!(res, serde_json::Value::Null, "Should have returned null");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_exists_false() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_file::init_module, "file").await?;
		let paths = &[
			"./no file .rs",
			"some/no-file.md",
			"./s do/",
			"no-dir/at/all",
			"non_existent_ns@non_existent_pack/file.txt",
		];

		// -- Exec & Check
		for path in paths {
			let code = format!(r#"return aip.file.exists("{path}")"#);
			let res = eval_lua(&lua, &code);

			let res = res?;
			assert!(
				!res.as_bool().ok_or("Result should be a bool")?,
				"Should NOT exist: {path}"
			);
		}
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_list_glob_direct() -> Result<()> {
		// -- Fixtures
		// This is the rust Path logic
		let glob = "*.*";

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.list("{glob}");"#), None).await?;

		// -- Check
		let res_paths = to_res_paths(&res)?;
		assert_eq!(res_paths.len(), 2, "result length");
		assert_contains(&res_paths, "file-01.txt");
		assert_contains(&res_paths, "file-02.txt");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_list_support_workspace() -> Result<()> {
		// -- Fixtures
		// This is the rust Path logic
		let glob = "ns_b@pack_b_2$base/**/*.*";

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.list("{glob}");"#), None).await?;

		// -- Check
		let res = res.as_array().ok_or("Should return an array")?;
		assert_eq!(res.len(), 1, "result length");
		let item = res.first().ok_or("Should have one item")?;
		assert_contains(item.x_get_str("path")?, "extra/test.txt");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_list_glob_deep() -> Result<()> {
		// -- Fixtures
		// This is the rust Path logic
		let glob = "sub-dir-a/**/*.*";

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.list("{glob}");"#), None).await?;

		// -- Check
		let res_paths = to_res_paths(&res)?;
		assert_eq!(res_paths.len(), 3, "result length");
		assert_contains(&res_paths, "sub-dir-a/sub-sub-dir/agent-hello-3.aip");
		assert_contains(&res_paths, "sub-dir-a/sub-sub-dir/main.aip");
		assert_contains(&res_paths, "sub-dir-a/agent-hello-2.aip");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_list_glob_abs_with_wild() -> Result<()> {
		// -- Fixtures
		let lua = setup_lua(aip_file::init_module, "file").await?;
		let dir = SPath::new("./tests-data/config");
		let dir = dir
			.canonicalize()
			.map_err(|err| format!("Cannot canonicalize {dir:?} cause: {err}"))?;

		// This is the rust Path logic
		let glob = format!("{dir}/*.*");
		let code = format!(r#"return aip.file.list("{glob}");"#);

		// -- Exec
		let res = eval_lua(&lua, &code)?;

		// -- Check
		let res = res.as_array().ok_or("Should be array")?;

		assert_eq!(res.len(), 1);
		let val = res.first().ok_or("Should have one item")?;
		assert_eq!(val.x_get_str("name")?, "config-current-with-aliases.toml");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_list_glob_with_base_dir_all_nested() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(super::super::init_module, "file").await?;
		let lua_code = r#"
local files = aip.file.list({"**/*.*"}, {base_dir = "sub-dir-a"})
return { files = files }
        "#;

		// -- Exec
		let res = eval_lua(&lua, lua_code)?;

		// -- Check
		let files = res
			.get("files")
			.ok_or("Should have .files")?
			.as_array()
			.ok_or("file should be array")?;

		assert_eq!(files.len(), 3, ".files.len() should be 3");

		// Just create a map fby name
		let file_by_name: HashMap<&str, &Value> =
			files.iter().map(|v| (v.x_get_str("name").unwrap_or_default(), v)).collect();

		let file = file_by_name.get("main.aip").ok_or("Should have 'main.aip'")?;
		assert_eq!(file.x_get_str("path")?, "sub-sub-dir/main.aip");
		assert!(file.x_get_i64("size")? >= 0, "Should have size >= 0");
		let file = file_by_name.get("agent-hello-3.aip").ok_or("Should have 'agent-hello-3.aip'")?;
		assert_eq!(file.x_get_str("path")?, "sub-sub-dir/agent-hello-3.aip");
		assert!(file.x_get_i64("size")? >= 0, "Should have size >= 0");
		let file = file_by_name.get("agent-hello-2.aip").ok_or("Should have 'agent-hello-2.aip'")?;
		assert_eq!(file.x_get_str("path")?, "agent-hello-2.aip");
		assert!(file.x_get_i64("size")? >= 0, "Should have size >= 0");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_list_glob_with_base_dir_one_level() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(super::super::init_module, "file").await?;
		let lua_code = r#"
local files = aip.file.list({"agent-hello-*.aip"}, {base_dir = "sub-dir-a"})
return { files = files }
        "#;

		// -- Exec
		let res = eval_lua(&lua, lua_code)?;

		// -- Check
		let files = res
			.get("files")
			.ok_or("Should have .files")?
			.as_array()
			.ok_or("file should be array")?;

		assert_eq!(files.len(), 1, ".files.len() should be 1");
		// NOTE: Here we assume the order will be deterministic and the same across OSes (tested on Mac).
		//       This logic might need to be changed, or actually, the list might need to have a fixed order.
		assert_eq!(
			"agent-hello-2.aip",
			files.first().ok_or("Should have a least one file")?.x_get_str("name")?
		);

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_first_glob_deep() -> Result<()> {
		// -- Fixtures
		// This is the rust Path logic
		let glob = "sub-dir-a/**/*-2.*";

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.first("{glob}");"#), None).await?;

		// -- Check
		// let res_paths = to_res_paths(&res);
		assert_eq!(res.x_get_str("name")?, "agent-hello-2.aip");
		assert_eq!(res.x_get_str("path")?, "sub-dir-a/agent-hello-2.aip");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_first_not_found() -> Result<()> {
		// -- Fixtures
		// This is the rust Path logic
		let glob = "sub-dir-a/**/*-not-a-thing.*";

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.first("{glob}")"#), None).await?;

		// -- Check
		assert_eq!(res, serde_json::Value::Null, "Should have returned null");

		Ok(())
	}

	// region:    --- Support for Tests

	fn to_res_paths(res: &serde_json::Value) -> Result<Vec<&str>> {
		let arr = res.as_array().ok_or("should have array of path")?;

		let v = arr
			.iter()
			.map(|v| v.x_get_as::<&str>("path").unwrap_or_default())
			.collect::<Vec<&str>>();

		Ok(v)
	}

	// endregion: --- Support for Tests
}

// endregion: --- Tests
