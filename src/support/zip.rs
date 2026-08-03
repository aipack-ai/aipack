use crate::{Error, Result};
use simple_fs::{SPath, get_glob_set};
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, Read as _};
use std::path::Path;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

/// Creates a zip archive from the directory `src_dir` and writes it to `dest_file`.
///
/// `src_dir` is the directory to be zipped.
/// `dest_file` is the destination file path for the zip archive.
///
/// This function recursively adds files and subdirectories from `src_dir` to the zip archive.
#[cfg_attr(not(test), expect(dead_code))]
pub fn zip_dir(src_dir: impl AsRef<SPath>, dest_file: impl AsRef<SPath>) -> Result<()> {
	zip_dir_with_globs(src_dir, dest_file, None::<&[String]>)
}

/// Creates a zip archive from the directory `src_dir` and writes it to `dest_file`,
/// optionally filtering source files by relative archive-style glob paths.
///
/// Directory entries are still emitted as needed for traversed directories, but file entries
/// are included only when `globs` is omitted, empty, or the relative archive path matches
/// at least one glob pattern.
pub fn zip_dir_with_globs(
	src_dir: impl AsRef<SPath>,
	dest_file: impl AsRef<SPath>,
	globs: Option<impl AsRef<[String]>>,
) -> Result<()> {
	let src_dir = src_dir.as_ref();
	let dest_file = dest_file.as_ref();

	if !src_dir.exists() {
		return Err(Error::ZipFail {
			zip_dir: src_dir.to_string(),
			cause: format!("Fail to zip directory. Source directory does not exist: '{src_dir}'"),
		});
	}
	if !src_dir.is_dir() {
		return Err(Error::ZipFail {
			zip_dir: src_dir.to_string(),
			cause: format!("Fail to zip directory. Source path is not a directory: '{src_dir}'"),
		});
	}

	// Create the destination zip file.
	let file = File::create(dest_file)?;
	let mut zip = ZipWriter::new(file);

	// Set default options with deflated compression (most standard).
	let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

	// Walk through the directory.
	for entry in WalkDir::new(src_dir) {
		let entry = entry.map_err(|err| Error::ZipFail {
			zip_dir: src_dir.to_string(),
			cause: format!("Fail to zip directory. Error on entry. Cause {err}"),
		})?;
		let Ok(path) = SPath::from_std_path(entry.path()) else {
			continue;
		};

		let relative_path = path.strip_prefix(src_dir).map_err(|err| Error::ZipFail {
			zip_dir: src_dir.to_string(),
			cause: format!("Fail strip_prefix '{src_dir}' on '{path}'. Cause {err}"),
		})?;

		// Convert relative path to UTF-8 string and normalize slashes for cross-platform compatibility.
		let name = relative_path.as_str().replace("\\", "/");

		if name.is_empty() {
			continue;
		}

		if path.is_dir() {
			// Add directory entry to zip archive.
			// Ensure directory name ends with '/'.
			let dir_name = if name.ends_with('/') {
				name.to_string()
			} else {
				format!("{name}/")
			};
			zip.add_directory(&dir_name, options).map_err(|err| Error::ZipFail {
				zip_dir: src_dir.to_string(),
				cause: format!("Fail add directory '{dir_name}'. Cause {err}"),
			})?;
		} else {
			if !matches_zip_globs(&name, globs.as_ref())? {
				continue;
			}

			// Add file entry to zip archive.
			zip.start_file(&name, options).map_err(|err| Error::ZipFail {
				zip_dir: src_dir.to_string(),
				cause: format!("Fail zip.start_file '{name}'. Cause {err}"),
			})?;
			let mut f = File::open(path)?;
			io::copy(&mut f, &mut zip)?;
		}
	}

	zip.finish().map_err(|err| Error::ZipFail {
		zip_dir: src_dir.to_string(),
		cause: format!("Fail zip.finish '{src_dir}'. Cause {err}"),
	})?;
	Ok(())
}

/// Creates a zip archive from selected files beneath `src_dir` and writes it to `dest_file`.
///
/// Selected files are stored using paths relative to `src_dir`. Only parent directories required
/// by selected files are emitted as directory entries.
pub fn zip_files<I, P>(src_dir: impl AsRef<SPath>, dest_file: impl AsRef<SPath>, files: I) -> Result<()>
where
	I: IntoIterator<Item = P>,
	P: AsRef<SPath>,
{
	let src_dir = src_dir.as_ref();
	let dest_file = dest_file.as_ref();

	if !src_dir.exists() {
		return Err(Error::ZipFail {
			zip_dir: src_dir.to_string(),
			cause: format!("Fail to zip selected files. Source directory does not exist: '{src_dir}'"),
		});
	}
	if !src_dir.is_dir() {
		return Err(Error::ZipFail {
			zip_dir: src_dir.to_string(),
			cause: format!("Fail to zip selected files. Source path is not a directory: '{src_dir}'"),
		});
	}

	let mut selected_files = Vec::new();
	for file in files {
		let file = file.as_ref();
		let relative_path = file.strip_prefix(src_dir).map_err(|err| Error::ZipFail {
			zip_dir: src_dir.to_string(),
			cause: format!("Selected file '{file}' is outside source directory '{src_dir}'. Cause: {err}"),
		})?;
		let name = relative_path.as_str().replace('\\', "/");

		if name.is_empty()
			|| name == "."
			|| name.starts_with('/')
			|| name.split('/').any(|component| component == "..")
			|| (name.len() >= 2 && name.as_bytes()[1] == b':' && name.as_bytes()[0].is_ascii_alphabetic())
		{
			return Err(Error::ZipFail {
				zip_dir: src_dir.to_string(),
				cause: format!("Selected file '{file}' has an unsafe relative archive path: '{name}'"),
			});
		}

		selected_files.push((file.as_std_path().to_path_buf(), name));
	}

	let file = File::create(dest_file)?;
	let mut zip = ZipWriter::new(file);
	let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

	let mut directory_names = HashSet::new();
	let mut directories = Vec::new();
	for (_, name) in &selected_files {
		let components = name.split('/').collect::<Vec<_>>();
		for index in 1..components.len() {
			let dir_name = format!("{}/", components[..index].join("/"));
			if directory_names.insert(dir_name.clone()) {
				directories.push(dir_name);
			}
		}
	}

	for dir_name in directories {
		zip.add_directory(&dir_name, options).map_err(|err| Error::ZipFail {
			zip_dir: src_dir.to_string(),
			cause: format!("Fail add directory '{dir_name}'. Cause {err}"),
		})?;
	}

	for (path, name) in selected_files {
		zip.start_file(&name, options).map_err(|err| Error::ZipFail {
			zip_dir: src_dir.to_string(),
			cause: format!("Fail zip.start_file '{name}'. Cause {err}"),
		})?;
		let mut source_file = File::open(&path)?;
		io::copy(&mut source_file, &mut zip)?;
	}

	zip.finish().map_err(|err| Error::ZipFail {
		zip_dir: src_dir.to_string(),
		cause: format!("Fail zip.finish '{src_dir}'. Cause {err}"),
	})?;
	Ok(())
}

/// Extracts the zip archive from `src_zip` into the directory `dest_dir`.
///
/// `src_zip` is the path to the zip archive.
/// `dest_dir` is the destination directory where the contents of the zip will be extracted.
pub fn unzip_file(src_zip: impl AsRef<SPath>, dest_dir: impl AsRef<SPath>) -> Result<()> {
	unzip_file_with_entries(src_zip, dest_dir).map(|_| ())
}

/// Extracts the zip archive from `src_zip` into the directory `dest_dir` and returns
/// the extracted file paths relative to `dest_dir`, preserving archive order.
///
/// Directory-only entries are not included in the returned list.
pub fn unzip_file_with_entries(src_zip: impl AsRef<SPath>, dest_dir: impl AsRef<SPath>) -> Result<Vec<String>> {
	unzip_file_with_entries_and_globs(src_zip, dest_dir, None::<&[String]>)
}

/// Extracts the zip archive from `src_zip` into the directory `dest_dir` and returns
/// the extracted file paths relative to `dest_dir`, preserving archive order.
///
/// Directory-only entries are not included in the returned list.
/// When `globs` is provided and not empty, matching is applied against stored archive entry paths.
pub fn unzip_file_with_entries_and_globs(
	src_zip: impl AsRef<SPath>,
	dest_dir: impl AsRef<SPath>,
	globs: Option<impl AsRef<[String]>>,
) -> Result<Vec<String>> {
	let src_zip = src_zip.as_ref();
	let dest_dir = dest_dir.as_ref();

	// Open the zip archive.
	let file = File::open(src_zip.as_std_path())?;
	let mut archive = ZipArchive::new(file).map_err(|err| Error::UnzipZipFail {
		zip_file: src_zip.to_string(),
		cause: format!("Fail to create new archive.\nCause: {err}"),
	})?;

	let mut extracted_files = Vec::new();

	// Iterate over zip entries.
	for i in 0..archive.len() {
		let mut file = archive.by_index(i).map_err(|err| Error::UnzipZipFail {
			zip_file: src_zip.to_string(),
			cause: format!("Fail to get item by_index {i}.\nCause: {err}"),
		})?;

		let entry_name = file.name().to_string();

		// Reject unsafe archive entry paths
		validate_zip_entry_name(&entry_name, src_zip)?;

		if !matches_zip_globs(&entry_name, globs.as_ref())? {
			continue;
		}

		let outpath = dest_dir.join(&entry_name);

		if file.name().ends_with('/') {
			// Create the directory if it doesn't exist.
			fs::create_dir_all(outpath.as_std_path())?;
		} else {
			// Ensure parent directory exists.
			if let Some(parent) = outpath.parent() {
				fs::create_dir_all(parent.as_std_path())?;
			}
			// Create and write the file.
			let mut outfile = File::create(outpath.as_std_path())?;
			io::copy(&mut file, &mut outfile)?;
			extracted_files.push(normalize_zip_entry_relative_path(&entry_name));
		}
	}

	Ok(extracted_files)
}

/// Validates a zip archive entry name for safety.
///
/// Rejects:
/// - Absolute paths (starting with `/` or a Windows drive letter like `C:`)
/// - Path traversal components (`..`)
fn validate_zip_entry_name(entry_name: &str, src_zip: &SPath) -> Result<()> {
	// Reject absolute paths (Unix-style leading slash)
	if entry_name.starts_with('/') || entry_name.starts_with('\\') {
		return Err(Error::UnzipZipFail {
			zip_file: src_zip.to_string(),
			cause: format!("Unsafe zip entry with absolute path: '{entry_name}'"),
		});
	}

	// Reject absolute paths (Windows-style drive letter, e.g. "C:" or "D:\")
	if entry_name.len() >= 2 && entry_name.as_bytes()[1] == b':' && entry_name.as_bytes()[0].is_ascii_alphabetic() {
		return Err(Error::UnzipZipFail {
			zip_file: src_zip.to_string(),
			cause: format!("Unsafe zip entry with absolute path: '{entry_name}'"),
		});
	}

	// Reject path traversal components (..)
	// Normalize backslashes to forward slashes for consistent checking
	let normalized = entry_name.replace('\\', "/");
	for component in normalized.split('/') {
		if component == ".." {
			return Err(Error::UnzipZipFail {
				zip_file: src_zip.to_string(),
				cause: format!("Unsafe zip entry with path traversal: '{entry_name}'"),
			});
		}
	}

	Ok(())
}

fn normalize_zip_entry_relative_path(entry_name: &str) -> String {
	Path::new(entry_name)
		.components()
		.map(|component| component.as_os_str().to_string_lossy().to_string())
		.collect::<Vec<_>>()
		.join("/")
}

pub fn extract_text_content(src_zip_path: impl AsRef<SPath>, content_path: &str) -> Result<String> {
	let src_zip_path = src_zip_path.as_ref();
	let file = File::open(src_zip_path)?;

	let mut archive = ZipArchive::new(file).map_err(|err| Error::Zip {
		zip_file: src_zip_path.name().to_string(),
		cause: err.to_string(),
	})?;

	let mut file = archive.by_name(content_path).map_err(|_| Error::ZipFileNotFound {
		zip_file: src_zip_path.name().to_string(),
		content_path: content_path.to_string(),
	})?;

	let mut data: Vec<u8> = Vec::new();
	file.read_to_end(&mut data).map_err(|err| Error::ZipContent {
		zip_file: src_zip_path.name().to_string(),
		content_path: content_path.to_string(),
		cause: format!("Fail to read content. Cause: {err}"),
	})?;
	let content = String::from_utf8(data).map_err(|err| Error::ZipContent {
		zip_file: src_zip_path.name().to_string(),
		content_path: content_path.to_string(),
		cause: format!("Not utf8. Info: {err}"),
	})?;

	Ok(content)
}

pub fn list_entries_with_globs(
	src_zip_path: impl AsRef<SPath>,
	globs: Option<impl AsRef<[String]>>,
) -> Result<Vec<String>> {
	let src_zip_path = src_zip_path.as_ref();
	let file = File::open(src_zip_path)?;

	let mut archive = ZipArchive::new(file).map_err(|err| Error::Zip {
		zip_file: src_zip_path.name().to_string(),
		cause: err.to_string(),
	})?;

	let mut entries = Vec::with_capacity(archive.len());
	for i in 0..archive.len() {
		let file = archive.by_index(i).map_err(|err| Error::Zip {
			zip_file: src_zip_path.name().to_string(),
			cause: format!("Fail to get item by_index {i}.\nCause: {err}"),
		})?;
		let entry_name = file.name().to_string();
		if matches_zip_globs(&entry_name, globs.as_ref())? {
			entries.push(entry_name);
		}
	}

	Ok(entries)
}

fn matches_zip_globs(entry_name: &str, globs: Option<&impl AsRef<[String]>>) -> Result<bool> {
	let Some(globs) = globs else {
		return Ok(true);
	};
	let globs = globs.as_ref();
	if globs.is_empty() {
		return Ok(true);
	}

	let glob_refs = globs.iter().map(String::as_str).collect::<Vec<_>>();
	let glob_set =
		get_glob_set(&glob_refs).map_err(|err| Error::custom(format!("Invalid zip glob patterns. Cause: {err}")))?;
	Ok(glob_set.is_match(entry_name))
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

	use super::*;
	use crate::_test_support::{gen_test_dir_path, remove_test_dir};

	#[test]
	fn test_support_zip_dir_requires_existing_directory() -> Result<()> {
		// -- Setup & Fixtures
		let root = gen_test_dir_path();
		let src_dir = root.join("missing_src");
		let dest_zip = root.join("missing_src_dest").join("archive.zip");

		// -- Exec
		let err = zip_dir(&src_dir, &dest_zip).err().ok_or("should have zip error")?;

		// -- Check
		let err_text = err.to_string();
		assert!(err_text.contains("Source directory does not exist"));

		Ok(())
	}

	#[test]
	fn test_support_zip_dir_requires_directory_source() -> Result<()> {
		// -- Setup & Fixtures
		let root = gen_test_dir_path();
		std::fs::create_dir_all(root.as_std_path())?;
		let src_file = root.join("source.txt");
		std::fs::write(src_file.as_std_path(), "hello")?;
		let dest_zip = root.join("archive.zip");

		// -- Exec
		let err = zip_dir(&src_file, &dest_zip).err().ok_or("should have zip error")?;

		// -- Check
		let err_text = err.to_string();
		assert!(err_text.contains("Source path is not a directory"));

		// -- Cleanup
		let _ = remove_test_dir(&root);

		Ok(())
	}

	#[test]
	fn test_support_zip_dir_writes_relative_archive_entries() -> Result<()> {
		// -- Setup & Fixtures
		let root = gen_test_dir_path();
		let src_dir = root.join("source");
		let nested_dir = src_dir.join("nested");
		std::fs::create_dir_all(nested_dir.as_std_path())?;
		std::fs::write(src_dir.join("root.txt").as_std_path(), "root file")?;
		std::fs::write(nested_dir.join("child.txt").as_std_path(), "nested file")?;
		let dest_zip = root.join("archive.zip");

		// -- Exec
		zip_dir(&src_dir, &dest_zip)?;
		let entries = list_entries_with_globs(&dest_zip, None::<&[String]>)?;

		// -- Check
		assert!(!entries.iter().any(|entry| entry.is_empty()));
		assert!(entries.iter().any(|entry| entry == "nested/"));
		assert!(entries.iter().any(|entry| entry == "root.txt"));
		assert!(entries.iter().any(|entry| entry == "nested/child.txt"));
		assert!(entries.iter().all(|entry| !entry.contains('\\')));

		// -- Cleanup
		let _ = remove_test_dir(&root);

		Ok(())
	}

	#[test]
	fn test_zip_files_writes_only_selected_relative_entries() -> Result<()> {
		// -- Setup & Fixtures
		let root = gen_test_dir_path();
		let src_dir = root.join("source");
		let nested_dir = src_dir.join("nested");
		let unused_dir = src_dir.join("unused");
		std::fs::create_dir_all(nested_dir.as_std_path())?;
		std::fs::create_dir_all(unused_dir.as_std_path())?;
		std::fs::write(src_dir.join("root.txt").as_std_path(), "root file")?;
		std::fs::write(nested_dir.join("child.txt").as_std_path(), "nested file")?;
		std::fs::write(unused_dir.join("ignored.txt").as_std_path(), "ignored file")?;
		let dest_zip = root.join("archive.zip");
		let selected_files = [src_dir.join("root.txt"), nested_dir.join("child.txt")];

		// -- Exec
		zip_files(&src_dir, &dest_zip, selected_files.iter())?;
		let entries = list_entries_with_globs(&dest_zip, None::<&[String]>)?;

		// -- Check
		assert!(entries.iter().any(|entry| entry == "root.txt"));
		assert!(entries.iter().any(|entry| entry == "nested/"));
		assert!(entries.iter().any(|entry| entry == "nested/child.txt"));
		assert!(!entries.iter().any(|entry| entry == "unused/"));
		assert!(!entries.iter().any(|entry| entry == "unused/ignored.txt"));
		assert!(entries.iter().all(|entry| !entry.contains('\\')));
		assert!(entries.iter().all(|entry| !entry.starts_with("source/")));
		assert_eq!(extract_text_content(&dest_zip, "nested/child.txt")?, "nested file");

		// -- Cleanup
		let _ = remove_test_dir(&root);

		Ok(())
	}

	#[test]
	fn test_zip_files_rejects_files_outside_source_directory() -> Result<()> {
		// -- Setup & Fixtures
		let root = gen_test_dir_path();
		let src_dir = root.join("source");
		std::fs::create_dir_all(src_dir.as_std_path())?;
		let outside_file = root.join("outside.txt");
		std::fs::write(outside_file.as_std_path(), "outside file")?;
		let dest_zip = root.join("archive.zip");

		// -- Exec
		let err = zip_files(&src_dir, &dest_zip, [outside_file])
			.err()
			.ok_or("should have zip error")?;

		// -- Check
		assert!(err.to_string().contains("outside source directory"));

		// -- Cleanup
		let _ = remove_test_dir(&root);

		Ok(())
	}

	#[test]
	fn test_matches_zip_globs_uses_archive_style_paths() -> Result<()> {
		// -- Setup & Fixtures
		let globs = vec!["nested/*.txt".to_string(), "root.txt".to_string()];

		// -- Check
		assert!(matches_zip_globs("root.txt", Some(&globs))?);
		assert!(matches_zip_globs("nested/child.txt", Some(&globs))?);
		assert!(!matches_zip_globs("nested/child.md", Some(&globs))?);

		Ok(())
	}
}

// endregion: --- Tests
