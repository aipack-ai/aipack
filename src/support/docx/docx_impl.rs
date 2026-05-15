//! From markdownify crate: https://github.com/Skardyy/mcat/tree/main/crates/markdownify
//! NOTE: Need to customize and use latest zip (will eventually do PRs)

use super::md_support::to_markdown_table;
use quick_xml::XmlVersion;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::io::{Cursor, Read};
use std::path::Path;
use zip::ZipArchive;

struct Styles {
	title: bool,       //w:pStyle empty w:val="includes title"
	subtitle: bool,    // w:pStyle empty w:val="includes subtitle"
	header: bool,      // w:pStyle empty w:val="includes heading"
	header_level: u32, // The level
	bold: bool,        //w:b empty
	strike: bool,      //w:strike
	underline: bool,   //w:u
	italics: bool,     //w:i
	indent: i8,        // w:ilvl w:val="0" (add 1 to it and -1 was indented)
	table: bool,       //w:tbl
}

impl Styles {
	pub fn default() -> Self {
		Styles {
			title: false,
			subtitle: false,
			header: false,
			header_level: 0,
			strike: false,
			italics: false,
			underline: false,
			bold: false,
			indent: 0,
			table: false,
		}
	}
}

fn get_attr(e: &quick_xml::events::BytesStart, key: &[u8]) -> Option<String> {
	for attr in e.attributes().with_checks(false).flatten() {
		if attr.key.as_ref() == key {
			let v = attr.normalized_value(XmlVersion::Implicit1_0);
			return v.ok().map(|v| v.to_string());
		}
	}
	None
}

// Track trailing newlines to efficiently ensure a blank line before block elements (e.g., headings).
fn count_trailing_newlines(s: &str) -> u8 {
	let mut cnt = 0u8;
	for b in s.as_bytes().iter().rev() {
		if *b == b'\n' {
			cnt += 1;
			if cnt == 2 {
				break;
			}
		} else {
			break;
		}
	}
	cnt
}

fn push_and_update(markdown: &mut String, s: &str, trailing_newlines: &mut u8, started: &mut bool) {
	let s_to_add = if !*started { s.trim_start() } else { s };

	if !*started && !s_to_add.is_empty() {
		*started = true;
	}

	markdown.push_str(s_to_add);
	let n = count_trailing_newlines(s_to_add);
	if n > 0 {
		*trailing_newlines = n;
	} else {
		*trailing_newlines = 0;
	}
}

fn ensure_blank_line_before_block(markdown: &mut String, trailing_newlines: &mut u8, started: &mut bool) {
	// Avoid leading blank lines at the very beginning.
	if !*started {
		return;
	}
	while *trailing_newlines < 2 {
		markdown.push('\n');
		*trailing_newlines += 1;
	}
}

/// convert docx into markdown
/// usuage:
/// ```rs
/// let path = Path::new("path/to/file.docx");
/// let md = docx_convert(&path)?;
/// println!("{}", md);
/// ```
pub fn docx_convert(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
	let data = std::fs::read(path)?;
	let cursor = Cursor::new(data);

	let mut archive = ZipArchive::new(cursor)?;
	let mut xml_content = String::new();

	for i in 0..archive.len() {
		let mut file = archive.by_index(i)?;
		if file.name() == "word/document.xml" {
			file.read_to_string(&mut xml_content)?;
			break;
		}
	}

	let mut reader = Reader::from_str(&xml_content);
	let mut buf = Vec::new();
	let mut markdown = String::new();
	let mut trailing_newlines: u8 = 0;
	let mut started: bool = false;

	let mut table_rows: Vec<Vec<String>> = Vec::new();
	let mut current_row: Vec<String> = Vec::new();
	let mut styles = Styles::default();

	loop {
		match reader.read_event_into(&mut buf) {
			Ok(Event::Start(e)) => match e.name().as_ref() {
				b"w:tbl" => styles.table = true,
				_ => {
					continue;
				}
			},
			Ok(Event::Empty(e)) => match e.name().as_ref() {
				b"w:b" => {
					if let Some(val) = get_attr(&e, b"w:val") {
						if val == "true" {
							styles.bold = true;
						}
					} else {
						styles.bold = true;
					}
				}
				b"w:i" => {
					if let Some(val) = get_attr(&e, b"w:val") {
						if val == "true" {
							styles.italics = true;
						}
					} else {
						styles.italics = true;
					}
				}
				b"w:strike" => {
					if let Some(val) = get_attr(&e, b"w:val") {
						if val == "true" {
							styles.strike = true;
						}
					} else {
						styles.strike = true;
					}
				}
				b"w:u" => {
					styles.underline = true;
				}
				b"w:pStyle" => {
					if let Some(val) = get_attr(&e, b"w:val") {
						let val_lower = val.to_lowercase();
						if val_lower.contains("subtitle") {
							styles.subtitle = true;
							styles.indent = 0;
						} else if val_lower.contains("title") {
							styles.title = true;
							styles.indent = 0;
							styles.header_level = 1;
						} else if val_lower.contains("heading") {
							// parse num
							let num_str = &val["heading".len()..];
							let num: u32 = num_str.parse().unwrap_or(5);
							// set styles
							styles.header_level = num + 1;
							styles.header = true;
							styles.indent = 0;
						}
					}
				}
				b"w:ilvl" => {
					if styles.header || styles.title {
						continue;
					}
					if let Some(val) = get_attr(&e, b"w:val")
						&& let Ok(val) = val.parse::<i8>()
					{
						styles.indent = val + 1
					}
				}
				_ => {}
			},
			Ok(Event::Text(e)) => {
				let mut text = e.decode()?.into_owned();
				if styles.bold {
					text = format!("**{}** ", text.trim());
					styles.bold = false;
				}
				if styles.underline {
					text = format!("<u>{}</u> ", text.trim());
					styles.underline = false;
				}
				if styles.strike {
					text = format!("~~{}~~ ", text.trim());
					styles.strike = false;
				}
				if styles.italics {
					text = format!("*{}* ", text.trim());
					styles.italics = false;
				}

				if styles.table {
					current_row.push(text);
					continue;
				}
				if styles.title {
					ensure_blank_line_before_block(&mut markdown, &mut trailing_newlines, &mut started);
					let header_prefix = "#".repeat(styles.header_level as usize);
					let line = format!("{header_prefix} {}", text);
					push_and_update(&mut markdown, &line, &mut trailing_newlines, &mut started);
					styles.title = false;
					continue;
				}
				if styles.subtitle {
					ensure_blank_line_before_block(&mut markdown, &mut trailing_newlines, &mut started);
					let line = format!("**{}**", text.trim());
					push_and_update(&mut markdown, &line, &mut trailing_newlines, &mut started);
					styles.subtitle = false;
					continue;
				}
				if styles.header {
					ensure_blank_line_before_block(&mut markdown, &mut trailing_newlines, &mut started);
					let header_prefix = "#".repeat(styles.header_level as usize);
					let line = format!("{header_prefix} {}", text);
					push_and_update(&mut markdown, &line, &mut trailing_newlines, &mut started);
					styles.header = false;
					continue;
				}
				if styles.indent > 0 {
					let indent_num = styles.indent.saturating_sub(1);
					let indent = "  ".repeat(indent_num as usize);
					let line = format!("{}- {}", indent, text);
					push_and_update(&mut markdown, &line, &mut trailing_newlines, &mut started);
					styles.indent = -1;
					continue;
				}
				push_and_update(&mut markdown, &text, &mut trailing_newlines, &mut started);
			}
			Ok(Event::End(e)) => match e.name().as_ref() {
				b"w:tbl" if !table_rows.is_empty() => {
					let headers = table_rows[0].clone();
					let data_rows = if table_rows.len() > 1 {
						table_rows[1..].to_vec()
					} else {
						Vec::new()
					};
					let tbl_md = to_markdown_table(&headers, &data_rows);
					push_and_update(&mut markdown, &tbl_md, &mut trailing_newlines, &mut started);
					push_and_update(&mut markdown, "\n", &mut trailing_newlines, &mut started);
					table_rows = Vec::new();
					styles = Styles::default();
				}
				b"w:tr" => {
					table_rows.push(current_row);
					current_row = Vec::new();
				}
				b"w:p" => {
					if styles.indent == -1 {
						push_and_update(&mut markdown, "  \n", &mut trailing_newlines, &mut started);
						styles.indent = 0;
					} else {
						push_and_update(&mut markdown, "\n\n", &mut trailing_newlines, &mut started);
					}
				}
				_ => {}
			},
			Ok(Event::Eof) => break,
			Err(e) => {
				return Err(format!("Error at position {}: {:?}", reader.buffer_position(), e).into());
			}
			_ => {}
		}
		buf.clear();
	}

	Ok(format(&markdown))
}

// Same format function from your ODT implementation
fn format(input: &str) -> String {
	let mut result = String::with_capacity(input.len());
	let mut newline_count = 0;
	let mut spaces_count = 0;

	for line in input.lines() {
		if line.trim() == "" {
			result.push('\n');
		} else {
			result.push_str(&format!("{}\n", line));
		}
	}
	let input = &result;
	let mut result = String::with_capacity(input.len());

	for c in input.chars() {
		if c == ' ' {
			spaces_count += 1;
		}
		if c == '\n' {
			newline_count += 1;
			if spaces_count >= 2 {
				newline_count += 1;
			}
			spaces_count = 0;
			if newline_count <= 2 {
				result.push(c);
			}
		} else {
			newline_count = 0;
			spaces_count = 0;
			result.push(c);
		}
	}

	result
}
