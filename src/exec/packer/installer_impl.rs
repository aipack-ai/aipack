use crate::dir_context::DirContext;
use crate::exec::packer::pack_toml::parse_validate_pack_toml;
use crate::exec::packer::support::PackUri;
use crate::exec::packer::{PackToml, provenance, support};
use crate::support::files::{DeleteCheck, safer_trash_dir, safer_trash_file};
use crate::support::zip;
use crate::{Error, Result};
use simple_fs::{SPath, ensure_dir};

pub enum InstallResponse {
	Installed(InstalledPack),
	UpToDate(InstalledPack),
}

pub struct InstalledPack {
	pub pack_toml: PackToml,
	pub path: SPath,
	#[allow(unused)]
	pub size: usize,
	pub zip_size: usize,
}

enum PackSource {
	Archive(SPath),
	Git {
		clone_dir: SPath,
		commit: String,
	},
}

impl PackSource {
	fn path(&self) -> &SPath {
		match self {
			Self::Archive(path) => path,
			Self::Git { clone_dir, .. } => clone_dir,
		}
	}
}

/// Install a `file.aipack` into the .aipack-base/pack/installed directory
///
/// IMPORTANT: Right now, very prelimealy. Should do the following:
///
/// TODO:
/// - Check for an existing installed pack.
/// - If an already installed pack has a semver greater than the new one,
///   return an error so that the caller can handle it with a prompt, and then provide a force flag, for example.
/// - Probably need to remove the existing pack files; otherwise, some leftover files can be an issue.
///
/// Returns the InstalledPack with information about the installed pack.
pub async fn install_pack(dir_context: &DirContext, pack_uri: &str, force: bool) -> Result<InstallResponse> {
	let original_pack_uri = pack_uri.to_string();
	let pack_uri = PackUri::parse(&original_pack_uri)?;

	// Get the aipack file path, downloading if needed
	let (source, pack_uri) = match pack_uri {
		pack_uri @ PackUri::RepoPack(_) => {
			let (aipack_zipped_file, pack_uri) = support::download_from_repo(dir_context, pack_uri).await?;
			(PackSource::Archive(aipack_zipped_file), pack_uri)
		}
		pack_uri @ PackUri::LocalPath(_) => {
			let (aipack_zipped_file, pack_uri) = support::resolve_local_path(dir_context, pack_uri)?;
			(PackSource::Archive(aipack_zipped_file), pack_uri)
		}
		pack_uri @ PackUri::HttpLink(_) => {
			let (aipack_zipped_file, pack_uri) = support::download_pack(dir_context, pack_uri).await?;
			(PackSource::Archive(aipack_zipped_file), pack_uri)
		}
		pack_uri @ PackUri::GitLink(_) => {
			let (git_dir, pack_uri, commit) = support::clone_from_git_with_commit(dir_context, pack_uri).await?;
			(
				PackSource::Git {
					clone_dir: git_dir,
					commit,
				},
				pack_uri,
			)
		}
	};

	let resolved_local_path = match (&pack_uri, &source) {
		(PackUri::LocalPath(_), PackSource::Archive(path)) => Some(path),
		_ => None,
	};
	let provenance_source = support::resolve_install_provenance_source(
		&original_pack_uri,
		&pack_uri,
		resolved_local_path,
	)?;

	// Validate file exists and has correct extension
	let zip_size = if let PackSource::Archive(aipack_zipped_file) = &source {
		support::validate_aipack_file(aipack_zipped_file, &pack_uri.to_string())?;
		support::get_file_size(aipack_zipped_file, &pack_uri.to_string())?
	} else {
		0
	};

	// Common installation steps for both local and remote files
	let install_result = match &source {
		PackSource::Archive(_) => {
			install_pack_source_with_provenance(dir_context, &source, &pack_uri, force, &provenance_source)
		}
		PackSource::Git { clone_dir, .. } => install_git_source_with_provenance(
			dir_context,
			&source,
			clone_dir,
			&pack_uri,
			force,
			&provenance_source,
		),
	};
	let mut install_res = install_result?;

	match install_res {
		InstallResponse::Installed(ref mut p) | InstallResponse::UpToDate(ref mut p) => {
			p.zip_size = zip_size;
		}
	}

	// If the file was downloaded (RepoPack or HttpLink), trash the temporary file
	if matches!(pack_uri, PackUri::RepoPack(_) | PackUri::HttpLink(_)) {
		safer_trash_file(source.path(), Some(DeleteCheck::CONTAINS_AIPACK_BASE))?;
	}

	Ok(install_res)
}


fn install_git_source_with_provenance(
	dir_context: &DirContext,
	source: &PackSource,
	clone_dir: &SPath,
	pack_uri: &PackUri,
	force: bool,
	provenance_source: &str,
) -> Result<InstallResponse> {
	let commit = match source {
		PackSource::Git { commit, .. } => Some(commit.as_str()),
		PackSource::Archive(_) => None,
	};
	let install_result = install_pack_source_with_provenance_and_commit(
		dir_context,
		source,
		pack_uri,
		force,
		provenance_source,
		commit,
	);

	match support::cleanup_git_clone(clone_dir) {
		Ok(()) => install_result,
		Err(cleanup_error) => {
			let cause = match install_result {
				Ok(_) => format!("Failed to clean up temporary Git clone '{}': {cleanup_error}", clone_dir.as_str()),
				Err(install_error) => format!(
					"Failed to install Git pack: {install_error}\nFailed to clean up temporary Git clone '{}': {cleanup_error}",
					clone_dir.as_str()
				),
			};

			Err(Error::FailToInstall {
				aipack_ref: pack_uri.to_string(),
				cause,
			})
		}
	}
}


#[cfg(test)]
fn install_git_source(
	dir_context: &DirContext,
	source: &PackSource,
	clone_dir: &SPath,
	pack_uri: &PackUri,
	force: bool,
) -> Result<InstallResponse> {
	let provenance_source = pack_uri.to_string();
	install_git_source_with_provenance(
		dir_context,
		source,
		clone_dir,
		pack_uri,
		force,
		&provenance_source,
	)
}


fn install_pack_source_with_provenance(
	dir_context: &DirContext,
	source: &PackSource,
	pack_uri: &PackUri,
	force: bool,
	provenance_source: &str,
) -> Result<InstallResponse> {
	install_pack_source_with_provenance_and_commit(dir_context, source, pack_uri, force, provenance_source, None)
}


/// Common installation logic for both local and remote aipack files
/// Return the InstalledPack containing pack information and installation details
fn install_pack_source_with_provenance_and_commit(
	dir_context: &DirContext,
	source: &PackSource,
	pack_uri: &PackUri,
	force: bool,
	provenance_source: &str,
	commit: Option<&str>,
) -> Result<InstallResponse> {
	// -- Get the aipack base pack install dir
	// This is the pack base dir and now, we need ot add `namespace/pack_name`
	let pack_installed_dir = dir_context.aipack_paths().get_base_pack_installed_dir()?;

	// Now, we automatically create, so we do not require it to be init-base
	ensure_dir(&pack_installed_dir)?;

	// Note: This should not happen, as it should have failed in the ensure_dir above.
	//       Howeer, for now,
	if !pack_installed_dir.exists() {
		return Err(Error::FailToInstall {
			aipack_ref: pack_uri.to_string(),
			cause: format!(
				"aipack base directory '{pack_installed_dir}' not found.\n   recommendation: Run 'aip init'"
			),
		});
	}

	let source_path = match source {
		PackSource::Archive(path) => path.clone(),
		PackSource::Git { clone_dir, .. } => support::resolve_git_pack_dir(clone_dir, pack_uri)?,
	};

	// -- Extract the pack.toml from zip and validate
	let new_pack_toml = match source {
		PackSource::Archive(aipack_zipped_file) => support::extract_pack_toml_from_pack_file(aipack_zipped_file)?,
		PackSource::Git { .. } => support::extract_pack_toml_from_pack_dir(&source_path, &pack_uri.to_string())?,
	};

	// NEW: Validate prerelease format for installation
	support::validate_version_for_install(&new_pack_toml.version)?;

	// -- Check if a pack with the same namespace/name is already installed
	let potential_existing_path = pack_installed_dir.join(&new_pack_toml.namespace).join(&new_pack_toml.name);

	if potential_existing_path.exists() && !force {
		let existing_pack_toml_path = potential_existing_path.join("pack.toml");

		// Try to get the existing pack toml (if it fails, we treat it as 0.0.0 and update)
		let existing_pack_toml = if existing_pack_toml_path.exists() {
			let content = std::fs::read_to_string(existing_pack_toml_path.path()).ok();
			content.and_then(|c| parse_validate_pack_toml(&c, existing_pack_toml_path.as_str()).ok())
		} else {
			None
		};

		if let Some(existing_pack_toml) = existing_pack_toml {
			let ord = support::validate_version_update(&existing_pack_toml.version, &new_pack_toml.version)?;
			match ord {
				std::cmp::Ordering::Equal => {
					return Ok(InstallResponse::UpToDate(InstalledPack {
						pack_toml: existing_pack_toml,
						path: potential_existing_path,
						size: 0,
						zip_size: 0,
					}));
				}
				std::cmp::Ordering::Less => {
					return Err(Error::InstallFailInstalledVersionAbove {
						installed_version: existing_pack_toml.version,
						new_version: new_pack_toml.version,
					});
				}
				std::cmp::Ordering::Greater => {}
			}
		}
	}

	// If we've gotten here, either there's no existing pack or the new version is greater than or equal to the installed version
	let pack_target_dir = pack_installed_dir.join(&new_pack_toml.namespace).join(&new_pack_toml.name);

	// If the directory exists, remove it first to ensure clean installation
	if pack_target_dir.exists() {
		safer_trash_dir(&pack_target_dir, Some(DeleteCheck::CONTAINS_AIPACK_BASE)).map_err(|e| {
			Error::FailToInstall {
				aipack_ref: pack_uri.to_string(),
				cause: format!("Failed to trash existing pack directory: {e}"),
			}
		})?;
	}

	match source {
		PackSource::Archive(aipack_zipped_file) => {
			zip::unzip_file(aipack_zipped_file, &pack_target_dir).map_err(|e| Error::FailToInstall {
				aipack_ref: pack_uri.to_string(),
				cause: format!("Failed to unzip pack: {e}"),
			})?;
		}
		PackSource::Git { .. } => {
			support::copy_git_pack(&source_path, &pack_target_dir).map_err(|e| Error::FailToInstall {
				aipack_ref: pack_uri.to_string(),
				cause: format!("Failed to copy Git pack: {e}"),
			})?;
		}
	}

	// Calculate the size of the installed pack
	let size = support::calculate_directory_size(&pack_target_dir)?;

	let installed_pack_toml_path = pack_target_dir.join("pack.toml");
	let provenance_result = match commit {
		Some(commit) => provenance::write_installation_provenance_with_commit(
			&installed_pack_toml_path,
			provenance_source,
			commit,
		),
		None => provenance::write_installation_provenance(&installed_pack_toml_path, provenance_source),
	};
	if let Err(provenance_error) = provenance_result {
		let cause = match safer_trash_dir(&pack_target_dir, Some(DeleteCheck::CONTAINS_AIPACK_BASE)) {
			Ok(_) => format!("Failed to write installation provenance: {provenance_error}"),
			Err(cleanup_error) => format!(
				"Failed to write installation provenance: {provenance_error}\nFailed to clean up installed pack directory '{}': {cleanup_error}",
				pack_target_dir.as_str()
			),
		};

		return Err(Error::FailToInstall {
			aipack_ref: pack_uri.to_string(),
			cause,
		});
	}

	Ok(InstallResponse::Installed(InstalledPack {
		pack_toml: new_pack_toml,
		path: pack_target_dir,
		size,
		zip_size: 0, // This will be populated by the caller
	}))
}

// region:    --- Tests

#[cfg(test)]
#[path = "../../_tests/tests_installer_impl.rs"]
mod tests;

// endregion: --- Tests
