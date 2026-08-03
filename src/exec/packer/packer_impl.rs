//! Module that pack the files into their .aipack

use crate::exec::packer::PackToml;
use crate::exec::packer::pack_toml::parse_validate_pack_toml;
use crate::support::files::list_options_with_default_excludes;
use crate::support::zip;
use crate::{Error, Result};
use simple_fs::SPath;
use std::fs;

/// Data returned when packing a directory
#[derive(Debug)]
pub struct PackDirData {
	pub pack_file: SPath,
	#[allow(unused)]
	pub pack_toml: PackToml,
}

/// Packs a directory into a .aipack file
///
/// # Parameters
/// - `pack_dir`: The directory containing the content to be packed
/// - `dest_dir`: The directory where the .aipack file will be created
///
/// # Returns
/// - Ok(PackDirData): If packing is successful, containing the path to the created .aipack file and pack.toml data
/// - Err(Error): If any error occurs during packing
pub fn pack_dir(pack_dir: impl AsRef<SPath>, dest_dir: impl AsRef<SPath>) -> Result<PackDirData> {
	let pack_dir = pack_dir.as_ref();
	let dest_dir = dest_dir.as_ref();

	// Verify if pack.toml exists
	let toml_path = pack_dir.join("pack.toml");
	if !toml_path.exists() {
		return Err(Error::AipackTomlMissing(toml_path.into()));
	}

	// Read and validate the TOML file
	let toml_content = fs::read_to_string(&toml_path)?;
	let pack_toml = parse_validate_pack_toml(&toml_content, toml_path.as_str())?;

	// Normalize version - replace special characters with hyphens
	let pack_version = &pack_toml.version;

	// Create the output filename
	let aipack_filename = format!("{}@{}-v{pack_version}.aipack", pack_toml.namespace, pack_toml.name);
	let aipack_path = dest_dir.join(aipack_filename);

	// Create the destination directory if it doesn't exist
	if !dest_dir.exists() {
		fs::create_dir_all(dest_dir)?;
	}

	let mut exclude_globs = Vec::new();
	let options = list_options_with_default_excludes(&[], &mut exclude_globs)?;
	let pack_files = simple_fs::list_files(pack_dir, None, Some(options))?;

	// Zip the directory
	zip::zip_files(pack_dir, &aipack_path, pack_files)?;

	Ok(PackDirData {
		pack_file: aipack_path,
		pack_toml,
	})
}

// region:    --- Tests

#[cfg(test)]
#[path = "../../_tests/tests_packer_impl.rs"]
mod tests;

// endregion: --- Tests
