// region:    --- RunCommandResponse

use crate::script::{serde_value_to_lua_value, serde_values_to_lua_values};
use mlua::IntoLua;
use serde::Serialize;
use serde_json::Value;

/// The response returned by a Run Command call.
/// TODO: Need to check why `outputs` is optional.
///       We might want to have an array of Null if no output or nil was returned (to keep in sync with inputs).
#[derive(Debug, Serialize, Default)]
pub struct RunAgentResponse {
	pub outputs: Option<Vec<Value>>,
	pub after_all: Option<Value>,
	pub redo_requested: bool,
}

impl IntoLua for RunAgentResponse {
	fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
		let table = lua.create_table()?;
		let outputs = self.outputs.map(|v| serde_values_to_lua_values(lua, v)).transpose()?;
		let after_all = self.after_all.map(|v| serde_value_to_lua_value(lua, v)).transpose()?;
		table.set("outputs", outputs)?;
		table.set("after_all", after_all)?;
		table.set("redo_requested", self.redo_requested)?;
		Ok(mlua::Value::Table(table))
	}
}

// endregion: --- RunCommandResponse
