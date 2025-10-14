use super::parse_attribute;
use std::collections::HashMap;

type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn test_support_tag_attrs_parser_simple() -> Result<()> {
	// -- Setup & Fixtures
	let raw = r#"path="a/b.txt" id=123 flag"#;

	// -- Exec
	let parsed = parse_attribute(Some(raw));

	// -- Check
	let mut expected = HashMap::new();
	expected.insert("path".to_string(), "a/b.txt".to_string());
	expected.insert("id".to_string(), "123".to_string());
	expected.insert("flag".to_string(), "".to_string());

	assert_eq!(parsed, Some(expected));

	Ok(())
}

#[test]
fn test_support_tag_attrs_parser_handles_spacing() -> Result<()> {
	// -- Setup & Fixtures
	let raw = "  key = 'value with spaces'    other =true\tline=42\nflag ";

	// -- Exec
	let parsed = parse_attribute(Some(raw));

	// -- Check
	let mut expected = HashMap::new();
	expected.insert("key".to_string(), "value with spaces".to_string());
	expected.insert("other".to_string(), "true".to_string());
	expected.insert("line".to_string(), "42".to_string());
	expected.insert("flag".to_string(), "".to_string());

	assert_eq!(parsed, Some(expected));

	Ok(())
}

#[test]
fn test_support_tag_attrs_parser_none_or_empty() -> Result<()> {
	// -- Setup & Fixtures
	let none_input = None;
	let empty_input = Some("   \n\t");

	// -- Exec
	let parsed_none = parse_attribute(none_input);
	let parsed_empty = parse_attribute(empty_input);

	// -- Check
	assert!(parsed_none.is_none());
	assert!(parsed_empty.is_none());

	Ok(())
}
