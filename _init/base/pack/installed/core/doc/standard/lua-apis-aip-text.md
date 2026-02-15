## aip.text

Text manipulation functions for cleaning, splitting, modifying, and extracting text content.

### Functions Summary

```lua
aip.text.escape_decode(content: string | nil): string | nil

aip.text.escape_decode_if_needed(content: string | nil): string | nil

aip.text.split_first(content: string | nil, sep: string): (string | nil, string | nil)

aip.text.split_last(content: string | nil, sep: string): (string | nil, string | nil)

aip.text.remove_first_line(content: string | nil): string | nil

aip.text.remove_first_lines(content: string | nil, n: number): string | nil

aip.text.remove_last_line(content: string | nil): string | nil

aip.text.remove_last_lines(content: string | nil, n: number): string | nil

aip.text.trim(content: string | nil): string | nil

aip.text.trim_start(content: string | nil): string | nil

aip.text.trim_end(content: string | nil): string | nil

aip.text.remove_last_lines(content: string | nil, n: number): string | nil

aip.text.truncate(content: string | nil, max_len: number, ellipsis?: string): string | nil

aip.text.replace_markers(content: string | nil, new_sections: list): string | nil

aip.text.ensure(content: string | nil, {prefix?: string, suffix?: string}): string | nil

aip.text.ensure_single_trailing_newline(content: string | nil): string | nil

aip.text.ensure_single_ending_newline(content: string | nil): string | nil -- Deprecated: Use aip.text.ensure_single_trailing_newline

aip.text.format_size(bytes: integer | nil, lowest_size_unit?: "B" | "KB" | "MB" | "GB"): string | nil -- lowest_size_unit default "B"

aip.text.extract_line_blocks(content: string | nil, options: {starts_with: string, extrude?: "content", first?: number}): (string[] | nil, string | nil)

aip.text.split_first_line(content: string | nil, sep: string): (string | nil, string | nil)

aip.text.split_last_line(content: string | nil, sep: string): (string | nil, string | nil)
```

### aip.text.escape_decode

HTML-decodes the entire content string. If `content` is `nil`, returns `nil`.

```lua
-- API Signature
aip.text.escape_decode(content: string | nil): string | nil
```

Useful for decoding responses from LLMs that might HTML-encode output.

#### Arguments

- `content: string | nil`: The content to HTML-decode. If `nil`, the function returns `nil`.

#### Returns

- `string | nil`: The decoded string, or `nil` if the input `content` was `nil`.

#### Error

Returns an error (Lua table `{ error: string }`) if decoding fails (and content was not `nil`).

### aip.text.escape_decode_if_needed

Selectively HTML-decodes content if needed (currently only decodes `&lt;`). If `content` is `nil`, returns `nil`.

```lua
-- API Signature
aip.text.escape_decode_if_needed(content: string | nil): string | nil
```

A more conservative version of `escape_decode` for cases where only specific entities need decoding.

#### Arguments

- `content: string | nil`: The content to process. If `nil`, the function returns `nil`.

#### Returns

- `string | nil`: The potentially decoded string, or `nil` if the input `content` was `nil`.

#### Error

Returns an error (Lua table `{ error: string }`) if decoding fails (and content was not `nil`).

### aip.text.split_first

Splits a string into two parts based on the first occurrence of a separator. If `content` is `nil`, returns `(nil, nil)`.

```lua
-- API Signature
aip.text.split_first(content: string | nil, sep: string): (string | nil, string | nil)
```

#### Arguments

- `content: string | nil`: The string to split. If `nil`, the function returns `(nil, nil)`.
- `sep: string`: The separator string.

#### Returns

- `string | nil`: The part before the first separator. `nil` if `content` was `nil` or separator not found.
- `string | nil`: The part after the first separator. `nil` if `content` was `nil` or separator not found. Empty string if separator is at the end.

#### Example

```lua
local content = "first part===second part"
local first, second = aip.text.split_first(content, "===")
-- first = "first part"
-- second = "second part"
```

#### Error

This function does not typically error.

### aip.text.split_last

Splits a string into two parts based on the last occurrence of a separator. If `content` is `nil`, returns `(nil, nil)`.

```lua
-- API Signature
aip.text.split_last(content: string | nil, sep: string): (string | nil, string | nil)
```

#### Arguments

- `content: string | nil`: The string to split. If `nil`, the function returns `(nil, nil)`.
- `sep: string`: The separator string.

#### Returns

- `string | nil`: The part before the last separator. `nil` if `content` was `nil` or separator not found.
- `string | nil`: The part after the last separator. `nil` if `content` was `nil` or separator not found. Empty string if separator is at the end.

#### Example

```lua
local content = "some == text == more"
local first, second = aip.text.split_last(content, "==")
-- first = "some == text "
-- second = " more"

local content = "no separator here"
local first, second = aip.text.split_last(content, "++")
-- first = "no separator here"
-- second = nil
```

#### Error

This function does not typically error.

### aip.text.remove_first_line

Removes the first line from the content. If `content` is `nil`, returns `nil`.

```lua
-- API Signature
aip.text.remove_first_line(content: string | nil): string | nil
```

#### Arguments

- `content: string | nil`: The content to process. If `nil`, the function returns `nil`.

#### Returns

- `string | nil`: The content with the first line removed, or `nil` if the input `content` was `nil`.

#### Error

This function does not typically error.

### aip.text.remove_first_lines

Removes the first `n` lines from the content. If `content` is `nil`, returns `nil`.

```lua
-- API Signature
aip.text.remove_first_lines(content: string | nil, n: number): string | nil
```

#### Arguments

- `content: string | nil`: The content to process. If `nil`, the function returns `nil`.
- `n: number`: The number of lines to remove.

#### Returns

- `string | nil`: The content with the first `n` lines removed, or `nil` if the input `content` was `nil`.

#### Error

This function does not typically error.

### aip.text.remove_last_line

Removes the last line from the content. If `content` is `nil`, returns `nil`.

```lua
-- API Signature
aip.text.remove_last_line(content: string | nil): string | nil
```

#### Arguments

- `content: string | nil`: The content to process. If `nil`, the function returns `nil`.

#### Returns

- `string | nil`: The content with the last line removed, or `nil` if the input `content` was `nil`.

#### Error

This function does not typically error.

### aip.text.remove_last_lines

Removes the last `n` lines from the content. If `content` is `nil`, returns `nil`.

```lua
-- API Signature
aip.text.remove_last_lines(content: string | nil, n: number): string | nil
```

#### Arguments

- `content: string | nil`: The content to process. If `nil`, the function returns `nil`.
- `n: number`: The number of lines to remove.

#### Returns

- `string | nil`: The content with the last `n` lines removed, or `nil` if the input `content` was `nil`.

#### Error

This function does not typically error.

### aip.text.trim

Trims leading and trailing whitespace from a string. If `content` is `nil`, returns `nil`.

```lua
-- API Signature
aip.text.trim(content: string | nil): string | nil
```

#### Arguments

- `content: string | nil`: The string to trim. If `nil`, the function returns `nil`.

#### Returns

- `string | nil`: The trimmed string, or `nil` if the input `content` was `nil`.

#### Error

This function does not typically error.

### aip.text.trim_start

Trims leading whitespace from a string. If `content` is `nil`, returns `nil`.

```lua
-- API Signature
aip.text.trim_start(content: string | nil): string | nil
```

#### Arguments

- `content: string | nil`: The string to trim. If `nil`, the function returns `nil`.

#### Returns

- `string | nil`: The trimmed string, or `nil` if the input `content` was `nil`.

#### Error

This function does not typically error.

### aip.text.trim_end

Trims trailing whitespace from a string. If `content` is `nil`, returns `nil`.

```lua
-- API Signature
aip.text.trim_end(content: string | nil): string | nil
```

#### Arguments

- `content: string | nil`: The string to trim. If `nil`, the function returns `nil`.

#### Returns

- `string | nil`: The trimmed string, or `nil` if the input `content` was `nil`.

#### Error

This function does not typically error.

### aip.text.truncate

Truncates content to a maximum length, optionally adding an ellipsis. If `content` is `nil`, returns `nil`.

```lua
-- API Signature
aip.text.truncate(content: string | nil, max_len: number, ellipsis?: string): string | nil
```

If `content` length exceeds `max_len`, truncates and appends `ellipsis` (if provided).

#### Arguments

- `content: string | nil`: The content to truncate. If `nil`, the function returns `nil`.
- `max_len: number`: The maximum length of the result.
- `ellipsis?: string` (optional): String to append if truncated (e.g., "...").

#### Returns

- `string | nil`: The truncated string, or `nil` if the input `content` was `nil`.

#### Error

This function does not typically error.

### aip.text.replace_markers

Replaces `<<START>>...<<END>>` markers in content with corresponding sections. If `content` is `nil`, returns `nil`.

```lua
-- API Signature
aip.text.replace_markers(content: string | nil, new_sections: list): string | nil
```

Replaces occurrences of `<<START>>...<<END>>` blocks sequentially with items from `new_sections`. Items in `new_sections` can be strings or tables with a `.content` field.

#### Arguments

- `content: string | nil`: The content containing `<<START>>...<<END>>` markers. If `nil`, the function returns `nil`.
- `new_sections: list`: A Lua list of strings or tables to replace the markers.

#### Returns

- `string | nil`: The string with markers replaced, or `nil` if the input `content` was `nil`.

#### Error

This function does not typically error.

### aip.text.ensure

Ensures the content starts with `prefix` and/or ends with `suffix`. If `content` is `nil`, returns `nil`.

```lua
-- API Signature
aip.text.ensure(content: string | nil, {prefix?: string, suffix?: string}): string | nil
```

Adds the prefix/suffix only if the content doesn't already start/end with it.

#### Arguments

- `content: string | nil`: The content to process. If `nil`, the function returns `nil`.
- `options: table`: A table with optional `prefix` and `suffix` string keys.

#### Returns

- `string | nil`: The ensured string, or `nil` if the input `content` was `nil`.

#### Error

This function does not typically error.

### aip.text.ensure_single_trailing_newline

Ensures the content ends with exactly one newline character (`\n`). If `content` is `nil`, returns `nil`.

```lua
-- API Signature
aip.text.ensure_single_trailing_newline(content: string | nil): string | nil
```

Removes trailing whitespace and adds a single newline if needed. Returns `\n` if content is empty. Useful for code normalization.

#### Arguments

- `content: string | nil`: The content to process. If `nil`, the function returns `nil`.

#### Returns

- `string | nil`: The string ending with a single newline, or `nil` if the input `content` was `nil`.

#### Error

This function does not typically error.

### aip.text.ensure_single_ending_newline (Deprecated)

Deprecated alias for `aip.text.ensure_single_trailing_newline`.

```lua
-- API Signature
aip.text.ensure_single_ending_newline(content: string | nil): string | nil
```

### aip.text.format_size

Formats a byte count (in bytes) into a human-readable, fixed-width string (9 characters, right-aligned).  
If `bytes` is `nil`, the function returns `nil`.

Optional lowest unit size to be used (by default "B" for Bytes)

```lua
-- API Signature
aip.text.format_size(bytes: integer | nil, lowest_size_unit?: "B" | "KB" | "MB" | "GB"): string | nil
```

### Examples

```lua
aip.text.format_size(777)          -- "   777 B "
aip.text.format_size(8_777)        -- "  8.78 KB"
aip.text.format_size(5_242_880)    -- "  5.24 MB"
aip.text.format_size(nil)          -- nil
```

### aip.text.extract_line_blocks

Extracts consecutive lines starting with a specific prefix. If `content` is `nil`, returns `(nil, nil)`.

```lua
-- API Signature
aip.text.extract_line_blocks(content: string | nil, options: {starts_with: string, extrude?: "content", first?: number}): (string[] | nil, string | nil)
```

Extracts blocks of consecutive lines from `content` where each line begins with `options.starts_with`.

#### Arguments

- `content: string | nil`: The content to extract from. If `nil`, the function returns `(nil, nil)`.
- `options: table`:
  - `starts_with: string` (required): The prefix indicating a line block.
  - `extrude?: "content"` (optional): If set, returns the remaining content after extraction as the second return value.
  - `first?: number` (optional): Limits the number of blocks extracted. Remaining lines (if any) contribute to the extruded content if `extrude` is set.

#### Returns

- `string[] | nil`: A Lua list of strings, each element being a block of consecutive lines starting with the prefix. `nil` if input `content` was `nil`.
- `string | nil`: The remaining content if `extrude = "content"`, otherwise `nil`. `nil` if input `content` was `nil`.

#### Example

```lua
local text = "> Block 1 Line 1\n> Block 1 Line 2\nSome other text\n> Block 2"
local blocks, remain = aip.text.extract_line_blocks(text, {starts_with = ">", extrude = "content"})
-- blocks = { "> Block 1 Line 1\n> Block 1 Line 2", "> Block 2" }
-- remain = "Some other text\n"
```

#### Error

Returns an error (Lua table `{ error: string }`) if arguments are invalid (and content was not `nil`).

### aip.text.split_first_line

Splits a string into two parts based on the *first* line that exactly matches the separator. If `content` is `nil`, returns `(nil, nil)`. If no line matches, returns `(original_content, nil)`.

```lua
-- API Signature
aip.text.split_first_line(content: string | nil, sep: string): (string | nil, string | nil)
```

The separator line itself is not included in either part.

#### Arguments

- `content: string | nil`: The string to split. If `nil`, the function returns `(nil, nil)`.
- `sep: string`: The exact string the line must match.

#### Returns

- `string | nil`: The part before the first matching line. `nil` if `content` was `nil`. Empty string if the first matching line was the first line.
- `string | nil`: The part after the first matching line. `nil` if `content` was `nil` or no line matched `sep`. Empty string if the first matching line was the last line.

#### Example

```lua
local text = "line one\n---\nline two\n---\nline three"
local first, second = aip.text.split_first_line(text, "---")
-- first = "line one"
-- second = "line two\n---\nline three"

local first, second = aip.text.split_first_line("START\ncontent", "START")
-- first = ""
-- second = "content"

local first, second = aip.text.split_first_line("no separator", "---")
-- first = "no separator"
-- second = nil
```

#### Error

This function does not typically error.

### aip.text.split_last_line

Splits a string into two parts based on the *last* line that exactly matches the separator. If `content` is `nil`, returns `(nil, nil)`. If no line matches, returns `(original_content, nil)`.

```lua
-- API Signature
aip.text.split_last_line(content: string | nil, sep: string): (string | nil, string | nil)
```

The separator line itself is not included in either part.

#### Arguments

- `content: string | nil`: The string to split. If `nil`, the function returns `(nil, nil)`.
- `sep: string`: The exact string the line must match.

#### Returns

- `string | nil`: The part before the last matching line. `nil` if `content` was `nil` or no line matched `sep`.
- `string | nil`: The part after the last matching line. `nil` if `content` was `nil` or no line matched `sep`. Empty string if the last matching line was the last line.

#### Example

```lua
local text = "line one\n---\nline two\n---\nline three"
local first, second = aip.text.split_last_line(text, "---")
-- first = "line one\n---\nline two"
-- second = "line three"

local first, second = aip.text.split_last_line("content\nEND", "END")
-- first = "content"
-- second = ""

local first, second = aip.text.split_last_line("no separator", "---")
-- first = "no separator"
-- second = nil
```

#### Error

This function does not typically error.
