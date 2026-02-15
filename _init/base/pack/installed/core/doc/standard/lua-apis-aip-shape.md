## aip.shape

Functions to shape row-like Lua tables (records), convert between row- and column-oriented data, and work with record keys.

### Functions Summary

```lua
aip.shape.to_record(names: string[], values: any[]): table

aip.shape.to_records(names: string[], rows: any[][]): object[]

aip.shape.record_to_values(record: table, names?: string[]): any[]

aip.shape.records_to_value_lists(records: object[], names: string[]): any[][]

aip.shape.columnar_to_records(cols: { [string]: any[] }): object[]

aip.shape.records_to_columnar(recs: object[]): { [string]: any[] }

aip.shape.select_keys(rec: table, keys: string[]): table

aip.shape.omit_keys(rec: table, keys: string[]): table

aip.shape.remove_keys(rec: table, keys: string[]): integer

aip.shape.extract_keys(rec: table, keys: string[]): table
```

### aip.shape.to_record

Build a single record (row object) from a list of column names and a list of values. Truncates to the shorter list.

```lua
-- API Signature
aip.shape.to_record(names: string[], values: any[]): table
```

#### Arguments

- `names: string[]`: Column names (all must be strings).

- `values: any[]`: Values list.

#### Returns

- `table`: A record with keys from `names` and values from `values`.

#### Example

```lua
local rec = aip.shape.to_record({ "id", "name", "email" }, { 1, "Alice", "a@x.com" })
-- rec == { id = 1, name = "Alice", email = "a@x.com" }
```

#### Error

Returns an error (Lua table `{ error: string }`) if any entry in `names` is not a string.

### aip.shape.to_records

Build multiple records from a list of column names and a list of rows (each row is a list of values). Each row is truncated to the shorter length relative to `names`.

```lua
-- API Signature
aip.shape.to_records(names: string[], rows: any[][]): object[]
```

#### Arguments

- `names: string[]`: Column names (all must be strings).

- `rows: any[][]`: List of value lists (each must be a table).

#### Returns

- `object[]`: A list of records.

#### Example

```lua
local names = { "id", "name" }
local rows  = { { 1, "Alice" }, { 2, "Bob" } }
local out = aip.shape.to_records(names, rows)
-- out == { { id = 1, name = "Alice" }, { id = 2, name = "Bob" } }
```

#### Error

Returns an error if a name is not a string or if any row is not a table.

### aip.shape.record_to_values

Convert a single record into an array (Lua list) of values.

```lua
-- API Signature
aip.shape.record_to_values(record: table, names?: string[]): any[]
```

- When `names` is provided, values are returned in the order of `names`.
  - Missing keys yield NA sentinel entries in the result list.
  - If `names` contains a non-string entry, an error is returned.

- When `names` is not provided, values are returned in alphabetical order of the record's string keys.
  - Non-string keys are ignored.

#### Example

```lua
local rec = { id = 1, name = "Alice", email = "a@x.com" }
local v1  = aip.shape.record_to_values(rec)
-- { 1, "a@x.com", "Alice" } (alpha by keys: email, id, name)

local v2  = aip.shape.record_to_values(rec, { "name", "id", "missing" })
-- { "Alice", 1, NA }
```

#### Error

Returns an error if `names` contains a non-string entry.

### aip.shape.records_to_value_lists

Converts a list of record objects into tables of ordered values using a fixed column order.

```lua
-- API Signature
aip.shape.records_to_value_lists(records: table[], names: string[]): any[][]
```

- `records`: List of record tables containing the data to convert.
- `names`: List of column names that defines the order of the values for each row.
  Each entry must be a string.

The returned value is a Lua list where each entry is itself a list of values that
correspond to the columns declared in `names`. Missing keys emit the shared
`null` sentinel to keep the row lengths consistent.

#### Example

```lua
local recs = {
  { id = 1, name = "Alice" },
  { id = 2 },
}
local names = { "name", "id", "missing" }
return aip.shape.records_to_value_lists(recs, names)
```

The result is:

```
{
  { "Alice", 1, null },
  { null, 2, null }
}
```

#### Error

Returns an error if any entry of `names` is not a string or if any element of
`records` is not a table.

### aip.shape.columnar_to_records

Convert a column-oriented table into a list of records. All columns must be tables of the same length and keys must be strings.

```lua
-- API Signature
aip.shape.columnar_to_records(cols: { [string]: any[] }): object[]
```

#### Arguments

- `cols: { [string]: any[] }`: Map of column name (string) to list of values (table).

#### Returns

- `object[]`: A list of row records.

#### Example

```lua
local cols = {
  id    = { 1, 2, 3 },
  name  = { "Alice", "Bob", "Cara" },
  email = { "a@x.com", "b@x.com", "c@x.com" },
}
local recs = aip.shape.columnar_to_records(cols)
-- recs == {
--   { id = 1, name = "Alice", email = "a@x.com" },
--   { id = 2, name = "Bob",   email = "b@x.com" },
--   { id = 3, name = "Cara",  email = "c@x.com" },
-- }
```

#### Error

Returns an error if any key is not a string, any value is not a table, or columns have different lengths.

### aip.shape.records_to_columnar

Convert a list of records into a column-oriented table. Uses the intersection of string keys across all records to ensure rectangular output.

```lua
-- API Signature
aip.shape.records_to_columnar(recs: object[]): { [string]: any[] }
```

#### Arguments

- `recs: object[]`: List of records (each must be a table with string keys).

#### Returns

- `{ [string]: any[] }`: Columns map with values aligned by record index. Only keys present in every record are included.

#### Example

```lua
local cols = aip.shape.records_to_columnar({
  { id = 1, name = "Alice" },
  { id = 2, name = "Bob"   },
})
-- cols == { id = {1, 2}, name = {"Alice", "Bob"} }
```

#### Error

Returns an error if any record is not a table or if any key is not a string.

### aip.shape.select_keys

Return a new record with only the specified keys (original record is unchanged). Missing keys are ignored.

```lua
-- API Signature
aip.shape.select_keys(rec: table, keys: string[]): table
```

#### Arguments

- `rec: table`: Source record.

- `keys: string[]`: Keys to select (all must be strings).

#### Returns

- `table`: New record with only the selected keys.

#### Example

```lua
local rec  = { id = 1, name = "Alice", email = "a@x.com" }
local out  = aip.shape.select_keys(rec, { "id", "email" })
-- out == { id = 1, email = "a@x.com" }
```

#### Error

Returns an error if any entry in `keys` is not a string.

### aip.shape.omit_keys

Return a new record without the specified keys (original record is unchanged). Missing keys are ignored.

```lua
-- API Signature
aip.shape.omit_keys(rec: table, keys: string[]): table
```

#### Arguments

- `rec: table`: Source record.

- `keys: string[]`: Keys to omit (all must be strings).

#### Returns

- `table`: New record with keys omitted.

#### Example

```lua
local rec  = { id = 1, name = "Alice", email = "a@x.com" }
local out  = aip.shape.omit_keys(rec, { "email" })
-- out == { id = 1, name = "Alice" }
```

#### Error

Returns an error if any entry in `keys` is not a string.

### aip.shape.remove_keys

Remove the specified keys from the original record (in place) and return the number of keys actually removed. Missing keys are ignored.

```lua
-- API Signature
aip.shape.remove_keys(rec: table, keys: string[]): integer
```

#### Arguments

- `rec: table`: Record to mutate.

- `keys: string[]`: Keys to remove (all must be strings).

#### Returns

- `integer`: Count of removed keys.

#### Example

```lua
local rec = { id = 1, name = "Alice", email = "a@x.com" }
local n   = aip.shape.remove_keys(rec, { "email", "missing" })
-- n   == 1
-- rec == { id = 1, name = "Alice" }
```

#### Error

Returns an error if any entry in `keys` is not a string.

### aip.shape.extract_keys

Return a new record containing only the specified keys and remove them from the original record (in place). Missing keys are ignored.

```lua
-- API Signature
aip.shape.extract_keys(rec: table, keys: string[]): table
```

#### Arguments

- `rec: table`: Record to extract from and mutate.

- `keys: string[]`: Keys to extract (all must be strings).

#### Returns

- `table`: New record containing the extracted key-value pairs.

#### Example

```lua
local rec      = { id = 1, name = "Alice", email = "a@x.com" }
local picked   = aip.shape.extract_keys(rec, { "id", "email" })
-- picked == { id = 1, email = "a@x.com" }
-- rec    == { name = "Alice" }
```

#### Error

Returns an error if any entry in `keys` is not a string.
