use crate::dir_context::DirContext;
use crate::exec::packer::PackToml;
use crate::exec::packer::pack_toml::{PartialPackToml, parse_validate_pack_toml};
use crate::support::files::{DeleteCheck, safer_trash_dir};
use crate::support::{
	proc::{proc_exec, proc_exec_to_output, ProcOptions},
	webc, zip,
};
use crate::types::PackIdentity;
use crate::{Error, Result};
use lazy_regex::regex;
use reqwest::Client;
use semver::Version;
use serde::Deserialize;
use simple_fs::{SPath, ensure_dir};
use std::str::FromStr;
use time::OffsetDateTime;
use time_tz::OffsetDateTimeExt;
use uuid::Uuid;

// region:    --- PackUri

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitSource {
	pub repository: String,
	pub subpath: Option<String>,
}

#[derive(Debug, Clone)]
pub enum PackUri {
	RepoPack(PackIdentity),
	LocalPath(String),
	HttpLink(String),
	GitLink(GitSource),
}

impl PackUri {
	pub fn parse(uri: &str) -> Result<Self> {
		// Try to parse as PackIdentity first
		if let Ok(pack_identity) = PackIdentity::from_str(uri) {
			return Ok(PackUri::RepoPack(pack_identity));
		}

		// If not a PackIdentity, check if it's an HTTP link
		if uri.starts_with("http://") || uri.starts_with("https://") {
			return Ok(PackUri::HttpLink(uri.to_string()));
		}

		if uri.starts_with("git://") || uri.starts_with("git+ssh://") {
			return Ok(PackUri::GitLink(Self::parse_git_source(uri)?));
		}

		// Otherwise, treat as local path
		Ok(PackUri::LocalPath(uri.to_string()))
	}

	fn parse_git_source(uri: &str) -> Result<GitSource> {
		let (repository, selector) = uri
			.split_once('#')
			.map_or((uri, None), |(repository, selector)| (repository, Some(selector)));

		if repository.is_empty() {
			return Err(Error::custom(format!("Invalid Git source '{uri}': repository URL is empty")));
		}

		let subpath = selector.map(Self::normalize_git_subpath).transpose()?;

		Ok(GitSource {
			repository: repository.to_string(),
			subpath,
		})
	}

	fn normalize_git_subpath(selector: &str) -> Result<String> {
		if selector.trim().is_empty() {
			return Err(Error::custom(format!(
				"Invalid Git pack selector '{selector}': selector cannot be empty"
			)));
		}

		let selector_bytes = selector.as_bytes();
		let is_windows_absolute = selector_bytes.len() >= 3
			&& selector_bytes[0].is_ascii_alphabetic()
			&& selector_bytes[1] == b':'
			&& (selector_bytes[2] == b'/' || selector_bytes[2] == b'\\');

		if selector.starts_with('/') || selector.starts_with('\\') || is_windows_absolute {
			return Err(Error::custom(format!(
				"Invalid Git pack selector '{selector}': absolute selectors are not allowed"
			)));
		}

		let normalized = selector.replace('\\', "/");
		let mut components = Vec::new();
		for component in normalized.split('/') {
			if component.is_empty() || component == "." {
				continue;
			}

			if component == ".." {
				return Err(Error::custom(format!(
					"Invalid Git pack selector '{selector}': parent-directory traversal is not allowed"
				)));
			}

			components.push(component);
		}

		if components.is_empty() {
			return Err(Error::custom(format!(
				"Invalid Git pack selector '{selector}': selector cannot be empty"
			)));
		}

		Ok(components.join("/"))
	}
}

impl std::fmt::Display for PackUri {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			PackUri::RepoPack(identity) => write!(f, "{identity}"),
			PackUri::LocalPath(path) => write!(f, "local file '{path}'"),
			PackUri::HttpLink(url) => write!(f, "URL '{url}'"),
			PackUri::GitLink(source) => match &source.subpath {
				Some(subpath) => write!(f, "Git URL '{}#{subpath}'", source.repository),
				None => write!(f, "Git URL '{}'", source.repository),
			},
		}
	}
}

pub(super) fn resolve_install_provenance_source(
	original_reference: &str,
	pack_uri: &PackUri,
	resolved_local_path: Option<&SPath>,
) -> Result<String> {
	match pack_uri {
		PackUri::RepoPack(_) => Ok("aipack.ai".to_string()),
		PackUri::HttpLink(_) | PackUri::GitLink(_) => Ok(original_reference.to_string()),
		PackUri::LocalPath(_) => resolved_local_path
			.map(|path| path.as_str().to_string())
			.ok_or_else(|| {
				Error::custom(format!(
					"Cannot resolve installation provenance source for local pack '{original_reference}': resolved local path is missing"
				))
			}),
	}
}

// endregion: --- PackUri

// region:    --- LatestToml

#[derive(Deserialize, Debug)]
pub(super) struct LatestToml {
	pub latest_stable: Option<LatestStableInfo>,
}

#[derive(Deserialize, Debug)]
pub(super) struct LatestStableInfo {
	pub version: Option<String>,
	pub rel_path: Option<String>,
}

impl LatestToml {
	pub fn validate(&self) -> Result<(&str, &str)> {
		// Check if latest_stable exists
		let latest_stable = self
			.latest_stable
			.as_ref()
			.ok_or_else(|| Error::custom("Missing 'latest_stable' section in latest.toml".to_string()))?;

		// Check if version is provided
		let version = latest_stable
			.version
			.as_deref()
			.ok_or_else(|| Error::custom("Missing 'version' in latest_stable section of latest.toml".to_string()))?;

		// Check if rel_path is provided
		let rel_path = latest_stable
			.rel_path
			.as_deref()
			.ok_or_else(|| Error::custom("Missing 'rel_path' in latest_stable section of latest.toml".to_string()))?;

		Ok((version, rel_path))
	}
}

// endregion: --- LatestToml

// region:    --- Shared Repo/Download Helpers

/// Fetches the latest.toml metadata from the remote repository for a given pack identity.
///
/// Returns the parsed LatestToml, which can be validated for version and rel_path.
pub(super) async fn fetch_repo_latest_toml(pack_identity: &PackIdentity) -> Result<LatestToml> {
	let latest_toml_url = format!(
		"https://repo.aipack.ai/pack/{}/{}/stable/latest.toml",
		pack_identity.namespace, pack_identity.name
	);

	let client = Client::new();
	let response = client.get(&latest_toml_url).send().await.map_err(|e| Error::FailToInstall {
		aipack_ref: pack_identity.to_string(),
		cause: format!("Failed to download latest.toml: {e}"),
	})?;

	if !response.status().is_success() {
		return Err(Error::FailToInstall {
			aipack_ref: pack_identity.to_string(),
			cause: format!("HTTP error when fetching latest.toml: {}", response.status()),
		});
	}

	let latest_toml_content = response.text().await.map_err(|e| Error::FailToInstall {
		aipack_ref: pack_identity.to_string(),
		cause: format!("Failed to read latest.toml content: {e}"),
	})?;

	let latest_toml: LatestToml = toml::from_str(&latest_toml_content).map_err(|e| Error::FailToInstall {
		aipack_ref: pack_identity.to_string(),
		cause: format!("Failed to parse latest.toml: {e}"),
	})?;

	Ok(latest_toml)
}

/// Constructs the full download URL for a pack from its identity and a relative path from latest.toml.
pub(super) fn build_repo_pack_url(pack_identity: &PackIdentity, rel_path: &str) -> String {
	format!(
		"https://repo.aipack.ai/pack/{}/{}/stable/{rel_path}",
		pack_identity.namespace, pack_identity.name
	)
}

/// Downloads a pack from a repo pack identity, resolving via latest.toml.
///
/// Returns the path to the downloaded `.aipack` file and the original PackUri.
pub(super) async fn download_from_repo(dir_context: &DirContext, pack_uri: PackUri) -> Result<(SPath, PackUri)> {
	if let PackUri::RepoPack(ref pack_identity) = pack_uri {
		let latest_toml = fetch_repo_latest_toml(pack_identity).await?;

		// Validate the latest.toml content
		let (_version, rel_path) = latest_toml.validate()?;

		// Construct the full URL to the .aipack file
		let aipack_url = build_repo_pack_url(pack_identity, rel_path);

		// Use HttpLink to download the actual pack
		let http_uri = PackUri::HttpLink(aipack_url);
		let (aipack_file, _) = download_pack(dir_context, http_uri).await?;

		return Ok((aipack_file, pack_uri));
	}

	Err(Error::custom(
		"Expected RepoPack variant but got a different one".to_string(),
	))
}

/// Clones a Git source into a unique temporary directory under the base download directory.
#[allow(dead_code)]
pub(super) async fn clone_from_git(dir_context: &DirContext, pack_uri: PackUri) -> Result<(SPath, PackUri)> {
	let (clone_dir, pack_uri, _) = clone_from_git_with_commit(dir_context, pack_uri).await?;
	Ok((clone_dir, pack_uri))
}

pub(super) async fn clone_from_git_with_commit(
	dir_context: &DirContext,
	pack_uri: PackUri,
) -> Result<(SPath, PackUri, String)> {
	let PackUri::GitLink(git_source) = pack_uri else {
		return Err(Error::custom(
			"Expected GitLink variant but got a different one".to_string(),
		));
	};

	let git_reference = PackUri::GitLink(git_source.clone()).to_string();
	let download_dir = dir_context.aipack_paths().get_base_pack_download_dir()?;
	ensure_dir(&download_dir)?;

	let clone_dir = download_dir.join(Uuid::now_v7().to_string());
	let clone_dir_str = clone_dir.as_str();
	let clone_result = proc_exec(
		"git",
		&["clone", git_source.repository.as_str(), clone_dir_str],
		None,
	)
	.await;

	if let Err(error) = clone_result {
		let cause = match cleanup_git_clone(&clone_dir) {
			Ok(()) => format!("Failed to clone Git source: {error}"),
			Err(cleanup_error) => format!(
				"Failed to clone Git source: {error}\nFailed to clean up temporary Git clone '{clone_dir_str}': {cleanup_error}"
			),
		};

		return Err(Error::FailToInstall {
			aipack_ref: git_reference,
			cause,
		});
	}

	let commit_options = ProcOptions::default().with_cwd(clone_dir_str);
	let commit_result = proc_exec_to_output("git", &["rev-parse", "HEAD"], Some(&commit_options)).await;
	let commit = match commit_result {
		Ok(commit) if !commit.trim().is_empty() => Ok(commit.trim().to_string()),
		Ok(_) => Err(Error::custom("Git commit resolution returned an empty hash")),
		Err(error) => Err(error),
	};

	let commit = match commit {
		Ok(commit) => commit,
		Err(error) => {
			let cause = match cleanup_git_clone(&clone_dir) {
				Ok(()) => format!("Failed to resolve Git commit: {error}"),
				Err(cleanup_error) => format!(
					"Failed to resolve Git commit: {error}\nFailed to clean up temporary Git clone '{clone_dir_str}': {cleanup_error}"
				),
			};

			return Err(Error::FailToInstall {
				aipack_ref: git_reference,
				cause,
			});
		}
	};

	Ok((clone_dir, PackUri::GitLink(git_source), commit))
}

/// Resolves a selected Git pack directory relative to the clone root.
pub(super) fn resolve_git_pack_dir(clone_dir: &SPath, pack_uri: &PackUri) -> Result<SPath> {
	let PackUri::GitLink(git_source) = pack_uri else {
		return Err(Error::custom(
			"Expected GitLink variant but got a different one".to_string(),
		));
	};

	let pack_dir = git_source
		.subpath
		.as_deref()
		.map_or_else(|| clone_dir.clone(), |subpath| clone_dir.join(subpath));

	if !pack_dir.is_dir() {
		return Err(Error::FailToInstall {
			aipack_ref: pack_uri.to_string(),
			cause: format!("Git pack directory '{}' does not exist", pack_dir.as_str()),
		});
	}

	Ok(pack_dir)
}

/// Removes a temporary Git clone after the caller has finished using it.
pub(super) fn cleanup_git_clone(clone_dir: &SPath) -> Result<()> {
	if clone_dir.exists() {
		safer_trash_dir(clone_dir, Some(DeleteCheck::CONTAINS_AIPACK_BASE))?;
	}

	Ok(())
}

/// Resolves a local path to an absolute SPath
pub(super) fn resolve_local_path(dir_context: &DirContext, pack_uri: PackUri) -> Result<(SPath, PackUri)> {
	if let PackUri::LocalPath(ref path) = pack_uri {
		let aipack_zipped_file = SPath::from(path);

		if aipack_zipped_file.path().is_absolute() {
			Ok((aipack_zipped_file, pack_uri))
		} else {
			let absolute_path = dir_context.current_dir().join(aipack_zipped_file.as_str());
			Ok((absolute_path, pack_uri))
		}
	} else {
		Err(Error::custom(
			"Expected LocalPath variant but got a different one".to_string(),
		))
	}
}

/// Downloads a pack from a URL and returns the path to the downloaded file
pub(super) async fn download_pack(dir_context: &DirContext, pack_uri: PackUri) -> Result<(SPath, PackUri)> {
	if let PackUri::HttpLink(ref url) = pack_uri {
		// Get the download directory
		let download_dir = dir_context.aipack_paths().get_base_pack_download_dir()?;

		// Create the download directory if it doesn't exist
		if !download_dir.exists() {
			ensure_dir(&download_dir)?;
		}

		// Extract the filename from the URL
		let url_path = url.split('/').next_back().unwrap_or("unknown.aipack");
		let filename = url_path.replace(' ', "-");

		// Create a timestamped filename using the time crate
		let now = OffsetDateTime::now_utc();
		// attempt to get local now (otherwise, no big deal, same machine so should be consistent return)
		let now = if let Ok(local) = time_tz::system::get_timezone() {
			now.to_timezone(local)
		} else {
			now
		};

		let timestamp =
			now.format(&time::format_description::well_known::Rfc3339)
				.map_err(|e| Error::FailToInstall {
					aipack_ref: pack_uri.to_string(),
					cause: format!("Failed to format timestamp: {e}"),
				})?;

		// Create a cleaner timestamp for filenames (removing colons, etc.)
		let file_timestamp = timestamp.replace([':', 'T'], "-");
		let file_timestamp = file_timestamp.split('.').next().unwrap_or(timestamp.as_str());
		let timestamped_filename = format!("{file_timestamp}-{filename}");
		let download_path = download_dir.join(&timestamped_filename);

		// Download the file
		webc::web_download_to_file(url, &download_path).await?;

		return Ok((download_path, pack_uri));
	}

	Err(Error::custom(
		"Expected HttpLink variant but got a different one".to_string(),
	))
}

/// Fetches the latest remote version string from the repository for a given pack identity.
///
/// Returns `Ok(Some(version))` if the remote latest.toml was successfully fetched and validated,
/// or `Ok(None)` if the remote could not be reached or the metadata was invalid.
pub(super) async fn fetch_repo_latest_version(pack_identity: &PackIdentity) -> Result<Option<String>> {
	match fetch_repo_latest_toml(pack_identity).await {
		Ok(latest_toml) => match latest_toml.validate() {
			Ok((version, _rel_path)) => Ok(Some(version.to_string())),
			Err(_) => Ok(None),
		},
		Err(_) => Ok(None),
	}
}

// endregion: --- Shared Repo/Download Helpers

/// Extracts and validates the pack.toml from an .aipack file
///
/// # Parameters
/// - `path_to_aipack`: The path to the .aipack file
///
/// # Returns
/// - Ok(PackToml): If extraction and validation are successful
/// - Err(Error): If any error occurs during extraction or validation
pub fn extract_pack_toml_from_pack_file(path_to_aipack: &SPath) -> Result<PackToml> {
	// Extract the pack.toml from zip
	let toml_content = zip::extract_text_content(path_to_aipack, "pack.toml").map_err(|e| Error::FailToInstall {
		aipack_ref: path_to_aipack.as_str().to_string(),
		cause: format!("Failed to extract pack.toml: {e}"),
	})?;

	// Parse and validate the pack.toml content
	let pack_toml =
		parse_validate_pack_toml(&toml_content, &format!("pack.toml for {path_to_aipack}")).map_err(|e| {
			Error::FailToInstall {
				aipack_ref: path_to_aipack.as_str().to_string(),
				cause: format!("Invalid pack.toml: {e}"),
			}
		})?;

	Ok(pack_toml)
}

pub(super) fn extract_pack_toml_from_pack_dir(path_to_pack: &SPath, reference: &str) -> Result<PackToml> {
	let pack_toml_path = path_to_pack.join("pack.toml");

	if !pack_toml_path.exists() {
		return Err(Error::FailToInstall {
			aipack_ref: reference.to_string(),
			cause: format!(
				"Git pack directory '{}' must contain 'pack.toml'",
				path_to_pack.as_str()
			),
		});
	}

	let toml_content = std::fs::read_to_string(pack_toml_path.path()).map_err(|e| Error::FailToInstall {
		aipack_ref: reference.to_string(),
		cause: format!("Failed to read root pack.toml: {e}"),
	})?;

	parse_validate_pack_toml(&toml_content, &format!("root pack.toml for {reference}")).map_err(|e| {
		Error::FailToInstall {
			aipack_ref: reference.to_string(),
			cause: format!("Invalid root pack.toml: {e}"),
		}
	})
}

pub(super) fn copy_git_pack(source_dir: &SPath, target_dir: &SPath) -> Result<()> {
	std::fs::create_dir_all(target_dir.path()).map_err(|e| Error::custom(format!("Failed to create Git pack directory: {e}")))?;

	let mut entries = walkdir::WalkDir::new(source_dir.path()).min_depth(1).into_iter();
	while let Some(entry) = entries.next() {
		let entry = entry.map_err(|e| Error::custom(format!("Failed to inspect Git pack: {e}")))?;
		let relative_path = entry
			.path()
			.strip_prefix(source_dir.path())
			.map_err(|e| Error::custom(format!("Failed to resolve Git pack path: {e}")))?;

		if relative_path
			.components()
			.next()
			.is_some_and(|component| component == std::path::Component::Normal(std::ffi::OsStr::new(".git")))
		{
			if entry.file_type().is_dir() {
				entries.skip_current_dir();
			}
			continue;
		}

		let relative_path = relative_path.to_str().ok_or_else(|| {
			Error::custom(format!(
				"Git pack path '{}' is not valid UTF-8",
				entry.path().display()
			))
		})?;

		let target_path = target_dir.join(relative_path);
		if entry.file_type().is_dir() {
			std::fs::create_dir_all(&target_path)
				.map_err(|e| Error::custom(format!("Failed to create Git pack directory '{}': {e}", target_path.as_str())))?;
		} else if entry.file_type().is_file() {
			if let Some(parent) = target_path.parent() {
				std::fs::create_dir_all(&parent)
					.map_err(|e| Error::custom(format!("Failed to create Git pack directory '{}': {e}", parent.as_str())))?;
			}

			std::fs::copy(entry.path(), &target_path)
				.map_err(|e| Error::custom(format!("Failed to copy Git pack file '{}': {e}", entry.path().display())))?;
		} else {
			return Err(Error::custom(format!(
				"Unsupported file type in Git pack '{}'",
				entry.path().display()
			)));
		}
	}

	Ok(())
}

/// Extracts the pack.toml from an .aipack file and returns it as a PartialPackToml without validation
///
/// This function is useful when custom error handling is needed or when only checking
/// specific fields without full validation.
///
/// # Parameters
/// - `path_to_aipack`: The path to the .aipack file
///
/// # Returns
/// - Ok(PartialPackToml): If extraction is successful
/// - Err(Error): If any error occurs during extraction
#[allow(unused)]
pub fn extract_partial_pack_toml_from_pack_file(path_to_aipack: &SPath) -> Result<PartialPackToml> {
	// Extract the pack.toml from zip
	let toml_content = zip::extract_text_content(path_to_aipack, "pack.toml").map_err(|e| Error::FailToInstall {
		aipack_ref: path_to_aipack.as_str().to_string(),
		cause: format!("Failed to extract pack.toml: {e}"),
	})?;

	// Parse the TOML content without validation
	let partial_pack_toml = toml::from_str(&toml_content).map_err(|e| Error::FailToInstall {
		aipack_ref: path_to_aipack.as_str().to_string(),
		cause: format!("Failed to parse pack.toml: {e}"),
	})?;

	Ok(partial_pack_toml)
}

/// Validates an .aipack file extension and existence
///
/// # Parameters
/// - `aipack_file`: The path to the .aipack file
/// - `reference`: A string representation of the file for error reporting
///
/// # Returns
/// - Ok(()): If validation passes
/// - Err(Error): If validation fails
pub fn validate_aipack_file(aipack_file: &SPath, reference: &str) -> Result<()> {
	if !aipack_file.exists() {
		return Err(Error::FailToInstall {
			aipack_ref: reference.to_string(),
			cause: "aipack file does not exist".to_string(),
		});
	}

	if aipack_file.ext() != "aipack" {
		return Err(Error::FailToInstall {
			aipack_ref: reference.to_string(),
			cause: format!("aipack file must be '.aipack' file, but was {}", aipack_file.name()),
		});
	}

	Ok(())
}

/// Validates if the new version is greater than or equal to the installed version
///
/// Returns Ok(()) if the new version is greater than or equal to the installed version
/// or if either version can't be parsed as a valid semver version.
///
/// Returns Err(Error::InstallFailInstalledVersionAbove) if the installed version is greater
/// than the new version.
///
/// # Parameters
/// - `installed_version`: The currently installed version
/// - `new_version`: The new version to be installed
///
/// # Returns
/// - Ok(()): If version comparison passes
/// - Err(Error): If validation fails
pub fn validate_version_update(installed_version: &str, new_version: &str) -> Result<std::cmp::Ordering> {
	// Remove leading 'v' if present for both versions
	let installed = installed_version.trim_start_matches('v');
	let new = new_version.trim_start_matches('v');

	// Parse versions into semver::Version
	if let (Ok(installed_semver), Ok(new_semver)) = (Version::parse(installed), Version::parse(new)) {
		Ok(new_semver.cmp(&installed_semver))
	} else {
		// If not valid semver, fallback to string comparison
		Ok(new.cmp(installed))
	}
}

/// Validates if the version format is valid for installation
///
/// In addition to standard semver validation, this function checks that
/// prerelease versions (e.g., -alpha, -beta) must end with a .number
///
/// Examples of valid versions:
/// - 0.1.1
/// - 0.1.1-alpha.1
/// - 0.1.1-beta.123
/// - 0.1.1-rc.1.2
///
/// Examples of invalid versions:
/// - 0.1.1-alpha (missing .number)
/// - 0.1.1-alpha.text (not ending with number)
///
/// # Parameters
/// - `version`: The version string to validate
///
/// # Returns
/// - Ok(()): If the version format is valid
/// - Err(Error): If the version format is invalid
pub fn validate_version_for_install(version: &str) -> Result<()> {
	// Remove leading 'v' if present
	let version_str = version.trim_start_matches('v');

	// Check if there's a prerelease portion (after a hyphen)
	if let Some(hyphen_idx) = version_str.find('-') {
		let prerelease = &version_str[hyphen_idx + 1..];

		// Regex to check if the prerelease ends with .number
		// This matches: any characters followed by a dot and then one or more digits at the end
		let prerelease_ending_with_number = regex!(r"\.[0-9]+$");

		if !prerelease_ending_with_number.is_match(prerelease) {
			return Err(Error::InvalidPrereleaseFormat {
				version: version.to_string(),
			});
		}
	}

	Ok(())
}

// /// Normalizes a version string by replacing dots and special characters with hyphens
// /// This is just to write the file names (cosmetic)
// /// and ensuring no consecutive hyphens
// pub fn normalize_version(version: &str) -> String {
// 	let mut result = String::new();
// 	let mut last_was_hyphen = false;

// 	for c in version.chars() {
// 		if c.is_alphanumeric() {
// 			result.push(c);
// 			last_was_hyphen = false;
// 		} else if !last_was_hyphen {
// 			result.push('-');
// 			last_was_hyphen = true;
// 		}
// 	}

// 	// Remove trailing hyphen if exists
// 	if result.ends_with('-') {
// 		result.pop();
// 	}

// 	result
// }

/// Get the size of a file in bytes
pub fn get_file_size(file_path: &SPath, reference: &str) -> Result<usize> {
	let metadata = std::fs::metadata(file_path.path()).map_err(|e| Error::FailToInstall {
		aipack_ref: reference.to_string(),
		cause: format!("Failed to get file metadata: {e}"),
	})?;

	Ok(metadata.len() as usize)
}

/// Calculate the total size of a directory recursively
pub fn calculate_directory_size(dir_path: &SPath) -> Result<usize> {
	use walkdir::WalkDir;

	let total_size = WalkDir::new(dir_path.path())
		.into_iter()
		.filter_map(|entry| entry.ok())
		.filter_map(|entry| entry.metadata().ok())
		.filter(|metadata| metadata.is_file())
		.map(|metadata| metadata.len() as usize)
		.sum();

	Ok(total_size)
}

// region:    --- Tests

#[cfg(test)]
#[path = "support_tests.rs"]
mod tests;

// endregion: --- Tests
