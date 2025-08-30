//! Defines the `lua` module, used in the lua engine.
//!
//! ---
//!
//! ## Lua documentation
//! The `lua` module exposes functions for inspecting and dumping Lua values.
//!
//! ### Functions
//!
//! - `aip.lua.dump(value: any) -> string`

use crate::Result;
use crate::runtime::Runtime;
use crate::script::lua_na::NASentinel;
use mlua::{Lua, Table, Value};

pub fn init_module(lua: &Lua, _runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	let dump_lua = lua.create_function(dump)?;
	table.set("dump", dump_lua)?;

	Ok(table)
}

// region: --- Rust Lua Support

/// ## Lua Documentation
///
/// Dump a Lua value into its string representation.
///
/// ```lua
/// -- API Signature
/// aip.lua.dump(value: any): string
/// ```
///
/// Given any Lua value, returns a string that recursively represents tables and their structure.
/// Useful for debugging and logging purposes.
///
/// ### Arguments
///
/// - `value`: The Lua value to be dumped. Can be any Lua type (nil, boolean, number, string, table, function, userdata, etc.).
///
/// ### Returns
///
/// - `string`: A string representation of the Lua value. Table contents are recursively dumped with indentation. Functions, userdata, and threads are represented by placeholder strings.
///
/// ```ts
/// string
/// ```
///
/// ### Example
///
/// ```lua
/// local tbl = { key = "value", nested = { subkey = 42 } }
/// print(aip.lua.dump(tbl))
/// -- Expected output structure (exact indentation/formatting might vary slightly based on internal Lua/dump implementation):
/// -- {
/// --   nested = {
/// --     key1 = "value1",
/// --     key2 = 42
/// --   },
/// --   bool = true,
/// --   num = 3.21
/// -- }
///
/// print(aip.lua.dump("Hello World"))
/// -- Expected output: "Hello World"
///
/// print(aip.lua.dump(123.45))
/// -- Expected output: 123.45
/// ```
///
/// ### Error
///
/// Returns an error message string if the conversion of a value (like a non-UTF8 string or specific userdata) within the dump process fails.
///
/// ```ts
/// {
///   error: string // Error message detailing the conversion failure
/// }
/// ```
pub fn dump(lua: &Lua, value: Value) -> mlua::Result<String> {
	fn dump_value(_lua: &Lua, value: Value, indent: usize) -> mlua::Result<String> {
		let indent_str = "  ".repeat(indent);
		match value {
			Value::Nil => Ok("nil".to_string()),
			Value::Boolean(b) => Ok(b.to_string()),
			Value::Integer(i) => Ok(i.to_string()),
			Value::Number(n) => Ok(n.to_string()),
			Value::String(s) => {
				let s: String = s.to_str()?.to_string();
				Ok(format!("\"{s}\""))
			}
			Value::Table(t) => {
				let mut entries: Vec<String> = Vec::new();
				for pair in t.clone().pairs::<Value, Value>() {
					let (key, val) = pair?;
					let dumped_key = match key {
						Value::String(s) => s.to_str()?.to_string(),
						_ => dump_value(_lua, key, 0)?,
					};
					let dumped_val = dump_value(_lua, val, indent + 1)?;
					entries.push(format!(
						"{indent_str_for_entry}{dumped_key} = {dumped_val}",
						indent_str_for_entry = "  ".repeat(indent + 1)
					));
				}
				let inner = entries.join(",\n");
				Ok(format!("{{\n{inner}\n{indent_str}}}"))
			}
			Value::Function(f) => {
				let name = f.info().name.unwrap_or("<anonymous>".to_string());
				Ok(format!("<function {name}>"))
			}
			Value::UserData(ud) => {
				if ud.is::<NASentinel>() {
					Ok("NA".into())
				} else {
					Ok("<UserData>".into())
				}
			}
			Value::LightUserData(_) => Ok("<LightUserData>".to_string()),
			Value::Thread(_) => Ok("<Thread>".to_string()),
			_ => Ok("<OtherType>".to_string()),
		}
	}

	dump_value(lua, value, 0)
}
// endregion: --- Rust Lua Support

// region: --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::{assert_contains, eval_lua, setup_lua};
	use crate::script::aip_modules::aip_lua;

	#[tokio::test]
	async fn test_lua_dump_ok() -> Result<()> {
		// -- Setup & Fixtures
		let lua = setup_lua(aip_lua::init_module, "lua").await?;
		let script = r#"
local tbl = {
  nested = { key1 = "value1", key2 = 42 },
  bool = true,
  num = 3.21
}
return aip.lua.dump(tbl)
	    "#;

		// -- Exec
		let res = eval_lua(&lua, script)?;
		let res = res.as_str().ok_or("res json value should be of type string")?;

		// -- Check
		assert_contains(res, "bool = {true}");
		assert_contains(res, "key1 = {\"value1\"}");
		assert_contains(res, "key2 = {42}");
		Ok(())
	}
}
// endregion: --- Tests
