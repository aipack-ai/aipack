use crate::{Error, Result};
use lazy_regex::{Lazy, Regex, regex};
use simple_fs::SPath;
use std::str::FromStr;

/// This is a complete Pack Identiy, with namespace, pack_name
///
/// It does not contain a sub_path, as this is not part of the pack identity,
/// but part of the pack ref
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackIdentity {
	pub namespace: String,
	pub name: String,
}

impl PackIdentity {
	/// Just return a new SPath `namespace/pack_name`
	pub fn identity_as_path(&self) -> SPath {
		SPath::new(format!("{}/{}", self.namespace, self.name))
	}
}

/// String parser
impl FromStr for PackIdentity {
	type Err = Error;

	fn from_str(full_ref: &str) -> Result<Self> {
		let parts: Vec<&str> = full_ref.split('@').collect();
		// Check for valid pattern: name@namespace
		match (parts.first(), parts.get(1), parts.get(2)) {
			(Some(namespace), Some(name), None) => {
				// name@namespace format
				Self::validate_namespace(namespace).map_err(|err| Error::InvalidPackIdentity {
					origin_path: full_ref.to_string(),
					cause: err.to_string(),
				})?;
				Self::validate_name(name).map_err(|err| Error::InvalidPackIdentity {
					origin_path: full_ref.to_string(),
					cause: err.to_string(),
				})?;

				Ok(PackIdentity {
					namespace: namespace.to_string(),
					name: name.to_string(),
				})
			}
			(Some(_), None, _) => {
				// Missing @ symbol
				Err(Error::InvalidPackIdentity {
					origin_path: full_ref.to_string(),
					cause: "Missing '@' symbol in pack identity. Format must be 'name@namespace'".to_string(),
				})
			}
			_ => {
				// Too many @ symbols or empty string
				Err(Error::InvalidPackIdentity {
					origin_path: full_ref.to_string(),
					cause: "Too many '@' symbols in pack identity".to_string(),
				})
			}
		}
	}
}

/// implement std::fmt::Display
impl std::fmt::Display for PackIdentity {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}@{}", self.namespace, self.name)
	}
}

// region:    --- Helper Validators
static RGX: &Lazy<Regex> = regex!(r"^[a-zA-Z_][a-zA-Z0-9_-]*$");

impl PackIdentity {
	pub fn validate_namespace(namespace: &str) -> Result<()> {
		if !RGX.is_match(namespace) {
			return Err(Error::InvalidNamespace {
				namespace: namespace.to_string(),
				cause: "Pack namespace can only contain alphanumeric characters, hyphens, and underscores, and cannot start with a number.",
			});
		}

		Ok(())
	}

	pub fn validate_name(name: &str) -> Result<()> {
		if name.trim().is_empty() {
			return Err(Error::InvalidPackName {
				name: name.to_string(),
				cause: "Pack name cannot be empty",
			});
		}
		if !RGX.is_match(name) {
			return Err(Error::InvalidPackIdentity {
				origin_path: name.to_string(),
				cause: "Pack name can only contain alphanumeric characters, hyphens, and underscores, and cannot start with a number.".to_string(),
			});
		}

		Ok(())
	}
}

// endregion: --- Helper Validators

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;
	use crate::_test_support::assert_contains;

	#[test]
	fn test_agent_pack_identity_valids() -> Result<()> {
		// -- Setup & Fixtures
		let data = &[
			("default@pack-name", "pack-name", "default"),
			("my-namespace@pack-name", "pack-name", "my-namespace"),
			(
				"other-namespace@complex_name_with_underscores",
				"complex_name_with_underscores",
				"other-namespace",
			),
			(
				"_namespace@_starts_with_underscore",
				"_starts_with_underscore",
				"_namespace",
			),
		];

		// -- Exec & Check
		for (input, expected_name, expected_namespace) in data {
			let identity = PackIdentity::from_str(input)?;
			assert_eq!(identity.name, *expected_name, "Name should match for input: {input}");
			assert_eq!(
				identity.namespace, *expected_namespace,
				"Namespace should match for input: {input}"
			);
		}

		Ok(())
	}

	#[test]
	fn test_agent_pack_identity_invalids() -> Result<()> {
		// -- Setup & Fixtures
		let data = &[
			(
				"pack-name",
				"Missing '@' symbol in pack identity. Format must be 'name@namespace'",
			),
			("namespace@pack-name@extra", "Too many '@' symbols in pack identity"),
			(
				"namespace@1pack-name",
				"Pack name can only contain alphanumeric characters, hyphens, and underscores, and cannot start with a number.",
			),
			(
				"1namespace@pack-name",
				"Pack namespace can only contain alphanumeric characters, hyphens, and underscores, and cannot start with a number.",
			),
			(
				"na me$%^@pack-name",
				"Pack namespace can only contain alphanumeric characters, hyphens, and underscores, and cannot start with a number.",
			),
			(
				"namespace@na me$%^",
				"Pack name can only contain alphanumeric characters, hyphens, and underscores, and cannot start with a number.",
			),
		];

		// -- Exec & Check
		for (invalid_input, expected_error) in data {
			let result = PackIdentity::from_str(invalid_input);
			assert!(result.is_err(), "Should fail for invalid input: {invalid_input}");
			let err = result.unwrap_err().to_string();
			assert_contains(&err, expected_error);
		}

		Ok(())
	}
}

// endregion: --- Tests
