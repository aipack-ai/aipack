## aip.html

Functions for processing HTML content.

### Functions Summary

```lua
aip.html.slim(html_content: string): string | {error: string}

aip.html.select(html_content: string, selectors: string | string[]): Elem[]

aip.html.to_md(html_content: string): string | {error: string}
```

### aip.html.slim

Strips non-content elements and most attributes from HTML.

```lua
-- API Signature
aip.html.slim(html_content: string): string | {error: string}
```

Removes `<script>`, `<link>`, `<style>`, `<svg>`, comments, empty lines, and most attributes (keeps `class`, `aria-label`, `href`).

#### Arguments

- `html_content: string`: The HTML content string.

#### Returns

- `string`: The cleaned HTML string on success.
- `{error: string}`: An error table on failure.

#### Example

```lua
local html = "<script>alert('hi')</script><p class='c' style='color:red'>Hello</p>"
local cleaned = aip.html.slim(html)
-- cleaned might be: "<p class=\"c\">Hello</p>" (exact output may vary)
```

#### Error

Returns an error (Lua table `{ error: string }`) if pruning fails.

### aip.html.select

Selects elements from HTML content using CSS selectors.

```lua
-- API Signature
aip.html.select(
  html_content: string,
  selectors: string | string[]
): Elem[]
```

Parses `html_content`, applies the CSS `selectors`, and returns a list of matching elements.

#### Arguments

- `html_content: string`: The HTML content to search within.
- `selectors: string | string[]`: One or more CSS selector strings.

#### Returns

- `Elem[]`: A Lua list of tables, where each table represents an element (`Elem`). Returns an empty list if no elements match.

#### Elem Structure

Each element table (`Elem`) has the following structure:

```ts
{
  tag: string,          // HTML tag name (e.g., "div", "a", "p")
  attrs?: table,        // Key/value map of attributes (only present if element has attributes)
  text?: string,        // Concatenated, trimmed text content inside the element (excluding child tags)
  inner_html?: string,  // Raw, trimmed inner HTML content (including child tags)
}
```

> Note: The `text` and `inner_html` fields are automatically trimmed of leading/trailing whitespace. The `attrs` field is omitted if the element has no attributes.

#### Example

```lua
local html = "<div><a href='#' class='link'>Click Here</a></div>"
local elements = aip.html.select(html, ".link")
-- elements[1].tag        -- "a"
-- elements[1].attrs.class -- "link"
-- elements[1].text       -- "Click Here"
```

#### Error

Returns an error (Lua table `{ error: string }`) if selector parsing fails or HTML parsing issues occur.

### aip.html.to_md

Converts HTML content to Markdown format.

```lua
-- API Signature
aip.html.to_md(html_content: string): string | {error: string}
```

#### Arguments

- `html_content: string`: The HTML content to be converted.

#### Returns

- `string`: The Markdown representation of the HTML content.
- `{error: string}`: An error table on failure.

#### Example

```lua
local markdown_content = aip.html.to_md("<h1>Hello</h1><p>World</p>")
-- markdown_content will be "# Hello\n\nWorld\n"
```

#### Error

Returns an error (Lua table `{ error: string }`) if the HTML content fails to be converted to Markdown.
