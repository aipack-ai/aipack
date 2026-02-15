## aip.rust

Functions for processing Rust code.

### Functions Summary

```lua
aip.rust.prune_to_declarations(code: string): string | {error: string}
```

### aip.rust.prune_to_declarations

Prunes Rust code, replacing function bodies with `{ ... }`.

```lua
-- API Signature
aip.rust.prune_to_declarations(code: string): string | {error: string}
```

Replaces function bodies with `{ ... }`, preserving comments, whitespace, and non-function code structures.

#### Arguments

- `code: string`: The Rust code to prune.

#### Returns

- `string`: The pruned Rust code string on success.
- `{error: string}`: An error table on failure.

#### Example

```lua
local rust_code = "fn greet(name: &str) {\n  println!(\"Hello, {}!\", name);\n}\n\nstruct Data;"
local pruned = aip.rust.prune_to_declarations(rust_code)
-- pruned might be: "fn greet(name: &str) { ... }\n\nstruct Data;" (exact spacing may vary)
```

#### Error

Returns an error (Lua table `{ error: string }`) if pruning fails.
