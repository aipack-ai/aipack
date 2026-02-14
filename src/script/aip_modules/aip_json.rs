//! Defines the `json` module, used in the lua engine.
//!
//! ---
//!
//! ## Lua documentation
//!
//! The `aip.json` module exposes functions to parse and stringify JSON content.
//!
//! IMPORTANT: By default, this support the parsing of jsonc content, meaning json with optional comments
//!
//! - Parse function will return nil if content is nil
//! - stringify return a single line
//! - use `stringify_pretty` for idented multi-line
//!
//! ### Functions
//!
//! - `aip.json.parse(content: string | nil) -> table | nil`
//! - `aip.json.parse_ndjson(content: string | nil) -> table[] | nil`
//! - `aip.json.stringify(content: table) -> string`
//! - `aip.json.stringify_pretty(content: table) -> string`
//!
//! ---
//!

use crate::runtime::Runtime;
use crate::script::{lua_value_to_serde_value, serde_value_to_lua_value};
use crate::support::jsons;
use crate::{Error, Result};
use mlua::{Lua, Table, Value};
use simple_fs::parse_ndjson_from_reader;
use std::io::BufReader;

pub fn init_module(lua: &Lua, _runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	let parse_fn = lua.create_function(move |lua, content: Option<String>| parse(lua, content))?;
	let parse_ndjson_fn = lua.create_function(move |lua, content: Option<String>| parse_ndjson(lua, content))?;
	let stringify_fn = lua.create_function(move |lua, content: Value| stringify(lua, content))?;
	let stringify_pretty_fn = lua.create_function(move |lua, content: Value| stringify_pretty(lua, content))?;
	// stringify_to_line is now an alias for stringify
	let stringify_to_line_fn = stringify_fn.clone();

	table.set("parse", parse_fn)?;
	table.set("parse_ndjson", parse_ndjson_fn)?;
	table.set("stringify", stringify_fn)?;
	table.set("stringify_pretty", stringify_pretty_fn)?;

	// deprecated, should use stringify
	table.set("stringify_to_line", stringify_to_line_fn)?;

	Ok(table)
}

/// ## Lua Documentation
/// ---
/// Parse a JSON string into a table.
///
/// ```lua
/// -- API Signature
/// aip.json.parse(content: string | nil): table | nil
/// ```
///
/// Parse a JSON string into a table that can be used in the Lua script.
///
/// IMPORTANT: Supports comments and trailing commas (e.g., jsonc + trailing commas)
///
/// ### Arguments
///
/// - `content: string | nil` - The JSON string to parse. If nil, returns nil.
///
/// ### Returns
///
/// - `table | nil` - A Lua table representing the parsed JSON structure. The structure
///   of the table depends on the input JSON. JSON objects become Lua tables,
///   JSON arrays become Lua lists (tables with integer keys), and JSON primitives
///   become corresponding Lua primitives (string, number, boolean, nil).
///
/// ### Example
///
/// ```lua
/// local json_str = '{"name": "John", "age": 30}'
/// local obj = aip.json.parse(json_str)
/// print(obj.name) -- prints "John"
/// print(obj.age)  -- prints 30
/// ```
///
/// ### Error
///
/// Returns an error if the input string is not valid JSON.
///
/// ```ts
/// {
///   error: string  // Error message from JSON parsing, e.g., "aip.json.parse failed. ..."
/// }
/// ```
fn parse(lua: &Lua, content: Option<String>) -> mlua::Result<Value> {
	let Some(content) = content else {
		return Ok(Value::Nil);
	};

	let json_value = match jsons::parse_jsonc_to_serde_value(&content) {
		Ok(val) => val,
		Err(err) => return Err(Error::custom(format!("aip.json.parse failed. {err}")).into()),
	};

	// -- Make it value::Null if None
	let json_value = json_value.unwrap_or_default();

	let lua_value = serde_value_to_lua_value(lua, json_value)?;

	Ok(lua_value)
}

/// ## Lua Documentation
/// ---
/// Parse a newline-delimited JSON (NDJSON) string into an array of tables.
///
/// ```lua
/// -- API Signature
/// aip.json.parse_ndjson(content: string | nil): table[] | nil
/// ```
///
/// Parses a string containing multiple JSON objects separated by newlines.
/// Each line should be a valid JSON object. Empty lines are skipped.
///
/// IMPORTANT: Unlike the aip.json.parse parser, this assumes the content is fully valid JSON
///            (no comments allowed or trailing comments)
///
/// ### Arguments
///
/// - `content: string | nil` - The NDJSON string to parse. If nil, returns nil.
///
/// ### Returns
///
/// - `table[] | nil` - A Lua list (table acting as an array) where each element is a table
///   representing a parsed JSON object from a line. The structure of each element
///   depends on the JSON object on that line.
///
/// ### Example
///
/// ```lua
/// local ndjson_str = '{"name": "John", "age": 30}\n{"name": "Jane", "age": 25}'
/// local list = aip.json.parse_ndjson(ndjson_str)
/// -- list will be:
/// -- {
/// --   { name = "John", age = 30 },
/// --   { name = "Jane", age = 25 }
/// -- }
/// print(list[1].name) -- prints "John"
/// print(list[2].name) -- prints "Jane"
/// ```
///
/// ### Error
///
/// Returns an error if any line contains invalid JSON.
///
/// ```ts
/// {
///   error: string  // Error message if any line fails JSON parsing, e.g., "aip.json.parse_ndjson failed. ..."
/// }
/// ```
fn parse_ndjson(lua: &Lua, content: Option<String>) -> mlua::Result<Value> {
	let Some(content) = content else {
		return Ok(Value::Nil);
	};

	let reader = BufReader::new(content.as_bytes());
	match parse_ndjson_from_reader(reader) {
		Ok(values) => {
			let values = serde_json::Value::Array(values);
			let lua_value = serde_value_to_lua_value(lua, values)?;
			Ok(lua_value)
		}
		Err(err) => Err(Error::custom(format!("aip.json.parse_ndjson failed. {err}")).into()),
	}
}

/// ## Lua Documentation
/// ---
/// Stringify a table into a single line JSON string.
///
/// Good for newline json or compact representation.
///
/// ```lua
/// -- API Signature
/// aip.json.stringify(content: table): string
/// ```
///
/// Convert a table into a single line JSON string.
///
/// ### Arguments
///
/// - `content: table` - The Lua table to stringify.
///
/// ### Returns
///
/// - `string` - A string containing the JSON representation of the input table,
///   without any indentation or extra whitespace (except within string values).
///
/// ### Example
///
/// ```lua
/// local obj = {
///     name = "John",
///     age = 30
/// }
/// local json_str = aip.json.stringify(obj)
/// -- Result will be:
/// -- {"name":"John","age":30}
/// ```
///
/// ### Error
///
/// Returns an error if the input Lua value cannot be serialized into JSON.
///
/// ```ts
/// {
///   error: string  // Error message from JSON stringification, e.g., "aip.json.stringify fail to stringify. ..."
/// }
/// ```
fn stringify(_lua: &Lua, content: Value) -> mlua::Result<String> {
	let json_value = lua_value_to_serde_value(content)?;
	match serde_json::to_string(&json_value) {
		Ok(str) => Ok(str),
		Err(err) => Err(Error::custom(format!("aip.json.stringify fail to stringify. {err}")).into()),
	}
}

/// ## Lua Documentation
/// ---
/// Stringify a table into a JSON string with pretty formatting.
///
/// ```lua
/// -- API Signature
/// aip.json.stringify_pretty(content: table): string
/// ```
///
/// Convert a table into a JSON string with pretty formatting using 2 spaces indentation.
///
/// ### Arguments
///
/// - `content: table` - The Lua table to stringify.
///
/// ### Returns
///
/// - `string` - A string containing the pretty-formatted JSON representation
///   of the input table, using newlines and 2-space indentation.
///
/// ### Example
///
/// ```lua
/// local obj = {
///     name = "John",
///     age = 30
/// }
/// local json_str = aip.json.stringify_pretty(obj)
/// -- Result will be similar to:
/// -- {
/// --   "name": "John",
/// --   "age": 30
/// -- }
/// ```
///
/// ### Error
///
/// Returns an error if the input Lua value cannot be serialized into JSON.
///
/// ```ts
/// {
///   error: string  // Error message from JSON stringification, e.g., "aip.json.stringify_pretty fail to stringify. ..."
/// }
/// ```
fn stringify_pretty(_lua: &Lua, content: Value) -> mlua::Result<String> {
	let json_value = lua_value_to_serde_value(content)?;
	match serde_json::to_string_pretty(&json_value) {
		Ok(str) => Ok(str),
		Err(err) => Err(Error::custom(format!("aip.json.stringify_pretty fail to stringify. {err}")).into()),
	}
}

// region:    --- Tests

#[cfg(test)]
#[path = "aip_json_tests.rs"]
mod tests;
// endregion: --- Tests
