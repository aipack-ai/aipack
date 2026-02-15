## aip.tag

Functions for extracting content based on custom XML-like tags (e.g., `<FILE>...</FILE>`).

### Functions Summary

```lua
aip.tag.extract(content: string, tag_names: string | string[], options?: {extrude?: "content"}): TagElem[] | (TagElem[], string)
aip.tag.extract_as_map(content: string, tag_names: string | string[], options?: {extrude?: "content"}): { [string]: TagElem } | ({ [string]: TagElem }, string)
aip.tag.extract_as_multi_map(content: string, tag_names: string | string[], options?: {extrude?: "content"}): { [string]: TagElem[] } | ({ [string]: TagElem[] }, string)
```

### aip.tag.extract

Extracts content blocks enclosed by matching start and end tags (e.g., `<TAG>content</TAG>`).

```lua
-- API Signature
aip.tag.extract(
  content: string,
  tag_names: string | string[],
  options?: { extrude?: "content" }
): TagElem[] | (TagElem[], string)
```

Finds matching tag pairs in `content` based on `tag_names`.

#### Arguments

- `content: string`: The content to search within.
- `tag_names: string | string[]`: A single tag name (e.g., `"FILE"`) or a list of tag names to look for. Case sensitive.
- `options?: table` (optional):
  - `extrude?: "content"` (optional): If set to `"content"`, the function returns two values: the list of extracted blocks and a string containing the remaining content outside the tags.

#### Returns

- If `extrude` is not set: `TagElem[]`: A Lua list of [TagElem](#tagelem) objects.
- If `extrude = "content"`: `(TagElem[], string)`: A tuple containing the list of [TagElem](#tagelem) objects and the extruded content string.

#### Example

```lua
local content = "Prefix <A file=readme.md>one</A> middle <B>two</B> Suffix"

-- Extract all blocks, return only the list
local blocks = aip.tag.extract(content, {"A", "B"})
-- blocks[1].tag == "A", blocks[1].content == "one"
-- blocks[1].attrs.file == "readme.md"

-- Extract blocks and also get the remaining text
local extracted_blocks, remaining_text = aip.tag.extract(content, "A", { extrude = "content" })
-- remaining_text == "Prefix  middle <B>two</B> Suffix" (approx)
```

#### Error

Returns an error (Lua table `{ error: string }`) if `tag_names` is invalid (e.g., empty string or list, or contains non-string elements).


### aip.tag.extract_as_map

Extracts content blocks and returns them as a map, where the key is the tag name. If multiple blocks share the same tag name, only the last one is returned.

```lua
-- API Signature
aip.tag.extract_as_map(
  content: string,
  tag_names: string | string[],
  options?: { extrude?: "content" }
): { [string]: TagElem } | ({ [string]: TagElem }, string)
```

Finds matching tag pairs in `content` based on `tag_names`.

#### Arguments

- `content: string`: The content to search within.
- `tag_names: string | string[]`: A single tag name (e.g., `"FILE"`) or a list of tag names to look for. Case sensitive.
- `options?: table` (optional):
  - `extrude?: "content"` (optional): If set to `"content"`, the function returns two values: the map of extracted blocks and a string containing the remaining content outside the tags.

#### Returns

- If `extrude` is not set: `{ [string]: TagElem }`: A Lua table mapping tag name (`string`) to the last extracted [TagElem](#tagelem) object.
- If `extrude = "content"`: `({ [string]: TagElem }, string)`: A tuple containing the map of [TagElem](#tagelem) objects and the extruded content string.

#### Example

```lua
local content = "First <A>one</A> <B>two</B> Last <A>three</A>"

-- Extract all blocks, return map (A will contain 'three')
local map = aip.tag.extract_as_map(content, {"A", "B"})
-- map.A.tag == "A", map.A.content == "three"
-- map.B.tag == "B", map.B.content == "two"
```

#### Error

Returns an error (Lua table `{ error: string }`) if `tag_names` is invalid (e.g., empty string or list, or contains non-string elements).


### aip.tag.extract_as_multi_map

Extracts content blocks and returns them as a map, where the key is the tag name and the value is a list of all matching [TagElem](#tagelem) blocks found for that tag.

```lua
-- API Signature
aip.tag.extract_as_multi_map(
  content: string,
  tag_names: string | string[],
  options?: { extrude?: "content" }
): { [string]: TagElem[] } | ({ [string]: TagElem[] }, string)
```

Finds matching tag pairs in `content` based on `tag_names`.

#### Arguments

- `content: string`: The content to search within.
- `tag_names: string | string[]`: A single tag name (e.g., `"FILE"`) or a list of tag names to look for. Case sensitive.
- `options?: table` (optional):
  - `extrude?: "content"` (optional): If set to `"content"`, the function returns two values: the map of extracted block lists and a string containing the remaining content outside the tags.

#### Returns

- If `extrude` is not set: `{ [string]: TagElem[] }`: A Lua table mapping tag name (`string`) to a list of all extracted [TagElem](#tagelem) objects for that tag.
- If `extrude = "content"`: `({ [string]: TagElem[] }, string)`: A tuple containing the map of lists of [TagElem](#tagelem) objects and the extruded content string.

#### Example

```lua
local content = "First <A>one</A> <B>two</B> Last <A>three</A>"

-- Extract all blocks, return map of lists (A will contain both 'one' and 'three')
local map = aip.tag.extract_as_multi_map(content, {"A", "B"})
-- map.A[1].tag == "A", map.A[1].content == "one"
-- map.A[2].tag == "A", map.A[2].content == "three"
-- map.B[1].tag == "B", map.B[1].content == "two"
```

#### Error

Returns an error (Lua table `{ error: string }`) if `tag_names` is invalid (e.g., empty string or list, or contains non-string elements).
