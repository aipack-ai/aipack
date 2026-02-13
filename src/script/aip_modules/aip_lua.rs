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
use mlua::{Lua, Table, Value};

pub fn init_module(lua: &Lua, _runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	let dump_lua = lua.create_function(dump)?;
	table.set("dump", dump_lua)?;

	let merge_lua = lua.create_function(merge)?;
	table.set("merge", merge_lua)?;

	let merge_deep_lua = lua.create_function(merge_deep)?;
	table.set("merge_deep", merge_deep_lua)?;

	Ok(table)
}

// region: --- Rust Lua Support

/// ## Lua Documentation
///
/// Shallow merge one or more tables into `target` (in place) and return target as well
///
/// ```lua
/// -- API Signature
/// aip.lua.merge(target: table, ...objs: table | nil): table
/// ```
///
/// IMPORTANT: `target` must be a table (cannot be `nil` or `null`). `objs` can be table, `nil` or `null`.
fn merge(_lua: &Lua, (target, objs): (Table, mlua::Variadic<Value>)) -> mlua::Result<Value> {
	for obj in objs {
		if let Some(obj) = extract_table(obj)? {
			for pair in obj.pairs::<Value, Value>() {
				let (k, v) = pair?;
				target.set(k, v)?;
			}
		}
	}
	Ok(Value::Table(target))
}

/// ## Lua Documentation
///
/// Deep merge one or more tables into `target` (in place) and return target as well.
///
/// ```lua
/// -- API Signature
/// aip.lua.merge_deep(target: table, ...objs: table | nil): table
/// ```
///
/// IMPORTANT: `target` must be a table (cannot be `nil` or `null`). `objs` can be table, `nil` or `null`.
fn merge_deep(_lua: &Lua, (target, objs): (Table, mlua::Variadic<Value>)) -> mlua::Result<Value> {
	fn deep_merge_into(target: &Table, overlay: &Table) -> mlua::Result<()> {
		for pair in overlay.pairs::<Value, Value>() {
			let (k, v_overlay) = pair?;
			let v_target = target.get::<Value>(k.clone())?;

			match (&v_target, &v_overlay) {
				(Value::Table(t_target), Value::Table(t_overlay)) => {
					// Both are tables, recursively merge
					deep_merge_into(t_target, t_overlay)?;
				}
				_ => {
					// Set the overlay value
					target.set(k, v_overlay)?;
				}
			}
		}
		Ok(())
	}

	for obj in objs {
		if let Some(obj) = extract_table(obj)? {
			deep_merge_into(&target, &obj)?
		}
	}
	Ok(Value::Table(target))
}

/// Extract the value if table, if nil/null return None, otherwise error
fn extract_table(obj: Value) -> Result<Option<Table>> {
	if obj == Value::NULL {
		Ok(None)
	} else {
		match obj {
			Value::Nil => Ok(None),
			Value::Table(table) => Ok(Some(table)),
			other => Err(format!("Cannot merge a non table type. Actual type: {}", other.type_name()).into()),
		}
	}
}

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
				if entries.is_empty() {
					Ok("{}".to_string())
				} else {
					let inner = entries.join(",\n");
					Ok(format!("{{\n{inner}\n{indent_str}}}"))
				}
			}
			Value::Function(f) => {
				let name = f.info().name.unwrap_or("<anonymous>".to_string());
				Ok(format!("<function {name}>"))
			}
			Value::UserData(_ud) => Ok("<UserData>".into()),
			Value::LightUserData(ld) => {
				if Value::LightUserData(ld) == Value::NULL {
					Ok("NULL".to_string())
				} else {
					Ok("<LightUserData>".to_string())
				}
			}
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
	use value_ext::JsonValueExt as _;

	#[tokio::test]
	async fn test_lua_lua_dump_ok() -> Result<()> {
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
		assert_contains(res, "bool = true");
		assert_contains(res, "key1 = \"value1\"");
		assert_contains(res, "key2 = 42");
		Ok(())
	}

	#[tokio::test]
	async fn test_lua_lua_merge_simple_ok() -> Result<()> {
		// -- Setup
		let lua = setup_lua(aip_lua::init_module, "lua").await?;

		// -- Test shallow merge in place
		let script = r#"
        local base = { a = 1, b = 2 }
        local ovl = { b = 3, c = 4 }
        aip.lua.merge(base, ovl)
        return base
    "#;
		let res = eval_lua(&lua, script)?;
		assert_eq!(res.x_get::<i32>("a")?, 1);
		assert_eq!(res.x_get::<i32>("b")?, 3);
		assert_eq!(res.x_get::<i32>("c")?, 4);

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_lua_merge_multi_ok() -> Result<()> {
		// -- Setup
		let lua = setup_lua(aip_lua::init_module, "lua").await?;

		// -- Test shallow merge with multiple tables
		let script = r#"
        local base = { a = 1 }
        local ovl1 = { b = 2 }
        local ovl2 = { c = 3, a = 10 }
        aip.lua.merge(base, ovl1, ovl2)
        return base
    "#;
		let res = eval_lua(&lua, script)?;
		assert_eq!(res.x_get::<i32>("a")?, 10);
		assert_eq!(res.x_get::<i32>("b")?, 2);
		assert_eq!(res.x_get::<i32>("c")?, 3);

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_lua_merge_deep_ok() -> Result<()> {
		// -- Setup
		let lua = setup_lua(aip_lua::init_module, "lua").await?;

		// -- Test deep merge in place
		let script = r#"
        local base = { a = 1, b = { x = 10, y = 20 } }
        local ovl = { b = { y = 22, z = 30 }, c = 3 }
        aip.lua.merge_deep(base, ovl)
        return base
    "#;
		let res = eval_lua(&lua, script)?;
		assert_eq!(res.x_get::<i32>("a")?, 1);
		assert_eq!(res.x_get::<i32>("c")?, 3);

		assert_eq!(res.x_get::<i32>("/b/x")?, 10);
		assert_eq!(res.x_get::<i32>("/b/y")?, 22);
		assert_eq!(res.x_get::<i32>("/b/z")?, 30);

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_lua_merge_deep_multi_ok() -> Result<()> {
		// -- Setup
		let lua = setup_lua(aip_lua::init_module, "lua").await?;

		// -- Test deep merge with multiple tables
		let script = r#"
        local base = { a = 1, b = { x = 10 } }
        local ovl1 = { b = { y = 20 } }
        local ovl2 = { b = { z = 30 }, c = 5 }
        aip.lua.merge_deep(base, ovl1, ovl2)
        return base
    "#;
		let res = eval_lua(&lua, script)?;
		assert_eq!(res.x_get::<i32>("a")?, 1);
		assert_eq!(res.x_get::<i32>("c")?, 5);

		assert_eq!(res.x_get::<i32>("/b/x")?, 10);
		assert_eq!(res.x_get::<i32>("/b/y")?, 20);
		assert_eq!(res.x_get::<i32>("/b/z")?, 30);

		Ok(())
	}
}
// endregion: --- Tests
