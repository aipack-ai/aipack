use crate::dir_context::DirContext;
use mlua::{IntoLua, Lua};
use serde::Serialize;
use simple_fs::SPath;

/// The FileInfo object contains the metadata of a file but not its content.
/// The ctime, mtime, and size metadata are generally loaded,
/// but this can be turned off when listing files using the `with_meta = false` option.
#[derive(Debug, Serialize)]
pub struct FileInfo {
	path: String,
	/// The dir/parent path of this file from path (will be empty if no parent of the rel path)
	dir: String,
	name: String,
	stem: String,
	ext: String,

	ctime: Option<i64>, // seconds since epoch, or nil in Lua
	mtime: Option<i64>,
	size: Option<i64>, // size in bytes
}

pub struct WithMeta<'a> {
	full_path: Option<&'a SPath>,
	with_meta: bool,
}
impl From<bool> for WithMeta<'_> {
	fn from(with_meta: bool) -> Self {
		WithMeta {
			full_path: None,
			with_meta,
		}
	}
}
impl<'a> From<&'a SPath> for WithMeta<'a> {
	fn from(full_path: &'a SPath) -> Self {
		WithMeta {
			full_path: Some(full_path),
			with_meta: true,
		}
	}
}

impl FileInfo {
	/// - with_meta: when true, will attempt to get the file meta. Will ignore if error
	/// - with_meta if SPath, then, it's true, and the SPath is the absolute path
	/// - `base_path` is only use with_meta true to attempt to get the meta
	pub fn new<'a>(dir_context: &DirContext, rel_path: impl Into<SPath>, with_meta: impl Into<WithMeta<'a>>) -> Self {
		let path: SPath = rel_path.into();
		// make it tild home
		let path = dir_context.maybe_home_path_into_tilde(path);

		let with_meta: WithMeta = with_meta.into();
		if with_meta.with_meta {
			// Here we preserve the ~ format if there is one
			let mut res = FileInfo::from_path(path.clone());

			// Here to resolve the ~ for the full_path.meta
			let full_path = match with_meta.full_path {
				Some(full_path) => full_path.clone(),
				None => dir_context.maybe_tilde_path_into_home(path),
			};

			if let Ok(meta) = full_path.meta() {
				res.ctime = Some(meta.created_epoch_us);
				res.mtime = Some(meta.modified_epoch_us);
				res.size = Some(meta.size as i64);
			}
			res
		} else {
			FileInfo::from_path(path)
		}
	}

	/// Internal from spath (note: do not make public)
	fn from_path(file: SPath) -> Self {
		let dir = file.parent().map(|p| p.to_string()).unwrap_or_default();
		FileInfo {
			path: file.to_string(),
			name: file.name().to_string(),
			dir,
			stem: file.stem().to_string(),
			ext: file.ext().to_string(),
			// -- when created _with_meta
			ctime: None,
			mtime: None,
			size: None,
		}
	}
}

// region:    --- Lua

impl IntoLua for FileInfo {
	fn into_lua(self, lua: &Lua) -> mlua::Result<mlua::Value> {
		let table = lua.create_table()?;
		table.set("path", self.path)?;
		table.set("dir", self.dir)?;
		table.set("name", self.name)?;
		table.set("stem", self.stem)?;
		table.set("ext", self.ext)?;
		if let Some(ctime) = self.ctime {
			table.set("ctime", ctime)?;
		}
		if let Some(mtime) = self.mtime {
			table.set("mtime", mtime)?;
		}
		if let Some(size) = self.size {
			table.set("size", size)?;
		}
		Ok(mlua::Value::Table(table))
	}
}

// endregion: --- Lua
