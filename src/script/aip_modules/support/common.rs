use crate::dir_context::{PathResolver, find_to_run_pack_dir, resolve_pack_ref_base_path};
use crate::runtime::Runtime;
use crate::script::support::{get_value_prop_as_string, into_vec_of_strings};
use crate::types::{DestOptions, FileRecord, FileRef, PackRef};
use crate::{Error, Result};
use mlua::{FromLua as _, Lua, Value};
use simple_fs::SPath;
use std::borrow::Cow;
use std::path::Path;
use std::str::FromStr;

/// Check if write access is granted.
/// TODO: Need to fix that. Should check that is workspace dir, or in .aipack-base/ dir, but right now, poor check.
// Check that if three is .., it ist still in a .aipack-base
// TODO: Would probably need to check that it can only write in it's own support folder
pub fn check_access_write(full_path: &SPath, wks_dir: &SPath) -> Result<()> {
	if let Some(rel_path) = full_path.diff(wks_dir)
		&& rel_path.as_str().starts_with("..")
	{
		// allow the .aipack-base
		if !full_path.as_str().contains(".aipack-base") {
			return Err(Error::custom(format!(
				"Save file protection - The path `{rel_path}` does not belong to the workspace dir `{wks_dir}` or to the .aipack-base.\nCannot save file out of workspace or aipack base at this point"
			)));
		}
	}
	Ok(())
}

/// Check if delete access is granted.
///
/// Same logic as write, but deletion is never allowed in `.aipack-base`.
pub fn check_access_delete(full_path: &SPath, wks_dir: &SPath) -> Result<()> {
	// Never allow delete operations in .aipack-base
	if full_path.as_str().contains(".aipack-base") {
		return Err(Error::custom(
			"Delete file protection - Deleting files from the `.aipack-base` folder is not allowed".to_string(),
		));
	}

	// If outside the workspace, deny deletion
	if let Some(rel_path) = full_path.diff(wks_dir)
		&& rel_path.as_str().starts_with("..")
	{
		return Err(Error::custom(format!(
			"Delete file protection - The path `{rel_path}` does not belong to the workspace dir `{wks_dir}`.\nCannot delete files outside the workspace"
		)));
	}

	Ok(())
}

/// Extracts base directory and glob patterns from options
///
/// Returns (base_path, globs)
pub fn base_dir_and_globs(
	runtime: &Runtime,
	include_globs: Value,
	options: Option<&Value>,
) -> Result<(Option<SPath>, Vec<String>)> {
	let globs: Vec<String> = into_vec_of_strings(include_globs, "file::file_list globs argument")?;
	let base_dir = compute_base_dir(runtime, options)?.or_else(|| runtime.dir_context().wks_dir().cloned());

	// Process any pack references in the globs
	let processed_globs = process_path_references(runtime, globs)?;

	Ok((base_dir, processed_globs))
}

/// Processes a single file path to handle pack references
///
/// - If a pack ref (with ..@../..) will process as pack ref
/// - Otherwise, just return the same path
///
/// Converts pack references like "jc@rust10x/common/file.md" to their actual paths
pub fn process_path_reference(runtime: &Runtime, path: &str) -> Result<SPath> {
	let dir_context = runtime.dir_context();

	// -- Resolve the eventual `~/` with the home_dir
	let path: Cow<str> = if let Some(path_from_home) = path.strip_prefix("~/") {
		format!("{}/{path_from_home}", dir_context.home_dir()).into()
	} else {
		path.into()
	};

	// -- Process if 'ns@pack_name...`
	if let Some(pack_ref_str) = extract_pack_reference(&path) {
		// Parse the pack reference
		if let Ok(partial_pack) = PackRef::from_str(pack_ref_str) {
			// Try to find the pack directory

			// Can be the pack base dir, or the support base dir.
			if let Ok(base_dir) = resolve_pack_ref_base_path(dir_context, &partial_pack) {
				// Replace the pack reference with the actual path
				let sub_path = partial_pack.sub_path.unwrap_or_default();
				let pack_path = base_dir.join(&sub_path);

				// Get the remaining path (after the pack reference)
				let remaining_path = path.strip_prefix(pack_ref_str).unwrap_or("").trim_start_matches('/');

				// Combine the pack path with the remaining path
				let resolved_path = if remaining_path.is_empty() {
					pack_path.to_string()
				} else {
					pack_path.join(remaining_path).to_string()
				};

				return Ok(resolved_path.into());
			}
		}
	}

	// -- Look if it is $tmp
	let path = SPath::new(&path);
	// It's a `$tmp/...` path
	if dir_context.is_tmp_path(&path) {
		dir_context.resolve_tmp_path(runtime.session(), &path)
	}
	// Nothing special, return the orginal path
	else {
		Ok(path)
	}
}

/// Processes globs to handle pack references
///
/// Converts pack references like "jc@rust10x/common/**/*.md" to their actual paths
pub fn process_path_references(runtime: &Runtime, globs: Vec<String>) -> Result<Vec<String>> {
	let mut processed_globs = Vec::with_capacity(globs.len());

	for glob in globs {
		let glob = process_path_reference(runtime, &glob)?;
		processed_globs.push(glob.to_string());
	}

	Ok(processed_globs)
}

/// Extracts a potential pack reference from a glob string
///
/// Returns Some(reference) if the glob appears to contain a pack reference,
/// or None if it doesn't match the pattern
fn extract_pack_reference(glob: &str) -> Option<&str> {
	// Look for patterns like "namespace@package/path"
	// Stop at the first wildcard character if present

	// First check if there's an @ symbol (required for namespace@package)
	if !glob.contains('@') {
		return None;
	}

	// Find the position of the first wildcard character
	let wildcard_pos = glob.find(['*', '?', '[']);

	// Extract the substring up to the wildcard or the entire string if no wildcard
	let reference = match wildcard_pos {
		Some(pos) => {
			// Find the last path separator before the wildcard
			match glob[..pos].rfind('/') {
				Some(sep_pos) => &glob[..=sep_pos],
				None => glob, // No separator before wildcard, use the whole string
			}
		}
		None => glob,
	};

	Some(reference)
}

/// Determines the base directory to use for file operations
///
/// If options.base_dir is provided, it resolves relative to workspace
///
/// Otherwise return none
pub fn compute_base_dir(runtime: &Runtime, options: Option<&Value>) -> Result<Option<SPath>> {
	let dir_context = runtime.dir_context();
	// the default base_path is the workspace dir.
	let workspace_path = dir_context.wks_dir().ok_or("Workspace dir is missing")?.clone();

	// if options, try to resolve the options.base_dir
	let base_dir = get_value_prop_as_string(options, "base_dir", "aip.file... options fail")?;

	let base_dir = match base_dir {
		Some(base_dir) => {
			// Check if the base_dir is a pack reference
			if let Some(pack_ref_str) = extract_pack_reference(&base_dir) {
				if let Ok(pack_ref) = PackRef::from_str(pack_ref_str) {
					if let Ok(pack_dir) = find_to_run_pack_dir(dir_context, &pack_ref) {
						// Get the complete path by joining the pack dir with any sub path
						let sub_path = pack_ref.sub_path.unwrap_or_default();
						let remaining_path = base_dir.strip_prefix(pack_ref_str).unwrap_or("").trim_start_matches('/');

						if remaining_path.is_empty() {
							Some(pack_dir.path.join(sub_path))
						} else {
							Some(pack_dir.path.join(sub_path).join(remaining_path))
						}
					} else {
						// Fall back to regular path resolution if pack not found
						if crate::support::paths::is_relative(&base_dir) {
							Some(workspace_path.join(&base_dir))
						} else {
							Some(SPath::from(base_dir))
						}
					}
				} else {
					// Not a valid pack reference, treat as regular path
					if crate::support::paths::is_relative(&base_dir) {
						Some(workspace_path.join(&base_dir))
					} else {
						Some(SPath::from(base_dir))
					}
				}
			} else {
				// Not a pack reference, treat as regular path
				if crate::support::paths::is_relative(&base_dir) {
					Some(workspace_path.join(&base_dir))
				} else {
					Some(SPath::from(base_dir))
				}
			}
		}
		None => None,
	};

	Ok(base_dir)
}

/// Creates a vector of FileRecords from file paths
///
/// Takes a list of file paths and base path, loads content and creates FileRecord objects
pub fn create_file_records(
	runtime: &Runtime,
	file_refs: Vec<FileRef>,
	base_path: Option<&SPath>,
	absolute: bool,
) -> Result<Vec<FileRecord>> {
	let mut has_base_path = false;
	let base_path = match base_path {
		Some(base_path) => {
			has_base_path = true;
			base_path.clone()
		}
		None => runtime
			.dir_context()
			.wks_dir()
			.ok_or("Cannot create file records, no workspace")?
			.clone(),
	};

	file_refs
		.into_iter()
		.map(|file_ref| -> Result<FileRecord> {
			if absolute {
				// So, here, the sfile is the full path (for laoding), and the rel_path
				let file_record =
					FileRecord::load_from_full_path(runtime.dir_context(), file_ref.as_ref(), file_ref.clone().spath)?;
				Ok(file_record)
			} else {
				let full_path = if has_base_path {
					base_path.join(file_ref.as_ref())
				} else {
					let dir_context = runtime.dir_context();
					dir_context.resolve_path(runtime.session(), file_ref.clone().spath, PathResolver::WksDir, None)?
				};

				// Need to cannonicalize because we need to compute the diff
				let sfile_abs = full_path.canonicalize()?;
				let diff = sfile_abs.try_diff(&base_path)?;
				// if the diff goes back from base_path, then, we put the absolute path
				// TODO: need to double check this
				let (base_path, rel_path) = if diff.as_str().starts_with("..") {
					(SPath::from(""), sfile_abs)
				} else {
					(base_path.clone(), diff)
				};
				let full_path = base_path.join(&rel_path);
				let file_record = FileRecord::load_from_full_path(runtime.dir_context(), &full_path, rel_path)?;
				Ok(file_record)
			}
		})
		.collect()
}

/// Resolves the destination path based on source path and destination options.
///
/// Returns a tuple of `(relative_destination_path, full_destination_path)`.
pub fn resolve_dest_path(
	lua: &Lua,
	runtime: &Runtime,
	src_rel_path: &SPath,
	dest_value: Value,
	target_ext: &str,
	default_stem_suffix: Option<&str>,
) -> Result<(SPath, SPath)> {
	let dir_context = runtime.dir_context();

	let opts: DestOptions = DestOptions::from_lua(dest_value, lua)
		.map_err(|e| Error::Custom(format!("Failed to parse destination options.\nCause: {e}")))?;

	let src_stem = Path::new(src_rel_path.as_str())
		.file_stem()
		.and_then(|s| s.to_str())
		.ok_or_else(|| Error::Custom(format!("Source path '{src_rel_path}' has no file stem.")))?;

	let rel_dest_path: SPath = match opts {
		DestOptions::Nil => {
			let effective_stem = if let Some(def_suf) = default_stem_suffix {
				format!("{src_stem}{def_suf}")
			} else {
				src_stem.to_string()
			};
			let filename_part = format!("{effective_stem}.{target_ext}");

			if let Some(parent_dir) = src_rel_path.parent() {
				parent_dir.join(filename_part)
			} else {
				SPath::new(filename_part)
			}
		}
		DestOptions::Path(p) => p,
		DestOptions::Custom(c) => {
			let filename_part = if let Some(name_opt) = c.file_name {
				name_opt
			} else {
				let effective_stem = if let Some(opt_suf) = c.suffix {
					format!("{src_stem}{opt_suf}")
				} else if let Some(def_suf) = default_stem_suffix {
					if c.base_dir.is_some() {
						// If base_dir is specified, default_stem_suffix is ignored as per slim requirement
						src_stem.to_string()
					} else {
						// No base_dir, apply default_stem_suffix
						format!("{src_stem}{def_suf}")
					}
				} else {
					src_stem.to_string()
				};
				format!("{effective_stem}.{target_ext}")
			};

			if let Some(base_dir_spath) = c.base_dir {
				if base_dir_spath.as_str().is_empty() {
					SPath::new(filename_part)
				} else {
					base_dir_spath.join(filename_part)
				}
			} else if let Some(parent_dir) = src_rel_path.parent() {
				parent_dir.join(filename_part)
			} else {
				SPath::new(filename_part)
			}
		}
	};

	let full_dest_path =
		dir_context.resolve_path(runtime.session(), rel_dest_path.clone(), PathResolver::WksDir, None)?;
	Ok((rel_dest_path, full_dest_path))
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use crate::_test_support::assert_contains;
	use crate::runtime::Runtime;
	use crate::script::aip_modules::support::{process_path_reference, process_path_references};
	use crate::support::AsStrsExt;

	#[tokio::test]
	async fn test_lua_file_support_process_pack_references() -> Result<()> {
		// -- Setup & Fixtures
		let runtime = Runtime::new_test_runtime_sandbox_01().await?;
		let fx_globs: Vec<String> = ["ns_b@pack_b_2/main.aip", "no_ns@pack_b_2/main.aip", "**/*.txt"]
			.into_iter()
			.map(|v| v.to_string())
			.collect();

		// -- Exec
		let res = process_path_references(&runtime, fx_globs)?;

		// -- Check
		// NOTE: Now the process_path_references do not process for existences
		let res = res.x_as_strs();
		assert_eq!(res.len(), 3, "Should have three globs");
		let first = res.first().ok_or("Should have first")?;
		assert_contains(*first, "ns_b/pack_b_2/main.aip");
		assert_contains(&res, "**/*.txt");

		Ok(())
	}

	#[tokio::test]
	async fn test_lua_file_support_process_path_reference() -> Result<()> {
		// -- Setup & Fixtures
		let runtime = Runtime::new_test_runtime_sandbox_01().await?;
		let fx_path = "ns_b@pack_b_2/main.aip";

		// -- Exec
		let res = process_path_reference(&runtime, fx_path)?;

		// -- Check
		assert_contains(res.as_str(), "ns_b/pack_b_2/main.aip");

		Ok(())
	}
}

// endregion: --- Tests
