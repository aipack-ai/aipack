//! Defines the `load_json`, `load_ndjson`, `append_json_line`, and `append_json_lines` functions for the `aip.file` Lua module.
//!
//! ---
//!
//! ## Lua documentation for `aip.file` JSON functions
//!
//! ### Functions
//!
//! - `aip.file.load_json(path: string): table | value`
//! - `aip.file.load_ndjson(path: string): table`
//! - `aip.file.append_json_line(path: string, data: value): FileInfo`
//! - `aip.file.append_json_lines(path: string, data: list): FileInfo`

use crate::Error;
use crate::dir_context::PathResolver;
use crate::runtime::Runtime;
use crate::script::{lua_value_list_to_serde_values, lua_value_to_serde_value, serde_value_to_lua_value};
use crate::support::jsons;
use crate::types::FileInfo;
use mlua::{IntoLua, Lua, Value};
use simple_fs::ensure_file_dir;

/// ## Lua Documentation
///
/// Load a file, parse its content as JSON, and return the corresponding Lua value.
///
/// ```lua
/// -- API Signature
/// aip.file.load_json(path: string): table | value
/// ```
///
/// Loads the content of the file specified by `path`, parses it as JSON,
/// and converts the result into a Lua value (typically a table, but can be
/// a string, number, boolean, or nil depending on the JSON content).
/// The path is resolved relative to the workspace root.
///
/// IMPORTANT:
/// - Supports comments and trailing commas.
/// - If file content is empty or no json content, will return Value::Nil
///
/// ### Arguments
///
/// - `path: string`: The path to the JSON file, relative to the workspace root.
///
/// ### Returns
///
/// - `table | value`: A Lua value representing the parsed JSON content.
///
/// ### Example
///
/// ```lua
/// -- Assuming 'config.json' contains {"port": 8080, "enabled": true}
/// local config = aip.file.load_json("config.json")
/// print(config.port)    -- Output: 8080
/// print(config.enabled) -- Output: true
///
/// -- Assuming 'data.json' contains ["item1", "item2"]
/// local data = aip.file.load_json("data.json")
/// print(data[1]) -- Output: item1
/// ```
///
/// ### Error
///
/// Returns an error if:
/// - The file cannot be found or read.
/// - The file content is not valid JSON.
/// - The JSON value cannot be converted to a Lua value.
///
/// ```ts
/// {
///   error: string // Error message (e.g., file not found, JSON parse error)
/// }
/// ```
pub(super) fn file_load_json(lua: &Lua, runtime: &Runtime, path: String) -> mlua::Result<Value> {
	// Resolve the path relative to the workspace directory
	let full_path =
		runtime
			.dir_context()
			.resolve_path(runtime.session(), path.clone().into(), PathResolver::WksDir, None)?;

	let json_value = jsons::load_json_to_serde_value(&full_path).map_err(|e| {
		Error::from(format!(
			"aip.file.load_json - Failed to read json file '{path}'.\nCause: {e}",
		))
	})?;

	// -- Make it value::Null if None
	let json_value = json_value.unwrap_or_default();

	// Convert the serde_json::Value to mlua::Value
	let lua_value = serde_value_to_lua_value(lua, json_value)?;

	Ok(lua_value)
}

/// ## Lua Documentation
///
/// Load a file containing newline-delimited JSON (NDJSON), parse each line, and return a Lua list (table) of the results.
///
/// ```lua
/// -- API Signature
/// aip.file.load_ndjson(path: string): list
/// ```
///
/// Loads the content of the file specified by `path`, assuming each line is a valid JSON object or value.
/// Parses each non-empty line and returns a Lua list (table indexed from 1) containing the parsed Lua values.
/// Empty lines or lines containing only whitespace are skipped.
/// The path is resolved relative to the workspace root.
///
/// IMPORTANT: Contrary to the aip.file.load_json, this requires valid json lines (no comments or trailing commas supported)
///
/// ### Arguments
///
/// - `path: string`: The path to the NDJSON file, relative to the workspace root.
///
/// ### Returns
///
/// - `list: table`: A Lua list (table) where each element corresponds to a successfully parsed JSON line from the file.
///
/// ### Example
///
/// ```lua
/// -- Assuming 'logs.ndjson' contains:
/// -- {"level": "info", "message": "Service started"}
/// -- {"level": "warn", "message": "Disk space low"}
/// -- <empty line>
/// -- "Simple string value"
///
/// local logs = aip.file.load_ndjson("logs.ndjson")
/// print(#logs) -- Output: 3
/// print(logs[1].message) -- Output: Service started
/// print(logs[2].level)   -- Output: warn
/// print(logs[3])         -- Output: Simple string value
/// ```
///
/// ### Error
///
/// Returns an error if:
/// - The file cannot be found or read.
/// - Any non-empty line contains invalid JSON.
/// - The parsed JSON values cannot be converted to Lua values.
///
/// ```ts
/// {
///   error: string // Error message (e.g., file not found, JSON parse error on line N)
/// }
/// ```
pub(super) fn file_load_ndjson(lua: &Lua, runtime: &Runtime, path: String) -> mlua::Result<Value> {
	// Resolve the path relative to the workspace directory
	let full_path =
		runtime
			.dir_context()
			.resolve_path(runtime.session(), path.clone().into(), PathResolver::WksDir, None)?;

	let json_values = simple_fs::load_ndjson(full_path).map_err(|e| {
		Error::from(format!(
			"aip.file.load_ndjson - Failed to load newline json file '{path}'.\nCause: {e}",
		))
	})?;

	let json_value = serde_json::Value::Array(json_values);

	let lua_values = serde_value_to_lua_value(lua, json_value)?;

	Ok(lua_values)
}

/// ## Lua Documentation
///
/// Convert a Lua value to a JSON string and append it as a new line to a file.
///
/// ```lua
/// -- API Signature
/// aip.file.append_json_line(path: string, data: value)
/// ```
///
/// Converts the provided Lua `data` (table, string, number, boolean, nil) into a JSON string
/// and appends this string followed by a newline character (`\n`) to the file specified by `path`.
/// The path is resolved relative to the workspace root.
/// If the file does not exist, it will be created. If the directory structure does not exist, it will be created.
///
/// ### Notes
///
/// - Lua `nil` values within tables might be omitted during JSON serialization, e.g., `{a = 1, b = nil}` becomes `{"a":1}`.
///
/// ### Arguments
///
/// - `path: string`: The path to the file where the JSON line should be appended, relative to the workspace root.
/// - `data: value`: The Lua data to be converted to JSON and appended. Can be any JSON-serializable Lua type.
///
/// ### Returns
///
/// - `FileInfo`: The updated [`FileInfo`] for the file.
///
/// ### Example
///
/// ```lua
/// aip.file.append_json_line("output.ndjson", {user = "test", score = 100})
/// aip.file.append_json_line("output.ndjson", {user = "another", score = 95, active = true, extra = nil})
/// aip.file.append_json_line("output.ndjson", "Just a string line")
///
/// --[[ content of output.ndjson after execution:
/// {"score":100,"user":"test"}
/// {"active":true,"score":95,"user":"another"}
/// "Just a string line"
/// ]]
/// ```
///
/// ### Error
///
/// Returns an error if:
/// - The Lua `data` cannot be converted to an internal JSON representation.
/// - The internal representation cannot be serialized to a JSON string.
/// - The directory structure cannot be created.
/// - The file cannot be opened for appending or written to (e.g., due to permissions or I/O errors).
///
/// ```ts
/// {
///   error: string // Error message (e.g., conversion error, serialization error, file I/O error)
/// }
/// ```
pub(super) fn file_append_json_line(lua: &Lua, runtime: &Runtime, path: String, data: Value) -> mlua::Result<Value> {
	// Resolve the path relative to the workspace directory
	let full_path =
		runtime
			.dir_context()
			.resolve_path(runtime.session(), path.clone().into(), PathResolver::WksDir, None)?;

	// Convert Lua value to serde_json::Value
	let json_value = lua_value_to_serde_value(data).map_err(|e| {
		Error::from(format!(
			"aip.file.append_json_line - Failed to convert Lua data to JSON for file '{path}'.\nCause: {e}",
		))
	})?;

	// Ensure directory exists
	ensure_file_dir(&full_path).map_err(Error::from)?;

	// Append using simple_fs
	simple_fs::append_json_line(full_path.clone(), &json_value).map_err(|e| {
		Error::from(format!(
			"aip.file.append_json_line - Failed to append json line to  '{path}'.\nCause: {e}",
		))
	})?;

	let file_info = FileInfo::new(runtime.dir_context(), path, &full_path);
	file_info.into_lua(lua)
}

/// ## Lua Documentation
///
/// Convert a Lua list (table) of values to JSON strings and append them as new lines to a file.
///
/// ```lua
/// -- API Signature
/// aip.file.append_json_lines(path: string, data: list)
/// ```
///
/// Iterates through the provided Lua `data` list (a table intended to be used as an array with sequential integer keys starting from 1).
/// For each element in the list, converts it into a JSON string and appends this string followed by a newline character (`\n`)
/// to the file specified by `path`. The path is resolved relative to the workspace root.
/// This operation uses buffering internally for potentially better performance when appending many lines compared to calling `append_json_line` repeatedly.
/// If the file does not exist, it will be created. If the directory structure does not exist, it will be created.
///
/// ### Notes
///
/// - The `data` argument MUST be a Lua table used as a list (array-like with sequential integer keys starting from 1). Behavior with non-list tables is undefined.
/// - Lua `nil` values within table elements might be omitted during JSON serialization, e.g., `{a = 1, b = nil}` becomes `{"a":1}`.
///
/// ### Arguments
///
/// - `path: string`: The path to the file where the JSON lines should be appended, relative to the workspace root.
/// - `data: list`: The Lua list (table) containing values to be converted to JSON and appended. Each element can be any JSON-serializable Lua type.
///
/// ### Returns
///
/// - `FileInfo`: The updated [`FileInfo`] for the file.
///
/// ### Example
///
/// ```lua
/// local users = {
///   {user = "alice", score = 88},
///   {user = "bob", score = 92, active = false},
///   {user = "charlie", score = 75, details = nil},
///   "Metadata comment" -- Example of non-table element
/// }
/// aip.file.append_json_lines("user_scores.ndjson", users)
///
/// --[[ content of user_scores.ndjson after execution:
/// {"score":88,"user":"alice"}
/// {"active":false,"score":92,"user":"bob"}
/// {"score":75,"user":"charlie"}
/// "Metadata comment"
/// ]]
/// ```
///
/// ### Error
///
/// Returns an error if:
/// - The `data` argument is not a Lua table or cannot be interpreted as a list.
/// - Any element in the list cannot be converted to an internal JSON representation.
/// - Any internal representation cannot be serialized to a JSON string.
/// - The directory structure cannot be created.
/// - The file cannot be opened for appending or written to (e.g., due to permissions or I/O errors).
///
/// ```ts
/// {
///   error: string // Error message (e.g., data not a table/list, conversion error, serialization error, file I/O error)
/// }
/// ```
pub(super) fn file_append_json_lines(lua: &Lua, runtime: &Runtime, path: String, data: Value) -> mlua::Result<Value> {
	// -- Get the json values
	let json_values = lua_value_list_to_serde_values(data).map_err(|e| {
		Error::from(format!(
			"aip.file.append_json_lines - Failed to append json lines to '{path}'.\nCause: {e}",
		))
	})?;

	// -- Resolve path and ensure directory
	let full_path =
		runtime
			.dir_context()
			.resolve_path(runtime.session(), path.clone().into(), PathResolver::WksDir, None)?;
	ensure_file_dir(&full_path).map_err(Error::from)?;

	// -- Append using simple_fs
	simple_fs::append_json_lines(full_path.clone(), &json_values).map_err(|e| {
		Error::from(format!(
			"aip.file.append_json_lines - Failed to append json line to  '{path}'.\nCause: {e}",
		))
	})?;

	let file_info = FileInfo::new(runtime.dir_context(), path, &full_path);
	file_info.into_lua(lua)
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{
		assert_contains, clean_sanbox_01_tmp_file, create_sanbox_01_tmp_file, gen_sandbox_01_temp_file_path,
		run_reflective_agent,
	};
	use simple_fs::read_to_string;
	use value_ext::JsonValueExt as _;

	// region:    --- load_json Tests

	#[tokio::test]
	async fn test_lua_file_load_json_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_path = "other/test_load_json.json"; // Relative to tests-data/sandbox-01/

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.load_json("{fx_path}")"#), None).await?;

		// -- Check
		assert_eq!(res.x_get_str("name")?, "Test JSON");
		assert_eq!(res.x_get_f64("version")?, 1.2);
		assert!(res.x_get_bool("enabled")?, "enabled should be true");

		let items = res
			.get("items")
			.ok_or("should have items")?
			.as_array()
			.ok_or("should be array")?;
		assert_eq!(items.len(), 2);
		assert_eq!(items[0].as_str().ok_or("should have item")?, "item1");
		assert_eq!(items[1].as_str().ok_or("should have item")?, "item2");

		let nested: &serde_json::Value = res.get("nested").ok_or("should have nested")?;
		assert_eq!(nested.x_get_str("key")?, "value");

		// Check null value handling
		let nullable = res.get("nullable").ok_or("should have nullable")?;
		assert!(nullable.is_null(), "nullable should be json null");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_load_json_with_comment_and_trailing() -> Result<()> {
		// -- Setup & Fixtures
		let fx_file = create_sanbox_01_tmp_file(
			"test_lua_file_load_json_with_comment_and_trailing.ndjson",
			r#"
{
	// Here are smoe comment
  "name": "Test JSON",
  "version": 1.2,
  "enabled": true,
  "items": [
    "item1",
    "item2"
  ],
  "nested": {
    "key": "value"
  },
	// Trailing comma
  "nullable": null,
}
"#,
		)?;
		let fx_path = fx_file.as_str(); // Relative to tests-data/sandbox-01/

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.load_json("{fx_path}")"#), None).await?;

		// -- Check
		assert_eq!(res.x_get_str("name")?, "Test JSON");
		assert_eq!(res.x_get_f64("version")?, 1.2);
		assert!(res.x_get_bool("enabled")?, "enabled should be true");

		let items = res
			.get("items")
			.ok_or("should have items")?
			.as_array()
			.ok_or("should be array")?;
		assert_eq!(items.len(), 2);
		assert_eq!(items[0].as_str().ok_or("should have item")?, "item1");
		assert_eq!(items[1].as_str().ok_or("should have item")?, "item2");

		let nested: &serde_json::Value = res.get("nested").ok_or("should have nested")?;
		assert_eq!(nested.x_get_str("key")?, "value");

		// Check null value handling
		let nullable = res.get("nullable").ok_or("should have nullable")?;
		assert!(nullable.is_null(), "nullable should be json null");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_load_json_file_not_found() -> Result<()> {
		// -- Setup & Fixtures
		let fx_path = "other/non_existent_file.json";

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.load_json("{fx_path}")"#), None).await;

		// -- Check
		let Err(err) = res else {
			panic!("Should have returned an error");
		};
		assert_contains(&err.to_string(), "aip.file.load_json - Failed to read json file");
		assert_contains(&err.to_string(), "non_existent_file.json");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_load_json_invalid_json() -> Result<()> {
		// -- Setup & Fixtures
		// Use an existing text file that is not JSON
		let fx_path = "file-01.txt"; // Content is "hello world file 01"

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.load_json("{fx_path}")"#), None).await;

		// -- Check
		let Err(err) = res else {
			panic!("Should have returned an error");
		};
		let err = err.to_string();
		assert_contains(&err, "aip.file.load_json - Failed to read json");
		assert_contains(&err, fx_path);

		Ok(())
	}

	// endregion: --- load_json Tests

	// region:    --- load_ndjson Tests

	#[tokio::test]
	async fn test_lua_file_load_jsonnd_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_path = "other/test_load_ndjson.ndjson"; // Relative to tests-data/sandbox-01/

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.load_ndjson("{fx_path}")"#), None).await?;

		// -- Check
		let arr = res.as_array().ok_or("Result should be an array")?;
		assert_eq!(arr.len(), 3, "Should have 3 items from the ndjson file");

		// Check first item
		let item1 = arr.first().ok_or("Should have item 1")?;
		assert_eq!(item1.x_get_str("name")?, "item1");
		assert_eq!(item1.x_get_i64("value")?, 10);

		// Check second item
		let item2 = arr.get(1).ok_or("Should have item 2")?;
		assert_eq!(item2.x_get_str("name")?, "item2");
		assert_eq!(item2.x_get_i64("value")?, 20);
		assert!(item2.x_get_bool("active")?);

		// Check third item (with null and array)
		let item3 = arr.get(2).ok_or("Should have item 3")?;
		assert_eq!(item3.x_get_str("name")?, "item3");
		assert!(item3.get("value").ok_or("item3 should have value")?.is_null());
		let tags = item3
			.get("tags")
			.ok_or("item3 should have tags")?
			.as_array()
			.ok_or("tags should be array")?;
		assert_eq!(tags.len(), 2);
		assert_eq!(tags[0].as_str().ok_or("tag should be string")?, "a");
		assert_eq!(tags[1].as_str().ok_or("tag should be string")?, "b");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_load_jsonnd_file_not_found() -> Result<()> {
		// -- Setup & Fixtures
		let fx_path = "other/non_existent_file.ndjson";

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.load_ndjson("{fx_path}")"#), None).await;

		// -- Check
		let Err(err) = res else {
			panic!("Should have returned an error");
		};
		assert_contains(
			&err.to_string(),
			"aip.file.load_ndjson - Failed to load newline json file",
		);
		assert_contains(&err.to_string(), "non_existent_file.ndjson");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_load_jsonnd_invalid_json_line() -> Result<()> {
		// -- Setup & Fixtures
		let fx_file = create_sanbox_01_tmp_file(
			"test_lua_file_load_jsonnd_invalid_json_line.ndjson",
			r#"{"valid": true}
invalid json line here
{"another_valid": 123}
"#,
		)?;
		let fx_path = fx_file.as_str();

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.load_ndjson("{fx_path}")"#), None).await;

		// -- Check
		let Err(err) = res else {
			panic!("Should have returned an error");
		};
		assert_contains(
			&err.to_string(),
			"aip.file.load_ndjson - Failed to parse JSON on line 2",
		);
		assert_contains(&err.to_string(), fx_path);

		// -- Clean
		clean_sanbox_01_tmp_file(fx_file)?;

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_load_jsonnd_empty_file() -> Result<()> {
		// -- Setup & Fixtures
		let fx_file = create_sanbox_01_tmp_file("test_lua_file_load_jsonnd_empty_file.ndjson", "")?;
		let fx_path = fx_file.as_str();

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.load_ndjson("{fx_path}")"#), None).await?;

		// -- Check
		let arr = res.as_array().ok_or("Result should be an array")?;
		assert_eq!(arr.len(), 0, "Should have 0 items from an empty file");

		// -- Clean
		clean_sanbox_01_tmp_file(fx_file)?;

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_load_jsonnd_empty_lines_file() -> Result<()> {
		// -- Setup & Fixtures
		let fx_file = create_sanbox_01_tmp_file(
			"test_lua_file_load_jsonnd_empty_lines_file.ndjson",
			r#"

{"valid": true}


{"another": "valid"}

"#,
		)?;
		let fx_path = fx_file.as_str();

		// -- Exec
		let res = run_reflective_agent(&format!(r#"return aip.file.load_ndjson("{fx_path}")"#), None).await?;

		// -- Check
		let arr = res.as_array().ok_or("Result should be an array")?;
		assert_eq!(arr.len(), 2, "Should have 2 items, skipping empty lines");
		assert!(arr[0].x_get_bool("valid")?);
		assert_eq!(arr[1].x_get_str("another")?, "valid");

		// -- Clean
		clean_sanbox_01_tmp_file(fx_file)?;

		Ok(())
	}

	// endregion: --- load_ndjson Tests

	// region:    --- append_json_line Tests

	#[tokio::test]
	async fn test_lua_file_append_json_line_new_file_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fix_file = gen_sandbox_01_temp_file_path("test_lua_file_append_json_line_new_file.ndjson");
		let fx_path = fix_file.as_str();
		let fx_data1 = r#"{name = "item1", value = 123}"#;
		let fx_data2 = r#"{name = "item2", active = true, tags = {"a", "b"}}"#;

		// -- Exec
		let res1 = run_reflective_agent(
			&format!(r#"return aip.file.append_json_line("{fx_path}", {fx_data1})"#),
			None,
		)
		.await?;
		let res2 = run_reflective_agent(
			&format!(r#"return aip.file.append_json_line("{fx_path}", {fx_data2})"#),
			None,
		)
		.await?;

		// -- Check
		// Check FileInfo
		assert_eq!(res1.x_get_str("path")?, fx_path);
		assert_eq!(res2.x_get_str("path")?, fx_path);
		assert!(res2.x_get_i64("size")? > res1.x_get_i64("size")?);

		// Check content
		let full_path = format!("tests-data/sandbox-01/{fx_path}");
		let content = read_to_string(&full_path)?;
		let lines: Vec<&str> = content.lines().collect();

		assert_eq!(lines.len(), 2, "Should have 2 lines");
		assert_eq!(lines[0], r#"{"name":"item1","value":123}"#);
		assert_eq!(lines[1], r#"{"active":true,"name":"item2","tags":["a","b"]}"#);

		// -- Clean
		// comment out cleanup for inspection
		// clean_sanbox_01_tmp_file(fix_file)?;

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_append_json_line_existing_file_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_file_name = "test_lua_file_append_json_line_existing_file.ndjson";
		let initial_content = r#"{"initial": true}
"#; // Note the newline
		let fx_file = create_sanbox_01_tmp_file(fx_file_name, initial_content)?;
		let fx_path = fx_file.as_str();
		let fx_data = r#"{appended = "yes", value = nil}"#;

		// -- Exec
		let res = run_reflective_agent(
			&format!(r#"return aip.file.append_json_line("{fx_path}", {fx_data})"#),
			None,
		)
		.await?;

		// -- Check
		// Check FileInfo
		assert_eq!(res.x_get_str("path")?, fx_path);
		assert!(res.x_get_i64("size")? > initial_content.len() as i64);

		// Check content
		let full_path = format!("tests-data/sandbox-01/{fx_path}");
		let content = read_to_string(&full_path)?;
		let lines: Vec<&str> = content.lines().collect();

		assert_eq!(lines.len(), 2, "Should have 2 lines (initial + appended)");
		assert_eq!(lines[0], r#"{"initial": true}"#);
		// IMPORTANT: We cannot preserve the "value = nil" as "value = null" as mlua return nil for absent as well
		assert_eq!(lines[1], r#"{"appended":"yes"}"#);

		// -- Clean
		// comment out cleanup for inspection
		// clean_sanbox_01_tmp_file(fx_file)?;

		Ok(())
	}

	// endregion: --- append_json_line Tests

	// region:    --- append_json_lines Tests

	#[tokio::test]
	async fn test_lua_file_append_json_lines_new_file_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fix_file = gen_sandbox_01_temp_file_path("test_lua_file_append_json_lines_new_file.ndjson");
		let fx_path = fix_file.as_str();
		let fx_data = r#"
        {
            {name = "line1", value = 1},
            {name = "line2", active = true},
            {name = "line3", tags = {"c", "d"}, data = nil}
        }
        "#;

		// -- Exec
		let res = run_reflective_agent(
			&format!(r#"return aip.file.append_json_lines("{fx_path}", {fx_data})"#),
			None,
		)
		.await?;

		// -- Check
		// Check FileInfo
		assert_eq!(res.x_get_str("path")?, fx_path);
		assert!(res.x_get_i64("size")? > 0);

		// Check content
		let full_path = format!("tests-data/sandbox-01/{fx_path}");
		let content = read_to_string(&full_path)?;
		let lines: Vec<&str> = content.lines().collect();

		assert_eq!(lines.len(), 3, "Should have 3 lines");
		assert_eq!(lines[0], r#"{"name":"line1","value":1}"#);
		assert_eq!(lines[1], r#"{"active":true,"name":"line2"}"#);
		assert_eq!(lines[2], r#"{"name":"line3","tags":["c","d"]}"#); // Note: data = nil is omitted

		// -- Clean
		// comment out cleanup for inspection
		// clean_sanbox_01_tmp_file(fix_file)?;

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_append_json_lines_existing_file_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_file_name = "test_lua_file_append_json_lines_existing_file.ndjson";
		let initial_content = r#"{"initial": true}
"#; // Note the newline
		let fx_file = create_sanbox_01_tmp_file(fx_file_name, initial_content)?;
		let fx_path = fx_file.as_str();
		let fx_data = r#"
        {
            {appended = "yes"},
            {another = 123}
        }
        "#;

		// -- Exec
		let res = run_reflective_agent(
			&format!(r#"return aip.file.append_json_lines("{fx_path}", {fx_data})"#),
			None,
		)
		.await?;

		// -- Check
		// Check FileInfo
		assert_eq!(res.x_get_str("path")?, fx_path);
		assert!(res.x_get_i64("size")? > initial_content.len() as i64);

		// Check content
		let full_path = format!("tests-data/sandbox-01/{fx_path}");
		let content = read_to_string(&full_path)?;
		let lines: Vec<&str> = content.lines().collect();

		assert_eq!(lines.len(), 3, "Should have 3 lines (initial + 2 appended)");
		assert_eq!(lines[0], r#"{"initial": true}"#);
		assert_eq!(lines[1], r#"{"appended":"yes"}"#);
		assert_eq!(lines[2], r#"{"another":123}"#);

		// -- Clean
		// comment out cleanup for inspection
		// clean_sanbox_01_tmp_file(fx_file)?;

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_append_json_lines_empty_list_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fix_file = gen_sandbox_01_temp_file_path("test_lua_file_append_json_lines_empty_list.ndjson");
		let fx_path = fix_file.as_str();
		let fx_data = r#"{}"#; // Empty list

		// -- Exec
		let res = run_reflective_agent(
			&format!(r#"return aip.file.append_json_lines("{fx_path}", {fx_data})"#),
			None,
		)
		.await?;

		// -- Check
		// Check FileInfo
		assert_eq!(res.x_get_str("path")?, fx_path);
		assert_eq!(res.x_get_i64("size")?, 0);

		// Check content
		let full_path = format!("tests-data/sandbox-01/{fx_path}");
		let content = read_to_string(&full_path)?;
		assert_eq!(content, "", "File should be empty");

		// -- Clean
		// comment out cleanup for inspection
		// clean_sanbox_01_tmp_file(fix_file)?;

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_append_json_lines_buffering_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fix_file = gen_sandbox_01_temp_file_path("test_lua_file_append_json_lines_buffering.ndjson");
		let fx_path = fix_file.as_str();
		// Create data larger than buffer size
		let mut lua_list = String::from("{");
		for i in 0..(100 + 5) {
			lua_list.push_str(&format!(r#"{{idx = {i}, name = "name-{i}""#));
			// Add a nil value occasionally to test handling
			if i % 10 == 0 {
				lua_list.push_str(", optional = nil");
			}
			lua_list.push_str("},");
		}
		lua_list.push('}');

		// -- Exec
		let res = run_reflective_agent(
			&format!(r#"return aip.file.append_json_lines("{fx_path}", {lua_list})"#),
			None,
		)
		.await?;

		// -- Check
		// Check FileInfo
		assert_eq!(res.x_get_str("path")?, fx_path);
		assert!(res.x_get_i64("size")? > 0);

		// Check content
		let full_path = format!("tests-data/sandbox-01/{fx_path}");
		let content = read_to_string(&full_path)?;
		let lines: Vec<&str> = content.lines().collect();

		assert_eq!(lines.len(), 100 + 5, "Should have correct number of lines");
		// Check first and last lines as a sample
		assert_eq!(lines[0], r#"{"idx":0,"name":"name-0"}"#); // optional = nil omitted
		assert_eq!(
			lines.last().ok_or("Should have last")?,
			&format!("{{\"idx\":{idx},\"name\":\"name-{idx}\"}}", idx = 100 + 4)
		);

		// -- Clean
		// comment out cleanup for inspection
		// clean_sanbox_01_tmp_file(fix_file)?;

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_append_json_lines_err_not_a_table() -> Result<()> {
		// -- Setup & Fixtures
		let fix_file = gen_sandbox_01_temp_file_path("test_lua_file_append_json_lines_err_not_table.ndjson");
		let fx_path = fix_file.as_str();
		let fx_data = r#""just a string""#; // Not a table

		// -- Exec
		let result =
			run_reflective_agent(&format!(r#"aip.file.append_json_lines("{fx_path}", {fx_data})"#), None).await;

		// -- Check
		let Err(err) = result else {
			panic!("Should have returned an error");
		};
		assert_contains(
			&err.to_string(),
			"aip.file.append_json_lines - Failed to append json lines",
		);
		assert_contains(&err.to_string(), "but got string");

		// -- Clean
		// File might have been created, attempt cleanup
		let _ = clean_sanbox_01_tmp_file(fix_file);

		Ok(())
	}

	// endregion: --- append_json_lines Tests
}

// endregion: --- Tests
