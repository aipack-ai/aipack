use crate::{Error, Result};
use semver::Version;
use simple_fs::SPath;
use std::env::consts::{ARCH, OS};
use std::fs;

// region:    --- Bin Path Resolver

const BASE_STABLE_URL: &str = "https://repo.aipack.ai/aip-dist/stable";
const BASE_STABLE_LATEST_URL: &str = "https://repo.aipack.ai/aip-dist/stable/latest";

/// Returns the stable URL for the `aip` binary archive based on the current OS and architecture.
pub fn get_aip_stable_url(version: Option<&Version>) -> Result<String> {
	let target_os = OS;
	let target_arch = ARCH;

	let target_triple = match (target_os, target_arch) {
		("macos", "aarch64") => "aarch64-apple-darwin",
		("macos", "x86_64") => "x86_64-apple-darwin",
		("linux", "x86_64") => "x86_64-unknown-linux-gnu",
		("linux", "aarch64") => "aarch64-unknown-linux-gnu",
		("windows", "x86_64") => "x86_64-pc-windows-msvc",
		("windows", "aarch64") => "aarch64-pc-windows-msvc",
		_ => {
			return Err(Error::custom(format!(
				"Unsupported OS/Architecture combination: {target_os}/{target_arch}"
			)));
		}
	};

	let url = if let Some(version) = version {
		format!("{BASE_STABLE_URL}/v{}/{}/aip.tar.gz", version, target_triple)
	} else {
		format!("{BASE_STABLE_LATEST_URL}/{}/aip.tar.gz", target_triple)
	};

	Ok(url)
}

// endregion: --- Bin Path Resolver

// region:    --- File Update

pub(super) fn atomic_replace(src: &SPath, dest: &SPath) -> Result<()> {
	fs::rename(src, dest).map_err(|err| {
		Error::custom(format!(
			"Failed to replace '{}' with '{}'. Cause: {}.\n\
					 On Windows, make sure all 'aip' processes are terminated before updating.",
			dest, src, err
		))
	})?;
	Ok(())
}

// endregion: --- File Update
