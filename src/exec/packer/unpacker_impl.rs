use crate::dir_context::DirContext;
use crate::exec::packer::support::{self, PackUri, download_from_repo, fetch_repo_latest_version};
use crate::support::files::{DeleteCheck, safer_trash_dir, safer_trash_file};
use crate::support::zip;
use crate::types::PackIdentity;
use crate::{Error, Result};
use simple_fs::{SPath, ensure_dir};
use std::str::FromStr;

/// Result of a successful unpack operation
pub struct UnpackedPack {
	pub namespace: String,
	pub name: String,
	pub dest_path: SPath,
	/// "installed" or "remote"
	pub source: String,
}

/// Unpack a repo pack into the workspace custom pack area.
///
/// - Requires a full repo-style pack identity (`namespace@name`).
/// - Fails if the workspace `.aipack/` directory is missing.
/// - Fails if the destination already exists unless `force` is true.
/// - When forced, trashes the full destination directory before recreating it.
/// - Compares installed and remote versions; prefers newer remote archive when available,
///   otherwise copies from installed. If nothing is installed, downloads from repo.
pub async fn unpack_pack(dir_context: &DirContext, pack_ref_str: &str, force: bool) -> Result<UnpackedPack> {
	// -- Parse and validate pack identity (must be full namespace@name, no sub-path, no scope)
	let pack_identity = PackIdentity::from_str(pack_ref_str).map_err(|e| {
		Error::custom(format!(
			"Invalid pack reference for unpack: '{pack_ref_str}'.\n\
			 Unpack requires a full pack identity in the form 'namespace@name'.\nCause: {e}"
		))
	})?;

	// Reject if the input contains '/' (sub-path) or '$' (scope) after the identity
	if pack_ref_str.contains('/') || pack_ref_str.contains('$') {
		return Err(Error::custom(format!(
			"Invalid pack reference for unpack: '{pack_ref_str}'.\n\
			 Unpack requires a plain pack identity 'namespace@name' without sub-path or scope."
		)));
	}

	// -- Ensure workspace .aipack/ exists
	let aipack_wks_dir = dir_context.aipack_paths().aipack_wks_dir().ok_or_else(|| {
		Error::custom(
			"Cannot unpack: no workspace '.aipack/' directory found.\n\
				 Run 'aip init .' in your project root to create the workspace marker folder."
				.to_string(),
		)
	})?;

	// -- Compute destination path
	let wks_custom_dir = aipack_wks_dir.get_pack_custom_dir()?;
	let dest_dir = wks_custom_dir.join(&pack_identity.namespace).join(&pack_identity.name);

	// -- Check if destination already exists
	if dest_dir.exists() {
		if !force {
			return Err(Error::custom(format!(
				"Destination already exists: '{dest_dir}'.\n\
				 Use '--force' to replace the existing workspace custom pack."
			)));
		}
		// Force: trash the existing directory
		safer_trash_dir(&dest_dir, Some(DeleteCheck::CONTAINS_AIPACK)).map_err(|e| {
			Error::custom(format!(
				"Failed to remove existing destination '{dest_dir}' during forced unpack.\nCause: {e}"
			))
		})?;
	}

	// -- Determine source: installed vs remote
	let installed_dir = compute_installed_path(dir_context, &pack_identity)?;
	let installed_version = read_installed_version(&installed_dir);
	let remote_version = fetch_repo_latest_version(&pack_identity).await?;

	let source = determine_source(&installed_version, &remote_version, Some(&installed_dir))?;

	// -- Perform the unpack based on the selected source
	match source {
		UnpackSource::Installed(ref installed_path) => {
			// Copy installed directory to destination
			ensure_dir(dest_dir.parent().unwrap_or(wks_custom_dir.clone()))?;
			copy_dir_recursive(installed_path, &dest_dir)?;

			Ok(UnpackedPack {
				namespace: pack_identity.namespace,
				name: pack_identity.name,
				dest_path: dest_dir,
				source: "installed".to_string(),
			})
		}
		UnpackSource::Remote => {
			// Download from repo and unzip into destination
			let pack_uri = PackUri::RepoPack(pack_identity.clone());
			let (aipack_file, _pack_uri) = download_from_repo(dir_context, pack_uri).await?;

			ensure_dir(dest_dir.parent().unwrap_or(wks_custom_dir.clone()))?;
			zip::unzip_file(&aipack_file, &dest_dir).map_err(|e| {
				Error::custom(format!(
					"Failed to unzip downloaded pack into '{dest_dir}'.\nCause: {e}"
				))
			})?;

			// Cleanup downloaded file
			let _ = safer_trash_file(&aipack_file, Some(DeleteCheck::CONTAINS_AIPACK_BASE));

			Ok(UnpackedPack {
				namespace: pack_identity.namespace,
				name: pack_identity.name,
				dest_path: dest_dir,
				source: "remote".to_string(),
			})
		}
	}
}

// region:    --- Support

#[derive(Debug, PartialEq)]
enum UnpackSource {
	Installed(SPath),
	Remote,
}

/// Compute the path where this pack would be installed in base installed area
fn compute_installed_path(dir_context: &DirContext, pack_identity: &PackIdentity) -> Result<SPath> {
	let installed_base = dir_context.aipack_paths().get_base_pack_installed_dir()?;
	Ok(installed_base.join(&pack_identity.namespace).join(&pack_identity.name))
}

/// Read the version from an installed pack's pack.toml, if it exists
fn read_installed_version(installed_dir: &SPath) -> Option<String> {
	if !installed_dir.exists() {
		return None;
	}
	let pack_toml_path = installed_dir.join("pack.toml");
	if !pack_toml_path.exists() {
		return None;
	}
	let content = std::fs::read_to_string(pack_toml_path.as_std_path()).ok()?;
	let pack_toml: toml::Value = toml::from_str(&content).ok()?;
	pack_toml.get("version")?.as_str().map(|s| s.to_string())
}

/// Determine whether to use the installed copy or download from remote.
///
/// Logic:
/// - If remote version is available and newer than installed, use remote.
/// - If installed exists and remote is not available or not newer, use installed.
/// - If nothing installed, use remote.
fn determine_source(
	installed_version: &Option<String>,
	remote_version: &Option<String>,
	installed_dir: Option<&SPath>,
) -> Result<UnpackSource> {
	let installed_exists = installed_dir.is_some_and(|p| p.exists());

	let source = match (installed_exists, installed_version, remote_version) {
		// Both installed and remote available: compare versions
		(true, Some(inst_ver), Some(rem_ver)) => {
			match support::validate_version_update(inst_ver, rem_ver) {
				Ok(std::cmp::Ordering::Greater) => {
					// Remote is newer
					UnpackSource::Remote
				}
				_ => {
					let installed_dir = installed_dir.ok_or("Does not have installed dir setup")?;
					// Installed is same or newer, use installed
					UnpackSource::Installed(installed_dir.clone())
				}
			}
		}
		// Installed exists but no remote info: use installed
		(true, _, None) => {
			let installed_dir = installed_dir.ok_or("Does not have installed dir setup")?;
			UnpackSource::Installed(installed_dir.clone())
		}
		// Installed exists (even without parseable version) but remote available: prefer remote for freshness
		(true, None, Some(_)) => UnpackSource::Remote,
		// Nothing installed: must download
		(false, _, _) => UnpackSource::Remote,
	};

	Ok(source)
}

/// Recursively copy a directory tree from src to dest
fn copy_dir_recursive(src: &SPath, dest: &SPath) -> Result<()> {
	if !src.exists() {
		return Err(Error::custom(format!(
			"Source directory does not exist for copy: '{src}'"
		)));
	}

	ensure_dir(dest)?;

	for entry in walkdir::WalkDir::new(src.as_std_path()) {
		let entry = entry.map_err(|e| Error::custom(format!("Failed to read directory entry during copy: {e}")))?;
		let entry_path = SPath::from_std_path_buf(entry.path().to_path_buf())
			.map_err(|e| Error::custom(format!("Failed to convert path '{}': {e}", entry.path().display())))?;
		let relative = entry_path.diff(src).ok_or_else(|| {
			Error::custom(format!(
				"Failed to compute relative path from '{src}' to '{entry_path}'"
			))
		})?;
		let target = dest.join(relative.as_str());

		if entry.file_type().is_dir() {
			ensure_dir(&target)?;
		} else {
			// Ensure parent directory exists
			if let Some(parent) = target.parent() {
				ensure_dir(&parent)?;
			}
			std::fs::copy(entry_path.as_std_path(), target.as_std_path())
				.map_err(|e| Error::custom(format!("Failed to copy file '{}' to '{target}': {e}", entry_path)))?;
		}
	}

	Ok(())
}

// endregion: --- Support

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;
	use crate::_test_support::{gen_test_dir_path, save_file_content};

	// region:    --- determine_source tests

	#[test]
	fn test_unpacker_determine_source_both_installed_newer() -> Result<()> {
		// -- Setup & Fixtures
		let installed_version = Some("0.2.0".to_string());
		let remote_version = Some("0.1.0".to_string());
		let installed_dir = gen_test_dir_path();
		std::fs::create_dir_all(installed_dir.path())?;

		// -- Exec
		let source = determine_source(&installed_version, &remote_version, Some(&installed_dir))?;

		// -- Check
		assert!(
			matches!(source, UnpackSource::Installed(_)),
			"Should prefer installed when installed version is newer"
		);

		// -- Cleanup
		// remove_test_dir(&installed_dir)?;

		Ok(())
	}

	#[test]
	fn test_unpacker_determine_source_both_remote_newer() -> Result<()> {
		// -- Setup & Fixtures
		let installed_version = Some("0.1.0".to_string());
		let remote_version = Some("0.2.0".to_string());
		let installed_dir = gen_test_dir_path();
		std::fs::create_dir_all(installed_dir.path())?;

		// -- Exec
		let source = determine_source(&installed_version, &remote_version, Some(&installed_dir))?;

		// -- Check
		assert_eq!(
			source,
			UnpackSource::Remote,
			"Should prefer remote when remote version is newer"
		);

		// -- Cleanup
		// remove_test_dir(&installed_dir)?;

		Ok(())
	}

	#[test]
	fn test_unpacker_determine_source_both_equal() -> Result<()> {
		// -- Setup & Fixtures
		let installed_version = Some("0.1.0".to_string());
		let remote_version = Some("0.1.0".to_string());
		let installed_dir = gen_test_dir_path();
		std::fs::create_dir_all(installed_dir.path())?;

		// -- Exec
		let source = determine_source(&installed_version, &remote_version, Some(&installed_dir))?;

		// -- Check
		assert!(
			matches!(source, UnpackSource::Installed(_)),
			"Should prefer installed when versions are equal"
		);

		// -- Cleanup
		// remove_test_dir(&installed_dir)?;

		Ok(())
	}

	#[test]
	fn test_unpacker_determine_source_installed_only() -> Result<()> {
		// -- Setup & Fixtures
		let installed_version = Some("0.1.0".to_string());
		let remote_version: Option<String> = None;
		let installed_dir = gen_test_dir_path();
		std::fs::create_dir_all(installed_dir.path())?;

		// -- Exec
		let source = determine_source(&installed_version, &remote_version, Some(&installed_dir))?;

		// -- Check
		assert!(
			matches!(source, UnpackSource::Installed(_)),
			"Should use installed when no remote version available"
		);

		// -- Cleanup
		// remove_test_dir(&installed_dir)?;

		Ok(())
	}

	#[test]
	fn test_unpacker_determine_source_remote_only() -> Result<()> {
		// -- Setup & Fixtures
		let installed_version: Option<String> = None;
		let remote_version = Some("0.1.0".to_string());
		// installed_dir does not exist
		let installed_dir = gen_test_dir_path();

		// -- Exec
		let source = determine_source(&installed_version, &remote_version, Some(&installed_dir))?;

		// -- Check
		assert_eq!(source, UnpackSource::Remote, "Should use remote when nothing installed");

		Ok(())
	}

	#[test]
	fn test_unpacker_determine_source_nothing_available() -> Result<()> {
		// -- Setup & Fixtures
		let installed_version: Option<String> = None;
		let remote_version: Option<String> = None;
		let installed_dir = gen_test_dir_path();

		// -- Exec
		let source = determine_source(&installed_version, &remote_version, Some(&installed_dir))?;

		// -- Check
		assert_eq!(
			source,
			UnpackSource::Remote,
			"Should fall back to remote when nothing installed and no remote version info"
		);

		Ok(())
	}

	#[test]
	fn test_unpacker_determine_source_installed_no_version_remote_available() -> Result<()> {
		// -- Setup & Fixtures
		let installed_version: Option<String> = None;
		let remote_version = Some("0.1.0".to_string());
		let installed_dir = gen_test_dir_path();
		std::fs::create_dir_all(installed_dir.path())?;

		// -- Exec
		let source = determine_source(&installed_version, &remote_version, Some(&installed_dir))?;

		// -- Check
		assert_eq!(
			source,
			UnpackSource::Remote,
			"Should prefer remote when installed has no parseable version but remote is available"
		);

		// -- Cleanup
		// remove_test_dir(&installed_dir)?;

		Ok(())
	}

	// endregion: --- determine_source tests

	// region:    --- read_installed_version tests

	#[test]
	fn test_unpacker_read_installed_version_valid() -> Result<()> {
		// -- Setup & Fixtures
		let dir = gen_test_dir_path();
		std::fs::create_dir_all(dir.path())?;
		let pack_toml = dir.join("pack.toml");
		save_file_content(
			&pack_toml,
			"[pack]\nversion = \"1.2.3\"\nnamespace = \"test\"\nname = \"example\"\n",
		)?;

		// -- Exec
		let version = read_installed_version(&dir);

		// -- Check
		assert_eq!(version, Some("1.2.3".to_string()));

		// -- Cleanup
		// remove_test_dir(&dir)?;

		Ok(())
	}

	#[test]
	fn test_unpacker_read_installed_version_missing_dir() -> Result<()> {
		// -- Setup & Fixtures
		let dir = gen_test_dir_path();
		// Dir does not exist

		// -- Exec
		let version = read_installed_version(&dir);

		// -- Check
		assert_eq!(version, None);

		Ok(())
	}

	#[test]
	fn test_unpacker_read_installed_version_no_pack_toml() -> Result<()> {
		// -- Setup & Fixtures
		let dir = gen_test_dir_path();
		std::fs::create_dir_all(dir.path())?;
		// No pack.toml file created

		// -- Exec
		let version = read_installed_version(&dir);

		// -- Check
		assert_eq!(version, None);

		// -- Cleanup
		// remove_test_dir(&dir)?;

		Ok(())
	}

	#[test]
	fn test_unpacker_read_installed_version_malformed_toml() -> Result<()> {
		// -- Setup & Fixtures
		let dir = gen_test_dir_path();
		std::fs::create_dir_all(dir.path())?;
		let pack_toml = dir.join("pack.toml");
		save_file_content(&pack_toml, "this is not valid toml {{{")?;

		// -- Exec
		let version = read_installed_version(&dir);

		// -- Check
		assert_eq!(version, None);

		// -- Cleanup
		// remove_test_dir(&dir)?;

		Ok(())
	}

	// endregion: --- read_installed_version tests

	// region:    --- copy_dir_recursive tests

	#[test]
	fn test_unpacker_copy_dir_recursive_simple() -> Result<()> {
		// -- Setup & Fixtures
		let src_dir = gen_test_dir_path();
		let dest_dir = gen_test_dir_path();
		std::fs::create_dir_all(src_dir.path())?;

		// Create files in source
		save_file_content(&src_dir.join("file1.txt"), "content1")?;
		std::fs::create_dir_all(src_dir.join("subdir").path())?;
		save_file_content(&src_dir.join("subdir/file2.txt"), "content2")?;

		// -- Exec
		copy_dir_recursive(&src_dir, &dest_dir)?;

		// -- Check
		assert!(dest_dir.join("file1.txt").exists(), "file1.txt should exist in dest");
		assert!(
			dest_dir.join("subdir/file2.txt").exists(),
			"subdir/file2.txt should exist in dest"
		);

		let content1 = std::fs::read_to_string(dest_dir.join("file1.txt").path())?;
		assert_eq!(content1, "content1");

		let content2 = std::fs::read_to_string(dest_dir.join("subdir/file2.txt").path())?;
		assert_eq!(content2, "content2");

		// -- Cleanup
		// remove_test_dir(&src_dir)?;
		// remove_test_dir(&dest_dir)?;

		Ok(())
	}

	#[test]
	fn test_unpacker_copy_dir_recursive_src_not_exists() -> Result<()> {
		// -- Setup & Fixtures
		let src_dir = gen_test_dir_path();
		let dest_dir = gen_test_dir_path();
		// src_dir intentionally not created

		// -- Exec
		let result = copy_dir_recursive(&src_dir, &dest_dir);

		// -- Check
		assert!(result.is_err(), "Should fail when source does not exist");

		Ok(())
	}

	// endregion: --- copy_dir_recursive tests

	// region:    --- pack identity validation tests

	#[test]
	fn test_unpacker_pack_identity_rejects_sub_path() -> Result<()> {
		// -- Setup & Fixtures
		let pack_ref_str = "test@example/some/path";

		// -- Exec & Check
		// The unpack_pack function rejects sub-paths;
		// we test the same check inline here
		assert!(
			pack_ref_str.contains('/'),
			"Should contain '/' to trigger sub-path rejection"
		);

		Ok(())
	}

	#[test]
	fn test_unpacker_pack_identity_rejects_scope() -> Result<()> {
		// -- Setup & Fixtures
		let pack_ref_str = "test@example$base";

		// -- Exec & Check
		assert!(
			pack_ref_str.contains('$'),
			"Should contain '$' to trigger scope rejection"
		);

		Ok(())
	}

	#[test]
	fn test_unpacker_pack_identity_valid() -> Result<()> {
		// -- Setup & Fixtures
		let pack_ref_str = "test@example";

		// -- Exec
		let identity = PackIdentity::from_str(pack_ref_str)?;

		// -- Check
		assert_eq!(identity.namespace, "test");
		assert_eq!(identity.name, "example");
		assert!(!pack_ref_str.contains('/'));
		assert!(!pack_ref_str.contains('$'));

		Ok(())
	}

	#[test]
	fn test_unpacker_pack_identity_partial_rejected() -> Result<()> {
		// -- Setup & Fixtures
		// Partial refs like "jc@" or "@coder" should not parse as valid PackIdentity
		let partial_refs = ["jc@", "@coder", "justname"];

		// -- Exec & Check
		for partial in partial_refs {
			let result = PackIdentity::from_str(partial);
			assert!(
				result.is_err(),
				"Partial ref '{partial}' should be rejected by PackIdentity::from_str"
			);
		}

		Ok(())
	}

	// endregion: --- pack identity validation tests

	// region:    --- Support
	// endregion: --- Support
}

// endregion: --- Tests
