use crate::runtime::Runtime;
use crate::support::files::list_options_with_default_excludes;
use crate::types::FileRef;
use crate::{Error, Result};
use simple_fs::{SPath, list_files};

/// Lists files based on provided glob patterns and options
///
/// Note: Common build/dependency folders (e.g., `target/`, `node_modules/`, `.build/`, `__pycache__/`)
/// are excluded by default unless explicitly matched by `include_globs`.
///
/// Returns a list of files that match the globs, with paths relative to the base_dir
/// or absolute depending on the options
pub fn list_files_with_options(
	runtime: &Runtime,
	base_path: Option<&SPath>,
	include_globs: &[&str],
	absolute: bool,
	glob_sort: bool,
) -> Result<Vec<FileRef>> {
	// we start with the full set of special exclude folders
	// (then if included in the include globs, they will be removed from the exclude set)

	// validate globs, and refine excludes
	// (cheap check for now. Should probably be in simple-fs)

	// -- Build the base_path
	let base_path = match base_path {
		Some(base_path) => base_path.clone(),
		None => runtime
			.dir_context()
			.wks_dir()
			.ok_or("Cannot create file records, no workspace")?
			.clone(),
	};

	// -- Build ListOptions

	// if there is some exlude special folders
	let mut exclude_globs = Vec::new();
	let options = list_options_with_default_excludes(include_globs, &mut exclude_globs)?;

	// -- Execute the list_files
	let sfiles = list_files(&base_path, Some(include_globs), Some(options)).map_err(Error::from)?;

	// Now, we put back the paths found relative to base_path
	let file_refs = sfiles
		.into_iter()
		.map(|f| {
			let smeta = f.meta().ok();
			let spath = if absolute {
				f
			} else {
				//
				let diff = f.try_diff(&base_path)?;
				// if the diff goes back from base_path, then, we put the absolute path
				if diff.as_str().starts_with("..") { f } else { diff }
			};

			Ok(FileRef { spath, smeta })
		})
		.collect::<simple_fs::Result<Vec<FileRef>>>()
		.map_err(|err| crate::Error::cc("Cannot list files to base", err))?;

	// sort by the globs (mke sure we use this files paths not the one before)
	let file_refs = if glob_sort {
		simple_fs::sort_by_globs(file_refs, include_globs, true)?
	} else {
		file_refs
	};

	Ok(file_refs)
}
