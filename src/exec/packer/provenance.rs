use crate::support::time;
use crate::{Error, Result};
use simple_fs::SPath;
use toml_edit::{DocumentMut, Item, Table, value};

#[allow(dead_code)]
pub(super) fn write_installation_provenance(pack_toml_path: &SPath, source: &str) -> Result<()> {
	let installed_time = time::now_rfc3339_local_sec()?;
	update_installation_provenance_with_commit(pack_toml_path, &installed_time, source, None)
}

#[allow(dead_code)]
pub(super) fn write_installation_provenance_with_commit(
	pack_toml_path: &SPath,
	source: &str,
	commit: &str,
) -> Result<()> {
	let installed_time = time::now_rfc3339_local_sec()?;
	update_installation_provenance_with_commit(pack_toml_path, &installed_time, source, Some(commit))
}

#[allow(dead_code)]
fn update_installation_provenance(pack_toml_path: &SPath, installed_time: &str, source: &str) -> Result<()> {
	update_installation_provenance_with_commit(pack_toml_path, installed_time, source, None)
}

fn update_installation_provenance_with_commit(
	pack_toml_path: &SPath,
	installed_time: &str,
	source: &str,
	commit: Option<&str>,
) -> Result<()> {
	let content = std::fs::read_to_string(pack_toml_path.path())?;
	let mut document = content.parse::<DocumentMut>().map_err(|error| {
		Error::custom(format!(
			"Failed to parse pack.toml '{}': {error}",
			pack_toml_path.as_str()
		))
	})?;

	let installed = document
		.entry("installed")
		.or_insert(Item::Table(Table::new()))
		.as_table_mut()
		.ok_or_else(|| {
			Error::custom(format!(
				"The installed section in '{}' is not a TOML table",
				pack_toml_path.as_str()
			))
		})?;

	installed["time"] = value(installed_time);
	installed["source"] = value(source);
	if let Some(commit) = commit {
		installed["commit"] = value(commit);
	}

	std::fs::write(pack_toml_path.path(), document.to_string())?;

	Ok(())
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	use super::*;
	use simple_fs::{SPath, ensure_dir};

	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	#[test]
	fn test_packer_provenance_write_insertion() -> Result<()> {
		// -- Setup & Fixtures
		let test_dir = SPath::from("tests-data/.tmp/test_packer_provenance_write_insertion");
		ensure_dir(&test_dir)?;
		let pack_toml_path = test_dir.join("pack.toml");
		std::fs::write(
			pack_toml_path.path(),
			r#"# Pack comment
[pack]
version = "1.0.0" # Version comment
namespace = "test"
name = "pack"
"#,
		)?;

		// -- Exec
		update_installation_provenance(&pack_toml_path, "2026-08-04T14:01:56-07:00", "aipack.ai")?;

		// -- Check
		let updated = std::fs::read_to_string(pack_toml_path.path())?;
		assert!(updated.contains("# Pack comment"));
		assert!(updated.contains("version = \"1.0.0\" # Version comment"));
		assert!(updated.contains("[installed]"));
		assert!(updated.contains("time = \"2026-08-04T14:01:56-07:00\""));
		assert!(updated.contains("source = \"aipack.ai\""));

		// -- Cleanup
		// std::fs::remove_dir_all(test_dir.path())?;

		Ok(())
	}

	#[test]
	fn test_packer_provenance_write_replacement() -> Result<()> {
		// -- Setup & Fixtures
		let test_dir = SPath::from("tests-data/.tmp/test_packer_provenance_write_replacement");
		ensure_dir(&test_dir)?;
		let pack_toml_path = test_dir.join("pack.toml");
		std::fs::write(
			pack_toml_path.path(),
			r#"# Pack comment
[pack]
version = "1.0.0"
namespace = "test"
name = "pack"

# Installed comment
[installed]
legacy = "keep" # Legacy comment
time = "old-time"
source = "old-source"
"#,
		)?;

		// -- Exec
		update_installation_provenance(&pack_toml_path, "2026-08-04T14:05:54-07:00", "https://example.com/pack.aipack")?;

		// -- Check
		let updated = std::fs::read_to_string(pack_toml_path.path())?;
		let document = updated.parse::<DocumentMut>()?;
		assert_eq!(installed_value(&document, "time")?, "2026-08-04T14:05:54-07:00");
		assert_eq!(installed_value(&document, "source")?, "https://example.com/pack.aipack");
		assert_eq!(installed_value(&document, "legacy")?, "keep");
		assert!(updated.contains("# Pack comment"));
		assert!(updated.contains("# Installed comment"));
		assert!(updated.contains("# Legacy comment"));

		// -- Cleanup
		// std::fs::remove_dir_all(test_dir.path())?;

		Ok(())
	}

	#[test]
	fn test_packer_provenance_write_commit() -> Result<()> {
		// -- Setup & Fixtures
		let test_dir = SPath::from("tests-data/.tmp/test_packer_provenance_write_commit");
		ensure_dir(&test_dir)?;
		let pack_toml_path = test_dir.join("pack.toml");
		std::fs::write(
			pack_toml_path.path(),
			r#"# Pack comment
[pack]
version = "1.0.0"
namespace = "test"
name = "pack"
"#,
		)?;

		// -- Exec
		update_installation_provenance_with_commit(
			&pack_toml_path,
			"2026-08-04T14:05:54-07:00",
			"git://example.com/pack.git",
			Some("0123456789abcdef"),
		)?;

		// -- Check
		let updated = std::fs::read_to_string(pack_toml_path.path())?;
		let document = updated.parse::<DocumentMut>()?;
		assert_eq!(installed_value(&document, "commit")?, "0123456789abcdef");
		assert!(updated.contains("# Pack comment"));

		// -- Cleanup
		// std::fs::remove_dir_all(test_dir.path())?;

		Ok(())
	}

	#[test]
	fn test_packer_provenance_write_preserves_unrelated_fields() -> Result<()> {
		// -- Setup & Fixtures
		let test_dir = SPath::from("tests-data/.tmp/test_packer_provenance_write_preserves_unrelated_fields");
		ensure_dir(&test_dir)?;
		let pack_toml_path = test_dir.join("pack.toml");
		std::fs::write(
			pack_toml_path.path(),
			r#"# Pack comment
[pack]
version = "1.0.0"
namespace = "test"
name = "pack"

# Metadata comment
[metadata]
description = "Keep this metadata"
"#,
		)?;

		// -- Exec
		update_installation_provenance(&pack_toml_path, "2026-08-04T14:05:54-07:00", "git://example.com/pack.git")?;

		// -- Check
		let updated = std::fs::read_to_string(pack_toml_path.path())?;
		let document = updated.parse::<DocumentMut>()?;
		assert_eq!(
			document
				.get("metadata")
				.and_then(|item| item.as_table())
				.and_then(|table| table.get("description"))
				.and_then(|item| item.as_str())
				.ok_or_else(|| "Missing metadata.description".to_string())?,
			"Keep this metadata"
		);
		assert!(updated.contains("# Pack comment"));
		assert!(updated.contains("# Metadata comment"));
		assert!(updated.contains("[installed]"));
		assert!(updated.contains("source = \"git://example.com/pack.git\""));

		// -- Cleanup
		// std::fs::remove_dir_all(test_dir.path())?;

		Ok(())
	}

	// region:    --- Support

	fn installed_value<'a>(document: &'a DocumentMut, key: &str) -> Result<&'a str> {
		let installed = document
			.get("installed")
			.and_then(|item| item.as_table())
			.ok_or_else(|| "Missing [installed] section".to_string())?;

		installed
			.get(key)
			.and_then(|item| item.as_str())
			.ok_or_else(|| format!("Missing installed.{key}"))
			.map_err(Into::into)
	}

	// endregion: --- Support
}

// endregion: --- Tests
