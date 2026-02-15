## aip.csv

CSV parsing and processing functions for both individual rows and complete CSV content.

### Functions Summary

```lua
aip.csv.parse_row(row: string, options?: CsvOptions): string[]

aip.csv.parse(content: string, options?: CsvOptions): CsvContent

aip.csv.values_to_row(values: any[], options?: CsvOptions): string

aip.csv.value_lists_to_rows(value_lists: any[][], options?: CsvOptions): string[]
```

### aip.csv.parse_row

Parse a single CSV row according to the specified options.

```lua
-- API Signature
aip.csv.parse_row(row: string, options?: CsvOptions): string[]
```

Parses a single CSV row string into an array of field values. Non-applicable options (`has_header`, `skip_empty_lines`, `comment`) are ignored for this function.

#### Arguments

- `row: string`: The CSV row string to parse.
- `options?: [CsvOptions](#csvoptions)` (optional): CSV parsing options. See [CsvOptions](#csvoptions) for details. Only `delimiter`, `quote`, `escape`, and `trim_fields` apply to this function.

#### Returns

- `string[]`: A Lua list of strings representing the parsed fields from the row.

#### Example

```lua
local row = 'a,"b,c",d'
local fields = aip.csv.parse_row(row)
-- fields = {"a", "b,c", "d"}

-- With custom delimiter
local fields_custom = aip.csv.parse_row("a;b;c", {delimiter = ";"})
-- fields_custom = {"a", "b", "c"}
```

#### Error

Returns an error (Lua table `{ error: string }`) if the row cannot be parsed or options are invalid.

### aip.csv.parse

Parse CSV content with optional header detection and comment skipping.

```lua
-- API Signature
aip.csv.parse(content: string, options?: CsvOptions): CsvContent
```

Parses complete CSV content and returns a `CsvContent` table containing headers (empty array when no header row is requested) and all data rows.

#### Arguments

- `content: string`: The CSV content string to parse.
- `options?: [CsvOptions](#csvoptions)` (optional): CSV parsing options. See [CsvOptions](#csvoptions) for details. All options are applicable to this function.

#### Returns

- `[CsvContent](#csvcontent)`: Matches the [CsvContent](#csvcontent) structure (same as `aip.file.load_csv`), including the `_type = "CsvContent"` marker alongside the `headers` and `rows` fields.

#### Example

```lua
local csv_content = [[
# This is a comment
name,age,city
John,30,New York
Jane,25,Boston
]]

local result = aip.csv.parse(csv_content, {
  has_header = true,
  comment = "#",
  skip_empty_lines = true
})
-- result.headers = {"name", "age", "city"}
-- result.rows = { {"John", "30", "New York"}, {"Jane", "25", "Boston"} }

-- Parse without headers
local result_no_headers = aip.csv.parse("a,b,c\n1,2,3", {has_header = false})
-- result_no_headers.headers = {}
-- result_no_headers.rows = { {"a", "b", "c"}, {"1", "2", "3"} }
```

#### Error

Returns an error (Lua table `{ error: string }`) if the content cannot be parsed or options are invalid.

### aip.csv.values_to_row

Converts a list of Lua values into a single CSV row string.

```lua
-- API Signature
aip.csv.values_to_row(values: any[], options?: CsvOptions): string
```

Each entry in `values` can be a string, number, boolean, `nil`, the AIPack `null` sentinel, or a table.
Tables are converted to JSON strings before being written. `nil` and `null` entries become empty fields.

#### Arguments

- `values: any[]`: Lua list of values to serialize into a CSV row.
- `options?: [CsvOptions](#csvoptions)` (optional): Optional `CsvOptions` (e.g., `delimiter`, `quote`, `escape`).

#### Returns

- `string`: A CSV-formatted row string following RFC 4180 quoting rules.

#### Example

```lua
local row = aip.csv.values_to_row({"a", 123, true, nil, { foo = "bar" }})
-- row == "a,123,true,,{\"foo\":\"bar\"}"
```

#### Error

Returns an error (Lua table `{ error: string }`) if an unsupported type (e.g., function, thread) is encountered or serialization fails.

### aip.csv.value_lists_to_rows

Converts a list of value lists into a list of CSV row strings.

```lua
-- API Signature
aip.csv.value_lists_to_rows(value_lists: any[][], options?: CsvOptions): string[]
```

Uses `aip.csv.values_to_row` for each inner list and returns all resulting CSV rows.

#### Arguments

- `value_lists: any[][]`: Lua list of lists representing rows. Each inner list follows the same type rules as `aip.csv.values_to_row`.
- `options?: [CsvOptions](#csvoptions)` (optional): Optional `CsvOptions` (e.g., `delimiter`, `quote`, `escape`).

#### Returns

- `string[]`: Lua list of CSV-formatted row strings.

#### Example

```lua
local rows = aip.csv.value_lists_to_rows({
  {"a", 1},
  {"b,c", 2}
})
-- rows == { "a,1", "\"b,c\",2" }
```

#### Error

Returns an error (Lua table `{ error: string }`) if `value_lists` is not a table, an entry is not a list, or any contained value cannot be serialized.
