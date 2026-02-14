//! Defines file writing functions for the `aip.file` Lua module.
//!
//! ---
//!
//! ## Lua API
//!
//! The `aip.file` module exposes helper functions that allow Lua scripts to
//! interact with the file-system in a workspace-aware and pack-aware manner.
//! This module specifically contains the write operations.
//!
//! ### Functions
//!
//! - `aip.file.save(rel_path: string, content: string, options?: SaveOptions) : FileInfo`
//! - `aip.file.copy(src_path: string, dest_path: string, options?: {overwrite?: boolean}) : FileInfo`
//! - `aip.file.move(src_path: string, dest_path: string, options?: {overwrite?: boolean}) : FileInfo`
//! - `aip.file.append(rel_path: string, content: string)       : FileInfo`
//! - `aip.file.ensure_exists(path: string, content?, options?)  : FileInfo`

use crate::Error;
use crate::dir_context::PathResolver;
use crate::hub::get_hub;
use crate::runtime::Runtime;
use crate::script::aip_modules::support::{check_access_delete, check_access_write, process_path_reference};
use crate::support::files::safer_trash_file;
use crate::support::text::{ensure_single_trailing_newline, trim_end_if_needed, trim_start_if_needed};
use crate::types::{FileInfo, FileOverOptions, SaveOptions};
use mlua::{FromLua, IntoLua, Lua, Value};
use simple_fs::ensure_file_dir;
use std::fs::{File, write};
use std::io::Write;

/// ## Lua Documentation
///
/// Saves string content to a file, returning a [`FileInfo`] object.
///
/// ```lua
/// -- API Signature
/// aip.file.save(rel_path: string, content: string, options?: SaveOptions): FileInfo
/// ```
///
/// Writes the provided `content` string to the file specified by `rel_path`.
/// The path is resolved relative to the workspace root. If the file exists, it will be overwritten.
/// If the directory structure does not exist, it will be created.
///
/// **Important Security Note:** For security reasons, this function currently restricts saving files
/// outside the workspace directory (`./`) or the shared base directory (`~/.aipack-base/`).
///
/// ### Arguments
///
/// - `rel_path: string` - The path to the file where the content should be saved, relative to the workspace root.
/// - `content: string`  - The string content to write to the file.
/// - `options: SaveOptions` (optional) - Options to pre-process content before saving:
///   - `trim_start?: boolean`: If true, remove leading whitespace.
///   - `trim_end?: boolean`: If true, remove trailing whitespace.
///   - `single_trailing_newline?: boolean`: If true, ensure exactly one trailing newline.
///
/// ### Returns
///
/// - `FileInfo`: A [`FileInfo`] object for the saved file.
///
/// ### Example
///
/// ```lua
/// -- Save documentation to a file in the 'docs' directory
/// aip.file.save("docs/new_feature.md", "# New Feature\n\nDetails about the feature.")
///
/// -- Overwrite an existing file, applying trimming
/// aip.file.save("config.txt", "  new_setting=true  \n", {trim_start = true, trim_end = true})
/// ```
///
/// ### Error
///
/// Returns an error if:
/// - The path attempts to write outside the allowed workspace or base directories.
/// - The directory structure cannot be created.
/// - The file cannot be written due to permissions or other I/O errors.
/// - The operation requires a workspace context, but none is found.
///
/// ```ts
/// {
///   error: string // Error message (e.g., save file protection, permission denied, ...)
/// }
/// ```
pub(super) fn file_save(
	lua: &Lua,
	runtime: &Runtime,
	rel_path: String,
	mut content: String,
	options: Option<SaveOptions>,
) -> mlua::Result<mlua::Value> {
	let dir_context = runtime.dir_context();
	let full_path = dir_context.resolve_path(runtime.session(), (&rel_path).into(), PathResolver::WksDir, None)?;

	// Apply options if present
	if let Some(opts) = options
		&& !opts.is_empty()
	{
		// 1. Apply trimming
		if opts.should_trim_start() {
			content = trim_start_if_needed(content);
		}
		if opts.should_trim_end() {
			content = trim_end_if_needed(content);
		}

		// 2. Ensure single trailing newline
		if opts.should_single_trailing_newline() {
			content = ensure_single_trailing_newline(content);
		}
	}

	// We might not want that once workspace is truely optional
	let wks_dir = dir_context.try_wks_dir_with_err_ctx("aip.file.save requires a aipack workspace setup")?;

	check_access_write(&full_path, wks_dir)?;

	ensure_file_dir(&full_path).map_err(Error::from)?;

	write(&full_path, content).map_err(|err| Error::custom(format!("Fail to save file {rel_path}.\nCause {err}")))?;

	let rel_path = full_path.diff(wks_dir).unwrap_or_else(|| full_path.clone());
	get_hub().publish_sync(format!("-> Lua aip.file.save called on: {rel_path}"));

	let file_info = FileInfo::new(runtime.dir_context(), full_path, true);
	let file_info = file_info.into_lua(lua)?;

	Ok(file_info)
}

/// ## Lua Documentation
///
/// Moves a file from `src_path` to `dest_path`, returning a [`FileInfo`] object for the destination.
///
/// ```lua
/// -- API Signature
/// aip.file.move(src_path: string, dest_path: string, options?: {overwrite?: boolean}): FileInfo
/// ```
///
/// Renames (moves) the file at `src_path` to `dest_path`.
/// Both paths are resolved relative to the workspace root and support pack references (`ns@pack/...`).
/// Parent directories for the destination are created automatically if they don't exist.
///
/// ### Arguments
///
/// - `src_path: string` - The source file path.
/// - `dest_path: string` - The destination file path.
/// - `options?: table` (optional) - Options:
///   - `overwrite?: boolean`: If `false`, the operation fails if the destination exists. Defaults to `false`.
///
/// ### Returns
///
/// - `FileInfo`: A [`FileInfo`] object for the moved destination file.
///
/// ### Error
///
/// Returns an error if the source file doesn't exist, if the source or destination is outside the workspace
/// (or restricted areas), or if an I/O error occurs (e.g., cross-device move failure).
pub(super) fn file_move(
	lua: &Lua,
	runtime: &Runtime,
	src_path: String,
	dest_path: String,
	options: Option<FileOverOptions>,
) -> mlua::Result<mlua::Value> {
	let dir_context = runtime.dir_context();
	let options = options.unwrap_or_default();

	let src_full = process_path_reference(runtime, &src_path)?;
	let dest_full = process_path_reference(runtime, &dest_path)?;

	// We might not want that once workspace is truely optional
	let wks_dir = dir_context.try_wks_dir_with_err_ctx("aip.file.move requires a aipack workspace setup")?;

	check_access_delete(&src_full, wks_dir)?;
	check_access_write(&dest_full, wks_dir)?;

	if !src_full.exists() {
		return Err(Error::custom(format!("Move file failed - Source `{src_path}` does not exist")).into());
	}

	if !options.overwrite() && dest_full.exists() {
		return Err(Error::custom(format!(
			"Move file failed - Destination `{dest_path}` already exists and overwrite is set to false.\nUse `aip.file.move(src_path, dst_path,{{overwrite = true}}` to allow overwrite."
		))
		.into());
	}

	ensure_file_dir(&dest_full).map_err(Error::from)?;

	std::fs::rename(&src_full, &dest_full)
		.map_err(|err| Error::custom(format!("Fail to move from `{src_path}` to `{dest_path}`.\nCause {err}")))?;

	let rel_dest = dest_full.diff(wks_dir).unwrap_or_else(|| dest_full.clone());
	get_hub().publish_sync(format!("-> Lua aip.file.move called to: {rel_dest}"));

	let file_info = FileInfo::new(runtime.dir_context(), dest_full, true);
	let file_info = file_info.into_lua(lua)?;

	Ok(file_info)
}

/// ## Lua Documentation
///
/// Copies a file from `src_path` to `dest_path`, returning a [`FileInfo`] object for the destination.
///
/// ```lua
/// -- API Signature
/// aip.file.copy(src_path: string, dest_path: string, options?: {overwrite?: boolean}): FileInfo
/// ```
///
/// Performs a binary, streaming copy of the file at `src_path` to `dest_path`.
/// Both paths are resolved relative to the workspace root and support pack references (`ns@pack/...`).
/// Parent directories for the destination are created automatically if they don't exist.
///
/// ### Arguments
///
/// - `src_path: string` - The source file path.
/// - `dest_path: string` - The destination file path.
/// - `options?: table` (optional) - Options:
///   - `overwrite?: boolean`: If `false`, the operation fails if the destination exists. Defaults to `false`.
///
/// ### Returns
///
/// - `FileInfo`: A [`FileInfo`] object for the copied destination file.
///
/// ### Error
///
/// Returns an error if the source file doesn't exist, if the destination is outside the workspace,
/// or if an I/O error occurs.
pub(super) fn file_copy(
	lua: &Lua,
	runtime: &Runtime,
	src_path: String,
	dest_path: String,
	options: Option<FileOverOptions>,
) -> mlua::Result<mlua::Value> {
	let dir_context = runtime.dir_context();
	let options = options.unwrap_or_default();

	let src_full = process_path_reference(runtime, &src_path)?;
	let dest_full = process_path_reference(runtime, &dest_path)?;

	// We might not want that once workspace is truely optional
	let wks_dir = dir_context.try_wks_dir_with_err_ctx("aip.file.copy requires a aipack workspace setup")?;

	check_access_write(&dest_full, wks_dir)?;

	if !options.overwrite() && dest_full.exists() {
		return Err(Error::custom(format!(
			"Copy file failed - Destination `{dest_path}` already exists and overwrite is set to false.\nUse `aip.file.copy(src_path, dst_path,{{overwrite = true}}` to allow overwrite."
		))
		.into());
	}

	ensure_file_dir(&dest_full).map_err(Error::from)?;

	let mut src_file = File::open(&src_full)
		.map_err(|err| Error::custom(format!("Fail to open source file `{src_path}` for copy.\nCause {err}")))?;

	let mut dest_file = File::create(&dest_full).map_err(|err| {
		Error::custom(format!(
			"Fail to create destination file `{dest_path}` for copy.\nCause {err}"
		))
	})?;

	std::io::copy(&mut src_file, &mut dest_file)
		.map_err(|err| Error::custom(format!("Fail to copy from `{src_path}` to `{dest_path}`.\nCause {err}")))?;

	let rel_dest = dest_full.diff(wks_dir).unwrap_or_else(|| dest_full.clone());
	get_hub().publish_sync(format!("-> Lua aip.file.copy called to: {rel_dest}"));

	let file_info = FileInfo::new(runtime.dir_context(), dest_full, true);
	let file_info = file_info.into_lua(lua)?;

	Ok(file_info)
}

/// ## Lua Documentation
///
/// Deletes a file, returning a boolean indicating success.
///
/// ```lua
/// -- API Signature
/// aip.file.delete(path: string): boolean
/// ```
///
/// Attempts to delete the file specified by `path`.
/// The path is resolved relative to the workspace root. If the file does not exist, returns `false`.
///
/// Security:
/// - Deleting files is only allowed within the current workspace directory.
/// - Deleting files under the shared base directory (`~/.aipack-base/`) is not allowed.
///
/// ### Arguments
///
/// - `path: string` - The path to the file to delete, relative to the workspace root.
///
/// ### Returns
///
/// - `boolean`: `true` if a file was deleted, `false` if the file did not exist.
///
/// ### Error
///
/// Returns an error if:
/// - The path attempts to delete outside the allowed workspace directory.
/// - The target is in the `.aipack-base` folder (always forbidden).
/// - The file cannot be deleted due to permissions or other I/O errors.
/// - The operation requires a workspace context, but none is found.
pub(super) fn file_delete(lua: &Lua, runtime: &Runtime, rel_path: String) -> mlua::Result<mlua::Value> {
	let dir_context = runtime.dir_context();
	let full_path = dir_context.resolve_path(runtime.session(), (&rel_path).into(), PathResolver::WksDir, None)?;

	// We might not want that once workspace is truely optional
	let wks_dir = dir_context.try_wks_dir_with_err_ctx("aip.file.delete requires a aipack workspace setup")?;

	check_access_delete(&full_path, wks_dir)?;

	let removed = if full_path.exists() {
		// std::fs::remove_file(&full_path).map(|_| true).map_err(Error::from)?
		safer_trash_file(&full_path, None)?
	} else {
		false
	};

	if removed {
		let rel_path = full_path.diff(wks_dir).unwrap_or_else(|| full_path.clone());
		get_hub().publish_sync(format!("-> Lua aip.file.delete called on: {rel_path}"));
	}

	removed.into_lua(lua)
}

/// ## Lua Documentation
///
/// Appends string content to a file, returning an updated [`FileInfo`] object.
///
/// ```lua
/// -- API Signature
/// aip.file.append(rel_path: string, content: string): FileInfo
/// ```
///
/// Appends the provided `content` string to the end of the file specified by `rel_path`.
/// The path is resolved relative to the workspace root. If the file does not exist, it will be created.
/// If the directory structure does not exist, it will be created.
///
/// ### Arguments
///
/// - `rel_path: string` - The path to the file where the content should be appended, relative to the workspace root.
/// - `content: string`  - The string content to append to the file.
///
/// ### Returns
///
/// - `FileInfo`: The updated [`FileInfo`] for the file.
///
/// ### Example
///
/// ```lua
/// -- Append a log entry to a log file
/// aip.file.append("logs/app.log", "INFO: User logged in.\n")
///
/// -- Create a file and append if it doesn't exist
/// aip.file.append("notes.txt", "- Remember to buy milk.\n")
/// aip.file.append("notes.txt", "- Finish report.\n")
/// ```
///
/// ### Error
///
/// Returns an error if:
/// - The directory structure cannot be created.
/// - The file cannot be opened for appending (e.g., due to permissions).
/// - An I/O error occurs during writing.
///
/// ```ts
/// {
///   error: string // Error message (e.g., permission denied, I/O error)
/// }
/// ```
pub(super) fn file_append(
	lua: &Lua,
	runtime: &Runtime,
	rel_path: String,
	content: String,
) -> mlua::Result<mlua::Value> {
	let path = runtime
		.dir_context()
		.resolve_path(runtime.session(), (&rel_path).into(), PathResolver::WksDir, None)?;

	ensure_file_dir(&path).map_err(Error::from)?;

	let mut file = std::fs::OpenOptions::new()
		.append(true)
		.create(true)
		.open(&path)
		.map_err(Error::from)?;

	file.write_all(content.as_bytes())?;

	// NOTE: Could be too many prints
	// get_hub().publish_sync(format!("-> Lua aip.file.append called on: {}", rel_path));

	let file_info = FileInfo::new(runtime.dir_context(), path, true);
	let file_info = file_info.into_lua(lua)?;

	Ok(file_info)
}

/// ## Lua Documentation
///
/// Ensures a file exists, returning its [`FileInfo`].
///
/// If the file does not exist, it's created with optional content.
/// If it exists and is empty, it can be optionally overwritten.
///
/// ```lua
/// -- API Signature
/// aip.file.ensure_exists(path: string, content?: string, options?: {content_when_empty?: boolean}): FileInfo
/// ```
///
/// Checks if the file at `path` (relative to the workspace root) exists.
/// - If the file does not exist:
///   - Creates the necessary directory structure.
///   - Creates the file.
///   - Writes the `content` (or an empty string if `content` is nil) to the new file.
/// - If the file exists:
///   - Checks if `options.content_when_empty` is true.
///   - If true, checks if the file is empty (contains only whitespace or is zero-length).
///   - If the file is empty and `content_when_empty` is true, overwrites the file with `content` (or an empty string if `content` is nil).
///
/// This function is intended for files, not directories.
///
/// ### Arguments
///
/// - `path: string` - The path to the file, relative to the workspace root.
/// - `content?: string` (optional) - The content to write to the file if it's created or if it's empty and `content_when_empty` is true. Defaults to an empty string if nil.
/// - `options?: table` (optional) - A table containing options:
///   - `content_when_empty?: boolean` (optional): If true, the `content` will be written to the file if the file is empty (or only contains whitespace). Defaults to `false`.
///
/// ### Returns
///
/// - `FileInfo`: The [`FileInfo`] for the file.
///
/// ### Example
///
/// ```lua
/// -- Ensure a config file exists, creating it with defaults if needed
/// local config_content = "-- Default Settings --\nenabled=true"
/// local file_info = aip.file.ensure_exists("config/settings.lua", config_content)
/// print("Ensured file:", file_info.path)
///
/// -- Ensure a log file exists, but don't overwrite if it has content
/// aip.file.ensure_exists("logs/activity.log")
///
/// -- Ensure a placeholder file exists, writing content only if it's empty
/// local placeholder = "-- TODO: Add content --"
/// aip.file.ensure_exists("src/module.lua", placeholder, {content_when_empty = true})
/// ```
///
/// ### Error
///
/// Returns an error if:
/// - The directory structure cannot be created.
/// - The file cannot be created or written to (e.g., due to permissions).
/// - Metadata cannot be retrieved for the file.
///
/// ```ts
/// {
///   error: string // Error message
/// }
/// ```
pub(super) fn file_ensure_exists(
	lua: &Lua,
	runtime: &Runtime,
	path: String,
	content: Option<String>,
	options: Option<EnsureExistsOptions>,
) -> mlua::Result<mlua::Value> {
	let options = options.unwrap_or_default();

	let rel_path = simple_fs::SPath::new(path);
	let full_path =
		runtime
			.dir_context()
			.resolve_path(runtime.session(), rel_path.clone(), PathResolver::WksDir, None)?;

	// if the file does not exist, create it.
	if !full_path.exists() {
		simple_fs::ensure_file_dir(&full_path).map_err(|err| Error::custom(err.to_string()))?;
		let content = content.unwrap_or_default();
		write(&full_path, content)?;
	}
	// if we have the options.content_when_empty flag, if empty
	else if options.content_when_empty && crate::support::files::is_file_empty(&full_path)? {
		let content = content.unwrap_or_default();
		write(&full_path, content)?;
	}

	let file_info = FileInfo::new(runtime.dir_context(), rel_path, &full_path);

	file_info.into_lua(lua)
}

// region:    --- Options
#[derive(Debug, Default)]
pub struct EnsureExistsOptions {
	/// Set the eventual provided content if the file is empty (only whitespaces)
	pub content_when_empty: bool,
}

impl FromLua for EnsureExistsOptions {
	fn from_lua(value: Value, _lua: &Lua) -> mlua::Result<Self> {
		let table = value
			.as_table()
			.ok_or(crate::Error::custom("EnsureExistsOptions should be a table"))?;
		let set_content_when_empty = table.get("content_when_empty")?;
		Ok(Self {
			content_when_empty: set_content_when_empty,
		})
	}
}

// endregion: --- Options

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{assert_contains, run_reflective_agent};
	use crate::runtime::Runtime;

	/// Note: need the multi-thread, because save do a `get_hub().publish_sync`
	///       which does a tokio blocking (requiring multi thread)
	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_lua_file_save_simple_ok() -> Result<()> {
		// -- Setup & Fixtures
		let runtime = Runtime::new_test_runtime_sandbox_01().await?;
		let dir_context = runtime.dir_context();
		let fx_dest_path = dir_context
			.wks_dir()
			.ok_or("Should have workspace setup")?
			.join(".tmp/test_lua_file_save_simple_ok.md");
		let fx_content = "hello from test_file_save_simple_ok";

		// -- Exec
		let _res = run_reflective_agent(
			&format!(r#"return aip.file.save("{fx_dest_path}", "{fx_content}");"#),
			None,
		)
		.await?;

		// -- Check
		let file_content = std::fs::read_to_string(fx_dest_path)?;
		assert_eq!(file_content, fx_content);

		Ok(())
	}

	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_lua_file_save_ok_in_base() -> Result<()> {
		// -- Setup & Fixtures
		let runtime = Runtime::new_test_runtime_sandbox_01().await?;
		let dir_context = runtime.dir_context();
		let fx_dest_path = dir_context
			.aipack_paths()
			.aipack_base_dir()
			.join(".tmp/test_lua_file_save_ok_in_base.md");
		let fx_content = "hello from test_lua_file_save_ok_in_base";

		// -- Exec
		let _res = run_reflective_agent(
			&format!(r#"return aip.file.save("{fx_dest_path}", "{fx_content}");"#),
			None,
		)
		.await?;

		// -- Check
		let file_content = std::fs::read_to_string(fx_dest_path)?;
		assert_eq!(file_content, fx_content);

		Ok(())
	}

	/// Note: need the multi-thread, because save do a `get_hub().publish_sync`
	///       which does a tokio blocking (requiring multi thread)
	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_lua_file_save_err_out_workspace() -> Result<()> {
		// -- Setup & Fixtures
		let runtime = Runtime::new_test_runtime_sandbox_01().await?;
		let dir_context = runtime.dir_context();
		let fx_dest_path = dir_context
			.wks_dir()
			.ok_or("Should have workspace setup")?
			.join("../.tmp/test_lua_file_save_err_out_workspace.md");
		let fx_content = "hello from test_lua_file_save_err_out_workspace";

		// -- Exec
		let res = run_reflective_agent(
			&format!(r#"return aip.file.save("{fx_dest_path}", "{fx_content}");"#),
			None,
		)
		.await;

		// -- Check
		let Err(err) = res else { panic!("Should return error") };
		assert!(err.to_string().contains("does not belong to the workspace dir"));

		Ok(())
	}

	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_lua_file_save_with_trim_start() -> Result<()> {
		// -- Setup & Fixtures
		let runtime = Runtime::new_test_runtime_sandbox_01().await?;
		let dir_context = runtime.dir_context();
		let fx_dest_path = dir_context
			.wks_dir()
			.ok_or("Should have workspace setup")?
			.join(".tmp/test_lua_file_save_with_trim_start.md");
		let fx_content_in = "   Leading spaces and content.\\n";
		let fx_content_expected = "Leading spaces and content.\n";
		let fx_options = "{trim_start = true}";

		// -- Exec
		let lua_code = format!(
			r#"return aip.file.save("{}", "{}", {});"#,
			fx_dest_path, fx_content_in, fx_options
		);
		let _res = run_reflective_agent(&lua_code, None).await?;

		// -- Check
		let file_content = std::fs::read_to_string(fx_dest_path)?;
		assert_eq!(file_content, fx_content_expected);

		Ok(())
	}

	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_lua_file_save_with_trim_end() -> Result<()> {
		// -- Setup & Fixtures
		let runtime = Runtime::new_test_runtime_sandbox_01().await?;
		let dir_context = runtime.dir_context();
		let fx_dest_path = dir_context
			.wks_dir()
			.ok_or("Should have workspace setup")?
			.join(".tmp/test_lua_file_save_with_trim_end.md");
		// Note: trailing spaces, tab, and newlines will be removed.
		let fx_content_in = "Content and trailing spaces.   \\n\\t";
		let fx_content_expected = "Content and trailing spaces.";
		let fx_options = "{trim_end = true}";

		// -- Exec
		let lua_code = format!(
			r#"return aip.file.save("{}", "{}", {});"#,
			fx_dest_path, fx_content_in, fx_options
		);
		let _res = run_reflective_agent(&lua_code, None).await?;

		// -- Check
		let file_content = std::fs::read_to_string(fx_dest_path)?;
		assert_eq!(file_content, fx_content_expected);

		Ok(())
	}

	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_lua_file_save_with_single_trailing_newline_add() -> Result<()> {
		// -- Setup & Fixtures
		let runtime = Runtime::new_test_runtime_sandbox_01().await?;
		let dir_context = runtime.dir_context();
		let fx_dest_path = dir_context
			.wks_dir()
			.ok_or("Should have workspace setup")?
			.join(".tmp/test_lua_file_save_with_single_trailing_newline_add.md");
		let fx_content_in = "Content without newline";
		let fx_content_expected = "Content without newline\n";
		let fx_options = "{single_trailing_newline = true}";

		// -- Exec
		let lua_code = format!(
			r#"return aip.file.save("{}", "{}", {});"#,
			fx_dest_path, fx_content_in, fx_options
		);
		let _res = run_reflective_agent(&lua_code, None).await?;

		// -- Check
		let file_content = std::fs::read_to_string(fx_dest_path)?;
		assert_eq!(file_content, fx_content_expected);

		Ok(())
	}

	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_lua_file_save_with_single_trailing_newline_remove_extra() -> Result<()> {
		// -- Setup & Fixtures
		let runtime = Runtime::new_test_runtime_sandbox_01().await?;
		let dir_context = runtime.dir_context();
		let fx_dest_path = dir_context
			.wks_dir()
			.ok_or("Should have workspace setup")?
			.join(".tmp/test_lua_file_save_with_single_trailing_newline_remove_extra.md");
		let fx_content_in = "Content with multiple newlines\\n\\n\\n"; // 3 newlines
		let fx_content_expected = "Content with multiple newlines\n"; // 1 newline
		let fx_options = "{single_trailing_newline = true}";

		// -- Exec
		let lua_code = format!(
			r#"return aip.file.save("{}", "{}", {});"#,
			fx_dest_path, fx_content_in, fx_options
		);
		let _res = run_reflective_agent(&lua_code, None).await?;

		// -- Check
		let file_content = std::fs::read_to_string(fx_dest_path)?;
		assert_eq!(file_content, fx_content_expected);

		Ok(())
	}

	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_lua_file_save_with_combo_options() -> Result<()> {
		// -- Setup & Fixtures
		let runtime = Runtime::new_test_runtime_sandbox_01().await?;
		let dir_context = runtime.dir_context();
		let fx_dest_path = dir_context
			.wks_dir()
			.ok_or("Should have workspace setup")?
			.join(".tmp/test_lua_file_save_with_combo_options.md");

		// Content: leading space, trailing space, multiple trailing newlines
		let fx_content_in = "   Content with all trims and newlines.  \\n\\n";

		// Expected result after trim_start, trim_end, then single_trailing_newline
		let fx_content_expected = "Content with all trims and newlines.\n";
		let fx_options = "{trim_start = true, trim_end = true, single_trailing_newline = true}";

		// -- Exec
		let lua_code = format!(
			r#"return aip.file.save("{}", "{}", {});"#,
			fx_dest_path, fx_content_in, fx_options
		);
		let _res = run_reflective_agent(&lua_code, None).await?;

		// -- Check
		let file_content = std::fs::read_to_string(fx_dest_path)?;
		assert_eq!(file_content, fx_content_expected);

		Ok(())
	}

	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_lua_file_copy_simple_ok() -> Result<()> {
		// -- Setup & Fixtures
		let runtime = Runtime::new_test_runtime_sandbox_01().await?;
		let dir_context = runtime.dir_context();
		let fx_src_path = "agent-script/agent-hello.aip";
		let fx_dest_path = dir_context
			.wks_dir()
			.ok_or("Should have workspace setup")?
			.join(".tmp/test_lua_file_copy_simple_ok.aip");

		// -- Exec
		let _res = run_reflective_agent(
			&format!(r#"return aip.file.copy("{fx_src_path}", "{fx_dest_path}");"#),
			None,
		)
		.await?;

		// -- Check
		assert!(fx_dest_path.exists());
		let src_content = std::fs::read_to_string(fx_src_path)?;
		let dest_content = std::fs::read_to_string(fx_dest_path)?;
		assert_eq!(src_content, dest_content);

		Ok(())
	}

	#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
	async fn test_lua_file_move_simple_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_src = ".tmp/move_src.txt";
		let fx_dest = ".tmp/move_dest.txt";
		let fx_content = "move content";

		// -- Exec
		let res = run_reflective_agent(
			&format!(
				r#"
                aip.file.save("{fx_src}", "{fx_content}")
                local info = aip.file.move("{fx_src}", "{fx_dest}")
                return {{
                    exists_src = aip.file.exists("{fx_src}"),
                    exists_dest = aip.file.exists("{fx_dest}"),
                    dest_path = info.path
                }}
            "#
			),
			None,
		)
		.await?;

		// -- Check
		assert!(!res.x_get_bool("exists_src")?);
		assert!(res.x_get_bool("exists_dest")?);
		assert_contains(res.x_get_str("dest_path")?, fx_dest);

		Ok(())
	}

	#[tokio::test(flavor = "multi_thread")]
	async fn test_lua_file_tmp_with_ctx() -> Result<()> {
		// -- Setup & Fixtures
		let fx_content = "Hello tmp content";
		let fx_path = "my-dir/some-tmp-file.aip";
		let fx_code = format!(
			r#"
local path = CTX.TMP_DIR .. "/{fx_path}"
aip.file.save(path,"{fx_content}")
return {{
   file    = aip.file.load(path),
	 session = CTX.SESSION_UID
}}
		"#
		);

		// -- Exec
		let res = run_reflective_agent(&fx_code, None).await?;

		// -- Check
		let content = res.x_get_str("/file/content")?;
		let path = res.x_get_str("/file/path")?;
		let session = res.x_get_str("session")?;

		assert_eq!(content, fx_content);
		assert_ends_with(path, &format!(".aipack/.session/{session}/tmp/{fx_path}"));

		Ok(())
	}

	#[tokio::test(flavor = "multi_thread")]
	async fn test_lua_file_tmp_with_var() -> Result<()> {
		// -- Setup & Fixtures
		let fx_content = "Hello tmp content";
		let fx_path = "my-dir/tmp_with_var_file.txt";
		let fx_tmp_path = format!("$tmp/{fx_path}");
		let fx_code = format!(
			r#"
	local path = "{fx_tmp_path}"
	aip.file.save(path,"{fx_content}")
	local files = aip.file.list_load("$tmp/**/*.*")
	return {{
	   file    = aip.file.load(path),
		 files   = files,
		 session = CTX.SESSION_UID
	}}
			"#
		);

		// -- Exec
		let res = run_reflective_agent(&fx_code, None).await?;

		// -- Check
		let content = res.x_get_str("/file/content")?;
		let path = res.x_get_str("/file/path")?;
		let file_name = res.x_get_str("/file/name")?;
		// let session = res.x_get_str("session")?;

		assert_eq!(content, fx_content);
		assert_eq!(file_name, "tmp_with_var_file.txt");
		assert_ends_with(path, &fx_tmp_path);
		// assert_ends_with(path, &format!("$tmp/{fx_path}"));

		Ok(())
	}

	// region:    --- Support for Tests

	use crate::_test_support::assert_ends_with;
	use value_ext::JsonValueExt as _;

	// endregion: --- Support for Tests
}

// endregion: --- Tests
