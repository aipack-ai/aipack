//! Defines DOCX-related helpers for the `aip.file` Lua module.
//!
//! ---
//!
//! ## Lua documentation for `aip.file` DOCX helpers
//!
//! ### Functions
//!
//! - `aip.file.save_docx_to_md(docx_path: string, dest?: DestOptions): FileInfo`
//! - `aip.file.load_docx_as_md(docx_path: string): string`
//!
//! This helper loads a DOCX file, converts it to Markdown, saves the result,
//! and returns the [`FileInfo`] describing the newly-created file.
//!
use crate::Error;
use crate::dir_context::PathResolver;
use crate::runtime::Runtime;
use crate::script::aip_modules;
use crate::types::FileInfo;
use mlua::{IntoLua, Lua, Value};
use simple_fs::SPath;
use std::fs::write;
use std::path::Path;

/// ## Lua Documentation
///
/// Loads a DOCX file, converts its content to Markdown, and saves it according to `dest` options.
///
/// ```lua
/// -- API Signature
/// aip.file.save_docx_to_md(
///   docx_path: string,
///   dest?: string | {
///     base_dir?: string,
///     file_name?: string,
///     suffix?: string
///   }
/// ): FileInfo
/// ```
///
/// ### Arguments
///
/// - `docx_path: string`
///   Path to the source DOCX file, relative to the workspace root.
///
/// - `dest: DestOptions (optional)`
///   Destination path or options table:
///
///   - `string`
///     Path to save the `.md` file (relative or absolute).
///
///   - `table` (`DestOptions`):
///       - `base_dir?: string`: Base directory for resolving the destination.
///       - `file_name?: string`: Custom file name for the Markdown output.
///       - `suffix?: string`: Suffix appended to the source file stem before `.md`.
///
/// ### Returns
///
/// - `FileInfo`
///   Metadata about the created Markdown file (path, name, stem, ext, timestamps, size).
///
/// ### Example
///
/// ```lua
/// -- Default (replaces .docx with .md):
/// aip.file.save_docx_to_md("docs/spec.docx")
///
/// -- Using a custom string path:
/// aip.file.save_docx_to_md("docs/spec.docx", "out/spec.md")
///
/// -- Using options table:
/// aip.file.save_docx_to_md("docs/spec.docx", {
///   base_dir = "output",
///   suffix = "_v2",
/// })
/// ```
pub(super) fn file_save_docx_to_md(
	lua: &Lua,
	runtime: &Runtime,
	docx_path: String,
	dest: Value,
) -> mlua::Result<Value> {
	let dir_context = runtime.dir_context();

	// -- resolve source path
	let rel_docx = SPath::new(docx_path.clone());
	let full_docx = dir_context.resolve_path(runtime.session(), rel_docx.clone(), PathResolver::WksDir, None)?;

	// -- convert to Markdown using support::docx
	let md_content = crate::support::docx::docx_convert(Path::new(full_docx.as_str())).map_err(|e| {
		Error::Custom(format!(
			"Failed to convert DOCX file '{docx_path}' to Markdown.\nCause: {e}"
		))
	})?;

	// -- determine destination paths using the helper
	let (rel_md, full_md) = aip_modules::support::resolve_dest_path(lua, runtime, &rel_docx, dest, "md", None)?;

	// -- write out and return metadata
	simple_fs::ensure_file_dir(&full_md).map_err(Error::from)?;
	write(&full_md, md_content)
		.map_err(|e| Error::Custom(format!("Failed to write Markdown file '{rel_md}'.\nCause: {e}")))?;

	let meta = FileInfo::new(runtime.dir_context(), rel_md, &full_md);
	meta.into_lua(lua)
}

/// ## Lua Documentation
///
/// Loads a DOCX file, converts it to Markdown, and returns the Markdown content as a string.
///
/// ```lua
/// -- API Signature
/// aip.file.load_docx_as_md(docx_path: string): string
/// ```
///
/// ### Arguments
///
/// - `docx_path: string`
///   Path to the source DOCX file, relative to the workspace root.
///
/// ### Returns
///
/// - `string`
///   The Markdown content converted from the DOCX file.
///
/// ### Error
///
/// Returns an error if:
/// - The DOCX file cannot be found or read,
/// - The DOCX cannot be converted to Markdown.
pub(super) fn file_load_docx_as_md(lua: &Lua, runtime: &Runtime, docx_path: String) -> mlua::Result<Value> {
	let dir_context = runtime.dir_context();

	// -- resolve source path
	let rel_docx = SPath::new(docx_path.clone());
	let full_docx = dir_context.resolve_path(runtime.session(), rel_docx, PathResolver::WksDir, None)?;

	// -- convert to Markdown using support::docx
	let md_content = crate::support::docx::docx_convert(Path::new(full_docx.as_str())).map_err(|e| {
		Error::Custom(format!(
			"Failed to convert DOCX file '{docx_path}' to Markdown.\nCause: {e}"
		))
	})?;

	md_content.into_lua(lua)
}
