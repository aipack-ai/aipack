use crate::{Error, Result};
use mlua::{Lua, LuaSerdeExt as _};

// region:    --- Json To Lua

/// Convert a json value to a lua value.
///
/// IMPORTANT: Use this to convert JSON Value to Lua Value, as the default mlua to_value,
///            converts serde_json::Value::Null to Lua user data, and not mlua::Value::Nil,
///            and we want it for aipack.
pub fn serde_value_to_lua_value(lua: &Lua, val: serde_json::Value) -> Result<mlua::Value> {
	let res = match val {
		serde_json::Value::Null => mlua::Value::Nil,
		other => lua.to_value(&other)?,
	};
	Ok(res)
}

/// Convert a vec of serde_json Value into a vec of Lua Value using serde_value_to_lua_value logic.
pub fn serde_values_to_lua_values(lua: &Lua, values: Vec<serde_json::Value>) -> Result<Vec<mlua::Value>> {
	values.into_iter().map(|val| serde_value_to_lua_value(lua, val)).collect()
}

// endregion: --- Json To Lua

// region:    --- Lua To Json

/// Convenient function to take lua value to serde value
///
/// NOTE: The app should use this one rather to call serde_json::to_value directly
///       This way we can normalize the behavior and error and such.
///
/// Custom logic:
/// - Maps the aipack NA (UserData NASentinel) to serde_json::Value::Null.
/// - Converts Lua tables either as arrays (when contiguous 1..n integer keys without gaps) or objects (stringified keys).
pub fn lua_value_to_serde_value(lua_value: mlua::Value) -> Result<serde_json::Value> {
	use mlua::Value;

	fn number_from_f64(v: f64) -> Result<serde_json::Number> {
		serde_json::Number::from_f64(v)
			.ok_or_else(|| Error::custom("Cannot convert non-finite Lua number to JSON (NaN or Infinity)"))
	}

	fn convert_table(table: mlua::Table) -> Result<serde_json::Value> {
		// Try to treat as an array (1..n contiguous integer keys, no gaps)
		// First pass: determine max index and ensure all keys are numeric positive
		let mut max_idx: usize = 0;
		let mut numeric_only = true;

		for pair in table.clone().pairs::<mlua::Value, mlua::Value>() {
			let (k, _) = pair?;
			match k {
				Value::Integer(i) if i > 0 => {
					let i = i as usize;
					if i > max_idx {
						max_idx = i;
					}
				}
				Value::Number(n) if n.is_finite() && n.fract() == 0.0 && n > 0.0 => {
					let i = n as usize;
					if i > max_idx {
						max_idx = i;
					}
				}
				_ => {
					numeric_only = false;
					break;
				}
			}
		}

		if numeric_only {
			let mut vec: Vec<Option<serde_json::Value>> = vec![None; max_idx];
			for pair in table.clone().pairs::<mlua::Value, mlua::Value>() {
				let (k, v) = pair?;
				let idx_opt = match k {
					Value::Integer(i) if i > 0 => Some(i as usize),
					Value::Number(n) if n.is_finite() && n.fract() == 0.0 && n > 0.0 => Some(n as usize),
					_ => None,
				};
				if let Some(idx) = idx_opt {
					if idx == 0 || idx > max_idx {
						numeric_only = false;
						break;
					}
					vec[idx - 1] = Some(lua_value_to_serde_value(v)?);
				} else {
					numeric_only = false;
					break;
				}
			}

			if numeric_only && vec.iter().all(|o| o.is_some()) {
				let arr = vec.into_iter().flatten().collect();
				return Ok(serde_json::Value::Array(arr));
			}
		}

		// Fallback: treat as object with stringified keys
		let mut map = serde_json::Map::new();
		for pair in table.pairs::<mlua::Value, mlua::Value>() {
			let (k, v) = pair?;

			let key = match k {
				Value::String(s) => s.to_str()?.to_string(),
				Value::Integer(i) => i.to_string(),
				Value::Number(n) => n.to_string(),
				Value::Boolean(b) => b.to_string(),
				// Do not attempt to stringify complex keys like tables/functions/userdata
				other => {
					return Err(Error::custom(format!(
						"Unsupported Lua table key type '{}' for JSON object",
						other.type_name()
					)));
				}
			};
			map.insert(key, lua_value_to_serde_value(v)?);
		}
		Ok(serde_json::Value::Object(map))
	}

	let res = match lua_value {
		Value::Nil => serde_json::Value::Null,
		Value::Boolean(b) => serde_json::Value::Bool(b),
		Value::Integer(i) => serde_json::Value::Number(serde_json::Number::from(i)),
		Value::Number(n) => serde_json::Value::Number(number_from_f64(n)?),
		Value::String(s) => serde_json::Value::String(s.to_str()?.to_string()),
		Value::Table(t) => convert_table(t)?,
		Value::LightUserData(ldata) => {
			if Value::LightUserData(ldata) == Value::NULL {
				serde_json::Value::Null
			} else {
				// for now, still null
				serde_json::Value::Null
			}
		}

		Value::Function(_) | Value::Thread(_) | Value::UserData(_) => {
			return Err(Error::custom(
				"Cannot serialize Lua value to JSON: unsupported type (Function/LigthUserData/UserData)",
			));
		}
		Value::Error(_) => {
			return Err(Error::custom(
				"Cannot serialize Lua value to JSON: unsupported type (error)",
			));
		}
		Value::Other(_) => {
			return Err(Error::custom(
				"Cannot serialize Lua value to JSON: unsupported type (other)",
			));
		}
	};

	Ok(res)
}

/// Convert a lua Value that should be a Table/List to a Vec of serde values
pub fn lua_value_list_to_serde_values(lua_value: mlua::Value) -> Result<Vec<serde_json::Value>> {
	let list = match lua_value {
		mlua::Value::Table(table) => table,
		other => {
			return Err(Error::custom(format!(
				"Lua Value is not a List. Expected a Lua table (list) as the second argument, but got {}",
				other.type_name()
			)));
		}
	};

	let iter = list.sequence_values::<mlua::Value>();

	let json_values: Vec<serde_json::Value> = iter
		.into_iter()
		.map(|val| lua_value_to_serde_value(val?))
		.collect::<Result<Vec<_>>>()
		.map_err(|e| format!("A mlua Value cannot be serialize to Json.\nCause: {e}"))?;

	Ok(json_values)
}

// endregion: --- Lua To Json
