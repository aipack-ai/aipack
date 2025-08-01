use crate::Result;
use crate::dir_context::{AipackBaseDir, AipackPaths, DirContext};
use crate::exec::init::assets;
use crate::hub::get_hub;
use crate::support::AsStrsExt;
use crate::support::files::safer_delete_dir;
use simple_fs::{SPath, ensure_dir};
use std::fs::write;
use std::io::BufRead;

/// Init the aipack base if needed,
/// and return a DirContext without wks dir (even if present)
/// NOTE: This is just for the commands that do not requires wks
pub async fn init_base_and_dir_context(force: bool) -> Result<DirContext> {
	init_base(force).await?;
	let aipack_paths = AipackPaths::new()?;
	let dir_context = DirContext::new(aipack_paths)?;
	Ok(dir_context)
}

/// `force` - except for `config.toml`
pub async fn init_base(force: bool) -> Result<()> {
	let hub = get_hub();
	// -- Check that the home dir exists

	let mut new = false;
	// -- Create the missing folders

	let base_dir = AipackBaseDir::new()?;
	if ensure_dir(base_dir.path())? {
		new = true;
	}

	// -- Determine version
	let is_new_version = check_is_new_version(&base_dir).await?;
	// if new version, then, force update
	let force = is_new_version || force;

	if force {
		clean_legacy_base_content(&base_dir).await?;
	}

	if new {
		hub.publish(format!("\n=== {} '{}'", "Initializing ~/.aipack-base at", &*base_dir))
			.await;
	} else if force {
		hub.publish(format!("\n=== {} '{}'", "Updating ~/.aipack-base at", &*base_dir))
			.await;
		if is_new_version {
			hub.publish("(needed because new aipack version)").await;
		}
	}

	// -- Create the config file (only if not present)
	// Note: For now, "config.toml" is not force, no matter what
	let config_path = base_dir.join("config.toml");
	if !config_path.exists() {
		let config_zfile = assets::extract_base_config_toml_zfile()?;
		write(&config_path, config_zfile.content)?;
		hub.publish(format!(
			"-> {label:<18} '{path}'",
			label = "Create config file",
			path = config_path
		))
		.await;
	}

	// -- Init the doc
	let doc_paths = assets::extract_base_doc_file_paths()?;
	assets::update_files("base", &base_dir, &doc_paths.x_as_strs(), force).await?;

	// -- Init the pack path
	let pack_paths = assets::extract_base_pack_file_paths()?;
	assets::update_files("base", &base_dir, &pack_paths.x_as_strs(), force).await?;

	// -- Display message
	if new || force {
		hub.publish("=== DONE\n").await;
	}

	Ok(())
}

/// Check is the `.aipack/version.txt` is present,
/// - read the first line, and compare with current version
/// - if match current version all good.
/// - if not recreate file with version,
async fn check_is_new_version(base_dir: &SPath) -> Result<bool> {
	let version_path = base_dir.join("version.txt");

	let mut is_new = true;

	// -- If exists, determine if is_new
	if version_path.exists() {
		// read efficiently only the first line of  version_path
		let mut reader = simple_fs::get_buf_reader(&version_path)?;
		let mut first_line = String::new();
		if reader.read_line(&mut first_line)? > 0 {
			let version_in_file = first_line.trim();
			is_new = version_in_file != crate::VERSION;
		}
	}

	// -- If is_new, rereate the file
	if is_new {
		let content = format!(
			r#"{}

DO NOT EDIT.

This file is used to keep track of the version and compare it during each `aip ...` execution.
If there is no match with the current version, this file will be recreated, and the documentation and other files will be updated.
		"#,
			crate::VERSION
		);
		write(&version_path, content)?;
		get_hub()
			.publish(format!(
				"-> {label:<18} '{path}'",
				label = "Create file",
				path = version_path
			))
			.await;
	}

	Ok(is_new)
}

async fn clean_legacy_base_content(aipack_base_dir: &SPath) -> Result<bool> {
	if !aipack_base_dir.as_str().contains(".aipack-base") {
		return Err(format!(
			"This dir '{aipack_base_dir}' does not see to be a aipack base dir, cannot clean legacy content"
		)
		.into());
	}

	let mut change = false;

	// -- clean the old ~aipack-base/doc
	let doc_path = aipack_base_dir.join("doc");
	if doc_path.exists() {
		let is_delete = safer_delete_dir(&doc_path)?;
		if is_delete {
			get_hub()
				.publish("-> legacy '~/.aipack-base/doc' dir deleted (now part of 'core@doc')".to_string())
				.await;
		}

		change |= is_delete;
	}

	Ok(change)
}
