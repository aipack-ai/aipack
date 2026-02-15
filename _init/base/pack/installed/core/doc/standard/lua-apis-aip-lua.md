## aip.lua

Lua value inspection and manipulation functions.

### Functions Summary

```lua
aip.lua.dump(value: any): string

aip.lua.merge(target: table, ...objs: table | nil): table

aip.lua.merge_deep(target: table, ...objs: table | nil): table
```

### aip.lua.dump

Dump a Lua value into its string representation.

```lua
-- API Signature
aip.lua.dump(value: any): string
```

Provides a detailed string representation of any Lua value, useful for debugging.

#### Arguments

- `value: any`: The Lua value to dump.

#### Returns

- `string`: A string representation of the value.

#### Example

```lua
local tbl = { key = "value", nested = { num = 42 } }
print(aip.lua.dump(tbl))
-- Output: Example: table: 0x... { key = "value", nested = table: 0x... { num = 42 } }
```

#### Error

Returns an error (Lua table `{ error: string }`) if the value cannot be converted to string.

### aip.lua.merge

Shallow merge one or more tables into `target` (in place) and return target as well.

```lua
-- API Signature
aip.lua.merge(target: table, ...objs: table | nil): table
```

Iterates through each table in `objs` and copies its key-value pairs into `target` and return `target` as well. Later tables override earlier ones for the same key. `target` must be a table (cannot be `nil` or `null`). `objs` can also be `nil` or `null`, in which case they are ignored.

#### Arguments

- `target: table`: The table to merge into (modified in place). Cannot be `nil` or `null`.
- `...objs: table`: One or more tables whose key-value pairs are merged into `target`. Can be `nil` or `null`.

#### Returns

- `table`: The target table after merging.

#### Example

```lua
local base = { a = 1, b = 2 }
local ovl1 = { b = 3, c = 4 }
local ovl2 = { d = 5 }
aip.lua.merge(base, ovl1, ovl2)
-- base is now { a = 1, b = 3, c = 4, d = 5 }
```

#### Error

Returns an error (Lua table `{ error: string }`) if table iteration fails.

### aip.lua.merge_deep

Deep merge one or more tables into `target` (in place) and return target as well.

```lua
-- API Signature
aip.lua.merge_deep(target: table, ...objs: table | nil): table
```

Recursively merges key-value pairs from each table in `objs` into `target`. When both `target` and the overlay have a table value for the same key, those nested tables are merged recursively. Otherwise, the overlay value replaces the target value and returns target as well. `target` must be a table (cannot be `nil` or `null`). `objs` can also be `nil` or `null`, in which case they are ignored.

#### Arguments

- `target: table`: The table to merge into (modified in place). Cannot be `nil` or `null`.
- `...objs: table`: One or more tables to deep merge into `target`. Can be `nil` or `null`.

#### Returns

- `table`: The target table after merging.

#### Example

```lua
local base = { a = 1, b = { x = 10, y = 20 } }
local ovl1 = { b = { y = 22, z = 30 } }
local ovl2 = { c = 3 }
aip.lua.merge_deep(base, ovl1, ovl2)
-- base is now { a = 1, b = { x = 10, y = 22, z = 30 }, c = 3 }
```

#### Error

Returns an error (Lua table `{ error: string }`) if table iteration fails.
