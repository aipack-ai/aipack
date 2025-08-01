use crate::dir_context::PathResolver;
use crate::runtime::Runtime;
use crate::support::W;
use crate::{Error, Result};
use mlua::{IntoLua, Lua, Value};

/// Process correctly the lua eval result
/// (Used by the lua engine eval, and test)
pub fn process_lua_eval_result(_lua: &Lua, res: mlua::Result<Value>, script: &str) -> Result<Value> {
	let res = match res {
		Ok(res) => res,
		Err(err) => return Err(Error::from_error_with_script(&err, script)),
	};

	let res = match res {
		// This is when we d with pcall(...), see test_lua_json_parse_invalid
		Value::Error(err) => {
			return Err(Error::from_error_with_script(&err, script));
			// return Err(Error::from(&*err));
		}
		res => res,
	};

	Ok(res)
}

// region:    --- mlua::Value utils

// Return a Vec<String> from a lua Value which can be String or Array of strings
pub fn into_vec_of_strings(value: Value, err_prefix: &'static str) -> mlua::Result<Vec<String>> {
	match value {
		// If the value is a string, return a Vec with that single string.
		Value::String(lua_string) => {
			let string_value = lua_string.to_str()?.to_string();
			Ok(vec![string_value])
		}

		// If the value is a table, try to interpret it as a list of strings.
		Value::Table(lua_table) => {
			let mut result = Vec::new();

			// Iterate over the table to collect strings.
			for pair in lua_table.sequence_values::<String>() {
				match pair {
					Ok(s) => result.push(s),
					Err(_) => {
						return Err(mlua::Error::FromLuaConversionError {
							from: "table",
							to: "Vec<String>".to_string(),
							message: Some(format!("{err_prefix} - Table contains non-string values")),
						});
					}
				}
			}

			Ok(result)
		}

		// Otherwise, return an error because the value is neither a string nor a list.
		_ => Err(mlua::Error::FromLuaConversionError {
			from: "unknown",
			to: "Vec<String>".to_string(),
			message: Some(format!("{err_prefix} - Expected a string or a list of strings")),
		}),
	}
}

pub fn into_option_string(value: mlua::Value, err_prefix: &str) -> mlua::Result<Option<String>> {
	match value {
		Value::Nil => Ok(None),
		Value::String(string) => Ok(Some(string.to_string_lossy())),
		other => Err(crate::Error::Custom(format!(
			"{err_prefix} - accepted argument types are String or Nil, but was {type_name}",
			type_name = other.type_name()
		))
		.into()),
	}
}

/// Pragmatic way to get a string property from an option lua value
/// TODO: To refactor/clean later
pub fn get_value_prop_as_string(
	value: Option<&mlua::Value>,
	prop_name: &str,
	err_prefix: &str,
) -> mlua::Result<Option<String>> {
	let Some(value) = value else { return Ok(None) };

	let table = value.as_table().ok_or_else(|| {
		crate::Error::custom(format!(
			"{err_prefix} - value should be of type lua table, but was of another type."
		))
	})?;

	match table.get::<Option<Value>>(prop_name)? {
		Some(Value::String(string)) => {
			// TODO: probaby need to normalize_dir to remove the eventual end "/"
			Ok(Some(string.to_string_lossy()))
		}
		Some(_other) => Err(crate::Error::custom(format!(
			"{err_prefix} options.base_dir must be of type string is present"
		))
		.into()),
		None => Ok(None),
	}
}

impl IntoLua for W<&String> {
	fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<Value> {
		Ok(Value::String(lua.create_string(self.0)?))
	}
}

impl IntoLua for W<String> {
	fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<Value> {
		Ok(Value::String(lua.create_string(&self.0)?))
	}
}

// endregion: --- mlua::Value utils

// region:    --- Common Paths Support

pub fn path_exits(runtime: &Runtime, path: &str) -> bool {
	let dir_context = runtime.dir_context();
	// Resolve the path relative to the workspace directory
	let full_path = dir_context
		.resolve_path(runtime.session(), path.into(), PathResolver::WksDir, None)
		.ok();

	full_path.map(|p| p.exists()).unwrap_or(false)
}

// endregion: --- Common Paths Support
