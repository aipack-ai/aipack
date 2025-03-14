use crate::{Error, Result};
use mlua::{IntoLua, Lua};
use serde::Serialize;
use simple_fs::SPath;
use std::fs::read_to_string;

/// FileRecord contains the metadata information about the file (name, ext, etc.) as well as the content.
#[derive(Serialize)]
pub struct FileRecord {
	/// The path, might and will probably be relative
	pub path: String,
	/// The dir/parent path of this file from path (will be empty if no parent of the rel path)
	pub dir: String,
	/// The name of the file with extension e.g., `main.rs`
	pub name: String,
	/// Stem
	pub stem: String,
	/// Empty if there is no extension
	pub ext: String,
	/// The full text content of the file
	pub content: String,
}

/// Constructors
impl FileRecord {
	pub fn load(base_path: &SPath, rel_path: &SPath) -> Result<Self> {
		let full_path = if rel_path.path().is_absolute() {
			rel_path
		} else {
			&base_path.join(rel_path)
		};

		let content = read_to_string(full_path).map_err(|err| Error::cc(format!("Fail to read {full_path}"), err))?;
		let dir = rel_path.parent().map(|p| p.to_string()).unwrap_or_default();

		Ok(FileRecord {
			path: rel_path.to_string(),
			dir,
			name: rel_path.name().to_string(),
			stem: rel_path.stem().to_string(),
			ext: rel_path.ext().to_string(),
			content,
		})
	}
}

// region:    --- Lua

impl IntoLua for FileRecord {
	fn into_lua(self, lua: &Lua) -> mlua::Result<mlua::Value> {
		let table = lua.create_table()?;
		table.set("path", self.path)?;
		table.set("dir", self.dir)?;
		table.set("name", self.name)?;
		table.set("stem", self.stem)?;
		table.set("ext", self.ext)?;
		table.set("content", self.content)?;
		Ok(mlua::Value::Table(table))
	}
}

// endregion: --- Lua
