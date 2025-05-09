//! Defines the `text` module, used in the lua engine.
//!
//! ---
//!
//! ## Lua documentation
//!
//! This module exposes functions that process text.
//!
//! ### Functions
//!
//! - `aip.text.escape_decode(content: string): string`
//! - `aip.text.escape_decode_if_needed(content: string): string`
//! - `aip.text.split_first(content: string, sep: string): (string, string|nil)`
//! - `aip.text.remove_first_line(content: string): string`
//! - `aip.text.remove_first_lines(content: string, n: int): string`
//! - `aip.text.remove_last_line(content: string): string`
//! - `aip.text.remove_last_lines(content: string, n: int): string`
//! - `aip.text.trim(content: string): string`
//! - `aip.text.trim_start(content: string): string`
//! - `aip.text.trim_end(content: string): string`
//! - `aip.text.truncate(content: string, max_len: int): string`
//! - `aip.text.truncate(content: string, max_len: int, ellipsis: string): string`
//! - `aip.text.replace_markers(content: string, new_sections: array): string`
//! - `aip.text.ensure(content: string, opt: table): string`
//! - `aip.text.ensure_single_ending_newline(content: string): string`
//! - `aip.text.extract_line_blocks(content: string, options: {starts_with: string, extrude?: "content", first?: number}): (table, string | nil)`

use crate::Result;
use crate::runtime::Runtime;
use crate::script::DEFAULT_MARKERS;
use crate::script::helpers::to_vec_of_strings;
use crate::support::Extrude;
use crate::support::html::decode_html_entities;
use crate::support::text::{self, EnsureOptions, truncate_with_ellipsis};
use crate::support::text::{LineBlockIter, LineBlockIterOptions};
use mlua::{FromLua, Lua, MultiValue, String as LuaString, Table, Value};
use std::borrow::Cow;

pub fn init_module(lua: &Lua, _runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	table.set("escape_decode", lua.create_function(escape_decode)?)?;
	table.set("escape_decode_if_needed", lua.create_function(escape_decode_if_needed)?)?;
	table.set("split_first", lua.create_function(split_first)?)?;
	table.set("remove_first_line", lua.create_function(remove_first_line)?)?;
	table.set("remove_first_lines", lua.create_function(remove_first_lines)?)?;
	table.set("remove_last_lines", lua.create_function(remove_last_lines)?)?;
	table.set("remove_last_line", lua.create_function(remove_last_line)?)?;
	table.set("trim", lua.create_function(trim)?)?;
	table.set("trim_start", lua.create_function(trim_start)?)?;
	table.set("trim_end", lua.create_function(trim_end)?)?;
	table.set("truncate", lua.create_function(truncate)?)?;
	table.set(
		"replace_markers",
		lua.create_function(replace_markers_with_default_parkers)?,
	)?;
	table.set("ensure", lua.create_function(ensure)?)?;
	table.set(
		"ensure_single_ending_newline",
		lua.create_function(ensure_single_ending_newline)?,
	)?;
	table.set("extract_line_blocks", lua.create_function(extract_line_blocks)?)?;

	Ok(table)
}

// region:    --- ensure

impl FromLua for EnsureOptions {
	fn from_lua(value: Value, _lua: &Lua) -> mlua::Result<Self> {
		let table = value.as_table().ok_or_else(|| {
			mlua::Error::runtime(
				"Ensure argument needs to be a table with the format {start = string, end = string} (both optional",
			)
		})?;

		//
		let prefix = table.get::<String>("prefix").ok();
		let suffix = table.get::<String>("suffix").ok();

		for (key, _value) in table.pairs::<Value, Value>().flatten() {
			if let Some(key) = key.as_str() {
				if key != "prefix" && key != "suffix" {
					let msg = format!(
						"Ensure argument contains invalid table property `{key}`. Can only contain `prefix` and/or `suffix`"
					);
					return Err(mlua::Error::RuntimeError(msg));
				}
			}
		}

		//
		Ok(EnsureOptions { prefix, suffix })
	}
}

/// ## Lua Documentation
///
/// Ensure the content start and/or end with the text given in the second argument dictionary.
///
/// ```lua
/// -- API Signature
/// aip.text.ensure(content: string, {prefix? = string, suffix? = string}): string
/// ```
///
/// This function is useful for code normalization.
///
/// ### Arguments
///
/// - `content: string`: The content to ensure.
/// - `options: table`: A table with optional `prefix` and `suffix` keys.
///
/// ### Returns
///
/// The ensured string.
fn ensure(_lua: &Lua, (content, inst): (String, Value)) -> mlua::Result<String> {
	let inst = EnsureOptions::from_lua(inst, _lua)?;
	let res = crate::support::text::ensure(&content, inst);
	let res = res.to_string();
	Ok(res)
}

/// ## Lua Documentation
///
/// Ensures that `content` ends with a single newline character.
/// If `content` is empty, it returns a newline character.
///
/// ```lua
/// -- API Signature
/// aip.text.ensure_single_ending_newline(content: string): string
/// ```
///
/// This function is useful for code normalization.
///
/// ### Arguments
///
/// - `content: string`: The content to process.
///
/// ### Returns
///
/// The string with a single ending newline.
fn ensure_single_ending_newline(_lua: &Lua, content: String) -> mlua::Result<String> {
	Ok(crate::support::text::ensure_single_ending_newline(content))
}

// endregion: --- ensure

// region:    --- Transform

/// ## Lua Documentation
///
/// Replaces markers in `content` with corresponding sections from `new_sections`.
/// Each section in `new_sections` can be a string or a map containing a `.content` string.
///
/// ```lua
/// -- API Signature
/// aip.text.replace_markers(content: string, new_sections: array): string
/// ```
///
/// Assumes the markers are `<<START>>` and `<<END>>`.
///
/// ### Arguments
///
/// - `content: string`: The content containing markers to replace.
/// - `new_sections: array`: An array of strings to replace the markers.
///
/// ### Returns
///
/// The string with markers replaced by the corresponding sections.
fn replace_markers_with_default_parkers(_lua: &Lua, (content, new_sections): (String, Value)) -> mlua::Result<String> {
	let sections = to_vec_of_strings(new_sections, "new_sections")?;
	let sections: Vec<&str> = sections.iter().map(|s| s.as_str()).collect();
	let new_content = text::replace_markers(&content, &sections, DEFAULT_MARKERS)?;
	Ok(new_content)
}

/// ## Lua Documentation
///
/// Returns `content` truncated to a maximum length of `max_len`.
/// If the content exceeds `max_len`, it appends the optional `ellipsis` string to indicate truncation.
/// If `ellipsis` is not provided, no additional characters are added after truncation.
///
/// ```lua
/// -- API Signature
/// aip.text.truncate(content: string, max_len: int, ellipsis?: string): string
/// ```
///
/// This function is useful for limiting the length of text output while preserving meaningful context.
///
/// ### Arguments
///
/// - `content: string`: The content to truncate.
/// - `max_len: int`: The maximum length of the truncated string.
/// - `ellipsis: string` (optional): The string to append if truncation occurs.
///
/// ### Returns
///
/// The truncated string.
fn truncate(_lua: &Lua, (content, max_len, ellipsis): (String, usize, Option<String>)) -> mlua::Result<String> {
	let ellipsis = ellipsis.unwrap_or_default();
	match truncate_with_ellipsis(&content, max_len, &ellipsis) {
		Cow::Borrowed(txt) => Ok(txt.to_string()),
		Cow::Owned(txt) => Ok(txt),
	}
}

// endregion: --- Transform

// region:    --- Split

/// ## Lua Documentation
///
/// Splits a string into two parts based on the first occurrence of a separator.
///
/// ```lua
/// -- API Signature
/// aip.text.split_first(content: string, sep: string): (string, string|nil)
/// ```
///
/// ```lua
/// local content = "some first content\n===\nsecond content"
/// local first, second = aip.text.split_first(content,"===")
/// -- first  = "some first content\n"
/// -- second = "\nsecond content"
/// -- NOTE: When no match, second is nil.
/// --       If match, but nothing after, second is ""
/// ```
///
/// NOTE: For optimization, this will use LuaString to avoid converting Lua String to Rust String and back
///
/// ### Arguments
///
/// - `content: string`: The string to split.
/// - `sep: string`: The separator string.
///
/// ### Returns
///
/// A tuple containing the first part and the second part (or nil if no match).
///
/// ```ts
/// [string, string | nil]
/// ```
fn split_first(_lua: &Lua, (content, sep): (LuaString, LuaString)) -> mlua::Result<MultiValue> {
	// Convert LuaStrings to Rust strings
	let content_str = content.to_str()?;
	let sep_str = sep.to_str()?;

	// Find the first occurrence of the separator
	if let Some(index) = content_str.find(&*sep_str) {
		// Split the content into two parts
		let first_part = &content_str[..index];
		let second_part = &content_str[index + sep_str.len()..];

		// Convert parts back to Lua strings and return as MultiValue
		Ok(MultiValue::from_vec(vec![
			Value::String(_lua.create_string(first_part)?),
			Value::String(_lua.create_string(second_part)?),
		]))
	} else {
		// Return the content as the first value and nil as the second
		Ok(MultiValue::from_vec(vec![Value::String(content), Value::Nil]))
	}
}

// endregion: --- Split

// region:    --- Trim

/// ## Lua Documentation
///
/// Trims leading and trailing whitespace from a string.
///
/// ```lua
/// -- API Signature
/// aip.text.trim(content: string): string
/// ```
///
/// ### Arguments
///
/// - `content: string`: The string to trim.
///
/// ### Returns
///
/// The trimmed string.
fn trim(_lua: &Lua, content: LuaString) -> mlua::Result<Value> {
	let original_str = content.to_str()?;
	let trimmed = original_str.trim();
	if trimmed.len() == original_str.len() {
		Ok(Value::String(content))
	} else {
		_lua.create_string(trimmed).map(Value::String)
	}
}

/// ## Lua Documentation
///
/// Trims leading whitespace from a string.
///
/// ```lua
/// -- API Signature
/// aip.text.trim_start(content: string): string
/// ```
///
/// ### Arguments
///
/// - `content: string`: The string to trim.
///
/// ### Returns
///
/// The trimmed string.
fn trim_start(_lua: &Lua, content: LuaString) -> mlua::Result<Value> {
	let original_str = content.to_str()?;
	let trimmed = original_str.trim_start();
	if trimmed.len() == original_str.len() {
		Ok(Value::String(content))
	} else {
		_lua.create_string(trimmed).map(Value::String)
	}
}

/// ## Lua Documentation
///
/// Trims trailing whitespace from a string.
///
/// ```lua
/// -- API Signature
/// aip.text.trim_end(content: string): string
/// ```
///
/// ### Arguments
///
/// - `content: string`: The string to trim.
///
/// ### Returns
///
/// The trimmed string.
fn trim_end(_lua: &Lua, content: LuaString) -> mlua::Result<Value> {
	let original_str = content.to_str()?;
	let trimmed = original_str.trim_end();
	if trimmed.len() == original_str.len() {
		Ok(Value::String(content))
	} else {
		_lua.create_string(trimmed).map(Value::String)
	}
}

// endregion: --- Trim

// region:    --- Remove

/// ## Lua Documentation
///
/// Returns `content` with the first line removed.
///
/// ```lua
/// -- API Signature
/// aip.text.remove_first_line(content: string): string
/// ```
///
/// ### Arguments
///
/// - `content: string`: The content to process.
///
/// ### Returns
///
/// The string with the first line removed.
fn remove_first_line(_lua: &Lua, content: String) -> mlua::Result<String> {
	Ok(remove_first_lines_impl(&content, 1).to_string())
}

/// ## Lua Documentation
///
/// Returns `content` with the first `n` lines removed.
///
/// ```lua
/// -- API Signature
/// aip.text.remove_first_lines(content: string, n: int): string
/// ```
///
/// ### Arguments
///
/// - `content: string`: The content to process.
/// - `n: int`: The number of lines to remove.
///
/// ### Returns
///
/// The string with the first `n` lines removed.
fn remove_first_lines(_lua: &Lua, (content, num_of_lines): (String, i64)) -> mlua::Result<String> {
	Ok(remove_first_lines_impl(&content, num_of_lines as usize).to_string())
}

fn remove_first_lines_impl(content: &str, num_of_lines: usize) -> &str {
	let mut start_idx = 0;
	let mut newline_count = 0;

	for (i, c) in content.char_indices() {
		if c == '\n' {
			newline_count += 1;
			if newline_count == num_of_lines {
				start_idx = i + 1;
				break;
			}
		}
	}

	if newline_count < num_of_lines {
		return "";
	}

	&content[start_idx..]
}

/// ## Lua Documentation
///
/// Returns `content` with the last line removed.
///
/// ```lua
/// -- API Signature
/// aip.text.remove_last_line(content: string): string
/// ```
///
/// ### Arguments
///
/// - `content: string`: The content to process.
///
/// ### Returns
///
/// The string with the last line removed.
fn remove_last_line(_lua: &Lua, content: String) -> mlua::Result<String> {
	Ok(remove_last_lines_impl(&content, 1).to_string())
}

/// ## Lua Documentation
///
/// Returns `content` with the last `n` lines removed.
///
/// ```lua
/// -- API Signature
/// aip.text.remove_last_lines(content: string, n: int): string
/// ```
///
/// ### Arguments
///
/// - `content: string`: The content to process.
/// - `n: int`: The number of lines to remove.
///
/// ### Returns
///
/// The string with the last `n` lines removed.
fn remove_last_lines(_lua: &Lua, (content, num_of_lines): (String, i64)) -> mlua::Result<String> {
	Ok(remove_last_lines_impl(&content, num_of_lines as usize).to_string())
}

fn remove_last_lines_impl(content: &str, num_of_lines: usize) -> &str {
	let mut end_idx = content.len();
	let mut newline_count = 0;

	for (i, c) in content.char_indices().rev() {
		if c == '\n' {
			newline_count += 1;
			if newline_count == num_of_lines {
				end_idx = i;
				break;
			}
		}
	}

	if newline_count < num_of_lines {
		return "";
	}

	&content[..end_idx]
}

// endregion: --- Remove

// region:    --- Escape Fns

/// ## Lua Documentation
///
/// Only escape if needed. Right now, the test only tests `&lt;`.
///
/// ```lua
/// -- API Signature
/// aip.text.escape_decode_if_needed(content: string): string
/// ```
///
/// Some LLMs HTML-encode their responses. This function returns `content`
/// after selectively decoding certain HTML tags.
///
/// Right now, the only tag that gets decoded is `&lt;`.
///
/// ### Arguments
///
/// - `content: string`: The content to process.
///
/// ### Returns
///
/// The HTML-decoded string.
fn escape_decode_if_needed(_lua: &Lua, content: String) -> mlua::Result<String> {
	if !content.contains("&lt;") {
		Ok(content)
	} else {
		escape_decode(_lua, content)
	}
}

/// ## Lua Documentation
///
/// Some LLMs HTML-encode their responses. This function returns `content`,
/// HTML-decoded.
///
/// ```lua
/// -- API Signature
/// aip.text.escape_decode(content: string): string
/// ```
///
/// ### Arguments
///
/// - `content: string`: The content to process.
///
/// ### Returns
///
/// The HTML-decoded string.
fn escape_decode(_lua: &Lua, content: String) -> mlua::Result<String> {
	Ok(decode_html_entities(&content))
}

// endregion: --- Escape Fns

// region: --- Extract Line Blocks

/// ## Lua Documentation
///
/// Extracts line blocks from `content` using the given options. The options table
/// must include a required `starts_with` field.
///
/// ```lua
/// -- API Signature
/// local blocks, extruded = aip.text.extract_line_blocks(content, { starts_with = ">", extrude = "content", first = number })
/// ```
///
/// Optionally, you can provide a `first` field as a number, which limits the number
/// of blocks returned by performing that many `next()` iterations. If `extrude` is set to "content",
/// the remaining lines (after extracting the specified number of blocks) are captured via `collect_remains`.
/// If the `extrude` option is not set, the extruded content is returned as `nil`.
///
/// ### Arguments
///
/// - `content: string`: The content to extract line blocks from.
/// - `options: table`: A table with the following keys:
///   - `starts_with: string` (required): The prefix that indicates the start of a line block.
///   - `extrude: "content"` (optional): If set to `"content"`, the remaining content after extracting the blocks is returned.
///   - `first: number` (optional): Limits the number of blocks returned (the rest will be treated as the remaining content)
///
/// ### Returns
///
/// A tuple containing:
///   - `blocks: table`: A Lua table (array-like) where each element is a string representing a line block.
///   - `extruded: string | nil`: The remaining content after extracting the blocks, if `extrude` is set to `"content"`. Otherwise, `nil`.
///
/// ```ts
/// [string[], string | nil]
/// ```
fn extract_line_blocks(_lua: &Lua, (content, options): (String, Table)) -> mlua::Result<MultiValue> {
	let starts_with: Option<String> = options.get("starts_with")?;
	let Some(starts_with) = starts_with else {
		return Err(crate::Error::custom(
			r#"aip.text.extract_line_blocks requires to options with {starts_with = ".."} "#,
		)
		.into());
	};
	let extrude_param: Option<String> = options.get("extrude").ok();
	let return_extrude = matches!(extrude_param.as_deref(), Some("content"));
	let first_opt: Option<i64> = options.get("first").ok();
	let first_count: Option<usize> = first_opt.map(|n| n as usize);

	let iter_options = LineBlockIterOptions {
		starts_with: &starts_with,
		extrude: if return_extrude { Some(Extrude::Content) } else { None },
	};

	let mut iterator = LineBlockIter::new(content.as_str(), iter_options);

	let (blocks, extruded_content) = if let Some(n) = first_count {
		let mut limited_blocks = Vec::new();
		for _ in 0..n {
			if let Some(block) = iterator.next() {
				limited_blocks.push(block);
			} else {
				break;
			}
		}
		let remains = if return_extrude {
			let (_ignored, extruded) = iterator.collect_remains();
			extruded
		} else {
			String::new()
		};
		(limited_blocks, remains)
	} else {
		iterator.collect_blocks_and_extruded_content()
	};

	let blocks_table = _lua.create_table()?;
	for block in blocks.iter() {
		// Use table.push so that the returned Lua table is an array-like table.
		blocks_table.push(block.as_str())?;
	}

	let extruded_value = if return_extrude {
		Value::String(_lua.create_string(&extruded_content)?)
	} else {
		Value::Nil
	};

	Ok(MultiValue::from_vec(vec![Value::Table(blocks_table), extruded_value]))
}

// endregion: --- Extract Line Blocks

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{assert_contains, eval_lua, setup_lua};
	use crate::script::aip_modules::aip_text;
	use value_ext::JsonValueExt as _;

	#[tokio::test]
	async fn test_lua_text_split_first_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_text::init_module, "text")?;
		// (content, separator, (first, second))
		let data = [
			// with matching
			(
				"some first content\n===\nsecond content",
				"===",
				("some first content\n", Some("\nsecond content")),
			),
			// no matching
			("some first content\n", "===", ("some first content\n", None)),
			// matching but nothing after separator
			("some first content\n===", "===", ("some first content\n", Some(""))),
		];

		for (content, sep, expected) in data {
			let script = format!(
				r#"
			local first, second = aip.text.split_first({content:?}, "{sep}")
			return {{first, second}}
			"#
			);
			let res = eval_lua(&lua, &script)?;

			// -- Check
			let values = res.as_array().ok_or("Should have returned an array")?;

			let first = values
				.first()
				.ok_or("Should always have at least a first return")?
				.as_str()
				.ok_or("First should be string")?;
			assert_eq!(expected.0, first);

			let second = values.get(1);
			if let Some(exp_second) = expected.1 {
				let second = second.ok_or("Should have second")?;
				assert_eq!(exp_second, second)
			} else {
				assert!(second.is_none(), "Second should not have been none");
			}
		}

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_text_ensure_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_text::init_module, "text")?;
		let data = [
			(
				"some- ! -path",
				r#"{prefix = "./", suffix = ".md"}"#,
				"./some- ! -path.md",
			),
			("some- ! -path", r#"{suffix = ".md"}"#, "some- ! -path.md"),
			(" ~ some- ! -path", r#"{prefix = " ~ "}"#, " ~ some- ! -path"),
			("~ some- ! -path", r#"{prefix = " ~ "}"#, " ~ ~ some- ! -path"),
		];

		for (content, arg, expected) in data {
			// -- Exec
			let script = format!("return aip.text.ensure(\"{content}\", {arg})");

			// -- Check
			let res = eval_lua(&lua, &script)?;
			assert_eq!(res, expected);
		}

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_text_extract_line_blocks_simple() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_text::init_module, "text")?;
		let lua_code = r#"
local content = [[
> one
> two
Some line A
> 3
The end
]]
local a, b = aip.text.extract_line_blocks(content, { starts_with = ">", extrude = "content" })
return {blocks = a, extruded = b}
		"#;

		// -- Exec
		let res = eval_lua(&lua, lua_code)?;

		// -- Check
		let block = res.x_get_str("/blocks/0")?;
		assert_eq!(block, "> one\n> two\n");
		let block = res.x_get_str("/blocks/1")?;
		assert_eq!(block, "> 3\n");
		let content = res.x_get_str("/extruded")?;
		assert_contains(content, "Some line A");
		assert_contains(content, "The end");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_text_extract_line_blocks_with_first_extrude() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_text::init_module, "text")?;
		let lua_code = r#"
local content = [[
> one
> two
line1
> three
line2
> four
line3
]]
local a, b = aip.text.extract_line_blocks(content, { starts_with = ">", extrude = "content", first = 2 })
return { blocks = a, extruded = b }
		"#;

		// -- Exec
		let res = eval_lua(&lua, lua_code)?;

		// -- Check
		let block1 = res.x_get_str("/blocks/0")?;
		assert_eq!(block1, "> one\n> two\n");
		let block2 = res.x_get_str("/blocks/1")?;
		assert_eq!(block2, "> three\n");
		let extruded = res.x_get_str("/extruded")?;
		assert_eq!(extruded, "line1\nline2\n> four\nline3\n");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_text_extract_line_blocks_with_first_no_extrude() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_text::init_module, "text")?;
		let lua_code = r#"
local content = [[
> one
> two
line1
> three
line2
> four
line3
]]
local a, b = aip.text.extract_line_blocks(content, { starts_with = ">", first = 2 })
return { blocks = a, extruded = b }
		"#;

		// -- Exec
		let res = eval_lua(&lua, lua_code)?;

		// -- Check
		let blocks = res.x_get_as::<Vec<&str>>("blocks")?;
		assert_eq!(blocks.len(), 2, "should have only 2 blocks");
		let block1 = res.x_get_str("/blocks/0")?;
		assert_eq!(block1, "> one\n> two\n");
		let block2 = res.x_get_str("/blocks/1")?;
		assert_eq!(block2, "> three\n");
		let extruded = res.get("extruded");
		assert!(
			extruded.is_none(),
			"extruded should be nil when extrude option is not set"
		);

		Ok(())
	}
}

// endregion: --- Tests
