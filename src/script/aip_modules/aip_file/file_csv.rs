//! Lua CSV helpers for `aip.file`.
//!
//! ---
//!
//! ## Lua documentation for `aip.file` CSV helpers
//!
//! ### Functions
//!
//! - `aip.file.load_csv_headers(path: string): string[]`
//! - `aip.file.load_csv(path: string, options?: CsvOptions): { _type: "CsvContent", headers: string[], rows: string[][] }`
//! - `aip.file.save_as_csv(path: string, data: matrix | {headers, rows}, options?: CsvOptions): FileInfo`
//! - `aip.file.save_records_as_csv(path: string, records: table[], header_keys: string[], options?: CsvOptions): FileInfo`
//! - `aip.file.append_csv_rows(path: string, value_lists: any[][], options?: CsvOptions): FileInfo`
//! - `aip.file.append_csv_row(path: string, values: any[], options?: CsvOptions): FileInfo`
//!
//! The `path` is resolved relative to the workspace root.

use crate::Error;
use crate::dir_context::PathResolver;
use crate::runtime::Runtime;
use crate::script::aip_modules::aip_csv::{lua_matrix_to_rows, lua_value_to_csv_string};
use crate::script::aip_modules::support::check_access_write;
use crate::script::support::{collect_string_sequence, expect_table};
use crate::support::W;
use crate::types::{CsvContent, CsvOptions, FileInfo};
use mlua::{FromLua as _, IntoLua, Lua, Value};

/// ## Lua Documentation
///
/// Loads a CSV file and returns its header row as a list of strings.
///
/// ```lua
/// -- API Signature
/// aip.file.load_csv_headers(path: string): string[]
/// ```
///
/// - `path: string` — CSV file path, relative to the workspace root (supports pack refs).
///
/// Returns a Lua array (list) of strings containing the header names.
///
/// ### Example
///
/// ```lua
/// local headers = aip.file.load_csv_headers("data/example.csv")
/// for i, h in ipairs(headers) do
///   print(i, h)
/// end
/// ```
///
/// ### Error
///
/// Returns an error if:
/// - The path cannot be resolved,
/// - The file cannot be found or read,
/// - CSV parsing fails.
pub(super) fn file_load_csv_headers(lua: &Lua, runtime: &Runtime, path: String) -> mlua::Result<Value> {
	let full_path =
		runtime
			.dir_context()
			.resolve_path(runtime.session(), path.clone().into(), PathResolver::WksDir, None)?;

	let headers = crate::support::csvs::load_csv_headers(&full_path, None).map_err(|e| {
		Error::from(format!(
			"aip.file.load_csv_headers - Failed to read csv file '{path}'.\nCause: {e}",
		))
	})?;

	let headers_tbl = W(headers).into_lua(lua)?;
	Ok(headers_tbl)
}

/// ## Lua Documentation
///
/// Loads a CSV file and returns its headers (optionally) and all rows as string arrays.
///
/// ```lua
/// -- API Signature
/// aip.file.load_csv(path: string, options?: CsvOptions): { _type: "CsvContent", headers: string[], rows: string[][] }
/// ```
///
/// - `path: string` — CSV file path, relative to the workspace root (supports pack refs).
/// - `options?: CsvOptions` — CSV parse options. Only `has_header` is honored by this API
///   (defaults to `true`), which controls whether the first row is treated as headers and
///   excluded from the returned `rows`.
///
/// ### Returns
///
/// - `{ _type: "CsvContent", headers: string[], rows: string[][] }`
///
/// ### Example
///
/// ```lua
/// local res = aip.file.load_csv("data/example.csv") -- defaults to with_headers = true
/// print("Type:", res._type) -- "CsvContent"
/// print("Headers:", table.concat(res.headers, ", "))
/// for _, row in ipairs(res.rows) do
///   print(table.concat(row, " | "))
/// end
/// ```
///
/// ### Error
///
/// Returns an error if:
/// - The path cannot be resolved,
/// - The file cannot be found or read,
/// - CSV parsing fails.
pub(super) fn file_load_csv(lua: &Lua, runtime: &Runtime, path: String, options: Option<Value>) -> mlua::Result<Value> {
	let full_path =
		runtime
			.dir_context()
			.resolve_path(runtime.session(), path.clone().into(), PathResolver::WksDir, None)?;

	let opts = match options {
		Some(v) => CsvOptions::from_lua(v, lua)?,
		None => CsvOptions::default(),
	};

	let csv_content = crate::support::csvs::load_csv(&full_path, Some(opts)).map_err(|e| {
		Error::from(format!(
			"aip.file.load_csv - Failed to read csv file '{path}'.\nCause: {e}",
		))
	})?;

	csv_content.into_lua(lua)
}

/// ## Lua Documentation
///
/// Save data as CSV file (overwrite).
///
/// ```lua
/// -- API Signature
/// aip.file.save_as_csv(
///   path: string,
///   data: any[][] | { headers: string[], rows: any[][] },
///   options?: CsvOptions
/// ): FileInfo
/// ```
pub(super) fn file_save_as_csv(
	lua: &Lua,
	runtime: &Runtime,
	path: String,
	data: Value,
	options: Option<Value>,
) -> mlua::Result<Value> {
	let dir_context = runtime.dir_context();
	let full_path = dir_context.resolve_path(runtime.session(), path.clone().into(), PathResolver::WksDir, None)?;

	// We might not want that once workspace is truely optional
	let wks_dir = dir_context.try_wks_dir_with_err_ctx("aip.file.save_as_csv requires a aipack workspace setup")?;

	check_access_write(&full_path, wks_dir)?;

	let opts = match options {
		Some(v) => Some(CsvOptions::from_lua(v, lua)?),
		None => None,
	};
	let has_header = opts.as_ref().and_then(|o| o.has_header);

	let content = normalize_csv_payload(lua, data, has_header)?;

	crate::support::csvs::save_csv(&full_path, &content, opts).map_err(|e| {
		Error::from(format!(
			"aip.file.save_as_csv - Failed to save csv file '{path}'.\nCause: {e}",
		))
	})?;

	let file_info = FileInfo::new(runtime.dir_context(), path, &full_path);
	file_info.into_lua(lua)
}

/// ## Lua Documentation
///
/// Save a list of records as CSV file (overwrite).
///
/// ```lua
/// -- API Signature
/// aip.file.save_records_as_csv(
///   path: string,
///   records: table[],
///   header_keys: string[],
///   options?: CsvOptions
/// ): FileInfo
/// ```
pub(super) fn file_save_records_as_csv(
	lua: &Lua,
	runtime: &Runtime,
	path: String,
	records: Value,
	header_keys: Vec<String>,
	options: Option<Value>,
) -> mlua::Result<Value> {
	let dir_context = runtime.dir_context();
	let full_path = dir_context.resolve_path(runtime.session(), path.clone().into(), PathResolver::WksDir, None)?;

	// We might not want that once workspace is truely optional
	let wks_dir =
		dir_context.try_wks_dir_with_err_ctx("aip.file.save_records_as_csv requires a aipack workspace setup")?;

	check_access_write(&full_path, wks_dir)?;

	let records_tbl = expect_table(records, "aip.file.save_records_as_csv", "records")?;

	// -- Build rows
	let mut rows = Vec::new();
	for rec_val in records_tbl.sequence_values::<Value>() {
		let rec_val = rec_val?;
		if let Value::Table(rec) = rec_val {
			let mut row = Vec::new();
			for key in &header_keys {
				let val = rec.get::<Value>(key.as_str())?;
				row.push(lua_value_to_csv_string(val)?);
			}
			rows.push(row);
		} else {
			return Err(mlua::Error::external("Records must be a list of tables"));
		}
	}

	let content = CsvContent {
		headers: header_keys,
		rows,
	};

	let opts = match options {
		Some(v) => Some(CsvOptions::from_lua(v, lua)?),
		None => None,
	};

	crate::support::csvs::save_csv(&full_path, &content, opts).map_err(|e| {
		Error::from(format!(
			"aip.file.save_records_as_csv - Failed to save csv file '{path}'.\nCause: {e}",
		))
	})?;

	let file_info = FileInfo::new(runtime.dir_context(), path, &full_path);
	file_info.into_lua(lua)
}

/// ## Lua Documentation
///
/// Append rows to a CSV file.
///
/// ```lua
/// -- API Signature
/// aip.file.append_csv_rows(
///   path: string,
///   value_lists: any[][],
///   options?: CsvOptions
/// ): FileInfo
/// ```
pub(super) fn file_append_csv_rows(
	lua: &Lua,
	runtime: &Runtime,
	path: String,
	value_lists: Value,
	options: Option<Value>,
) -> mlua::Result<Value> {
	let dir_context = runtime.dir_context();
	let full_path = dir_context.resolve_path(runtime.session(), path.clone().into(), PathResolver::WksDir, None)?;

	// We might not want that once workspace is truely optional
	let wks_dir = dir_context.try_wks_dir_with_err_ctx("aip.file.append_csv_rows requires a aipack workspace setup")?;

	check_access_write(&full_path, wks_dir)?;

	let rows = match value_lists {
		Value::Table(t) => lua_matrix_to_rows(t)?,
		_ => return Err(mlua::Error::external("value_lists must be a table (list of lists)")),
	};

	let content = CsvContent {
		headers: Vec::new(),
		rows,
	};

	let opts = match options {
		Some(v) => Some(CsvOptions::from_lua(v, lua)?),
		None => None,
	};

	crate::support::csvs::append_csv(&full_path, &content, opts).map_err(|e| {
		Error::from(format!(
			"aip.file.append_csv_rows - Failed to append rows to csv file '{path}'.\nCause: {e}",
		))
	})?;

	let file_info = FileInfo::new(runtime.dir_context(), path, &full_path);
	file_info.into_lua(lua)
}

/// ## Lua Documentation
///
/// Append a single row to a CSV file.
///
/// ```lua
/// -- API Signature
/// aip.file.append_csv_row(
///   path: string,
///   values: any[],
///   options?: CsvOptions
/// ): FileInfo
/// ```
pub(super) fn file_append_csv_row(
	lua: &Lua,
	runtime: &Runtime,
	path: String,
	values: Value,
	options: Option<Value>,
) -> mlua::Result<Value> {
	let dir_context = runtime.dir_context();
	let full_path = dir_context.resolve_path(runtime.session(), path.clone().into(), PathResolver::WksDir, None)?;

	// We might not want that once workspace is truely optional
	let wks_dir = dir_context.try_wks_dir_with_err_ctx("aip.file.append_csv_row requires a aipack workspace setup")?;

	check_access_write(&full_path, wks_dir)?;

	let row = match values {
		Value::Table(t) => {
			let mut r = Vec::new();
			for val in t.sequence_values::<Value>() {
				r.push(lua_value_to_csv_string(val?)?);
			}
			r
		}
		_ => return Err(mlua::Error::external("values must be a table (list)")),
	};

	let content = CsvContent {
		headers: Vec::new(),
		rows: vec![row],
	};

	let opts = match options {
		Some(v) => Some(CsvOptions::from_lua(v, lua)?),
		None => None,
	};

	crate::support::csvs::append_csv(&full_path, &content, opts).map_err(|e| {
		Error::from(format!(
			"aip.file.append_csv_row - Failed to append row to csv file '{path}'.\nCause: {e}",
		))
	})?;

	let file_info = FileInfo::new(runtime.dir_context(), path, &full_path);
	file_info.into_lua(lua)
}

// region:    --- Support

/// Normalize the CSV payload (data argument) into a CsvContent struct.
///
/// It handles both:
/// - A matrix (list of lists) `any[][]`.
/// - A structured table `{ headers?: string[], rows?: any[][] }`.
///
/// If a matrix is provided and `has_header` is true (via options), the first row is treated as headers.
fn normalize_csv_payload(_lua: &Lua, data: Value, has_header_opt: Option<bool>) -> mlua::Result<CsvContent> {
	let has_header = has_header_opt.unwrap_or(false);

	if let Value::Table(t) = &data {
		// Check for structured data: presence of "rows" or "headers"
		let rows_val = t.get::<Value>("rows")?;
		let headers_val = t.get::<Value>("headers")?;

		let is_structured = !rows_val.is_nil() || !headers_val.is_nil();

		if is_structured {
			// -- Headers
			let headers = match headers_val {
				Value::Table(ht) => {
					let seq = collect_string_sequence(Value::Table(ht), "CsvContent", "headers")?;
					seq.into_iter().map(|s| s.to_string_lossy()).collect()
				}
				Value::Nil => Vec::new(),
				other => {
					return Err(mlua::Error::external(format!(
						"'headers' must be a table, found {}",
						other.type_name()
					)));
				}
			};

			// -- Rows
			let rows = match rows_val {
				Value::Table(rt) => lua_matrix_to_rows(rt)?,
				Value::Nil => Vec::new(),
				other => {
					return Err(mlua::Error::external(format!(
						"'rows' must be a table, found {}",
						other.type_name()
					)));
				}
			};

			return Ok(CsvContent { headers, rows });
		}

		// Otherwise assume it is a matrix
		let mut rows = lua_matrix_to_rows(t.clone())?;
		let headers = if has_header && !rows.is_empty() {
			rows.remove(0)
		} else {
			Vec::new()
		};

		Ok(CsvContent { headers, rows })
	} else {
		Err(mlua::Error::external(
			"Data must be a table (matrix or structured {headers, rows})",
		))
	}
}

// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	use crate::_test_support::{clean_sanbox_01_tmp_file, gen_sandbox_01_temp_file_path, run_reflective_agent};
	use simple_fs::read_to_string;
	use value_ext::JsonValueExt as _;

	#[tokio::test]
	async fn test_lua_file_csv_save_as_csv_matrix_ok() -> Result<()> {
		let fx_path = gen_sandbox_01_temp_file_path("test_save_as_csv_matrix.csv");
		let fx_lua = format!(
			r#"
            local data = {{
                {{"name", "age"}},
                {{"Alice", 30}},
                {{"Bob", 25}}
            }}
            return aip.file.save_as_csv("{fx_path}", data, {{has_header = true}})
        "#
		);

		let res = run_reflective_agent(&fx_lua, None).await?;
		assert_eq!(res.x_get_str("path")?, fx_path.as_str());

		let content = read_to_string(format!("tests-data/sandbox-01/{fx_path}"))?;
		let lines: Vec<&str> = content.lines().collect();
		assert_eq!(lines.len(), 3);
		assert_eq!(lines[0], "name,age");
		assert_eq!(lines[1], "Alice,30");
		assert_eq!(lines[2], "Bob,25");

		clean_sanbox_01_tmp_file(fx_path)?;
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_csv_append_csv_rows_ok() -> Result<()> {
		let fx_path = gen_sandbox_01_temp_file_path("test_append_csv_rows.csv");

		// Create file with header first
		let fx_lua_header = format!(
			r#"
            aip.file.save_as_csv("{fx_path}", {{headers={{"name", "age"}}, rows={{}}}})
            "#
		);
		run_reflective_agent(&fx_lua_header, None).await?;

		// Now append rows
		let fx_lua = format!(
			r#"
            local data = {{
                {{"Alice", 30}},
                {{"Bob", 25}}
            }}
            -- has_header should be ignored for append_csv_rows
            return aip.file.append_csv_rows("{fx_path}", data, {{has_header = true}})
        "#
		);

		run_reflective_agent(&fx_lua, None).await?;

		let content = read_to_string(format!("tests-data/sandbox-01/{fx_path}"))?;
		let lines: Vec<&str> = content.lines().collect();
		assert_eq!(lines.len(), 3);
		assert_eq!(lines[0], "name,age");
		assert_eq!(lines[1], "Alice,30");
		assert_eq!(lines[2], "Bob,25");

		clean_sanbox_01_tmp_file(fx_path)?;
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_csv_append_csv_row_ok() -> Result<()> {
		let fx_path = gen_sandbox_01_temp_file_path("test_append_csv_row.csv");

		// Append one row
		let fx_lua = format!(
			r#"
            return aip.file.append_csv_row("{fx_path}", {{"Alice", 30}})
        "#
		);

		run_reflective_agent(&fx_lua, None).await?;

		let content = read_to_string(format!("tests-data/sandbox-01/{fx_path}"))?;
		let lines: Vec<&str> = content.lines().collect();
		assert_eq!(lines.len(), 1);
		assert_eq!(lines[0], "Alice,30");

		clean_sanbox_01_tmp_file(fx_path)?;
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_csv_save_as_csv_only_headers() -> Result<()> {
		let fx_path = gen_sandbox_01_temp_file_path("test_save_as_csv_only_headers.csv");
		let fx_lua = format!(
			r#"
            local data = {{
                headers = {{"name", "age"}}
            }}
            return aip.file.save_as_csv("{fx_path}", data)
        "#
		);

		run_reflective_agent(&fx_lua, None).await?;
		let content = read_to_string(format!("tests-data/sandbox-01/{fx_path}"))?;
		let lines: Vec<&str> = content.lines().collect();
		assert_eq!(lines.len(), 1);
		assert_eq!(lines[0], "name,age");

		clean_sanbox_01_tmp_file(fx_path)?;
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_csv_save_as_csv_only_rows() -> Result<()> {
		let fx_path = gen_sandbox_01_temp_file_path("test_save_as_csv_only_rows.csv");
		let fx_lua = format!(
			r#"
            local data = {{
                rows = {{ {{"Alice", 30}} }}
            }}
            return aip.file.save_as_csv("{fx_path}", data)
        "#
		);

		run_reflective_agent(&fx_lua, None).await?;
		let content = read_to_string(format!("tests-data/sandbox-01/{fx_path}"))?;
		let lines: Vec<&str> = content.lines().collect();
		assert_eq!(lines.len(), 1);
		assert_eq!(lines[0], "Alice,30");

		clean_sanbox_01_tmp_file(fx_path)?;
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_csv_save_as_csv_empty_table() -> Result<()> {
		let fx_path = gen_sandbox_01_temp_file_path("test_save_as_csv_empty.csv");
		let fx_lua = format!(
			r#"
            local data = {{}}
            return aip.file.save_as_csv("{fx_path}", data)
        "#
		);

		run_reflective_agent(&fx_lua, None).await?;
		// Should exist and be empty
		assert!(std::path::Path::new(&format!("tests-data/sandbox-01/{fx_path}")).exists());
		let content = read_to_string(format!("tests-data/sandbox-01/{fx_path}"))?;
		assert!(content.is_empty());

		clean_sanbox_01_tmp_file(fx_path)?;
		Ok(())
	}
}

// endregion: --- Tests
