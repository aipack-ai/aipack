use crate::{Error, Result};
use simple_fs::{ListOptions, SPath};
use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use walkdir::WalkDir;

// Those folders need to be explicitly include in the include globs or they will be ignored with `**..**` glob (e.g. `**target/**`)
pub const SPECIAL_DEFAULT_FOLDER_EXCLUDES: &[&str] = &[
	//
	".aipack/", // otherwise, when `**/*` it get included and can confuse
	".git/",
	"target/",
	"node_modules/",
	".build/",
	"__pycache__/",
];

const SPECIAL_DEFAULT_FOLDER_EXCLUDE_GLOBS: &[&str] = &[
	"**/.aipack/**",
	"**/.git/**",
	"**/target/**",
	"**/node_modules/**",
	"**/.build/**",
	"**/__pycache__/**",
];

pub const GLOBS_TO_ALWAYS_EXLUDES: &[&str] = &["**/.DS_Store", ".DS_Store", "**/Thumbs.db", "**/*.swp"];

/// Builds list options with the default file-list exclusions.
///
/// Explicitly matching a special folder in an include glob removes that folder
/// from the default exclusion set.
pub fn list_options_with_default_excludes<'a>(
	include_globs: &[&str],
	exclude_globs: &'a mut Vec<&'a str>,
) -> Result<ListOptions<'a>> {
	// we start with the full set of special exclude folders
	// (then if included in the include globs, they will be removed from the exclude set)
	let mut special_folder_excludes: HashSet<&'static str> = SPECIAL_DEFAULT_FOLDER_EXCLUDES.iter().copied().collect();

	// validate globs, and refine excludes
	// (cheap check for now. Should probably be in simple-fs)
	for glob in include_globs {
		// this normalize the path with `/`
		let glob = SPath::new(glob);
		let glob = glob.as_str().trim();

		// -- Update the exclude
		let excludes_tmp: Vec<&'static str> = special_folder_excludes.iter().copied().collect();
		for exc in excludes_tmp {
			if glob.contains(exc) && special_folder_excludes.contains(exc) {
				special_folder_excludes.remove(exc);
			}
		}

		// -- Validate that it does not start wiht ../ or ./.. (not supported for now)
		// NOTE: This not a complete check, but at least will warn the user for common mistake
		if glob.starts_with("../") || glob.starts_with("./..") {
			return Err(Error::custom(format!(
				"Glob '{glob}' starting with '../'.\nStarting glob with '../' is not supported at the moment."
			)));
		}
	}

	exclude_globs.clear();
	if !special_folder_excludes.is_empty() {
		for (folder, glob) in SPECIAL_DEFAULT_FOLDER_EXCLUDES
			.iter()
			.zip(SPECIAL_DEFAULT_FOLDER_EXCLUDE_GLOBS.iter())
		{
			if special_folder_excludes.contains(*folder) {
				exclude_globs.push(*glob);
			}
		}
	}
	exclude_globs.extend_from_slice(GLOBS_TO_ALWAYS_EXLUDES);

	let mut options = ListOptions::from_relative_glob(true);
	if !exclude_globs.is_empty() {
		options = options.with_exclude_globs(&*exclude_globs);
	}

	Ok(options)
}

/// Returns the current dir
pub fn current_dir() -> Result<SPath> {
	let dir = env::current_dir().map_err(|err| Error::cc("Current dir error", err))?;
	let dir = SPath::from_std_path_buf(dir)?;
	Ok(dir)
}

/// Returns the AIPACK home directory, or the root path if no home directory is found.
/// This strategy allows AIPACK to run on systems that do not have a home directory,
/// avoiding downstream complexity.
pub fn home_dir() -> SPath {
	let home_dir = simple_fs::home_dir().ok();
	let home_dir: SPath = home_dir.unwrap_or_else(|| SPath::new("/"));
	home_dir
}

/// Lists directories under the base_dir up to the specified depth.
///
/// # Parameters
/// - `base_dir`: The base directory to start listing from
/// - `depth`: Maximum directory depth to traverse (0 means just the base_dir)
/// - `only_leaf`: If true, only returns directories exactly at the specified depth.
///   (Callers can pass true to use the default behavior.)
///
/// # Returns
/// A vector of PathBuf for the directories that match the criteria
pub fn list_dirs(base_dir: impl AsRef<Path>, depth: usize, only_leaf: bool) -> Vec<SPath> {
	let base_path = base_dir.as_ref();
	let base_depth = base_path.components().count();

	let mut dirs = Vec::new();

	for entry in WalkDir::new(base_path)
		.min_depth(if only_leaf { depth } else { 1 })
		.max_depth(depth)
		.follow_links(true) // Add this to follow symlinks when walking
		.into_iter()
		.filter_entry(|e| {
			// Include both actual directories and symlinks to directories
			e.file_type().is_dir() || (e.path().is_dir() && e.file_type().is_symlink())
		}) {
		let entry = entry.expect("Error walking directory");
		// Now check if the path is a directory (will be true for symlinks to directories)
		if entry.path().is_dir() {
			// Skip the base directory itself if only_leaf is true and depth is 0
			if only_leaf && depth == 0 && entry.path() == base_path {
				continue;
			}

			// Calculate current depth relative to base_path
			let current_depth = entry.path().components().count() - base_depth;

			// If only_leaf is true, we only want directories exactly at the specified depth.
			// Otherwise, include all directories up to and including the max depth.
			if (!only_leaf || current_depth == depth)
				&& let Ok(spath) = SPath::from_walkdir_entry(entry)
			{
				dirs.push(spath);
			}
		}
	}

	dirs
}

/// Relatively efficient way to determine if a file is empty, meaning length == 0, or only whitespace.
pub fn is_file_empty(file_path: impl AsRef<Path>) -> Result<bool> {
	let path = file_path.as_ref();
	let file = File::open(path).map_err(|err| {
		//
		Error::cc(
			"Cannot determine if file empty",
			format!("File '{}' open error.\nCause: {err}", path.to_string_lossy()),
		)
	})?;
	let mut reader = BufReader::new(file);

	// First read with a small buffer of 64 bytes
	let mut small_buffer = [0; 64];
	let num_bytes = reader.read(&mut small_buffer)?;
	if num_bytes == 0 {
		return Ok(true);
	}
	if !is_buff_empty(&small_buffer[..num_bytes]) {
		return Ok(false);
	}
	// If we read less than the small buffer size, we've reached the end of the file.
	if num_bytes < small_buffer.len() {
		return Ok(true);
	}

	// Subsequent reads with a larger buffer of 1024 bytes
	let mut large_buffer = [0; 1024];
	loop {
		let num_bytes = reader.read(&mut large_buffer)?;
		if num_bytes == 0 {
			break;
		}
		if !is_buff_empty(&large_buffer[..num_bytes]) {
			return Ok(false);
		}
	}
	Ok(true)
}

fn is_buff_empty(buff: &[u8]) -> bool {
	let s = std::str::from_utf8(buff).unwrap_or("");
	s.chars().all(|c| c.is_whitespace())
}
// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	use super::*;

	#[test]
	fn test_support_files_list_dirs_only_leaf() -> Result<()> {
		// -- Setup & Fixtures
		// Assuming the following directory structure exists relative to the crate root:
		// src/agent/
		// src/cli/
		// src/script/support/
		// src/support/code/
		// src/support/md/
		// src/support/text/
		let base_dir = "src";
		let depth = 2;
		let only_leaf = true;

		// -- Exec
		let dirs = list_dirs(base_dir, depth, only_leaf);

		// -- Check
		// Expected directories at exactly depth 2 (relative to "src")
		let expected = vec![
			"src/script/aip_modules",
			"src/support/code",
			"src/support/md",
			"src/support/text",
		];

		for exp in expected {
			let exp_path = SPath::new(exp).canonicalize()?;
			let found = dirs
				.iter()
				.any(|d| d.canonicalize().map(|p| p.as_str() == exp_path.as_str()).unwrap_or(false));
			assert!(found, "Expected directory {exp:?} not found in the returned list");
		}

		Ok(())
	}

	#[test]
	fn test_support_files_list_dirs_all() -> Result<()> {
		// -- Setup & Fixtures
		let base_dir = "src";
		let depth = 2;
		let only_leaf = false;

		// -- Exec
		let dirs = list_dirs(base_dir, depth, only_leaf);

		// -- Check
		// For only_leaf = false, expected directories include those at depth 1 and depth 2.
		let expected = vec!["src/agent", "src/exec", "src/script", "src/support"];

		for exp in expected {
			let exp_path = SPath::new(exp).canonicalize()?;
			let found = dirs
				.iter()
				.any(|d| d.canonicalize().map(|p| p.as_str() == exp_path.as_str()).unwrap_or(false));
			assert!(found, "Expected directory {exp:?} not found in the returned list");
		}

		Ok(())
	}

	#[test]
	fn test_support_files_list_dirs_depth_zero() -> Result<()> {
		// -- Setup & Fixtures
		let base_dir = "src";
		let depth = 0;
		let only_leaf = true;

		// -- Exec
		let dirs = list_dirs(base_dir, depth, only_leaf);

		// -- Check
		// For depth = 0 with only_leaf true, the expected result is an empty vector.
		assert!(dirs.is_empty(), "Expected empty directory list for depth=0");

		Ok(())
	}
}

// endregion: --- Tests
