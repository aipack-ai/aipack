## aip.hbs

Functions for rendering Handlebars templates.

### Functions Summary

```lua
aip.hbs.render(content: string, data: any): string | {error: string}
```

### aip.hbs.render

Renders a Handlebars template string with Lua data.

```lua
-- API Signature
aip.hbs.render(content: string, data: any): string | {error: string}
```

Converts Lua `data` to JSON internally and renders the Handlebars `content` template.

#### Arguments

- `content: string`: The Handlebars template string.
- `data: any`: The data as a Lua value (table, number, string, boolean, nil). Note that function types or userdata are not supported.

#### Returns

- `string`: The rendered template string on success.
- `{error: string}`: An error table on failure (data conversion or template rendering).

#### Example

```lua
local template = "Hello, \{{name}}!"
local data = {name = "World"}
local rendered_content = aip.hbs.render(template, data)
print(rendered_content) -- Output: Hello, World!

local data_list = {
    name  = "Jen Donavan",
    todos = {"Bug Triage AIPack", "Fix Windows Support"}
}
local template_list = [[
Hello \{{name}},

Your tasks today:

\{{#each todos}}
- \{{this}}
\{{/each}}

Have a good day (after you completed this tasks)
]]
local content_list = aip.hbs.render(template_list, data_list)
print(content_list)
```

_NOTE: Do not prefix with `\` the `\{{` (this is just for internal templating for the website)_

#### Error

Returns an error (Lua table `{ error: string }`) if Lua data cannot be converted to JSON or if Handlebars rendering fails.
