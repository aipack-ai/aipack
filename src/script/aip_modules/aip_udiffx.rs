//! Defines the `aip.udiffx` module, used in the Lua engine.
//!
//! This module provides functionality to apply multi-file changes (New, Patch, Rename, Delete)
//! encoded in the `<FILE_CHANGES>` envelope format, using the `udiffx` crate.
//!
//! ---
//!
//! ## Lua documentation for `aip.udiffx` functions
//!
//! ### Functions
//!
//! - `aip.udiffx.apply_file_changes(content: string, base_dir?: string, options?: {extrude?: "content"}): status, remaining`
//! - `aip.udiffx.load_files_context(include_globs: string | string[], options?: {base_dir?: string, absolute?: boolean}): string | nil`
//! - `aip.udiffx.file_changes_instruction(): string`

use crate::runtime::Runtime;
use crate::script::LuaValueExt;
use crate::script::aip_modules::support::{base_dir_and_globs, list_files_with_options};
use crate::support::{AsStrsExt, W};
use crate::types::Extrude;
use crate::{Error, Result};
use mlua::{IntoLua, Lua, MultiValue, Table, Value};
use udiffx::ApplyChangesStatus;

// region:    --- Module Init

pub fn init_module(lua: &Lua, runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	let runtime_inner = runtime.clone();
	let apply_file_changes_fn = lua.create_function(move |lua, args| apply_file_changes(lua, &runtime_inner, args))?;
	let runtime_inner = runtime.clone();
	let load_files_context_fn = lua.create_function(move |lua, args| load_files_context(lua, &runtime_inner, args))?;
	let file_changes_instruction_fn = lua.create_function(file_changes_instruction)?;

	table.set("apply_file_changes", apply_file_changes_fn)?;
	table.set("load_files_context", load_files_context_fn)?;
	table.set("file_changes_instruction", file_changes_instruction_fn)?;

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

/// ## Lua Documentation
///
/// Loads file context blocks for matched files, using the `<FILE_CONTENT>` format.
///
/// ```lua
/// -- API Signatures
/// aip.udiffx.load_files_context(include_globs: string | list<string>, options?: {base_dir?: string, absolute?: boolean}): string | nil
/// ```
/// Finds files matching `include_globs` and returns their content wrapped in `<FILE_CONTENT>` tags.
/// This format is used to provide file context to LLMs.
///
/// Returns `nil` when no files match the globs.
fn load_files_context(
	lua: &Lua,
	runtime: &Runtime,
	(include_globs, options): (Value, Option<Value>),
) -> mlua::Result<Value> {
	let (base_path, include_globs) = base_dir_and_globs(runtime, include_globs, options.as_ref())?;
	let absolute = options.x_get_bool("absolute").unwrap_or(false);

	let file_refs = list_files_with_options(runtime, base_path.as_ref(), &include_globs.x_as_strs(), absolute, true)?;

	if file_refs.is_empty() {
		return Ok(Value::Nil);
	}

	let base_path = match base_path {
		Some(bp) => bp,
		None => runtime
			.dir_context()
			.wks_dir()
			.ok_or_else(|| Error::custom("Workspace dir is missing"))?
			.clone(),
	};

	let mut context = String::new();
	for file_ref in file_refs {
		let full_path = if absolute {
			file_ref.spath.clone()
		} else {
			base_path.join(&file_ref.spath)
		};

		let content = std::fs::read_to_string(full_path).map_err(Error::from)?;

		if !context.is_empty() {
			context.push('\n');
		}

		context.push_str(&format!(r#"<FILE_CONTENT path="{}">"#, file_ref.spath));
		context.push('\n');
		context.push_str(&content);
		if !content.ends_with('\n') {
			context.push('\n');
		}
		context.push_str("</FILE_CONTENT>\n");
	}

	match context.is_empty() {
		false => Ok(Value::String(lua.create_string(&context)?)),
		true => Ok(Value::Nil),
	}
}

/// ## Lua Documentation
///
/// Returns the instruction text describing the `<FILE_CHANGES>` format.
///
/// ```lua
/// -- API Signatures
/// aip.udiffx.file_changes_instruction(): string
/// ```
fn file_changes_instruction(lua: &Lua, (): ()) -> mlua::Result<Value> {
	Ok(Value::String(lua.create_string(udiffx::prompt_file_changes())?))
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
