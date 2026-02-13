//! Defines the `aip.udiffx` module, used in the Lua engine.
//!
//! This module provides functionality to apply multi-file changes (New, Patch, Rename, Delete)
//! encoded in the `<FILE_CHANGES>` envelope format, using the `udiffx` crate.

use crate::Result;
use crate::runtime::Runtime;
use crate::support::W;
use crate::types::Extrude;
use mlua::{IntoLua, Lua, MultiValue, Table, Value};
use udiffx::ApplyChangesStatus;

// region:    --- Module Init

pub fn init_module(lua: &Lua, runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	let runtime_inner = runtime.clone();
	let apply_file_changes_fn = lua.create_function(move |lua, args| apply_file_changes(lua, &runtime_inner, args))?;

	table.set("apply_file_changes", apply_file_changes_fn)?;

	Ok(table)
}

// endregion: --- Module Init

/// ## Lua Documentation
///
/// Applies multi-file changes from a `<FILE_CHANGES>` envelope.
///
/// ```lua
/// -- API Signatures
/// aip.udiffx.apply_file_changes(content: string, base_dir?: string, options?: {extrude?: "content"}): status, remaining
/// ```
///
/// Scans `content` for a `<FILE_CHANGES>` block and applies the directives within it.
/// Directives include `New`, `Patch` (supporting Unified Diff and simplified `@@` hunk headers), `Rename`, and `Delete`.
/// All paths in the envelope are resolved relative to `base_dir`.
///
/// ### Arguments
///
/// - `content: string`: The raw text containing the `<FILE_CHANGES>...</FILE_CHANGES>` envelope.
/// - `base_dir: string | nil` (optional): The directory where file changes will be applied, relative to the workspace root.
///   Defaults to the workspace root if `nil` or `""`.
/// - `options: table` (optional):
///   - `extrude?: "content"` (optional): If set, returns the `content` string without the first `<FILE_CHANGES>` block as a second return value.
///
/// ### Returns
///
/// 1. `status: table`:
///    - `success: boolean`: `true` if all directives were applied successfully.
///    - `total_count: number`: Total number of directives found.
///    - `success_count: number`: Number of successful directives.
///    - `fail_count: number`: Number of failed directives.
///    - `items: array<table>`: List of results for each directive:
///      - `file_path: string`: Path of the affected file.
///      - `kind: string`: One of `"New"`, `"Patch"`, `"Rename"`, `"Delete"`, or `"Fail"`.
///      - `success: boolean`: `true` if this directive succeeded.
///      - `error_msg: string | nil`: Error details if `success` is `false`.
/// 2. `remaining: string | nil`: The content without the extracted block (only if `options.extrude == "content"`).
///
/// ### Example
///
/// ```lua
/// local ai_response = [[
/// Here are the changes:
/// <FILE_CHANGES>
/// <FILE_NEW file_path="src/new_file.rs">
/// pub fn hello() { println!("Hello"); }
/// </FILE_NEW>
/// </FILE_CHANGES>
/// ]]
///
/// local status, remaining = aip.udiffx.apply_file_changes(ai_response, ".", {extrude = "content"})
/// if status.success then
///     print("Changes applied successfully!")
/// end
/// ```
fn apply_file_changes(
	lua: &Lua,
	runtime: &Runtime,
	(content, base_dir, options): (String, Option<String>, Option<Value>),
) -> mlua::Result<MultiValue> {
	// -- 1) Process Options (extrude)
	let extrude = match options {
		Some(Value::Table(table)) => Extrude::extract_from_table_value(&table)?,
		_ => None,
	};
	let do_extrude = matches!(extrude, Some(Extrude::Content));

	// -- 2) Extract changes
	let (file_changes, extruded_content) =
		udiffx::extract_file_changes(&content, do_extrude).map_err(crate::Error::from)?;

	// -- 3) Resolve base_dir
	let base_dir_str = base_dir.unwrap_or_default();
	let abs_base_dir = runtime.resolve_path_default(base_dir_str.into(), None)?;

	// -- 4) Apply changes
	let status = udiffx::apply_file_changes(&abs_base_dir, file_changes).map_err(crate::Error::from)?;

	// -- 5) Build return values
	let mut values = MultiValue::new();
	values.push_back(W(status).into_lua(lua)?);

	if do_extrude {
		let remain = extruded_content.unwrap_or_default();
		values.push_back(Value::String(lua.create_string(&remain)?));
	}

	Ok(values)
}

// region:    --- IntoLua Implementations

impl IntoLua for W<ApplyChangesStatus> {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		let status = self.0;
		let mut total_count = 0;
		let mut success_count = 0;
		let mut fail_count = 0;

		let table = lua.create_table()?;
		let items_table = lua.create_table()?;

		for (i, item) in status.items.iter().enumerate() {
			total_count += 1;
			if item.success() {
				success_count += 1;
			} else {
				fail_count += 1;
			}

			let info_table = lua.create_table()?;
			info_table.set("file_path", item.file_path())?;
			info_table.set("kind", item.kind())?;
			info_table.set("success", item.success())?;
			info_table.set("error_msg", item.error_msg())?;

			items_table.set(i + 1, info_table)?;
		}

		table.set("total_count", total_count)?;
		table.set("success_count", success_count)?;
		table.set("fail_count", fail_count)?;
		table.set("items", items_table)?;
		table.set("success", fail_count == 0)?;

		Ok(Value::Table(table))
	}
}

// endregion: --- IntoLua Implementations
