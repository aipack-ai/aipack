use crate::_test_support::{gen_test_dir_path, remove_test_dir};
use crate::exec::packer::{self};
use crate::runtime::Runtime;
use crate::support::zip::{extract_text_content, list_entries_with_globs};
use simple_fs::SPath;
use std::fs;

pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

#[tokio::test]
async fn test_packer_impl_pack_simple() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_for_temp_dir().await?;
	let dir_context = runtime.dir_context();
	let to_pack_dir = SPath::new("tests-data/test_packs_folder/test_pack_01");

	// -- Exec
	let pack_result = packer::pack_dir(to_pack_dir, dir_context.current_dir())?;

	// -- Check
	// Verify that the pack file was created with correct structure
	verify_aipack_file(&pack_result.pack_file)?;

	// Verify pack information is correct
	assert_eq!(pack_result.pack_toml.namespace, "test");
	assert_eq!(pack_result.pack_toml.name, "test_pack_01");
	assert_eq!(pack_result.pack_toml.version, "0.1.0");

	// Verify the filename follows the expected pattern
	let filename = pack_result.pack_file.name();
	assert!(
		filename.starts_with("test@test_pack_01-v0.1.0"),
		"Unexpected filename: {filename}"
	);

	// -- Cleanup
	remove_test_dir(dir_context.current_dir())?;

	Ok(())
}

#[test]
fn test_packer_impl_pack_filters_default_excludes() -> Result<()> {
	// -- Setup & Fixtures
	let root = gen_test_dir_path();
	let pack_dir = root.join("pack");
	let dest_dir = root.join("dest");
	let test_files = [
		(
			"pack.toml",
			"namespace = \"test\"\nname = \"filtered_pack\"\nversion = \"0.1.0\"\n",
		),
		("root.txt", "root file"),
		("nested/child.txt", "nested file"),
		(".aipack/generated.txt", "excluded"),
		(".git/config", "excluded"),
		("target/debug/app", "excluded"),
		("node_modules/package/index.js", "excluded"),
		(".build/generated/output.txt", "excluded"),
		("__pycache__/module.pyc", "excluded"),
		(".DS_Store", "excluded"),
		("nested/.DS_Store", "excluded"),
		("Thumbs.db", "excluded"),
		("nested/Thumbs.db", "excluded"),
		("root.swp", "excluded"),
		("nested/child.swp", "excluded"),
	];

	for (relative_path, content) in test_files {
		write_pack_test_file(&pack_dir, relative_path, content)?;
	}

	// -- Exec
	let pack_result = packer::pack_dir(&pack_dir, &dest_dir)?;
	let entries = list_entries_with_globs(&pack_result.pack_file, None::<&[String]>)?;

	// -- Check
	verify_aipack_file(&pack_result.pack_file)?;
	assert!(entries.iter().any(|entry| entry == "pack.toml"));
	assert!(entries.iter().any(|entry| entry == "root.txt"));
	assert!(entries.iter().any(|entry| entry == "nested/child.txt"));
	assert_eq!(extract_text_content(&pack_result.pack_file, "root.txt")?, "root file");
	assert_eq!(
		extract_text_content(&pack_result.pack_file, "nested/child.txt")?,
		"nested file"
	);
	assert!(entries.iter().all(|entry| !entry.contains('\\')));
	assert!(entries.iter().all(|entry| !entry.starts_with('/')));
	assert!(entries.iter().all(|entry| !entry.contains(pack_dir.as_str())));

	for excluded_dir in [".aipack/", ".git/", "target/", "node_modules/", ".build/", "__pycache__/"] {
		assert!(
			!entries
				.iter()
				.any(|entry| entry == excluded_dir || entry.starts_with(excluded_dir)),
			"Excluded directory was archived: {excluded_dir}"
		);
	}

	for excluded_file in [
		".DS_Store",
		"nested/.DS_Store",
		"Thumbs.db",
		"nested/Thumbs.db",
		"root.swp",
		"nested/child.swp",
	] {
		assert!(
			!entries.iter().any(|entry| entry == excluded_file),
			"Excluded file was archived: {excluded_file}"
		);
	}

	assert_eq!(pack_result.pack_toml.namespace, "test");
	assert_eq!(pack_result.pack_toml.name, "filtered_pack");
	assert_eq!(pack_result.pack_toml.version, "0.1.0");
	assert_eq!(pack_result.pack_file.name(), "test@filtered_pack-v0.1.0.aipack");

	// -- Cleanup
	remove_test_dir(&root)?;

	Ok(())
}

// region:    --- Support

// Test helper to verify the structure of a created .aipack file
fn verify_aipack_file(aipack_path: &SPath) -> Result<()> {
	// Check that the file exists
	assert!(
		aipack_path.exists(),
		"The .aipack file was not created at {aipack_path}"
	);

	// Check that it has the correct extension
	assert_eq!(aipack_path.ext(), "aipack", "The file does not have .aipack extension");

	// Check that the file size is reasonable (greater than a minimal size)
	let metadata = fs::metadata(aipack_path.path())?;
	assert!(
		metadata.len() > 100,
		"The .aipack file is too small: {bytes} bytes",
		bytes = metadata.len()
	);

	Ok(())
}

fn write_pack_test_file(pack_dir: &SPath, relative_path: &str, content: &str) -> Result<()> {
	let path = pack_dir.join(relative_path);
	if let Some(parent) = path.parent() {
		fs::create_dir_all(parent.path())?;
	}
	fs::write(path.path(), content)?;
	Ok(())
}

// endregion: --- Support
