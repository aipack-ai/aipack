//! Defines HTML-related helpers for the `aip.file` Lua module.
//!
//! ---
//!
//! ## Lua documentation for `aip.file` HTML helpers
//!
//! ### Functions
//!
//! - `aip.file.save_html_to_md(html_path: string, dest?: DestOptions): FileInfo`
//! - `aip.file.save_html_to_slim(html_path: string, dest?: DestOptions): FileInfo`
//!
//! These helpers load an HTML file, transform it (conversion to Markdown or
//! slimming), save the result, and return the [`FileInfo`] describing the
//! newly-created file.
//!
use crate::Error;
use crate::dir_context::PathResolver;
use crate::runtime::Runtime;
use crate::script::{LuaValueExt, aip_modules};
use crate::types::{DestOptions, FileInfo};
use mlua::{FromLua as _, IntoLua, Lua, Value};
use simple_fs::SPath;
use std::fs::{read_to_string, write};

/// ## Lua Documentation
///
/// Loads an HTML file, converts its content to Markdown, and saves it according to `dest` options.
///
/// ```lua
/// -- API Signature
/// aip.file.save_html_to_md(
///   html_path: string,
///   dest?: string | {
///     base_dir?: string,
///     file_name?: string,
///     suffix?: string,
///     slim?: boolean
///   }
/// ): FileInfo
/// ```
///
/// ### Arguments
///
/// - `html_path: string`
///   Path to the source HTML file, relative to the workspace root.
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
///       - `slim?: boolean`: Default true. When `true`, slims HTML (removes scripts, etc.) before conversion to markdown. Defaults to `false`.
///
/// ### Returns
///
/// - `FileInfo`
///   Metadata about the created Markdown file (path, name, stem, ext, timestamps, size).
///
/// ### Example
///
/// ```lua
/// -- Default (replaces .html with .md):
/// aip.file.save_html_to_md("docs/page.html")
///
/// -- Using a custom string path:
/// aip.file.save_html_to_md("docs/page.html", "out/custom.md")
///
/// -- Using options table:
/// aip.file.save_html_to_md("docs/page.html", {
///   base_dir = "output",
///   suffix = "_v2",
///   slim = true,
/// })
/// ```
pub(super) fn file_save_html_to_md(
	lua: &Lua,
	runtime: &Runtime,
	html_path: String,
	dest: Value,
) -> mlua::Result<Value> {
	let dir_context = runtime.dir_context();

	// -- get dest options
	// TODO: Avoid the clone there, the resolve_dest_path should take &DestOptions
	let dest_options: DestOptions = DestOptions::from_lua(dest.clone(), lua)?;
	let do_slim = if let DestOptions::Custom(c) = dest_options {
		c.slim.unwrap_or(true)
	} else {
		true
	};

	// -- resolve and read source
	let rel_html = SPath::new(html_path.clone());
	let full_html = dir_context.resolve_path(runtime.session(), rel_html.clone(), PathResolver::WksDir, None)?;
	let html_content = read_to_string(&full_html)
		.map_err(|e| Error::Custom(format!("Failed to read HTML file '{html_path}'.\nCause: {e}")))?;

	// -- slim if requested
	let html_content = if do_slim {
		crate::support::html::slim(html_content)
			.map_err(|e| Error::Custom(format!("Failed to slim HTML file '{html_path}'.\nCause: {e}")))?
	} else {
		html_content
	};

	// -- convert to Markdown
	let md_content = crate::support::html::to_md(html_content).map_err(|e| {
		Error::Custom(format!(
			"Failed to convert HTML file '{html_path}' to Markdown.\nCause: {e}"
		))
	})?;

	// -- determine destination paths using the helper
	let (rel_md, full_md) = aip_modules::support::resolve_dest_path(lua, runtime, &rel_html, dest, "md", None)?;

	// -- write out and return metadata
	simple_fs::ensure_file_dir(&full_md).map_err(Error::from)?;
	write(&full_md, md_content)
		.map_err(|e| Error::Custom(format!("Failed to write Markdown file '{rel_md}'.\nCause: {e}")))?;

	let meta = FileInfo::new(runtime.dir_context(), rel_md, &full_md);
	meta.into_lua(lua)
}

/// ## Lua Documentation
///
/// Loads an HTML file, "slims" its content (removes scripts, styles, comments, etc.),
/// and saves the slimmed HTML content according to `dest` options.
///
/// ```lua
/// -- API Signature
/// aip.file.save_html_to_slim(
///   html_path: string,
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
/// - `html_path: string`
///   Path to the source HTML file, relative to the workspace root.
///
/// - `dest: DestOptions (optional)`
///   Destination path or options table for the output `.html` file:
///
///   - `nil`: Saves as `[original_name]-slim.html` in the same directory.
///   - `string`: Path to save the slimmed `.html` file (relative or absolute).
///   - `table` (`DestOptions`):
///       - `base_dir?: string`: Base directory for resolving the destination. If provided without `file_name` or `suffix`, the output will be `[original_name].html` in this directory.
///       - `file_name?: string`: Custom file name for the slimmed HTML output.
///       - `suffix?: string`: Suffix appended to the source file stem (e.g., `_slimmed`).
///
/// ### Returns
///
/// - `FileInfo`
///   Metadata about the created slimmed HTML file.
///
/// ### Example
///
/// ```lua
/// -- Default (saves as original-slim.html):
/// aip.file.save_html_to_slim("web/page.html")
/// -- Result: web/page-slim.html
///
/// -- Using a custom string path:
/// aip.file.save_html_to_slim("web/page.html", "output/slim_page.html")
///
/// -- Using options table (base_dir, uses original name):
/// aip.file.save_html_to_slim("web/page.html", { base_dir = "slim_output" })
/// -- Result: slim_output/page.html
///
/// -- Using options table (suffix):
/// aip.file.save_html_to_slim("web/page.html", { suffix = "_light" })
/// -- Result: web/page_light.html
/// ```
pub(super) fn file_save_html_to_slim(
	lua: &Lua,
	runtime: &Runtime,
	html_path: String,
	dest: Value,
) -> mlua::Result<Value> {
	let dir_context = runtime.dir_context();

	// -- resolve and read source
	let rel_html_src = SPath::new(html_path.clone());
	let full_html_src =
		dir_context.resolve_path(runtime.session(), rel_html_src.clone(), PathResolver::WksDir, None)?;
	let html_content = read_to_string(&full_html_src)
		.map_err(|e| Error::Custom(format!("Failed to read HTML file '{html_path}'.\nCause: {e}")))?;

	// -- slim the HTML content
	let slim_html_content = crate::support::html::slim(html_content)
		.map_err(|e| Error::Custom(format!("Failed to slim HTML file '{html_path}'.\nCause: {e}")))?;

	// -- determine destination paths using the helper
	let (rel_html_dest, full_html_dest) =
		aip_modules::support::resolve_dest_path(lua, runtime, &rel_html_src, dest, "html", Some("-slim"))?;

	// -- write out and return metadata
	simple_fs::ensure_file_dir(&full_html_dest).map_err(Error::from)?;
	write(&full_html_dest, slim_html_content).map_err(|e| {
		Error::Custom(format!(
			"Failed to write slimmed HTML file '{rel_html_dest}'.\nCause: {e}"
		))
	})?;

	let meta = FileInfo::new(runtime.dir_context(), rel_html_dest, &full_html_dest);
	meta.into_lua(lua)
}

/// ## Lua Documentation
///
/// Loads an HTML file, "slims" its content (removes scripts, styles, comments, etc.),
/// and returns the slimmed HTML as a string.
///
/// ```lua
/// -- API Signature
/// aip.file.load_html_as_slim(html_path: string): string
/// ```
///
/// ### Arguments
///
/// - `html_path: string`
///   Path to the source HTML file, relative to the workspace root.
///
/// ### Returns
///
/// - `string`
///   The slimmed HTML content.
///
/// ### Error
///
/// Returns an error if:
/// - The HTML file cannot be found or read,
/// - The HTML content cannot be slimmed.
pub(super) fn file_load_html_as_slim(lua: &Lua, runtime: &Runtime, html_path: String) -> mlua::Result<Value> {
	let dir_context = runtime.dir_context();

	// -- resolve and read source
	let rel_html = SPath::new(html_path.clone());
	let full_html = dir_context.resolve_path(runtime.session(), rel_html, PathResolver::WksDir, None)?;
	let html_content = read_to_string(&full_html)
		.map_err(|e| Error::Custom(format!("Failed to read HTML file '{html_path}'.\nCause: {e}")))?;

	// -- slim HTML
	let slim_html_content = crate::support::html::slim(html_content)
		.map_err(|e| Error::Custom(format!("Failed to slim HTML file '{html_path}'.\nCause: {e}")))?;

	slim_html_content.into_lua(lua)
}

/// ## Lua Documentation
///
/// Loads an HTML file, optionally "trims" (slims) its content, converts it to Markdown,
/// and returns the Markdown content as a string.
///
/// ```lua
/// -- API Signature
/// aip.file.load_html_as_md(
///   html_path: string,
///   options?: {
///     trim?: boolean -- default true. When true, slim HTML before converting to Markdown.
///   }
/// ): string
/// ```
///
/// ### Arguments
///
/// - `html_path: string`
///   Path to the source HTML file, relative to the workspace root.
///
/// - `options: table (optional)`
///   - `trim?: boolean` (default: true)
///     When `true`, trims/slims the HTML (removes scripts, styles, comments, etc.) before conversion.
///     Note: For compatibility, `slim` can also be used instead of `trim`.
///
/// ### Returns
///
/// - `string`
///   The Markdown content converted from the (optionally slimmed) HTML.
///
/// ### Error
///
/// Returns an error if:
/// - The HTML file cannot be found or read,
/// - The HTML content cannot be slimmed,
/// - The content cannot be converted to Markdown.
pub(super) fn file_load_html_as_md(
	lua: &Lua,
	runtime: &Runtime,
	html_path: String,
	options: Option<Value>,
) -> mlua::Result<Value> {
	let dir_context = runtime.dir_context();

	// -- resolve and read source
	let rel_html = SPath::new(html_path.clone());
	let full_html = dir_context.resolve_path(runtime.session(), rel_html, PathResolver::WksDir, None)?;
	let html_content = read_to_string(&full_html)
		.map_err(|e| Error::Custom(format!("Failed to read HTML file '{html_path}'.\nCause: {e}")))?;

	// -- decide whether to slim (default true)
	let do_slim = match options {
		Some(Value::Table(t)) => t.x_get_bool("trim").or_else(|| t.x_get_bool("slim")).unwrap_or(true),
		_ => true,
	};

	// -- optionally slim
	let html_content = if do_slim {
		crate::support::html::slim(html_content)
			.map_err(|e| Error::Custom(format!("Failed to slim HTML file '{html_path}'.\nCause: {e}")))?
	} else {
		html_content
	};

	// -- convert to Markdown
	let md_content = crate::support::html::to_md(html_content).map_err(|e| {
		Error::Custom(format!(
			"Failed to convert HTML file '{html_path}' to Markdown.\nCause: {e}"
		))
	})?;

	md_content.into_lua(lua)
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{
		assert_contains, assert_not_contains, clean_sanbox_01_tmp_file, create_sanbox_01_tmp_file,
		gen_sandbox_01_temp_file_path, resolve_sandbox_01_path, run_reflective_agent,
	};
	use simple_fs::{SPath, read_to_string as sfs_read_to_string};
	use value_ext::JsonValueExt;

	const FX_HTML_CONTENT_FULL: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Test Page</title>
    <script>console.log("script")</script>
    <style>.body { margin: 0; }</style>
    <link rel="stylesheet" href="style.css">
</head>
<body>
    <!-- A comment -->
    <h1>Main Title</h1>
    <p>This is a paragraph with <strong>strong</strong> text and <em>emphasized</em> text.</p>
    <ul>
        <li>  Item 1</li>
        <li>Item 2</li>
    </ul>
    <a href="https://example.com">A Link</a>
    <svg><circle cx="50" cy="50" r="40" stroke="black" stroke-width="3" fill="red" /></svg>
</body>
</html>"#;

	#[tokio::test]
	async fn test_script_aip_file_save_html_to_md_simple_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_html_path = create_sanbox_01_tmp_file(
			"test_script_aip_file_save_html_to_md_simple_ok-input.html",
			FX_HTML_CONTENT_FULL,
		)?;

		let md_path = fx_html_path.new_sibling("test_script_aip_file_save_html_to_md_simple_ok-input.md");

		// -- Exec
		let lua_code = format!(r#"return aip.file.save_html_to_md("{fx_html_path}", "{md_path}")"#);
		let res = run_reflective_agent(&lua_code, None).await?;

		// -- Check
		// Check FileInfo result
		assert_eq!(res.x_get_str("path")?, md_path.as_str());
		assert_eq!(res.x_get_str("ext")?, "md");
		assert!(res.x_get_i64("size")? > 0);

		// Check MD content
		let md_full_path = resolve_sandbox_01_path(&md_path);
		let md_content = sfs_read_to_string(&md_full_path)?;
		assert_contains(&md_content, "# Main Title");
		assert_contains(
			&md_content,
			"This is a paragraph with **strong** text and _emphasized_ text.",
		);
		assert_contains(&md_content, "- Item 1");
		assert_contains(&md_content, "- Item 2");
		assert_contains(&md_content, "[A Link](https://example.com)");

		// -- Cleanup
		clean_sanbox_01_tmp_file(md_full_path)?;

		Ok(())
	}

	#[tokio::test]
	async fn test_script_aip_file_save_html_to_md_html_not_found() -> Result<()> {
		let fx_src_path = gen_sandbox_01_temp_file_path("test_script_aip_file_save_html_to_md_html_not_found.html");
		let fx_dst_path = gen_sandbox_01_temp_file_path("test_script_aip_file_save_html_to_md_html_not_found.md");

		// -- Exec
		let lua_code = format!(r#"return aip.file.save_html_to_md("{fx_src_path}", "{fx_dst_path}")"#);

		let Err(res) = run_reflective_agent(&lua_code, None).await else {
			panic!("Should have returned a error")
		};

		// -- Check
		let res = res.to_string();
		assert_contains(&res, "Failed to read HTML file ");

		Ok(())
	}

	// region:    --- Tests for save_html_to_slim

	#[tokio::test]
	async fn test_script_aip_file_save_html_to_slim_default_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_input_filename = "test_slim_default_input.html";
		let fx_html_path = create_sanbox_01_tmp_file(fx_input_filename, FX_HTML_CONTENT_FULL)?;
		let expected_slim_path_rel = fx_html_path.new_sibling(format!("{}-slim.html", fx_html_path.stem()));

		// -- Exec
		let lua_code = format!(r#"return aip.file.save_html_to_slim("{fx_html_path}")"#);
		let res = run_reflective_agent(&lua_code, None).await?;

		// -- Check FileInfo result
		assert_eq!(res.x_get_str("path")?, expected_slim_path_rel.as_str());
		assert_eq!(res.x_get_str("name")?, expected_slim_path_rel.name());
		assert_eq!(res.x_get_str("ext")?, "html");
		assert!(res.x_get_i64("size")? > 0);

		// -- Check slimmed HTML content
		let slim_full_path = resolve_sandbox_01_path(&expected_slim_path_rel);
		let slim_content = sfs_read_to_string(&slim_full_path)?;
		assert_contains(&slim_content, "<h1>Main Title</h1>");
		assert_contains(
			&slim_content,
			"<p>This is a paragraph with <strong>strong</strong> text and <em>emphasized</em> text.</p>",
		);
		assert_not_contains(&slim_content, "<script>");
		assert_not_contains(&slim_content, "<style>");
		assert_not_contains(&slim_content, "<!-- A comment -->");
		assert_not_contains(&slim_content, "<link rel=");
		assert_not_contains(&slim_content, "<svg>");

		// -- Cleanup
		clean_sanbox_01_tmp_file(slim_full_path)?;
		Ok(())
	}

	#[tokio::test]
	async fn test_script_aip_file_save_html_to_slim_dest_string_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_html_path = create_sanbox_01_tmp_file("test_slim_dest_string_input.html", FX_HTML_CONTENT_FULL)?;
		let fx_custom_output_path_str = ".tmp/custom_slim_output.html"; // Relative to sandbox_01
		let expected_slim_path_rel = SPath::new(fx_custom_output_path_str);

		// -- Exec
		let lua_code = format!(r#"return aip.file.save_html_to_slim("{fx_html_path}", "{fx_custom_output_path_str}")"#);
		let res = run_reflective_agent(&lua_code, None).await?;

		// -- Check FileInfo result
		assert_eq!(res.x_get_str("path")?, expected_slim_path_rel.as_str());
		assert_eq!(res.x_get_str("name")?, "custom_slim_output.html");

		// -- Check content (briefly)
		let slim_full_path = resolve_sandbox_01_path(&expected_slim_path_rel);
		let slim_content = sfs_read_to_string(&slim_full_path)?;
		assert_contains(&slim_content, "<h1>Main Title</h1>");

		// -- Cleanup
		clean_sanbox_01_tmp_file(slim_full_path)?;
		Ok(())
	}

	#[tokio::test]
	async fn test_script_aip_file_save_html_to_slim_dest_options_base_dir_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_html_path = create_sanbox_01_tmp_file(
			&format!("{}.html", "test_slim_opts_base_dir_input"),
			FX_HTML_CONTENT_FULL,
		)?;
		let fx_base_dir = ".tmp/output_slim_base"; // Relative to sandbox_01
		// Expected: original name in new base_dir
		let expected_slim_path_rel = SPath::new(format!("{}/{}.html", fx_base_dir, fx_html_path.stem()));

		// -- Exec
		let lua_code =
			format!(r#"return aip.file.save_html_to_slim("{fx_html_path}", {{ base_dir = "{fx_base_dir}" }})"#);
		let res = run_reflective_agent(&lua_code, None).await?;

		// -- Check FileInfo result
		assert_eq!(res.x_get_str("path")?, expected_slim_path_rel.as_str());
		assert_eq!(res.x_get_str("name")?, format!("{}.html", fx_html_path.stem()));

		// -- Check content
		let slim_full_path = resolve_sandbox_01_path(&expected_slim_path_rel);
		let slim_content = sfs_read_to_string(&slim_full_path)?;
		assert_contains(&slim_content, "<h1>Main Title</h1>");

		// -- Cleanup
		clean_sanbox_01_tmp_file(slim_full_path)?;
		// Note: This test creates a directory `.tmp/output_slim_base`.
		// The `clean_sanbox_01_tmp_file` only removes the file.
		// For robust cleanup, the directory might need removal if empty or using a more general cleanup.
		// However, standard test practice often leaves temp dirs for inspection.
		Ok(())
	}

	#[tokio::test]
	async fn test_script_aip_file_save_html_to_slim_dest_options_suffix_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_html_path =
			create_sanbox_01_tmp_file(&format!("{}.html", "test_slim_opts_suffix_input"), FX_HTML_CONTENT_FULL)?;
		let fx_suffix = "_customslim";
		let expected_output_filename = format!("{}{}.html", fx_html_path.stem(), fx_suffix);
		let expected_slim_path_rel = fx_html_path.new_sibling(&expected_output_filename);

		// -- Exec
		let lua_code = format!(r#"return aip.file.save_html_to_slim("{fx_html_path}", {{ suffix = "{fx_suffix}" }})"#);
		let res = run_reflective_agent(&lua_code, None).await?;

		// -- Check FileInfo result
		assert_eq!(res.x_get_str("path")?, expected_slim_path_rel.as_str());
		assert_eq!(res.x_get_str("name")?, expected_output_filename);

		// -- Check content
		let slim_full_path = resolve_sandbox_01_path(&expected_slim_path_rel);
		let slim_content = sfs_read_to_string(&slim_full_path)?;
		assert_contains(&slim_content, "<h1>Main Title</h1>");

		// -- Cleanup
		clean_sanbox_01_tmp_file(slim_full_path)?;
		Ok(())
	}

	#[tokio::test]
	async fn test_script_aip_file_save_html_to_slim_dest_options_file_name_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_html_path = create_sanbox_01_tmp_file("test_slim_opts_filename_input.html", FX_HTML_CONTENT_FULL)?;
		let fx_file_name = "new_slim_name.html";
		let expected_slim_path_rel = fx_html_path.new_sibling(fx_file_name);

		// -- Exec
		let lua_code =
			format!(r#"return aip.file.save_html_to_slim("{fx_html_path}", {{ file_name = "{fx_file_name}" }})"#);
		let res = run_reflective_agent(&lua_code, None).await?;

		// -- Check FileInfo result
		assert_eq!(res.x_get_str("path")?, expected_slim_path_rel.as_str());
		assert_eq!(res.x_get_str("name")?, fx_file_name);

		// -- Check content
		let slim_full_path = resolve_sandbox_01_path(&expected_slim_path_rel);
		let slim_content = sfs_read_to_string(&slim_full_path)?;
		assert_contains(&slim_content, "<h1>Main Title</h1>");

		// -- Cleanup
		clean_sanbox_01_tmp_file(slim_full_path)?;
		Ok(())
	}

	#[tokio::test]
	async fn test_script_aip_file_save_html_to_slim_html_not_found() -> Result<()> {
		// -- Setup & Fixtures
		let fx_src_path = gen_sandbox_01_temp_file_path("test_slim_html_not_found.html");
		// No need to specify dest, as it should fail before path resolution.

		// -- Exec
		let lua_code = format!(r#"return aip.file.save_html_to_slim("{fx_src_path}")"#);
		let Err(res) = run_reflective_agent(&lua_code, None).await else {
			panic!("Should have returned an error")
		};

		// -- Check
		let res_string = res.to_string();
		assert_contains(&res_string, "Failed to read HTML file");
		assert_contains(&res_string, fx_src_path.as_str());

		Ok(())
	}

	// endregion: --- Tests for save_html_to_slim

	#[tokio::test]
	async fn test_script_aip_file_save_html_to_md_with_slim() -> Result<()> {
		// -- Setup & Fixtures
		const FX_HTML_WITH_STYLE: &str = r#"
<!DOCTYPE html>
<html><head><title>Test Page</title><style>#some-id { content: "this should not appear"; }</style></head>
<body><h1>Main Title</h1><p>Some paragraph.</p></body></html>"#;

		let fx_html_path = create_sanbox_01_tmp_file(
			"test_script_aip_file_save_html_to_md_with_slim.html",
			FX_HTML_WITH_STYLE,
		)?;

		let fx_md_dir = ".tmp/test-save-html-to-md-slim/";
		let fx_md_name = "output.md";
		let expected_md_path = SPath::new(fx_md_dir).join(fx_md_name);

		// -- Exec
		let lua_code = format!(
			r#"return aip.file.save_html_to_md("{}", {{base_dir = "{}", file_name = "{}", slim = true}})"#,
			fx_html_path, fx_md_dir, fx_md_name
		);
		let res = run_reflective_agent(&lua_code, None).await?;

		// -- Check
		assert_eq!(res.x_get_str("path")?, expected_md_path.as_str());

		let md_full_path = resolve_sandbox_01_path(&expected_md_path);
		let md_content = sfs_read_to_string(&md_full_path)?;

		assert_contains(&md_content, "# Main Title");
		assert_not_contains(&md_content, "this should not appear");

		// -- Cleanup
		clean_sanbox_01_tmp_file(md_full_path)?;
		Ok(())
	}
}

// endregion: --- Tests
