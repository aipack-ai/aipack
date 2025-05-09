use crate::Result;
use crate::dir_context::path_consts::{AIPACK_DIR_NAME, CONFIG_FILE_NAME, PACK_CUSTOM};
use simple_fs::SPath;
use std::ops::Deref;

/// AipackWksDir is the typed wrapper of the absolute path to the workspace `.aipack` directory.
#[derive(Debug, Clone)]
pub struct AipackWksDir {
	path: SPath,
}

impl AipackWksDir {
	/// Builds the absolute path for the `.aipack/` directory relative to a workspace directory.
	///
	/// # Arguments
	///
	/// * `wks_dir` - The absolute, canonicalized path to the workspace directory.
	///
	/// # Returns
	///
	/// * `Result<Self>` - The `AipackWksDir` instance.
	///
	/// NOTE: This does not test if the `.aipack` directory exists.
	pub fn new_from_wks_dir(wks_dir: &SPath) -> Result<Self> {
		// Assume wks_dir is already absolute and canonicalized.
		let aipack_wks_path = wks_dir.join(AIPACK_DIR_NAME);
		Ok(Self { path: aipack_wks_path })
	}

	pub fn path(&self) -> &SPath {
		&self.path
	}

	// region:    --- Path Getters
	pub fn get_config_toml_path(&self) -> Result<SPath> {
		let path = self.join(CONFIG_FILE_NAME);
		Ok(path)
	}

	pub fn get_pack_custom_dir(&self) -> Result<SPath> {
		let dir = self.join(PACK_CUSTOM);
		Ok(dir)
	}
	// endregion: --- Path Getters
}

/// Path-through methods to SPath
impl AipackWksDir {
	pub fn exists(&self) -> bool {
		self.path.exists()
	}
	pub fn join(&self, leaf_path: impl Into<SPath>) -> SPath {
		self.path.join(leaf_path.into())
	}
}

impl AsRef<SPath> for AipackWksDir {
	fn as_ref(&self) -> &SPath {
		&self.path
	}
}

impl Deref for AipackWksDir {
	type Target = SPath;

	fn deref(&self) -> &Self::Target {
		&self.path
	}
}

#[cfg(test)]
impl AipackWksDir {
	/// Creates an AipackWksDir directly from a path. For testing purposes.
	pub fn new_for_test(path: impl Into<SPath>) -> Result<Self> {
		let path = path.into();
		// In tests, we might not need strict canonicalization checks.
		Ok(Self { path })
	}
}
