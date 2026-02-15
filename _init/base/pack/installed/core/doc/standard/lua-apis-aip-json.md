## aip.json

JSON parsing and stringification functions.

### Functions Summary

```lua
aip.json.parse(content: string | nil): table | value | nil

aip.json.parse_ndjson(content: string | nil): object[] | nil

aip.json.stringify(content: table): string

aip.json.stringify_pretty(content: table): string

aip.json.stringify_to_line(content: table): string -- Deprecated alias for `stringify`
```

### aip.json.parse

Parse a JSON string into a Lua table or value.

```lua
-- API Signature
aip.json.parse(content: string | nil): table | value | nil
```

#### Arguments

- `content: string | nil`: The JSON string to parse. If `nil`, returns `nil`.

#### Returns

- `table | value | nil`: A Lua value representing the parsed JSON. Returns `nil` if `content` was `nil`.

#### Example

```lua
local obj = aip.json.parse('{"name": "John", "age": 30}')
print(obj.name) -- Output: John
```

#### Error

Returns an error (Lua table `{ error: string }`) if `content` is not valid JSON.

### aip.json.parse_ndjson

Parse a newline-delimited JSON (NDJSON) string into a list of tables/values.

```lua
-- API Signature
aip.json.parse_ndjson(content: string | nil): object[] | nil
```

Parses each non-empty line as a separate JSON object/value.

#### Arguments

- `content: string | nil`: The NDJSON string. If `nil`, returns `nil`.

#### Returns

- `object[] | nil`: A Lua list containing the parsed value from each line, or `nil` if `content` was `nil`.

#### Example

```lua
local ndjson = '{"id":1}\n{"id":2}'
local items = aip.json.parse_ndjson(ndjson)
print(items[1].id) -- Output: 1
print(items[2].id) -- Output: 2
```

#### Error

Returns an error (Lua table `{ error: string }`) if any line contains invalid JSON.

### aip.json.stringify

Stringify a Lua table/value into a compact, single-line JSON string.

```lua
-- API Signature
aip.json.stringify(content: table): string
```

#### Arguments

- `content: table`: The Lua table/value to stringify.

#### Returns

- `string`: A single-line JSON string representation.

#### Example

```lua
local obj = {name = "John", age = 30}
local json_str = aip.json.stringify(obj)
-- json_str = '{"age":30,"name":"John"}' (order may vary)
```

#### Error

Returns an error (Lua table `{ error: string }`) if `content` cannot be serialized.

### aip.json.stringify_pretty

Stringify a Lua table/value into a pretty-formatted JSON string (2-space indent).

```lua
-- API Signature
aip.json.stringify_pretty(content: table): string
```

#### Arguments

- `content: table`: The Lua table/value to stringify.

#### Returns

- `string`: A formatted JSON string with newlines and indentation.

#### Example

```lua
local obj = {name = "John", age = 30}
local json_str = aip.json.stringify_pretty(obj)
-- json_str =
-- {
--   "age": 30,
--   "name": "John"
-- } (order may vary)
```

#### Error

Returns an error (Lua table `{ error: string }`) if `content` cannot be serialized.

### aip.json.stringify_to_line (Deprecated)

Deprecated alias for `aip.json.stringify`.

```lua
-- API Signature
aip.json.stringify_to_line(content: table): string
```
