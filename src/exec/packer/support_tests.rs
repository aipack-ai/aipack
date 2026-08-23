use super::*;
use crate::Error;

type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

// #[test]
// fn test_packer_support_normalize_version_simple() -> Result<()> {
// 	assert_eq!(normalize_version("1.0.0"), "1-0-0");
// 	assert_eq!(normalize_version("1.0-alpha"), "1-0-alpha");
// 	assert_eq!(normalize_version("1.0 beta"), "1-0-beta");
// 	assert_eq!(normalize_version("1.0-beta-2"), "1-0-beta-2");
// 	assert_eq!(normalize_version("1.0--beta--2"), "1-0-beta-2");
// 	assert_eq!(normalize_version("v1.0.0_rc1"), "v1-0-0-rc1");
// 	assert_eq!(normalize_version("1.0.0!@#$%^&*()"), "1-0-0");

// 	Ok(())
// }

#[test]
fn test_packer_support_validate_version_update_simple() -> Result<()> {
	use std::cmp::Ordering;

	// Test case: New version is greater than installed
	assert_eq!(validate_version_update("1.0.0", "1.0.1")?, Ordering::Greater);
	assert_eq!(validate_version_update("1.0.0", "1.1.0")?, Ordering::Greater);
	assert_eq!(validate_version_update("1.0.0", "2.0.0")?, Ordering::Greater);

	// Test case: New version is equal to installed
	assert_eq!(validate_version_update("1.0.0", "1.0.0")?, Ordering::Equal);

	// Test case: New version is less than installed
	assert_eq!(validate_version_update("1.0.1", "1.0.0")?, Ordering::Less);

	// Test with leading 'v'
	assert_eq!(validate_version_update("v1.0.0", "1.0.1")?, Ordering::Greater);
	assert_eq!(validate_version_update("1.0.0", "v1.0.1")?, Ordering::Greater);

	// Test with invalid versions (string comparison fallback)
	assert_eq!(validate_version_update("a", "b")?, Ordering::Greater);
	assert_eq!(validate_version_update("b", "a")?, Ordering::Less);
	assert_eq!(validate_version_update("a", "a")?, Ordering::Equal);

	Ok(())
}

#[test]
fn test_packer_support_validate_version_for_install_valid() -> Result<()> {
	// Test valid versions
	assert!(validate_version_for_install("0.1.0").is_ok());
	assert!(validate_version_for_install("1.0.0").is_ok());
	assert!(validate_version_for_install("0.1.1-alpha.1").is_ok());
	assert!(validate_version_for_install("0.1.1-beta.123").is_ok());
	assert!(validate_version_for_install("0.1.1-rc.1.2").is_ok());
	assert!(validate_version_for_install("v1.0.0-alpha.1").is_ok());

	Ok(())
}

#[test]
fn test_packer_support_validate_version_for_install_invalid() -> Result<()> {
	// Test invalid versions
	let err = validate_version_for_install("0.1.1-alpha").unwrap_err();
	match err {
		Error::InvalidPrereleaseFormat { version } => {
			assert_eq!(version, "0.1.1-alpha");
		}
		_ => panic!("Expected InvalidPrereleaseFormat error"),
	}

	let err = validate_version_for_install("0.1.1-alpha.text").unwrap_err();
	match err {
		Error::InvalidPrereleaseFormat { version } => {
			assert_eq!(version, "0.1.1-alpha.text");
		}
		_ => panic!("Expected InvalidPrereleaseFormat error"),
	}

	let err = validate_version_for_install("0.1.1-alpha.1.some").unwrap_err();
	match err {
		Error::InvalidPrereleaseFormat { version } => {
			assert_eq!(version, "0.1.1-alpha.1.some");
		}
		_ => panic!("Expected InvalidPrereleaseFormat error"),
	}

	Ok(())
}

#[test]
fn test_packer_support_pack_uri_parse_repo() -> Result<()> {
	// -- Setup & Fixtures
	let uri = "pro@coder";

	// -- Exec
	let pack_uri = PackUri::parse(uri)?;

	// -- Check
	assert!(matches!(pack_uri, PackUri::RepoPack(_)));
	if let PackUri::RepoPack(identity) = &pack_uri {
		assert_eq!(identity.namespace, "pro");
		assert_eq!(identity.name, "coder");
	}

	Ok(())
}

#[test]
fn test_packer_support_pack_uri_parse_http() -> Result<()> {
	// -- Setup & Fixtures
	let uri = "https://example.com/some-pack.aipack";

	// -- Exec
	let pack_uri = PackUri::parse(uri)?;

	// -- Check
	assert!(matches!(pack_uri, PackUri::HttpLink(_)));
	if let PackUri::HttpLink(url) = &pack_uri {
		assert_eq!(url, "https://example.com/some-pack.aipack");
	}

	Ok(())
}

#[test]
fn test_packer_support_pack_uri_parse_local() -> Result<()> {
	// -- Setup & Fixtures
	let uri = "./path/to/pack.aipack";

	// -- Exec
	let pack_uri = PackUri::parse(uri)?;

	// -- Check
	assert!(matches!(pack_uri, PackUri::LocalPath(_)));
	if let PackUri::LocalPath(path) = &pack_uri {
		assert_eq!(path, "./path/to/pack.aipack");
	}

	Ok(())
}

#[test]
fn test_packer_support_pack_uri_display() -> Result<()> {
	// -- Setup & Fixtures
	let data = [
		("pro@coder", "pro@coder"),
		(
			"https://example.com/pack.aipack",
			"URL 'https://example.com/pack.aipack'",
		),
		("./local.aipack", "local file './local.aipack'"),
	];

	// -- Exec & Check
	for (input, expected_display) in data {
		let pack_uri = PackUri::parse(input)?;
		assert_eq!(pack_uri.to_string(), expected_display, "Input: {input}");
	}

	Ok(())
}

#[test]
fn test_packer_support_pack_uri_parse_git() -> Result<()> {
	// -- Setup & Fixtures
	let uri = "git://example.com/team/pack.git";

	// -- Exec
	let pack_uri = PackUri::parse(uri)?;

	// -- Check
	assert!(matches!(pack_uri, PackUri::GitLink(_)));
	if let PackUri::GitLink(source) = &pack_uri {
		assert_eq!(source.repository, uri);
		assert!(source.subpath.is_none());
	}
	assert_eq!(pack_uri.to_string(), format!("Git URL '{uri}'"));

	Ok(())
}

#[test]
fn test_packer_support_pack_uri_parse_git_subpath() -> Result<()> {
	// -- Setup & Fixtures
	let uri = "git://example.com/team/pack.git#path/to/pack_dir";

	// -- Exec
	let pack_uri = PackUri::parse(uri)?;

	// -- Check
	assert!(matches!(pack_uri, PackUri::GitLink(_)));
	if let PackUri::GitLink(source) = &pack_uri {
		assert_eq!(source.repository, "git://example.com/team/pack.git");
		assert_eq!(source.subpath.as_deref(), Some("path/to/pack_dir"));
	}
	assert_eq!(pack_uri.to_string(), format!("Git URL '{uri}'"));

	Ok(())
}

#[test]
fn test_packer_support_pack_uri_parse_git_ssh_subpath() -> Result<()> {
	// -- Setup & Fixtures
	let uri = "git+ssh://git@github.com/owner/repository.git#path/to/pack_dir";

	// -- Exec
	let pack_uri = PackUri::parse(uri)?;

	// -- Check
	assert!(matches!(pack_uri, PackUri::GitLink(_)));
	if let PackUri::GitLink(source) = &pack_uri {
		assert_eq!(source.repository, "git+ssh://git@github.com/owner/repository.git");
		assert_eq!(source.subpath.as_deref(), Some("path/to/pack_dir"));
	}

	Ok(())
}

#[test]
fn test_packer_support_pack_uri_parse_git_subpath_normalizes_separators() -> Result<()> {
	// -- Setup & Fixtures
	let uri = r"git://example.com/team/pack.git#path\to\pack";

	// -- Exec
	let pack_uri = PackUri::parse(uri)?;

	// -- Check
	if let PackUri::GitLink(source) = &pack_uri {
		assert_eq!(source.subpath.as_deref(), Some("path/to/pack"));
	} else {
		return Err("Expected GitLink variant".into());
	}

	Ok(())
}

#[test]
fn test_packer_support_pack_uri_parse_git_invalid_subpath() -> Result<()> {
	// -- Setup & Fixtures
	let invalid_sources = [
		"git://example.com/team/pack.git#",
		"git://example.com/team/pack.git#/absolute",
		r"git://example.com/team/pack.git#\absolute",
		r"git://example.com/team/pack.git#C:\absolute",
		"git://example.com/team/pack.git#nested/../pack",
	];

	// -- Exec & Check
	for uri in invalid_sources {
		assert!(PackUri::parse(uri).is_err(), "Input: {uri}");
	}

	Ok(())
}

#[test]
fn test_packer_support_pack_uri_parse_git_suffix_not_inferred() -> Result<()> {
	// -- Setup & Fixtures
	let uri = "git://example.com/team/repository.git/nested";

	// -- Exec
	let pack_uri = PackUri::parse(uri)?;

	// -- Check
	if let PackUri::GitLink(source) = &pack_uri {
		assert_eq!(source.repository, uri);
		assert!(source.subpath.is_none());
	} else {
		return Err("Expected GitLink variant".into());
	}

	Ok(())
}

#[test]
fn test_packer_support_pack_uri_parse_https_suffix_remains_archive() -> Result<()> {
	// -- Setup & Fixtures
	let uri = "https://example.com/team/repository.git#path/to/pack";

	// -- Exec
	let pack_uri = PackUri::parse(uri)?;

	// -- Check
	if let PackUri::HttpLink(url) = &pack_uri {
		assert_eq!(url, uri);
	} else {
		return Err("Expected HttpLink variant".into());
	}

	Ok(())
}

#[test]
fn test_packer_support_resolve_install_provenance_source_variants() -> Result<()> {
	// -- Setup & Fixtures
	use simple_fs::SPath;

	let repo_reference = "pro@coder";
	let repo_uri = PackUri::parse(repo_reference)?;
	let http_reference = "https://example.com/pack.aipack";
	let http_uri = PackUri::parse(http_reference)?;
	let git_reference = "git+ssh://git@example.com/team/pack.git#packs/example";
	let git_uri = PackUri::parse(git_reference)?;
	let local_reference = "./packs/example.aipack";
	let local_uri = PackUri::parse(local_reference)?;
	let resolved_local_path = SPath::new("/workspace/packs/example.aipack");
	let scp_reference = "git@github.com:team/pack.git#packs/example";
	let scp_uri = PackUri::parse(scp_reference)?;

	// -- Exec
	let repo_source = resolve_install_provenance_source(repo_reference, &repo_uri, None)?;
	let http_source = resolve_install_provenance_source(http_reference, &http_uri, None)?;
	let git_source = resolve_install_provenance_source(git_reference, &git_uri, None)?;
	let local_source = resolve_install_provenance_source(local_reference, &local_uri, Some(&resolved_local_path))?;
	let scp_source = resolve_install_provenance_source(scp_reference, &scp_uri, None)?;

	// -- Check
	assert_eq!(repo_source, "aipack.ai");
	assert_eq!(http_source, http_reference);
	assert_eq!(git_source, git_reference);
	assert_eq!(local_source, resolved_local_path.as_str());
	assert_eq!(scp_source, scp_reference);

	Ok(())
}

#[test]
fn test_packer_support_pack_uri_parse_scp_root() -> Result<()> {
	// -- Setup & Fixtures
	let uri = "git@github.com:rust10x/rust10x.git";

	// -- Exec
	let pack_uri = PackUri::parse(uri)?;

	// -- Check
	assert_eq!(
		pack_uri,
		PackUri::GitLink(GitSource {
			repository: "git+ssh://git@github.com/rust10x/rust10x.git".to_string(),
			subpath: None,
		})
	);
	assert_eq!(
		pack_uri.to_string(),
		"Git URL 'git+ssh://git@github.com/rust10x/rust10x.git'"
	);

	Ok(())
}

#[test]
fn test_packer_support_pack_uri_parse_scp_subpath() -> Result<()> {
	// -- Setup & Fixtures
	let uri = "git@github.com:rust10x/rust10x.git#path/to/pack_dir";

	// -- Exec
	let pack_uri = PackUri::parse(uri)?;

	// -- Check
	let PackUri::GitLink(source) = &pack_uri else {
		return Err("Expected GitLink variant".into());
	};
	assert_eq!(source.repository, "git+ssh://git@github.com/rust10x/rust10x.git");
	assert_eq!(source.subpath.as_deref(), Some("path/to/pack_dir"));

	Ok(())
}

#[test]
fn test_packer_support_pack_uri_parse_scp_invalid_selectors() -> Result<()> {
	// -- Setup & Fixtures
	let invalid_sources = [
		"git@github.com:rust10x/rust10x.git#",
		"git@github.com:rust10x/rust10x.git#/absolute",
		r"git@github.com:rust10x/rust10x.git#\absolute",
		r"git@github.com:rust10x/rust10x.git#C:\absolute",
		"git@github.com:rust10x/rust10x.git#nested/../pack",
	];

	// -- Exec & Check
	for uri in invalid_sources {
		assert!(PackUri::parse(uri).is_err(), "Input: {uri}");
	}

	Ok(())
}

#[test]
fn test_packer_support_pack_uri_parse_explicit_git_sources_unchanged() -> Result<()> {
	// -- Setup & Fixtures
	let sources = [
		(
			"git://example.com/team/pack.git#path/to/pack",
			"git://example.com/team/pack.git",
			Some("path/to/pack"),
		),
		(
			"git+ssh://git@example.com/team/pack.git#path/to/pack",
			"git+ssh://git@example.com/team/pack.git",
			Some("path/to/pack"),
		),
	];

	// -- Exec & Check
	for (uri, expected_repository, expected_subpath) in sources {
		let PackUri::GitLink(source) = PackUri::parse(uri)? else {
			return Err("Expected GitLink variant".into());
		};
		assert_eq!(source.repository, expected_repository);
		assert_eq!(source.subpath.as_deref(), expected_subpath);
	}

	Ok(())
}
