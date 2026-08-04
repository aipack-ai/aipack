use super::*;
use crate::_test_support::{remove_test_dir, save_file_content};
use crate::exec::packer::{self, install_pack};
use crate::runtime::Runtime;
use simple_fs::{SPath, ensure_dir};

type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::test]
async fn test_installer_impl_local_file_simple() -> Result<()> {
	// -- Setup & Fixtures
	// this will create the .tests-data/.tmp/... and the base dir for .aipack/ and .aipack-base
	let runtime = Runtime::new_test_runtime_for_temp_dir().await?;
	let dir_context = runtime.dir_context();
	// Prep the pack dir
	let to_pack_dir = SPath::new("tests-data/test_packs_folder/test_pack_01");
	let pack_result = packer::pack_dir(to_pack_dir, dir_context.current_dir())?;
	let aipack_file_path = pack_result.pack_file;

	// -- Exec
	let installed_pack = install_pack(dir_context, aipack_file_path.as_str(), true).await?;

	let InstallResponse::Installed(installed_pack) = installed_pack else {
		return Err("Should be installed_pack".into());
	};
	// -- Check
	// Verify that the pack was installed correctly
	assert_eq!(installed_pack.pack_toml.namespace, "test");
	assert_eq!(installed_pack.pack_toml.name, "test_pack_01");
	assert_eq!(installed_pack.pack_toml.version, "0.1.0");

	// Verify the installation path follows the expected pattern
	let expected_install_path = dir_context
		.aipack_paths()
		.get_base_pack_installed_dir()?
		.join("test/test_pack_01");
	assert_eq!(installed_pack.path.as_str(), expected_install_path.as_str());

	// Verify that the main.aip file was extracted
	let main_aip_path = expected_install_path.join("main.aip");
	assert!(main_aip_path.exists(), "main.aip should have been extracted");

	// Verify pack.toml was extracted
	let pack_toml_path = expected_install_path.join("pack.toml");
	assert!(pack_toml_path.exists(), "pack.toml should have been extracted");

	// Verify zip_size is set correctly
	assert!(installed_pack.zip_size > 0, "zip_size should be greater than 0");

	// -- Cleanup
	// This will check that it is a `tests-data/.tmp`
	remove_test_dir(dir_context.current_dir())?;

	Ok(())
}

// TODO: This fails sometime. Probably a race condition. Needs to investigate.
#[tokio::test]
async fn test_installer_impl_local_version_above_err() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_for_temp_dir().await?;
	let dir_context = runtime.dir_context();

	// Create common main.aip content for both packs
	let main_aip_content = r#"# Test Main

This is a test agent file for installation testing."#;

	// Step 1: Create old pack directory (version 0.2.0)
	let old_pack_dir = dir_context.current_dir().join("pack_to_install/test_old/test-pack-01");
	ensure_dir(&old_pack_dir)?;
	let old_pack_toml = r#"
[pack]
namespace = "test_ns"
name = "test-pack-01"
version = "0.2.0"
"#;
	save_file_content(&old_pack_dir.join("pack.toml"), old_pack_toml)?;

	let old_pack_data = packer::pack_dir(&old_pack_dir, dir_context.current_dir())?;
	let old_pack_file = old_pack_data.pack_file;
	// Install the old pack (version 0.2.0)
	let _installed_old_pack = install_pack(dir_context, old_pack_file.as_str(), true).await?;
	// (Optional: assert that installed_old_pack.pack_toml.version == "0.2.0")

	// Step 2: Create new pack directory (version 0.1.0)
	let new_pack_dir = dir_context.current_dir().join("pack_to_install/test_new/test-pack-01");
	ensure_dir(&new_pack_dir)?;
	let new_pack_toml = r#"
[pack]
namespace = "test_ns"
name = "test-pack-01"
version = "0.1.0"
"#;
	save_file_content(&new_pack_dir.join("pack.toml"), new_pack_toml)?;
	save_file_content(&new_pack_dir.join("main.aip"), main_aip_content)?;
	let new_pack_data = packer::pack_dir(&new_pack_dir, dir_context.current_dir())?;
	let new_pack_file = new_pack_data.pack_file;

	// -- Execute: Try to install the new pack (version 0.1.0)
	let result = install_pack(dir_context, new_pack_file.as_str(), true).await;

	// -- Check: The new pack installation should fail.
	assert!(result.is_err(), "Installing lower version should fail");

	if let Err(error) = result {
		match error {
			Error::InstallFailInstalledVersionAbove {
				installed_version,
				new_version,
			} => {
				assert_eq!(installed_version, "0.2.0");
				assert_eq!(new_version, "0.1.0");
			}
			other => panic!("Expected InstallFailInstalledVersionAbove error, got: {other:?}"),
		}
	}

	// -- Cleanup
	remove_test_dir(dir_context.current_dir())?;
	Ok(())
}

// TODO: This fails sometime. Probably a race condition. Needs to investigate.
#[tokio::test]
async fn test_installer_impl_invalid_prerelease_err() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_for_temp_dir().await?;
	let dir_context = runtime.dir_context();

	// Create a pack directory with an invalid prerelease version "0.1.0-alpha"
	// (the prerelease must end with ".number" such as "-alpha.1")
	let invalid_pack_dir = dir_context
		.current_dir()
		.join("pack_to_install/invalid_prerelease/test-pack-02");
	ensure_dir(&invalid_pack_dir)?;
	let invalid_pack_toml = r#"
[pack]
namespace = "test_ns"
name = "test-pack-02"
version = "0.1.0-alpha"
"#;
	save_file_content(&invalid_pack_dir.join("pack.toml"), invalid_pack_toml)?;
	save_file_content(
		&invalid_pack_dir.join("main.aip"),
		"# Test Main\nInvalid prerelease version.",
	)?;

	// Pack the directory into a .aipack file
	let pack_data = packer::pack_dir(&invalid_pack_dir, dir_context.current_dir())?;
	let pack_file_str = pack_data.pack_file.as_str();

	// Attempt to install the pack, expecting an error due to invalid prerelease format
	let result = install_pack(dir_context, pack_file_str, true).await;

	assert!(
		result.is_err(),
		"Installation should fail due to invalid prerelease format"
	);

	if let Err(error) = result {
		match error {
			Error::InvalidPrereleaseFormat { version } => {
				assert_eq!(version, "0.1.0-alpha");
			}
			other => panic!("Expected InvalidPrereleaseFormat error, got: {other:?}"),
		}
	}

	// -- Cleanup
	remove_test_dir(dir_context.current_dir())?;
	Ok(())
}

#[tokio::test]
async fn test_installer_impl_git_local_pack_installation() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_for_temp_dir().await?;
	let dir_context = runtime.dir_context();
	let repo_dir = dir_context.current_dir().join("git-source/local-pack");
	let pack_toml = git_pack_toml("git", "local-pack", "0.1.0");
	create_git_repository(
		&repo_dir,
		Some(&pack_toml),
		&[
			("main.aip", "# Git Main\nInstalled from a Git source."),
			("data/fixture.txt", "Git fixture data"),
		],
	)?;

	let (clone_dir, pack_uri) = clone_git_repository(dir_context, &repo_dir).await?;
	let source = PackSource::Git(clone_dir.clone());

	// -- Exec
	let result = install_git_source(dir_context, &source, &clone_dir, &pack_uri, false);

	// -- Check
	let installed_pack = match result? {
		InstallResponse::Installed(pack) => pack,
		InstallResponse::UpToDate(_) => return Err("Git installation should report Installed".into()),
	};
	let expected_install_path = dir_context
		.aipack_paths()
		.get_base_pack_installed_dir()?
		.join("git")
		.join("local-pack");

	assert_eq!(installed_pack.pack_toml.namespace, "git");
	assert_eq!(installed_pack.pack_toml.name, "local-pack");
	assert_eq!(installed_pack.pack_toml.version, "0.1.0");
	assert_eq!(installed_pack.path.as_str(), expected_install_path.as_str());
	assert!(expected_install_path.join("main.aip").exists());
	assert!(expected_install_path.join("data/fixture.txt").exists());
	assert!(!expected_install_path.join(".git").exists());
	assert!(installed_pack.size > 0);
	assert_eq!(installed_pack.zip_size, 0);
	assert!(!clone_dir.exists());

	// -- Cleanup
	remove_test_dir(dir_context.current_dir())?;

	Ok(())
}

#[tokio::test]
async fn test_installer_impl_git_missing_pack_toml_err() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_for_temp_dir().await?;
	let dir_context = runtime.dir_context();
	let repo_dir = dir_context.current_dir().join("git-source/missing-pack-toml");
	create_git_repository(
		&repo_dir,
		None,
		&[("main.aip", "A Git pack without a root manifest.")],
	)?;

	let (clone_dir, pack_uri) = clone_git_repository(dir_context, &repo_dir).await?;
	let source = PackSource::Git(clone_dir.clone());

	// -- Exec
	let result = install_git_source(dir_context, &source, &clone_dir, &pack_uri, false);

	// -- Check
	let error = match result {
		Ok(_) => return Err("Missing pack.toml should fail installation".into()),
		Err(error) => error,
	};
	match error {
		Error::FailToInstall { cause, .. } => {
			assert!(cause.contains("pack.toml"), "Unexpected installation cause: {cause}");
		}
		other => return Err(format!("Expected missing pack.toml error, got: {other:?}").into()),
	}
	assert!(!clone_dir.exists());

	// -- Cleanup
	remove_test_dir(dir_context.current_dir())?;

	Ok(())
}

#[tokio::test]
async fn test_installer_impl_git_invalid_pack_toml_err() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_for_temp_dir().await?;
	let dir_context = runtime.dir_context();
	let repo_dir = dir_context.current_dir().join("git-source/invalid-pack-toml");
	let invalid_pack_toml = r#"
[pack]
version = "0.1.0"
namespace =
name = "invalid"
"#;
	create_git_repository(
		&repo_dir,
		Some(invalid_pack_toml),
		&[("main.aip", "A Git pack with an invalid manifest.")],
	)?;

	// -- Exec
	let result = install_git_repository(dir_context, &repo_dir, false).await;

	// -- Check
	let error = result.err().ok_or("Invalid pack.toml should fail installation")?;
	match error {
		Error::FailToInstall { cause, .. } => {
			assert!(
				cause.contains("Invalid root pack.toml"),
				"Unexpected installation cause: {cause}"
			);
		}
		other => return Err(format!("Expected invalid pack.toml error, got: {other:?}").into()),
	}

	// -- Cleanup
	remove_test_dir(dir_context.current_dir())?;

	Ok(())
}

#[tokio::test]
async fn test_installer_impl_git_invalid_install_version_err() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_for_temp_dir().await?;
	let dir_context = runtime.dir_context();
	let repo_dir = dir_context.current_dir().join("git-source/invalid-version");
	let pack_toml = git_pack_toml("git", "invalid-version", "0.1.0-alpha");
	create_git_repository(&repo_dir, Some(&pack_toml), &[("main.aip", "Invalid version.")])?;

	// -- Exec
	let result = install_git_repository(dir_context, &repo_dir, false).await;

	// -- Check
	let error = result.err().ok_or("Invalid installation version should fail")?;
	match error {
		Error::InvalidPrereleaseFormat { version } => assert_eq!(version, "0.1.0-alpha"),
		other => return Err(format!("Expected invalid prerelease error, got: {other:?}").into()),
	}

	// -- Cleanup
	remove_test_dir(dir_context.current_dir())?;

	Ok(())
}

#[tokio::test]
async fn test_installer_impl_git_lower_version_err() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_for_temp_dir().await?;
	let dir_context = runtime.dir_context();

	let old_repo_dir = dir_context.current_dir().join("git-source/old-version");
	let old_pack_toml = git_pack_toml("git", "versioned-pack", "0.2.0");
	create_git_repository(&old_repo_dir, Some(&old_pack_toml), &[("main.aip", "Old version.")])?;
	let _ = install_git_repository(dir_context, &old_repo_dir, false).await?;

	let new_repo_dir = dir_context.current_dir().join("git-source/new-version");
	let new_pack_toml = git_pack_toml("git", "versioned-pack", "0.1.0");
	create_git_repository(&new_repo_dir, Some(&new_pack_toml), &[("main.aip", "New lower version.")])?;

	// -- Exec
	let result = install_git_repository(dir_context, &new_repo_dir, false).await;

	// -- Check
	let error = result.err().ok_or("Lower installed version should fail")?;
	match error {
		Error::InstallFailInstalledVersionAbove {
			installed_version,
			new_version,
		} => {
			assert_eq!(installed_version, "0.2.0");
			assert_eq!(new_version, "0.1.0");
		}
		other => return Err(format!("Expected installed version error, got: {other:?}").into()),
	}

	// -- Cleanup
	remove_test_dir(dir_context.current_dir())?;

	Ok(())
}

#[tokio::test]
async fn test_installer_impl_git_equal_version_up_to_date() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_for_temp_dir().await?;
	let dir_context = runtime.dir_context();

	let first_repo_dir = dir_context.current_dir().join("git-source/first-equal");
	let pack_toml = git_pack_toml("git", "equal-pack", "0.1.0");
	create_git_repository(&first_repo_dir, Some(&pack_toml), &[("main.aip", "First version.")])?;
	let _ = install_git_repository(dir_context, &first_repo_dir, false).await?;

	let second_repo_dir = dir_context.current_dir().join("git-source/second-equal");
	create_git_repository(
		&second_repo_dir,
		Some(&pack_toml),
		&[("main.aip", "Second source with equal version.")],
	)?;

	// -- Exec
	let result = install_git_repository(dir_context, &second_repo_dir, false).await?;

	// -- Check
	let InstallResponse::UpToDate(installed_pack) = result else {
		return Err("Equal Git version should report UpToDate".into());
	};
	assert_eq!(installed_pack.pack_toml.version, "0.1.0");
	assert_eq!(
		std::fs::read_to_string(installed_pack.path.join("main.aip").path())?,
		"First version."
	);

	// -- Cleanup
	remove_test_dir(dir_context.current_dir())?;

	Ok(())
}

#[tokio::test]
async fn test_installer_impl_git_force_replaces_equal_version() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_for_temp_dir().await?;
	let dir_context = runtime.dir_context();

	let first_repo_dir = dir_context.current_dir().join("git-source/first-force");
	let pack_toml = git_pack_toml("git", "force-pack", "0.1.0");
	create_git_repository(&first_repo_dir, Some(&pack_toml), &[("main.aip", "Original content.")])?;
	let _ = install_git_repository(dir_context, &first_repo_dir, false).await?;

	let second_repo_dir = dir_context.current_dir().join("git-source/second-force");
	create_git_repository(
		&second_repo_dir,
		Some(&pack_toml),
		&[("main.aip", "Forced replacement content.")],
	)?;

	// -- Exec
	let result = install_git_repository(dir_context, &second_repo_dir, true).await?;

	// -- Check
	let installed_pack = match result {
		InstallResponse::Installed(pack) => pack,
		InstallResponse::UpToDate(_) => return Err("Forced Git installation should report Installed".into()),
	};
	assert_eq!(
		std::fs::read_to_string(installed_pack.path.join("main.aip").path())?,
		"Forced replacement content."
	);

	// -- Cleanup
	remove_test_dir(dir_context.current_dir())?;

	Ok(())
}

#[tokio::test]
async fn test_installer_impl_git_clone_failure_cleans_up() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_for_temp_dir().await?;
	let dir_context = runtime.dir_context();
	let download_dir = dir_context.aipack_paths().get_base_pack_download_dir()?;
	let missing_repo_dir = dir_context.current_dir().join("git-source/missing-repository");
	let pack_uri = crate::exec::packer::support::PackUri::GitLink(crate::exec::packer::support::GitSource {
		repository: missing_repo_dir.as_str().to_string(),
		subpath: None,
	});

	// -- Exec
	let result = crate::exec::packer::support::clone_from_git(dir_context, pack_uri).await;

	// -- Check
	assert!(result.is_err(), "Cloning a missing repository should fail");
	let clone_count = if download_dir.exists() {
		std::fs::read_dir(download_dir.path())?.count()
	} else {
		0
	};
	assert_eq!(clone_count, 0, "Failed Git clones should be cleaned up");

	// -- Cleanup
	remove_test_dir(dir_context.current_dir())?;

	Ok(())
}

#[tokio::test]
async fn test_installer_impl_git_nested_pack_requires_root_manifest() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_for_temp_dir().await?;
	let dir_context = runtime.dir_context();
	let repo_dir = dir_context.current_dir().join("git-source/nested-pack");
	let nested_pack_toml = git_pack_toml("nested", "pack", "0.1.0");
	let nested_files = [
		("nested/pack.toml", nested_pack_toml.as_str()),
		("nested/main.aip", "Nested Git pack."),
	];
	create_git_repository(&repo_dir, None, &nested_files)?;

	let (clone_dir, pack_uri) = clone_git_repository(dir_context, &repo_dir).await?;
	let source = PackSource::Git(clone_dir.clone());
	let installed_dir = dir_context.aipack_paths().get_base_pack_installed_dir()?;

	// -- Exec
	let result = install_git_source(dir_context, &source, &clone_dir, &pack_uri, false);

	// -- Check
	let error = result.err().ok_or("Nested-only pack should fail installation")?;
	match error {
		Error::FailToInstall { cause, .. } => {
			assert!(cause.contains("pack.toml"), "Unexpected installation cause: {cause}");
		}
		other => return Err(format!("Expected root manifest error, got: {other:?}").into()),
	}
	assert!(!installed_dir.join("nested").join("pack").exists());
	assert!(!clone_dir.exists());

	// -- Cleanup
	remove_test_dir(dir_context.current_dir())?;

	Ok(())
}

#[tokio::test]
async fn test_installer_impl_git_selected_subdirectory_installation() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_for_temp_dir().await?;
	let dir_context = runtime.dir_context();
	let repo_dir = dir_context.current_dir().join("git-source/selected-pack");
	let selected_pack_toml = git_pack_toml("selected", "nested-pack", "0.1.0");
	create_git_repository(
		&repo_dir,
		None,
		&[
			("packs/nested/pack.toml", selected_pack_toml.as_str()),
			("packs/nested/main.aip", "Selected Git pack."),
			("packs/nested/data/fixture.txt", "Selected fixture data"),
		],
	)?;

	let (clone_dir, pack_uri) = clone_git_repository_with_subpath(dir_context, &repo_dir, "packs/nested").await?;
	let source = PackSource::Git(clone_dir.clone());

	// -- Exec
	let result = install_git_source(dir_context, &source, &clone_dir, &pack_uri, false)?;

	// -- Check
	let installed_pack = match result {
		InstallResponse::Installed(pack) => pack,
		InstallResponse::UpToDate(_) => return Err("Selected Git installation should report Installed".into()),
	};
	let expected_install_path = dir_context
		.aipack_paths()
		.get_base_pack_installed_dir()?
		.join("selected")
		.join("nested-pack");
	assert_eq!(installed_pack.pack_toml.namespace, "selected");
	assert_eq!(installed_pack.pack_toml.name, "nested-pack");
	assert_eq!(installed_pack.path.as_str(), expected_install_path.as_str());
	assert_eq!(
		std::fs::read_to_string(expected_install_path.join("main.aip").path())?,
		"Selected Git pack."
	);
	assert_eq!(
		std::fs::read_to_string(expected_install_path.join("data/fixture.txt").path())?,
		"Selected fixture data"
	);
	assert!(!expected_install_path.join(".git").exists());
	assert!(installed_pack.size > 0);
	assert_eq!(installed_pack.zip_size, 0);
	assert!(!clone_dir.exists());

	// -- Cleanup
	remove_test_dir(dir_context.current_dir())?;

	Ok(())
}

#[tokio::test]
async fn test_installer_impl_git_selected_directory_missing_err() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_for_temp_dir().await?;
	let dir_context = runtime.dir_context();
	let repo_dir = dir_context.current_dir().join("git-source/missing-selected-pack");
	create_git_repository(&repo_dir, None, &[("README.md", "No selected pack.")])?;
	let (clone_dir, pack_uri) =
		clone_git_repository_with_subpath(dir_context, &repo_dir, "packs/missing").await?;
	let source = PackSource::Git(clone_dir.clone());

	// -- Exec
	let result = install_git_source(dir_context, &source, &clone_dir, &pack_uri, false);

	// -- Check
	let error = result.err().ok_or("Missing selected Git directory should fail installation")?;
	match error {
		Error::FailToInstall { cause, .. } => {
			assert!(cause.contains("Git pack directory"), "Unexpected installation cause: {cause}");
		}
		other => return Err(format!("Expected missing Git directory error, got: {other:?}").into()),
	}
	assert!(!clone_dir.exists());

	// -- Cleanup
	remove_test_dir(dir_context.current_dir())?;

	Ok(())
}

#[tokio::test]
async fn test_installer_impl_git_selected_directory_missing_manifest_err() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_for_temp_dir().await?;
	let dir_context = runtime.dir_context();
	let repo_dir = dir_context.current_dir().join("git-source/missing-selected-manifest");
	create_git_repository(
		&repo_dir,
		None,
		&[("packs/no-manifest/main.aip", "The selected pack has no manifest.")],
	)?;
	let (clone_dir, pack_uri) =
		clone_git_repository_with_subpath(dir_context, &repo_dir, "packs/no-manifest").await?;
	let source = PackSource::Git(clone_dir.clone());

	// -- Exec
	let result = install_git_source(dir_context, &source, &clone_dir, &pack_uri, false);

	// -- Check
	let error = result.err().ok_or("Missing selected Git manifest should fail installation")?;
	match error {
		Error::FailToInstall { cause, .. } => {
			assert!(cause.contains("pack.toml"), "Unexpected installation cause: {cause}");
		}
		other => return Err(format!("Expected missing Git manifest error, got: {other:?}").into()),
	}
	assert!(!clone_dir.exists());

	// -- Cleanup
	remove_test_dir(dir_context.current_dir())?;

	Ok(())
}

#[tokio::test]
async fn test_installer_impl_git_selected_directory_invalid_manifest_err() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_for_temp_dir().await?;
	let dir_context = runtime.dir_context();
	let repo_dir = dir_context.current_dir().join("git-source/invalid-selected-manifest");
	let invalid_pack_toml = r#"
[pack]
version = "0.1.0"
namespace =
name = "invalid"
"#;
	create_git_repository(
		&repo_dir,
		None,
		&[
			("packs/invalid/pack.toml", invalid_pack_toml),
			("packs/invalid/main.aip", "The selected pack has an invalid manifest."),
		],
	)?;
	let (clone_dir, pack_uri) = clone_git_repository_with_subpath(dir_context, &repo_dir, "packs/invalid").await?;
	let source = PackSource::Git(clone_dir.clone());

	// -- Exec
	let result = install_git_source(dir_context, &source, &clone_dir, &pack_uri, false);

	// -- Check
	let error = result.err().ok_or("Invalid selected Git manifest should fail installation")?;
	match error {
		Error::FailToInstall { cause, .. } => {
			assert!(cause.contains("Invalid root pack.toml"), "Unexpected installation cause: {cause}");
		}
		other => return Err(format!("Expected invalid Git manifest error, got: {other:?}").into()),
	}
	assert!(!clone_dir.exists());

	// -- Cleanup
	remove_test_dir(dir_context.current_dir())?;

	Ok(())
}

// region:    --- Support

fn git_pack_toml(namespace: &str, name: &str, version: &str) -> String {
	format!(
		"[pack]\nnamespace = \"{namespace}\"\nname = \"{name}\"\nversion = \"{version}\"\n"
	)
}

fn create_git_repository(
	repo_dir: &SPath,
	pack_toml: Option<&str>,
	files: &[(&str, &str)],
) -> Result<()> {
	ensure_dir(repo_dir)?;

	if let Some(pack_toml) = pack_toml {
		save_file_content(&repo_dir.join("pack.toml"), pack_toml)?;
	}

	for &(path, content) in files {
		save_file_content(&repo_dir.join(path), content)?;
	}

	run_git_command(repo_dir, &["init"])?;
	run_git_command(repo_dir, &["config", "user.name", "AIPack Test"])?;
	run_git_command(repo_dir, &["config", "user.email", "aipack-test@example.invalid"])?;
	run_git_command(repo_dir, &["add", "--all"])?;
	run_git_command(repo_dir, &["commit", "-m", "Add Git pack fixture"])?;

	Ok(())
}

async fn clone_git_repository(
	dir_context: &crate::dir_context::DirContext,
	repo_dir: &SPath,
) -> crate::Result<(SPath, crate::exec::packer::support::PackUri)> {
	let pack_uri = crate::exec::packer::support::PackUri::GitLink(crate::exec::packer::support::GitSource {
		repository: repo_dir.as_str().to_string(),
		subpath: None,
	});
	let (clone_dir, _) = crate::exec::packer::support::clone_from_git(dir_context, pack_uri.clone()).await?;
	Ok((clone_dir, pack_uri))
}

async fn clone_git_repository_with_subpath(
	dir_context: &crate::dir_context::DirContext,
	repo_dir: &SPath,
	subpath: &str,
) -> crate::Result<(SPath, crate::exec::packer::support::PackUri)> {
	let pack_uri = crate::exec::packer::support::PackUri::GitLink(crate::exec::packer::support::GitSource {
		repository: repo_dir.as_str().to_string(),
		subpath: Some(subpath.to_string()),
	});
	let (clone_dir, _) = crate::exec::packer::support::clone_from_git(dir_context, pack_uri.clone()).await?;
	Ok((clone_dir, pack_uri))
}

async fn install_git_repository(
	dir_context: &crate::dir_context::DirContext,
	repo_dir: &SPath,
	force: bool,
) -> crate::Result<InstallResponse> {
	let (clone_dir, pack_uri) = clone_git_repository(dir_context, repo_dir).await?;
	let source = PackSource::Git(clone_dir.clone());

	install_git_source(dir_context, &source, &clone_dir, &pack_uri, force)
}

fn run_git_command(repo_dir: &SPath, args: &[&str]) -> Result<()> {
	let output = std::process::Command::new("git")
		.args(args)
		.current_dir(repo_dir.path())
		.output()?;

	if !output.status.success() {
		return Err(format!(
			"Git command {args:?} failed: {}",
			String::from_utf8_lossy(&output.stderr)
		)
		.into());
	}

	Ok(())
}

// endregion: --- Support
