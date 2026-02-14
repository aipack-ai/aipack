//! Helper functions for JSON operations.

use crate::support::text::truncate_with_ellipsis;
use crate::{Error, Result};
use jsonc_parser::ParseOptions;
use serde_json::Value;
use simple_fs::SPath;

/// Prase a json string content that can have
/// - Comments
/// - Trailing commas
///
/// Note: Property names still need to be quoted.
pub fn parse_jsonc_to_serde_value(content: &str) -> Result<Option<serde_json::Value>> {
	static OPTIONS: ParseOptions = ParseOptions {
		allow_comments: true,
		allow_trailing_commas: true,
		// this one is set to FALSE, for better IDE compatibility
		allow_loose_object_property_names: false,
		allow_single_quoted_strings: false,
		allow_hexadecimal_numbers: false,
		allow_unary_plus_numbers: false,
		allow_missing_commas: false,
	};

	let json_value = jsonc_parser::parse_to_serde_value(content, &OPTIONS).map_err(|err| {
		let content = truncate_with_ellipsis(content, 300, "...");
		Error::custom(format!("Fail to parse json.\nCause: {err}\nJson Content:\n{content}"))
	})?;

	Ok(json_value)
}

/// Read & parse a json or jsonc/trailing-commas
pub fn load_json_to_serde_value(file: &SPath) -> Result<Option<serde_json::Value>> {
	let content = simple_fs::read_to_string(file)?;

	let value = parse_jsonc_to_serde_value(&content)?;

	Ok(value)
}

/// Converts a `Vec<T>` where `T` is serializable into a `Result<Vec<Value>>`.
///
/// Serializes each item in the input vector into a `serde_json::Value`.
///
/// # Arguments
///
/// * `vals` - A `Vec` of items implementing `serde::Serialize`.
///
/// # Returns
///
/// Returns `Ok(Vec<Value>)` on success, or an `Error` if serialization fails.
pub fn into_values<T: serde::Serialize>(vals: Vec<T>) -> Result<Vec<Value>> {
	let inputs: Vec<Value> = vals
		.into_iter()
		.map(|v| serde_json::to_value(v).map_err(Error::custom))
		.collect::<Result<Vec<_>>>()?;

	Ok(inputs)
}
