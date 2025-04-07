//! This module defines the AgentRef enum used to reference an agent either as a local file path
//! or as a PackRef, which can be parsed from a string using the '@' delimiter.

use crate::pack::{LocalPackRef, PartialPackRef};

/// AgentRef represents a reference to an agent.
/// It can either be a LocalPath (a direct file path) or a PackRef (a parsed representation).
#[derive(Debug, Clone)]
pub enum PartialAgentRef {
	LocalPath(String),
	PackRef(PartialPackRef),
}

/// Constructors
impl PartialAgentRef {
	/// Create a new AgentRef from an input string.
	///
	/// The function determines the type of AgentRef based on the presence of the '@' character.
	///
	/// If the input string contains '@':
	///   - It is parsed into a PackRef.
	///   - Example: "pro@coder" will be parsed as
	/// ```
	/// PackRef {
	///     namespace: Some("jc"),
	///     pack_name: "coder",
	///     sub_path: None
	/// }
	///````
	///   - If a subpath is provided (e.g., "pro@coder/explain"), the sub_path field is set.
	///
	/// If the input string does not contain '@':
	///   - It is treated as a local file path.
	///
	/// TODO: Probably need to use PatialPackRef
	pub fn new(input: &str) -> Self {
		// Check if the input contains the delimiter '@'
		if input.contains('@') {
			// Split the input into namespace and remainder using '@'
			let parts: Vec<&str> = input.splitn(2, '@').collect();
			let ns_part = parts[0].trim();
			let remainder = parts[1].trim();

			// Split the remainder into pack_name and an optional sub_path using '/'
			let mut rem_parts = remainder.splitn(2, '/');
			let pack_name = rem_parts.next().unwrap().to_string();
			let sub_path = rem_parts.next().map(|s| s.to_string());

			// Determine if namespace is provided or empty
			let namespace = if ns_part.is_empty() {
				None
			} else {
				Some(ns_part.to_string())
			};

			// Return a PackRef wrapped in the AgentRef enum
			PartialAgentRef::PackRef(PartialPackRef {
				namespace,
				name: pack_name,
				sub_path,
			})
		} else {
			// If no '@' is found, treat the input as a local file path and return it as LocalPath.
			PartialAgentRef::LocalPath(input.to_string())
		}
	}
}

/// Implement the Display trait for AgentRef
impl std::fmt::Display for PartialAgentRef {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			PartialAgentRef::LocalPath(path) => write!(f, "{}", path),
			PartialAgentRef::PackRef(pack_ref) => {
				write!(f, "{}", pack_ref)?;
				Ok(())
			}
		}
	}
}

// region:    --- Final AgentRef

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum AgentRef {
	LocalPath(String),
	PackRef(LocalPackRef),
}

// endregion: --- Final AgentRef

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

	use super::*;

	#[test]
	fn test_agent_ref_new_localpath() -> Result<()> {
		// -- Setup & Fixtures
		let input = "path/to/local/file.rs";

		// -- Exec
		let agent_ref = PartialAgentRef::new(input);

		// -- Check
		match agent_ref {
			PartialAgentRef::LocalPath(ref path) => {
				assert_eq!(path, input, "The local path should match the input string.");
			}
			_ => panic!("Expected AgentRef::LocalPath but got a different variant."),
		}

		Ok(())
	}

	#[test]
	fn test_agent_ref_new_packref_without_subpath() -> Result<()> {
		// -- Setup & Fixtures
		let input = "pro@coder";

		// -- Exec
		let agent_ref = PartialAgentRef::new(input);

		// -- Check
		match agent_ref {
			PartialAgentRef::PackRef(ref pack_ref) => {
				assert_eq!(pack_ref.namespace.as_deref(), Some("pro"), "Namespace should be 'pro'.");
				assert_eq!(pack_ref.name, "coder", "Pack name should be 'coder'.");
				assert!(pack_ref.sub_path.is_none(), "Sub path should be None.");
			}
			_ => panic!("Expected AgentRef::PackRef but got a different variant."),
		}

		Ok(())
	}

	#[test]
	fn test_agent_ref_new_packref_with_subpath() -> Result<()> {
		// -- Setup & Fixtures
		let input = " jc @ coder/example/path ";
		// note: the input contains spaces which should be trimmed

		// -- Exec
		let agent_ref = PartialAgentRef::new(input);

		// -- Check
		match agent_ref {
			PartialAgentRef::PackRef(ref pack_ref) => {
				assert_eq!(pack_ref.namespace.as_deref(), Some("jc"), "Namespace should be 'jc'.");
				assert_eq!(pack_ref.name, "coder", "Pack name should be 'coder'.");
				assert_eq!(
					pack_ref.sub_path.as_deref(),
					Some("example/path"),
					"Sub path should be 'example/path'."
				);
			}
			_ => panic!("Expected AgentRef::PackRef but got a different variant."),
		}

		Ok(())
	}
}

// endregion: --- Tests
