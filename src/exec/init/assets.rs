// Auto-generated file. Do not edit.
use crate::{Error, Result};
use std::fs::write;

pub const ASSETS_ZIP: &[u8] = include_bytes!(env!("INIT_ASSETS_ZIP"));

use crate::hub::get_hub;
use simple_fs::{SPath, ensure_dir, list_files};
use std::collections::HashSet;
use std::io::{Cursor, Read};
use zip::ZipArchive;

#[derive(Debug)]
pub struct ZFile {
	#[allow(unused)]
	pub path: String,
	pub content: Vec<u8>,
}

// region:    --- Workspace ZFiles
pub fn extract_workspace_config_toml_zfile() -> Result<ZFile> {
	extract_workspace_zfile("config.toml")
}

pub fn extract_workspace_zfile(path: &str) -> Result<ZFile> {
	extract_zfile("workspace", path)
}

#[allow(unused)]
pub fn extract_workspace_pack_file_paths() -> Result<Vec<String>> {
	list_workspace_file_paths_start_with("pack")
}

pub fn list_workspace_file_paths_start_with(prefix: &str) -> Result<Vec<String>> {
	list_file_paths_start_with("workspace", prefix)
}

// endregion: --- Workspace ZFiles

// region:    --- Template ZFiles

pub fn extract_template_pack_toml_zfile() -> Result<ZFile> {
	extract_template_zfile("pack.toml")
}

pub fn extract_template_zfile(path: &str) -> Result<ZFile> {
	extract_zfile("_template", path)
}

// endregion: --- Template ZFiles

// region:    --- Base ZFiles

pub fn extract_base_config_default_toml_zfile() -> Result<ZFile> {
	extract_base_zfile("config-default.toml")
}

pub fn extract_base_config_user_toml_zfile() -> Result<ZFile> {
	extract_base_zfile("config-user.toml")
}

pub fn extract_base_doc_file_paths() -> Result<Vec<String>> {
	list_base_file_paths_start_with("doc")
}

pub fn extract_base_pack_file_paths() -> Result<Vec<String>> {
	list_base_file_paths_start_with("pack")
}

fn extract_base_zfile(path: &str) -> Result<ZFile> {
	extract_zfile("base", path)
}

fn list_base_file_paths_start_with(prefix: &str) -> Result<Vec<String>> {
	list_file_paths_start_with("base", prefix)
}

// endregion: --- Base ZFiles

// region:    --- Setup Files

pub fn extract_setup_aip_env_sh_zfile() -> Result<ZFile> {
	extract_zfile("_setup", "aip-env")
}

// endregion: --- Setup Files

// region:    --- Support

/// Update all of the files in a dest_dir base on the pre_path (workspace or base)
pub async fn update_files(pre_path: &str, dest_dir: &SPath, file_paths: &[&str], force_update: bool) -> Result<()> {
	let existing_files = list_files(dest_dir, Some(&["**/*.aip", "**/*.lua", "**/*.md"]), None)?;

	let existing_names: HashSet<String> = existing_files
		.iter()
		.filter_map(|f| f.try_diff(dest_dir).ok().map(|p| p.to_string()))
		.collect();

	for &file_path in file_paths {
		if force_update || !existing_names.contains(file_path) {
			let dest_rel_path = SPath::from(file_path);
			let dest_path = SPath::new(dest_dir).join(dest_rel_path.as_str());
			// if the rel_path had a parent
			if let Some(parent_dir) = dest_rel_path.parent() {
				let parent_dir = dest_dir.join(parent_dir);
				ensure_dir(parent_dir)?;
			}
			let zfile = extract_zfile(pre_path, dest_rel_path.as_str())?;
			write(&dest_path, zfile.content)?;
			get_hub()
				.publish(format!("-> {:<18} '{}'", "Create file", dest_path.try_diff(dest_dir)?))
				.await;
		}
	}

	Ok(())
}

pub fn extract_zfile(pre_path: &str, path: &str) -> Result<ZFile> {
	let path = format!("{pre_path}/{path}");
	let content = extract_asset_content(&path)?;
	Ok(ZFile {
		path: path.to_string(),
		content,
	})
}

/// List the paths nder the `workspace/_prefix_` path and remove
fn list_file_paths_start_with(pre_path: &str, prefix: &str) -> Result<Vec<String>> {
	let archive = new_asset_archive_reader()?;

	let mut paths = Vec::new();

	for path in archive.file_names() {
		if !path.ends_with('/') && path.starts_with(pre_path) {
			let Some(path_sub) = path.strip_prefix(pre_path) else {
				continue;
			};
			let path_sub = path_sub.strip_prefix("/").unwrap_or(path_sub);
			if path_sub.starts_with(prefix) {
				paths.push(path_sub.to_string());
			}
		}
	}

	Ok(paths)
}

fn extract_asset_content(path: &str) -> Result<Vec<u8>> {
	let mut archive = new_asset_archive_reader()?;

	let mut file = archive
		.by_name(path)
		.map_err(|err| Error::custom(format!("Fail to extract assets from zip '{path}'. Cause: {err} ")))?;

	let mut data: Vec<u8> = Vec::new();

	file.read_to_end(&mut data)?;

	Ok(data)
}

fn new_asset_archive_reader() -> Result<ZipArchive<Cursor<&'static [u8]>>> {
	let reader = Cursor::new(ASSETS_ZIP);

	let archive = ZipArchive::new(reader)
		.map_err(|err| Error::custom(format!("Cannot create zip archive reader. Cause: {err}")))?;

	Ok(archive)
}

// endregion: --- Support
