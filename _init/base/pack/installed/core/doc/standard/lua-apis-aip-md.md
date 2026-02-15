## aip.md

Markdown processing functions for extracting structured information like code blocks and metadata.

### Functions Summary

```lua
aip.md.extract_blocks(md_content: string): MdBlock[]

aip.md.extract_blocks(md_content: string, lang: string): MdBlock[]

aip.md.extract_blocks(md_content: string, {lang?: string, extrude: "content"}): (MdBlock[], string)

aip.md.extract_meta(md_content: string | nil): (table | nil, string | nil)

aip.md.outer_block_content_or_raw(md_content: string): string

aip.md.extract_refs(md_content: string | nil): MdRef[]
```

### aip.md.extract_blocks

Extracts fenced code blocks ([MdBlock](#mdblock)) from markdown content.

```lua
-- API Signatures
-- Extract all blocks:
aip.md.extract_blocks(md_content: string): MdBlock[]
-- Extract blocks by language:
aip.md.extract_blocks(md_content: string, lang: string): MdBlock[]
-- Extract blocks and remaining content:
aip.md.extract_blocks(md_content: string, {lang?: string, extrude: "content"}): (MdBlock[], string)
```

Parses `md_content` and extracts fenced code blocks (``` ```).

#### Arguments

- `md_content: string`: The markdown content.
- `options?: string | table` (optional):
  - If string: Filter blocks by this language identifier.
  - If table:
    - `lang?: string`: Filter by language.
    - `extrude?: "content"`: If set, also return content outside the extracted blocks.

#### Returns

- If `extrude` is not set: `MdBlock[]`: A Lua list of [MdBlock](#mdblock) objects.
- If `extrude = "content"`: `(MdBlock[], string)`: A tuple containing the list of [MdBlock](#mdblock) objects and the remaining content string.

#### Example

```lua
local md = "```rust\nfn main() {}\n```\nSome text.\n```lua\nprint('hi')\n```"
local rust_blocks = aip.md.extract_blocks(md, "rust")
-- rust_blocks = { { content = "fn main() {}", lang = "rust" } }

local lua_blocks, remain = aip.md.extract_blocks(md, { lang = "lua", extrude = "content" })
-- lua_blocks = { { content = "print('hi')", lang = "lua", info = "" } }
-- lua_blocks = { { content = "print('hi')", lang = "lua" } }
-- remain = "Some text.\n" (approx.)
```

#### Error

Returns an error (Lua table `{ error: string }`) on invalid options or parsing errors.

### aip.md.extract_meta

Extracts and merges metadata from `#!meta` TOML blocks.

```lua
-- API Signature
aip.md.extract_meta(md_content: string | nil): (table | nil, string | nil)
```

Finds all ```toml #!meta ... ``` blocks, parses their TOML content, merges them into a single Lua table, and returns the table along with the original content stripped of the meta blocks.

#### Arguments

- `md_content: string`: The markdown content.

#### Returns

- `table`: Merged metadata from all `#!meta` blocks (empty object if not found)
- `string`: Original content with meta blocks removed.

If `md_content` the return will be `(nil, nil)`

#### Example

```lua
local content = "Intro.\n```toml\n#!meta\ntitle=\"T\"\n```\nMain.\n```toml\n#!meta\nauthor=\"A\"\n```"
local meta, remain = aip.md.extract_meta(content)
-- meta = { title = "T", author = "A" }
-- remain = "Intro.\n\nMain.\n" (approx.)
```

#### Error

Returns an error (Lua table `{ error: string }`) if any meta block contains invalid TOML.

### aip.md.extract_refs

Extracts all markdown references (links and images) from markdown content.

```lua
-- API Signature
aip.md.extract_refs(md_content: string | nil): MdRef[]
```

Scans the provided `md_content` for markdown references in the forms:
- Links: `[text](target)`
- Images: `![alt text](target)`

References inside code blocks (fenced with ``` or ````) and inline code (backticks) are skipped.

#### Arguments

- `md_content: string | nil`: The markdown content string to process.

#### Returns

- `MdRef[]`: A Lua list (table) of [MdRef](#mdref) objects. Each object represents a parsed reference.

If `md_content` is `nil`, returns an empty list (`{}`).

#### Example

```lua
local content = [[
Check out [this link](https://example.com) and [docs](docs/page.md).

Also see ![image](assets/photo.jpg) for reference.

```
[not a link](https://fake.com)
```
]]

local refs = aip.md.extract_refs(content)
print(#refs) -- Output: 3

for _, ref in ipairs(refs) do
  print(ref.target, ref.kind, ref.inline)
end
-- Output:
-- https://example.com    Url    false
-- docs/page.md           File   false
-- assets/photo.jpg       File   true
```

#### Error

Returns an error (Lua table `{ error: string }`) if an internal error occurs during processing.

### aip.md.outer_block_content_or_raw

Extracts content from the outermost code block, or returns raw content.

```lua
-- API Signature
aip.md.outer_block_content_or_raw(md_content: string): string
```

If `md_content` starts and ends with a fenced code block (```), returns the content inside. Otherwise, returns the original `md_content`. Useful for processing LLM responses.

#### Arguments

- `md_content: string`: The markdown content.

#### Returns

- `string`: Content inside the outer block, or the original string.

#### Example

```lua
local block = "```rust\ncontent\n```"
local raw = "no block"
print(aip.md.outer_block_content_or_raw(block)) -- Output: "content\n"
print(aip.md.outer_block_content_or_raw(raw))   -- Output: "no block"
```
