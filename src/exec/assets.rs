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

pub fn extract_template_zfile(path: &str) -> Result<ZFile> {
	extract_zfile("_template", path)
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
pub fn list_file_paths_start_with(pre_path: &str, prefix: &str) -> Result<Vec<String>> {
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

/// Update all of the files in a dest_dir base on the pre_path (workspace or base)
pub async fn update_files(pre_path: &str, dest_dir: &SPath, file_paths: &[&str], force_update: bool) -> Result<()> {
	let existing_files = list_files(dest_dir, Some(&["**/*.aip", "**/*.lua", "**/*.md"]), None)?;

	let existing_names: HashSet<String> = existing_files
		.iter()
		.filter_map(|f| f.try_diff(dest_dir).ok().map(|p| p.to_string()))
		.collect();

	let hub = get_hub();

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
			hub.publish(format!("-> {:<18} '{}'", "Create file", dest_path.try_diff(dest_dir)?))
				.await;
		}
	}

	Ok(())
}

fn extract_asset_content(path: &str) -> Result<Vec<u8>> {
	let mut archive = new_asset_archive_reader()?;

	let mut file = archive
		.by_name(path)
		.map_err(|err| Error::custom(format!("Fail to extract assets from zip '{path}'.\nCause: {err} ")))?;

	let mut data: Vec<u8> = Vec::new();

	file.read_to_end(&mut data)?;

	Ok(data)
}

fn new_asset_archive_reader() -> Result<ZipArchive<Cursor<&'static [u8]>>> {
	let reader = Cursor::new(ASSETS_ZIP);

	let archive = ZipArchive::new(reader)
		.map_err(|err| Error::custom(format!("Cannot create zip archive reader.\nCause: {err}")))?;

	Ok(archive)
}

// region:    --- Hashing

pub fn compute_assets_hash(pre_path: &str, prefix: &str) -> Result<blake3::Hash> {
	let mut archive = new_asset_archive_reader()?;
	let mut entries = Vec::new();

	for i in 0..archive.len() {
		let file = archive
			.by_index(i)
			.map_err(|err| Error::custom(format!("Fail to read zip entry. Cause: {err}")))?;
		let path = file.name();
		if !path.ends_with('/') && path.starts_with(pre_path) {
			let Some(path_sub) = path.strip_prefix(pre_path) else {
				continue;
			};
			let path_sub = path_sub.strip_prefix("/").unwrap_or(path_sub);
			if path_sub.starts_with(prefix) {
				entries.push(path.to_string());
			}
		}
	}
	entries.sort();

	let base_prefix = format!("{pre_path}/");
	let mut hasher = blake3::Hasher::new();
	for path in entries {
		let content = extract_asset_content(&path)?;

		// This way, we do not fail, but the hasher won't probably match, so ok
		let Some(rel_path) = path
			.strip_prefix(&base_prefix)
			.and_then(|s| s.strip_prefix(prefix))
			.and_then(|s| s.strip_prefix("/"))
		else {
			continue;
		};

		hasher.update(rel_path.as_bytes());
		hasher.update(&content);
	}

	Ok(hasher.finalize())
}

// endregion: --- Hashing
