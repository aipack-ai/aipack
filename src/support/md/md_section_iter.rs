use crate::support::CowLines;
use crate::support::md::InBlockState;
use crate::support::md::md_section_split::MdSectionSplit;
use crate::types::{MdHeading, MdSection, ParseResponse};
use crate::{Error, Result};
use std::borrow::{BorrowMut, Cow};
use std::path::Path;
use std::str;

/// The Section filter pattern.
/// Currently, it supports headings only, which is a good start and efficient since there is no need to look ahead.
#[derive(Debug)]
struct SectionPattern {
	heading_level: usize,
	heading_name: String,
}

impl SectionPattern {
	fn new(heading_level: usize, heading_name: &str) -> Self {
		Self {
			heading_level,
			heading_name: heading_name.trim().to_string(),
		}
	}

	fn matches(&self, level: usize, name: &str) -> bool {
		self.heading_level == level && self.heading_name == name
	}

	fn matches_heading(&self, heading: &MdHeading) -> bool {
		self.matches(heading.level(), heading.name())
	}
}

/// Represents an iterator over Markdown sections with multiple section filters.
/// IMPORTANT:
/// - When filter: [] (empty), the No Filter mode will return every section as a new item, regardless if they are nested.
/// - When filter: [...] is not empty, it will respect the hierarchy, meaning the "# Heading 1"
///   will include the descendant heading and content as string content.
/// - When a filter item is an empty string, it means only the top content, before any heading, and in this case, subheadings are not captured.
pub struct MdSectionIter<'a> {
	// -- Iterator data
	lines: CowLines<'a>,
	/// A filter is a list of SectionPatterns
	filter: Vec<SectionPattern>,

	// -- iterator managed states
	passed_first_heading: bool,
	action_state: ActionState,
	last_heading: Option<MdHeading>,
}

/// Constructors
impl<'a> MdSectionIter<'a> {
	pub fn from_path(path: impl AsRef<Path>, heading_patterns: Option<&[&str]>) -> Result<Self> {
		let lines = CowLines::from_path(path)?;
		Self::new(lines, heading_patterns)
	}

	#[allow(unused)]
	pub fn from_str(content: &'a str, heading_patterns: Option<&[&str]>) -> Result<Self> {
		let lines = CowLines::from_str(content);
		Self::new(lines, heading_patterns)
	}

	/// Creates a new MdSection iterator from the given content source.
	fn new(source: CowLines<'a>, heading_patterns: Option<&[&str]>) -> Result<Self> {
		let section_filter = Self::resolve_heading_patterns(heading_patterns)?;

		Ok(MdSectionIter {
			lines: source,
			filter: section_filter,
			passed_first_heading: false,
			action_state: ActionState::NoCapture,
			last_heading: None,
		})
	}

	fn resolve_heading_patterns(heading_patterns: Option<&[&str]>) -> Result<Vec<SectionPattern>> {
		let Some(heading_patterns) = heading_patterns else {
			return Ok(Vec::new());
		};

		let ref_headings: Vec<SectionPattern> = heading_patterns
			.iter()
			.map(|&ref_heading| {
				if ref_heading.is_empty() {
					Ok(SectionPattern::new(0, ""))
				} else {
					MdHeading::peek_line(ref_heading)
						.ok_or_else(|| {
							Error::custom(format!(
								"read_md_section - ref '{ref_heading}' is not a valid markdown heading"
							))
						})
						.map(|(level, name)| SectionPattern::new(level, name))
				}
			})
			.collect::<Result<Vec<_>>>()?;

		Ok(ref_headings)
	}
}

/// Lexer / next_section
impl<'a> MdSectionIter<'a> {
	/// Retrieves the next `MdSection` that matches the section filters, if any.
	fn next_section(&mut self) -> Option<MdSection> {
		// -- Helper function to close and return the section
		fn close_section(
			current_captured_content: &mut Option<Vec<Cow<str>>>,
			current_captured_heading: &mut Option<MdHeading>,
			add_new_line: bool,
		) -> Option<MdSection> {
			current_captured_content.take().map(|mut content| {
				if add_new_line {
					content.push("".into());
				}
				let content = content.join("\n");
				MdSection::new(content, current_captured_heading.take())
			})
		}

		// -- Helper closures to look in the filters
		let filter_matches_fn =
			|heading: &MdHeading| self.filter.iter().position(|pattern| pattern.matches_heading(heading));
		let filter_has_level_0_fn = || self.filter.iter().position(|pattern| pattern.heading_level == 0);

		// -- function states
		// if filter list is empty, then, we capture all sections
		let all_sections = self.filter.is_empty();
		let mut current_captured_content: Option<Vec<Cow<'a, str>>> = None;
		let mut current_captured_heading: Option<MdHeading> = self.last_heading.take();
		let mut current_matching_ref: Option<&SectionPattern> = None;
		// to make sure we
		let mut block_state = InBlockState::Out;
		// This is a flag to capture level 0

		// -- Capture the content and return when found
		for line in self.lines.borrow_mut() {
			block_state = block_state.compute_new(line.as_ref());
			let is_inside_code_block = !block_state.is_out();

			// -- Capture the LineData
			let line_data = {
				if is_inside_code_block {
					LineData::Content(line)
				} else {
					//
					match MdHeading::peek_line(line.as_ref()) {
						Some((_level, _name)) => {
							self.passed_first_heading = true;
							// TODO: Needs to handle the correct add_new_line flag
							match MdHeading::parse_line(line.as_ref() as &str) {
								ParseResponse::Item(heading) => LineData::Heading(heading),
								// TODO: Here we should never have Other, as the peek_line returns Some.
								//       But just in case, fall back to other
								ParseResponse::Other(_) => LineData::Content(line),
							}
						}
						None => {
							if line.starts_with('>') {
								LineData::Blockquote(line)
							} else {
								LineData::Content(line)
							}
						}
					}
				}
			};

			// -- Compute the action
			self.action_state = match &line_data {
				// -- We are on a Heading
				LineData::Heading(line_heading) => {
					// if we capture all sections, this is a new section
					if all_sections {
						ActionState::NewHeadingForAllSections
					}
					// If the current matching ref is 0 and this is a heading
					// we set the matching current_matching_ref = None
					else if current_matching_ref
						.map(|pattern| pattern.heading_level == 0)
						.unwrap_or_default()
					{
						ActionState::CloseCurrentSection
					} else {
						match current_captured_heading.as_ref() {
							// If we were capturing the heading, then only capture if lower
							Some(captured_heading) => {
								if line_heading.level() > captured_heading.level() {
									ActionState::CaptureLine
								} else {
									ActionState::CloseCurrentSection
								}
							}
							// If we were not in a capture heading
							None => {
								if let Some(ref_idx) = filter_matches_fn(line_heading) {
									ActionState::NewMatchingHeading { ref_idx }
								} else {
									// same state as before (for now clone, but ok because stack/size. Can be optimized later.)
									self.action_state.clone()
								}
							}
						}
					}
				}
				// -- If we are on a content line
				LineData::Content(_) | LineData::Blockquote(_) => match self.action_state {
					ActionState::NoCapture => {
						if !self.passed_first_heading {
							if let Some(ref_idx) = filter_has_level_0_fn() {
								ActionState::NewMatchingHeading { ref_idx }
							} else {
								// Now, we check if all_sections
								if all_sections {
									ActionState::CaptureLine
								} else {
									ActionState::NoCapture
								}
							}
						} else {
							// Now, we check if all_sections
							if all_sections {
								ActionState::CaptureLine
							} else {
								ActionState::NoCapture
							}
						}
					}
					ActionState::SkipLineInCapture => ActionState::CaptureLine,
					ActionState::NewMatchingHeading { .. } => ActionState::CaptureLine,
					ActionState::NewHeadingForAllSections => ActionState::CaptureLine,
					ActionState::CaptureLine => ActionState::CaptureLine,
					ActionState::CloseCurrentSection => ActionState::NoCapture,
				},
				// NOTE: This as before, when we wanted to skip blockquote even when in capture
				//       This might comeback later, but when and config of some sort.
				// LineData::Blockquote(_) => match self.action_state {
				// 	ActionState::NoCapture | ActionState::CloseCurrentSection => ActionState::NoCapture,
				// 	_ => ActionState::SkipLineInCapture,
				// },
			};

			// -- Execute the action
			match self.action_state {
				ActionState::NoCapture | ActionState::SkipLineInCapture => (),
				ActionState::CaptureLine => match line_data {
					LineData::Content(line) | LineData::Blockquote(line) => {
						current_captured_content.get_or_insert_with(Vec::new).push(line)
					}
					LineData::Heading(line_heading) => current_captured_content
						.get_or_insert_with(Vec::new)
						.push(line_heading.into_content().into()),
				},
				ActionState::NewHeadingForAllSections => match line_data {
					LineData::Heading(line_heading) => {
						// if we are in a NewHeadingForAllSections and already capturing something, we close current
						if current_captured_heading.is_some() || current_captured_content.is_some() {
							self.last_heading = Some(line_heading);
							return close_section(&mut current_captured_content, &mut current_captured_heading, true);
						}
						// if we start a new capture, just set it
						else {
							current_captured_heading = Some(line_heading);
						}
					}
					LineData::Content(line) => {
						current_captured_content.get_or_insert_with(Vec::new).push(line);
					}
					_ => (),
				},
				ActionState::NewMatchingHeading { ref_idx } => {
					match line_data {
						LineData::Heading(line_heading) => {
							// TODO: Probably need to have the same logic as above if we start something and during capture

							let Some(matching_ref) = self.filter.get(ref_idx) else {
								// NOTE: Given the logic, this should not happen, but do not panic just return None.
								return None;
							};
							current_matching_ref = Some(matching_ref);
							current_captured_heading = Some(line_heading);
						}
						// if we are here, it must be a level 0 heading catch
						LineData::Content(line) => {
							let Some(matching_ref) = self.filter.get(ref_idx) else {
								// NOTE: Given the logic, this should not happen, but do not panic just return None.
								return None;
							};
							current_matching_ref = Some(matching_ref);
							current_captured_content.get_or_insert_with(Vec::new).push(line);
						}
						_ => (),
					}
				}
				ActionState::CloseCurrentSection => {
					return close_section(&mut current_captured_content, &mut current_captured_heading, false);
				}
			}
		}

		// - Close the last section
		close_section(&mut current_captured_content, &mut current_captured_heading, true)
	}
}

/// Iterator impl
impl Iterator for MdSectionIter<'_> {
	type Item = MdSection;

	fn next(&mut self) -> Option<Self::Item> {
		self.next_section()
	}
}

// region:    --- Splitter

/// Splitter extractor uses the iterator to split the md content in certain ways
impl MdSectionIter<'_> {
	/// Split the first MdSection out, and have the before and after as strings.
	/// NOTE: Right now, jsut take the top MdSection, so `.before` will always be empty string
	pub fn split_first(&mut self) -> MdSectionSplit {
		// -- Capture the first section
		let first = self.next_section();

		// -- Today hardcode before the ""
		let before = String::new();

		// -- Capture the after
		let after_lines: Vec<Cow<str>> = if let Some(heading) = self.last_heading.take() {
			let heading_content = heading.into_content();
			let heading_line = std::iter::once(Cow::Owned(heading_content));
			let lines = heading_line.chain(self.lines.borrow_mut());
			lines.collect()
		} else {
			self.lines.borrow_mut().collect()
		};

		let after = after_lines.join("\n");

		MdSectionSplit { before, first, after }
	}
}

// endregion: --- Splitter

// region:    --- Support Types

#[derive(Debug)]
enum LineData<'a> {
	Content(Cow<'a, str>),
	Heading(MdHeading),
	#[allow(unused)]
	Blockquote(Cow<'a, str>),
}

#[derive(Debug, Clone)]
enum ActionState {
	NoCapture,
	#[allow(unused)]
	SkipLineInCapture,
	/// We are in all sections match, and this is a new heading
	NewHeadingForAllSections,
	/// We have a new matching heading for a ref_idx in the SectionPattern array
	NewMatchingHeading {
		ref_idx: usize,
	},
	CaptureLine,
	CloseCurrentSection,
}

// endregion: --- Support Types

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;
	use crate::_test_support::{assert_contains, assert_not_contains};

	// region:    --- consts

	const FX_MD_01: &str = r#"
Some early text
# Heading 1

> Some heading-1-blockquote

Some heading-1-content

And some more heading-1-other-content

## sub heading 1-a

> Some heading-1-a-blockquote

Some heading-1-a-content

Some heading-1-a-other-content



# Heading 2

Some other content-2

```
# Some block content for content-2
```

# Heading three

 "#;

	const FX_MD_02: &str = r#"
# First heading

first heading content

## Second Heading

second heading content"#;

	// endregion: --- consts

	// TODO: Need to fix with the last Heading `# Heading three` has empty content (it wil give 4 sections and not 5)
	#[test]
	fn test_md_section_iter_no_filter_with_md_01() -> Result<()> {
		// -- Setup & Fixtures
		let fx_md = FX_MD_01;

		// -- Exec
		let sec_iter = MdSectionIter::from_str(fx_md, None)?;
		let sections = sec_iter.collect::<Vec<_>>();

		// -- Check
		assert_eq!(sections.len(), 5, "Should have only 5 sections match");
		// check first section
		let first = sections.first().ok_or("Should have first section")?;
		assert!(first.heading().is_none(), "First section heading should be none");
		assert_eq!(first.content().trim(), "Some early text");
		// Check last
		let last = sections.last().ok_or("Should have last section")?;
		let last_heading = last.heading().ok_or("Last section heading should be some")?;
		assert_eq!(last_heading.name(), "Heading three");
		assert_eq!(last_heading.level(), 1);
		assert_contains(last.content().trim(), "");

		Ok(())
	}

	#[test]
	fn test_md_section_iter_no_filter_with_md_02() -> Result<()> {
		// -- Setup & Fixtures
		let fx_md = FX_MD_02;

		// -- Exec
		let sec_iter = MdSectionIter::from_str(fx_md, None)?;
		let sections = sec_iter.collect::<Vec<_>>();

		// -- Check
		// Note: Since no filter, all section will be captured separatly
		assert_eq!(sections.len(), 3, "Should have only 3 sections");

		Ok(())
	}

	#[test]
	fn test_md_section_iter_heading_1_root() -> Result<()> {
		// -- Setup & Fixtures
		let fx_md = FX_MD_01;
		let fx_headings = &["# Heading 1"];

		// -- Exec
		let sec_iter = MdSectionIter::from_str(fx_md, Some(fx_headings))?;
		let sections = sec_iter.collect::<Vec<_>>();

		// -- Check
		assert_eq!(sections.len(), 1, "Should have only one match");
		// extract first
		let MdSection { heading, content } = sections.into_iter().next().ok_or("Should have returned a result")?;
		// check heading
		let heading = heading.ok_or("Should have a heading")?;
		assert_eq!(heading.content(), fx_headings[0].to_string());
		assert_eq!(heading.level(), 1);
		// Should contain
		assert_contains(&content, "heading-1-content");
		assert_contains(&content, "Some heading-1-a-content");
		assert_contains(&content, "Some heading-1-a-other-content");
		assert_contains(&content, "heading-1-blockquote");
		assert_contains(&content, "heading-1-a-blockquote");
		// Should not contain
		assert_not_contains(&content, fx_headings[0]);
		assert_not_contains(&content, "Some early text");
		assert_not_contains(&content, "content-2");

		Ok(())
	}

	#[test]
	fn test_md_section_iter_heading_1_a() -> Result<()> {
		// -- Setup & Fixtures
		let fx_md = FX_MD_01;
		let fx_headings = &["## sub heading 1-a"];

		// -- Exec
		let sec_iter = MdSectionIter::from_str(fx_md, Some(fx_headings))?;
		let sections = sec_iter.collect::<Vec<_>>();

		// -- Check
		assert_eq!(sections.len(), 1, "Should have only one match");
		// extract first
		let MdSection { heading, content } = sections.into_iter().next().ok_or("Should have returned a result")?;
		// check heading
		let heading = heading.ok_or("Should have a heading")?;
		assert_eq!(heading.content(), fx_headings[0].to_string());
		assert_eq!(heading.level(), 2);
		// Should contain
		assert_contains(&content, "Some heading-1-a-content");
		assert_contains(&content, "Some heading-1-a-other-content");
		assert_contains(&content, "> Some heading-1-a-blockquote");
		// Should not contain
		assert_not_contains(&content, fx_headings[0]);
		assert_not_contains(&content, "Some early text");
		assert_not_contains(&content, "content-2");
		assert_not_contains(&content, "heading-1-content");

		Ok(())
	}

	#[test]
	fn test_md_section_iter_level_0() -> Result<()> {
		// -- Setup & Fixtures
		let fx_md = FX_MD_01;
		let fx_headings = &[""];

		// -- Exec
		let sec_iter = MdSectionIter::from_str(fx_md, Some(fx_headings))?;
		let sections = sec_iter.collect::<Vec<_>>();

		// -- Check
		assert_eq!(sections.len(), 1, "Should have only one match");
		// extract first
		let MdSection { heading, content } = sections.into_iter().next().ok_or("Should have returned a result")?;

		// // check heading
		assert!(heading.is_none(), "heading should be none");
		// // Should not content
		assert_contains(&content, "Some early text");
		assert_not_contains(&content, "# Heading 1");
		assert_not_contains(&content, "content-2");
		assert_not_contains(&content, "heading-1-content");

		Ok(())
	}

	#[test]
	fn test_md_section_iter_split_first_simple() -> Result<()> {
		// -- Setup & Fixtures
		let fx_md = FX_MD_01;

		// -- Exec
		let mut sec_iter = MdSectionIter::from_str(fx_md, None)?;
		let split_first = sec_iter.split_first();

		// -- Check
		let before = split_first.before;
		let after = split_first.after;
		let first = split_first.first.ok_or("Should have a first section")?;

		// Check before
		assert_eq!(before, "", "for now, before, always empty string");
		// Check first
		assert!(first.heading().is_none(), "Should not have heading");
		// Check after

		assert_contains(&after, "# Heading 1");
		assert_contains(&after, "> Some heading-1-blockquote");
		assert_contains(&after, "## sub heading 1-a");
		assert_contains(&after, "# Heading 2");

		let heading_content = first.heading().map(|h| format!("{}\n", h.content())).unwrap_or_default();

		// Check to reconstitute
		let recon = format!("{before}{}{}{after}", heading_content, first.content);
		assert_eq!(recon, FX_MD_01);

		Ok(())
	}
}

// endregion: --- Tests
