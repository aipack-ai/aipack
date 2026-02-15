## aip.toml

Functions Summary

```lua
aip.toml.parse(content: string): table
aip.toml.stringify(content: table): string
```

### aip.toml.parse

Parse a TOML string into a Lua table.

```lua
-- API Signature
aip.toml.parse(content: string): table
```

#### Arguments

- `content: string`: The TOML string to parse.

#### Returns

- `table`: A Lua table representing the parsed TOML structure.

#### Example

```lua
local toml_str = [[
title = "Example"

[owner]
name = "John"
]]
local obj = aip.toml.parse(toml_str)
print(obj.title)
print(obj.owner.name)
```

#### Error

Returns an error (Lua table `{ error: string }`) if `content` is not valid TOML.

### aip.toml.stringify

Stringify a Lua table into a TOML string.

```lua
-- API Signature
aip.toml.stringify(content: table): string
```

#### Arguments

- `content: table`: The Lua table to stringify.

#### Returns

- `string`: A TOML-formatted string representing the table.

#### Example

```lua
local obj = {
    title = "Example",
    owner = { name = "John" }
}
local toml_str = aip.toml.stringify(obj)
```

#### Error

Returns an error (Lua table `{ error: string }`) if `content` cannot be serialized into TOML.

## aip.yaml

The `aip.yaml` module exposes functions to parse and stringify YAML content.

- Parse function will return nil if content is nil.
- Parse supports multi-document YAML and returns a [YamlDocs](#yamldocs).
- `stringify` will assume single document.
- `stringify_multi_docs` will error if content is not a [YamlDocs](#yamldocs).

### Functions Summary

```lua
aip.yaml.parse(content: string | nil): YamlDocs | nil

aip.yaml.stringify(content: any): string

aip.yaml.stringify_multi_docs(content: YamlDocs): string
```

### aip.yaml.parse

Parse a YAML string into a [YamlDocs](#yamldocs).

```lua
-- API Signature
aip.yaml.parse(content: string | nil): YamlDocs | nil
```

Parse a YAML string, which can contain multiple documents separated by `---`,
into a [YamlDocs](#yamldocs).

#### Arguments

- `content: string | nil` - The YAML string to parse. If nil, returns nil.

#### Returns

- `YamlDocs | nil` - A [YamlDocs](#yamldocs) (table with integer keys) where each element
  represents one YAML document from the input.

#### Example

```lua
local yaml_str = "name: John\n---\nname: Jane"
local docs = aip.yaml.parse(yaml_str)
print(docs[1].name) -- prints "John"
print(docs[2].name) -- prints "Jane"
```

#### Error

Returns an error (Lua table `{ error: string }`) if the input string is not valid YAML.

### aip.yaml.stringify

Stringify a value into a YAML string.

```lua
-- API Signature
aip.yaml.stringify(content: any): string
```

Convert a Lua value (usually a table) into a YAML string.

#### Arguments

- `content: any` - The Lua value to stringify.

#### Returns

- `string` - A string containing the YAML representation of the input.

#### Example

```lua
local obj = { name = "John", age = 30 }
local yaml_str = aip.yaml.stringify(obj)
```

#### Error

Returns an error (Lua table `{ error: string }`) if the value cannot be serialized into YAML.

### aip.yaml.stringify_multi_docs

Stringify a [YamlDocs](#yamldocs) into a multi-document YAML string.

```lua
-- API Signature
aip.yaml.stringify_multi_docs(content: YamlDocs): string
```

Converts a [YamlDocs](#yamldocs) into a single YAML string where each table
becomes a separate YAML document separated by `---`.

#### Arguments

- `content: YamlDocs` - A [YamlDocs](#yamldocs) to stringify.

#### Returns

- `string` - A multi-document YAML string.

#### Error

Returns an error (Lua table `{ error: string }`) if serialization fails or if the content is not a list.
