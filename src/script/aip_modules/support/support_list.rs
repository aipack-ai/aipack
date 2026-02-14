use crate::runtime::Runtime;
use crate::support::AsStrsExt;
use crate::types::FileRef;
use crate::{Error, Result};
use simple_fs::{ListOptions, SPath, list_files};
use std::collections::HashSet;

// Those folders need to be explicitly include in the include globs or they will be ignored with `**..**` glob (e.g. `**target/**`)
const SPECIAL_DEFAULT_FOLDER_EXCLUDES: &[&str] = &[
	//
	".aipack/", // otherwise, when `**/*` it get included and can confuse
	".git/",
	"target/",
	"node_modules/",
	".build/",
	"__pycache__/",
];

const GLOBS_TO_ALWAYS_EXLUDES: &[&str] = &["**/.DS_Store", ".DS_Store", "**/Thumbs.db", "**/*.swp"];

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
	let mut special_folder_excludes: HashSet<&'static str> = SPECIAL_DEFAULT_FOLDER_EXCLUDES.iter().copied().collect();

	// validate globs, and refine excludes
	// (cheap check for now. Should probably be in simple-fs)
	for glob in include_globs {
		// this normalize the path with `/`
		let glob = SPath::new(glob);
		let glob = glob.as_str().trim();

		// -- Update the exclude
		let excludes_tmp: Vec<&'static str> = special_folder_excludes.iter().copied().collect();
		for exc in excludes_tmp {
			if glob.contains(exc) && special_folder_excludes.contains(exc) {
				special_folder_excludes.remove(exc);
			}
		}

		// -- Validate that it does not start wiht ../ or ./.. (not supported for now)
		// NOTE: This not a complete check, but at least will warn the user for common mistake
		if glob.starts_with("../") || glob.starts_with("./..") {
			return Err(Error::custom(format!(
				"Glob '{glob}' starting with '../'.\nStarting glob with '../' is not supported at the moment."
			)));
		}
	}

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
	let mut options = ListOptions::from_relative_glob(!absolute);

	// if there is some exlude special folders
	let exclude_globs = if !special_folder_excludes.is_empty() {
		special_folder_excludes
			.into_iter()
			.map(|exc| format!("**/{exc}**"))
			.collect::<Vec<_>>()
	} else {
		Vec::new()
	};
	let mut exclude_globs = exclude_globs.x_as_strs();
	exclude_globs.extend_from_slice(GLOBS_TO_ALWAYS_EXLUDES);
	if !exclude_globs.is_empty() {
		options = options.with_exclude_globs(&exclude_globs);
	}

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
