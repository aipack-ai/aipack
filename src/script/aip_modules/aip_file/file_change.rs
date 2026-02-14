#![allow(clippy::doc_markdown)]
//! Defines change-saving helpers for the `aip.file` Lua module.
//!
//! ---
//!
//! ## Lua documentation for `aip.file` change helpers
//!
//! ### Functions
//!
//! - `aip.file.save_changes(rel_path: string, changes: string): FileInfo, ChangesInfo`
//!
//! The helper applies an *aip change-block* to a file, saves it, and returns
//! the resulting [`FileInfo`].
//!
use crate::Error;
use crate::dir_context::PathResolver;
use crate::hub::get_hub;
use crate::runtime::Runtime;
use crate::script::aip_modules::support::check_access_write;
use crate::support::text;
use crate::types::{ChangesInfo, FileInfo};
use mlua::{IntoLua, Lua, Value};
use simple_fs::{SPath, ensure_file_dir};
use std::fs::write;

/// ## Lua Documentation
///
/// Applies a set of changes to a file and saves it, returning [`FileInfo`].
///
/// This function is typically used with `aip.rust.find_items` which can generate
/// a changes string.
///
/// ```lua
/// -- API Signature
/// aip.file.save_changes(rel_path: string, changes: string): FileInfo, ChangesInfo
/// ```
///
/// ### Arguments
///
/// - `rel_path: string` - The path to the file to be changed.
/// - `changes: string` - The change block string.
///
/// ### Returns
///
/// - `FileInfo` - A [`FileInfo`] object for the saved file.
/// - `ChangesInfo` - A table containing `changed_count` and `failed_changes`.
///
/// ### Example
///
/// ```lua
/// local changes = aip.rust.find_items(
///   {
///     file = "src/main.rs",
///     find = {
///       kind = "fn",
///       name = "main"
///     }
///   },
///   {
///     replace = {
///       with = "pub fn main() { .. }"
///     }
///   })
/// if changes then
///   aip.file.save_changes("src/main.rs", changes)
/// end
/// ```
pub(super) fn file_save_changes(
	lua: &Lua,
	runtime: &Runtime,
	rel_path: String,
	changes: String,
) -> mlua::Result<(Value, Value)> {
	let dir_context = runtime.dir_context();
	let full_path = dir_context.resolve_path(runtime.session(), (&rel_path).into(), PathResolver::WksDir, None)?;

	// We might not want that once workspace is truely optional
	let wks_dir = dir_context.try_wks_dir_with_err_ctx("aip.file.save requires a aipack workspace setup")?;

	check_access_write(&full_path, wks_dir)?;

	ensure_file_dir(&full_path).map_err(Error::from)?;

	let (content, apply_changes_info) = if full_path.exists() {
		let content = simple_fs::read_to_string(&full_path).map_err(Error::custom)?;
		text::apply_changes(content, changes)?
	} else {
		(
			changes,
			ChangesInfo {
				changed_count: 1,
				failed_changes: Vec::new(),
			},
		)
	};

	write(&full_path, content).map_err(|err| Error::custom(format!("Fail to save file {rel_path}.\nCause {err}")))?;

	let rel_path_for_hub = full_path.diff(wks_dir).unwrap_or_else(|| full_path.clone());
	get_hub().publish_sync(format!("-> Lua aip.file.save called on: {rel_path_for_hub}"));

	let file_info = FileInfo::new(runtime.dir_context(), SPath::new(rel_path), &full_path);
	Ok((file_info.into_lua(lua)?, apply_changes_info.into_lua(lua)?))
}
