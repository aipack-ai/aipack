## aip.udiffx

Functions for applying multi-file changes (New, Patch, Rename, Delete) encoded in the `<FILE_CHANGES>` envelope format.

### Functions Summary

```lua
aip.udiffx.apply_file_changes(content: string, base_dir?: string, options?: {extrude?: "content"}): ApplyChangesStatus, remaining

aip.udiffx.load_files_context(include_globs: string | string[], options?: {base_dir?: string, absolute?: boolean}): string | nil

aip.udiffx.file_changes_instruction(): string
```

### aip.udiffx.apply_file_changes

Applies multi-file changes from a `<FILE_CHANGES>` envelope.

```lua
-- API Signatures
aip.udiffx.apply_file_changes(content: string, base_dir?: string, options?: {extrude?: "content"}): ApplyChangesStatus, remaining
```

Scans `content` for a `<FILE_CHANGES>` block and applies the directives within it.
Directives include `New`, `Patch` (supporting Unified Diff and simplified `@@` hunk headers), `Rename`, and `Delete`.
All paths in the envelope are resolved relative to `base_dir`.

#### Arguments

- `content: string`: The raw text containing the `<FILE_CHANGES>...</FILE_CHANGES>` envelope.
- `base_dir: string | nil` (optional): The directory where file changes will be applied, relative to the workspace root.
  Defaults to the workspace root if `nil` or `""`.
  **Note**: If `options` is provided, `base_dir` must be provided as well (can be `nil`).
- `options: table` (optional):
  - `extrude?: "content"` (optional): If set, returns the `content` string without the first `<FILE_CHANGES>` block as a second return value.

#### Returns

1. `status: ApplyChangesStatus`: An [ApplyChangesStatus](#applychangesstatus) table indicating the result of the operation.
2. `remaining: string | nil`: The content without the extracted block (only if `options.extrude == "content"`).

#### Example

```lua
local ai_response = [[
Here are the changes:
<FILE_CHANGES>
<FILE_NEW file_path="src/new_file.rs">
pub fn hello() { println!("Hello"); }
</FILE_NEW>
</FILE_CHANGES>
]]

local status, remaining = aip.udiffx.apply_file_changes(ai_response, ".", {extrude = "content"})
if status.success then
    print("Changes applied successfully!")
end
```

#### Error

Returns an error (Lua table `{ error: string }`) if:
- The `<FILE_CHANGES>` block cannot be parsed.
- An I/O error occurs during the application process that prevents finishing the cycle.
- The `base_dir` cannot be resolved.


### aip.udiffx.load_files_context

Loads file context blocks for matched files, using the `<FILE_CONTENT>` format.

```lua
-- API Signatures
aip.udiffx.load_files_context(
  include_globs: string | string[],
  options?: {
    base_dir?: string,
    absolute?: boolean
  }
): string | nil
```

Finds files matching `include_globs` and returns their content wrapped in `<FILE_CONTENT>` tags. This format is used to provide file context to LLMs.

#### Arguments

- `include_globs: string | string[]`: A single glob pattern string or a Lua list of glob pattern strings.
- `options?: table` (optional):
  - `base_dir?: string`: The directory relative to which the `include_globs` are applied. Defaults to the workspace root.
  - `absolute?: boolean`: If `true`, the paths in the `<FILE_CONTENT>` tags will be absolute.

#### Returns

- `string | nil`: A string containing all the file contents wrapped in tags, or `nil` when no files match the globs.

#### Example

```lua
local context = aip.udiffx.load_files_context("src/**/*.rs")
if context then
    print(context)
end
```

### aip.udiffx.file_changes_instruction

Returns the instruction text describing the `<FILE_CHANGES>` format.

```lua
-- API Signatures
aip.udiffx.file_changes_instruction(): string
```

#### Returns

- `string`: The prompt instruction text.
