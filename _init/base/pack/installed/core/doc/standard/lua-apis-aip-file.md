## aip.file

File manipulation functions for loading, saving, listing, and managing files and their content, including specialized functions for JSON and Markdown.

### Functions Summary

```lua
aip.file.load(rel_path: string, options?: {base_dir: string}): FileRecord

aip.file.save(rel_path: string, content: string, options?: SaveOptions): FileInfo

aip.file.copy(src_path: string, dest_path: string, options?: {overwrite?: boolean}): FileInfo

aip.file.move(src_path: string, dest_path: string, options?: {overwrite?: boolean}): FileInfo

aip.file.append(rel_path: string, content: string)

aip.file.delete(path: string): boolean

aip.file.ensure_exists(path: string, content?: string, options?: {content_when_empty?: boolean}): FileInfo

aip.file.exists(path: string): boolean

aip.file.list(include_globs: string | string[], options?: {base_dir?: string, absolute?: boolean, with_meta?: boolean}): FileInfo[]

aip.file.list_load(include_globs: string | string[], options?: {base_dir?: string, absolute?: boolean}): FileRecord[]

aip.file.first(include_globs: string | string[], options?: {base_dir?: string, absolute?: boolean}): FileInfo | nil

aip.file.info(path: string): FileInfo | nil

aip.file.load_json(path: string | nil): table | value | nil

aip.file.load_toml(path: string): table | value

aip.file.load_yaml(path: string): list

aip.file.load_ndjson(path: string | nil): object[] | nil

aip.file.append_json_line(path: string, data: value): FileInfo

aip.file.append_json_lines(path: string, data: list): FileInfo

aip.file.save_changes(path: string, changes: string): FileInfo

aip.file.load_md_sections(path: string, headings?: string | string[]): MdSection[]

aip.file.load_md_split_first(path: string): {before: string, first: MdSection, after: string}

aip.file.load_csv_headers(path: string): string[]

aip.file.load_csv(path: string, options?: CsvOptions): {headers: string[], rows: string[][]}

aip.file.save_as_csv(path: string, data: any[][] | {headers?: string[], rows?: any[][]}, options?: CsvOptions): FileInfo
aip.file.save_records_as_csv(path: string, records: table[], header_keys: string[], options?: CsvOptions): FileInfo
aip.file.append_csv_rows(path: string, value_lists: any[][], options?: CsvOptions): FileInfo
aip.file.append_csv_row(path: string, values: any[], options?: CsvOptions): FileInfo

aip.file.save_html_to_md(html_path: string, dest?: string | table): FileInfo

aip.file.save_html_to_slim(html_path: string, dest?: string | table): FileInfo
aip.file.load_html_as_slim(html_path: string): string
aip.file.load_html_as_md(html_path: string, options?: table): string

aip.file.save_docx_to_md(docx_path: string, dest?: string | table): FileInfo

aip.file.load_docx_as_md(docx_path: string): string

aip.file.line_spans(path: string): [start: number, end: number][]

aip.file.csv_row_spans(path: string): [start: number, end: number][]

aip.file.read_span(path: string, start: number, end: number): string

aip.file.hash_sha256(path: string): string
aip.file.hash_sha256_b64(path: string): string
aip.file.hash_sha256_b64u(path: string): string
aip.file.hash_sha256_b58u(path: string): string

aip.file.hash_sha512(path: string): string
aip.file.hash_sha512_b64(path: string): string
aip.file.hash_sha512_b64u(path: string): string
aip.file.hash_sha512_b58u(path: string): string

aip.file.hash_blake3(path: string): string
aip.file.hash_blake3_b64(path: string): string
aip.file.hash_blake3_b64u(path: string): string
aip.file.hash_blake3_b58u(path: string): string
```


> Note: All relative paths are relative to the workspace directory (parent of `.aipack/`). Unless a `base_dir` option is specified. Pack references (e.g., `ns@pack/`) can be used in paths and `base_dir`.

### aip.file.load

Load a [FileRecord](#filerecord) object with its content.

```lua
-- API Signature
aip.file.load(rel_path: string, options?: {base_dir: string}): FileRecord
```

Loads the file specified by `rel_path` and returns a [FileRecord](#filerecord) object containing the file's metadata and its content.

#### Arguments

- `rel_path: string`: The path to the file, relative to the `base_dir` or workspace root.
- `options?: table`: An optional table containing:
  - `base_dir: string` (optional): The base directory from which `rel_path` is resolved. Defaults to the workspace root. Pack references (e.g., `ns@pack/`) can be used.

#### Returns

- `[FileRecord](#filerecord)`: A [FileRecord](#filerecord) table representing the file.

#### Example

```lua
local readme = aip.file.load("doc/README.md")
print(readme.path)    -- Output: "doc/README.md" (relative path used)
print(#readme.content) -- Output: <length of content>

local agent_file = aip.file.load("agent.aip", { base_dir = "ns@pack/" })
print(agent_file.path) -- Output: "agent.aip" (relative to the resolved base_dir)
```

#### Error

Returns an error (Lua table `{ error: string }`) if the file cannot be found, read, or metadata retrieved, or if `base_dir` is invalid.

### aip.file.save

Save string content to a file at the specified path.

```lua
-- API Signature
aip.file.save(rel_path: string, content: string, options?: SaveOptions): FileInfo
```

Writes the `content` string to the file specified by `rel_path`. Overwrites existing files. Creates directories as needed. Restricts saving outside the workspace or shared base directory for security.

#### Arguments

- `rel_path: string`: The path relative to the workspace root.
- `content: string`: The string content to write.
- `options?: [SaveOptions](#saveoptions)` (optional): Options ([SaveOptions](#saveoptions)) to pre-process content before saving:
  - `trim_start?: boolean`: If true, remove leading whitespace.
  - `trim_end?: boolean`: If true, remove trailing whitespace.
  - `single_trailing_newline?: boolean`: If true, ensure exactly one trailing newline (`\n`).

#### Returns

- [FileInfo](#fileinfo): Metadata ([FileInfo](#fileinfo)) about the saved file.

#### Example

```lua
-- Save documentation to a file in the 'docs' directory
aip.file.save("docs/new_feature.md", "# New Feature\n\nDetails.")

-- Overwrite an existing file, applying trimming and normalization
aip.file.save("config.txt", "  new_setting=true  \n\n", {
  trim_start = true,
  trim_end = true,
  single_trailing_newline = true
})
```

#### Error

Returns an error (Lua table `{ error: string }`) on write failure, permission issues, path restrictions, or if no workspace context.

### aip.file.copy

Copies a file from `src_path` to `dest_path`, returning a [FileInfo](#fileinfo) object for the destination. (since 0.8.15)

```lua
-- API Signature
aip.file.copy(src_path: string, dest_path: string, options?: {overwrite?: boolean}): FileInfo
```

Performs a binary copy of the file at `src_path` to `dest_path`.
Both paths are resolved relative to the workspace root and support pack references (`ns@pack/...`).
Parent directories for the destination are created automatically if they don't exist.

#### Arguments

- `src_path: string` - The source file path.
- `dest_path: string` - The destination file path.
- `options?: table` (optional) - Options:
  - `overwrite?: boolean`: If `false` (default), the operation fails if the destination exists.

#### Returns

- `[FileInfo](#fileinfo)`: Metadata ([FileInfo](#fileinfo)) about the copied destination file.

#### Error

Returns an error if the source file doesn't exist, if the destination is outside the workspace, or if the destination exists and `overwrite` is false.

### aip.file.move

Moves (renames) a file from `src_path` to `dest_path`, returning a [FileInfo](#fileinfo) object for the destination. (since 0.8.15)

```lua
-- API Signature
aip.file.move(src_path: string, dest_path: string, options?: {overwrite?: boolean}): FileInfo
```

Renames the file at `src_path` to `dest_path`.
Both paths are resolved relative to the workspace root and support pack references (`ns@pack/...`).
Parent directories for the destination are created automatically if they don't exist.

#### Arguments

- `src_path: string` - The source file path.
- `dest_path: string` - The destination file path.
- `options?: table` (optional) - Options:
  - `overwrite?: boolean`: If `false` (default), the operation fails if the destination exists.

#### Returns

- `[FileInfo](#fileinfo)`: Metadata ([FileInfo](#fileinfo)) about the moved destination file.

#### Error

Returns an error if the source file doesn't exist, if the source or destination is outside the workspace (e.g., `.aipack-base`), or if the destination exists and `overwrite` is false.

### aip.file.append

Append string content to a file at the specified path.

```lua
-- API Signature
aip.file.append(rel_path: string, content: string): FileInfo
```

Appends `content` to the end of the file at `rel_path`. Creates the file and directories if they don't exist.

#### Arguments

- `rel_path: string`: The path relative to the workspace root.
- `content: string`: The string content to append.

#### Returns

- `[FileInfo](#fileinfo)`: Metadata ([FileInfo](#fileinfo)) about the file.

#### Example

```lua
aip.file.append("logs/app.log", "INFO: User logged in.\n")
```

#### Error

Returns an error (Lua table `{ error: string }`) on write failure, permission issues, or I/O errors.

### aip.file.delete

Deletes a file at the specified path.

```lua
-- API Signature
aip.file.delete(path: string): boolean
```

Attempts to delete the file specified by `path`. The path is resolved relative to the workspace root.

Security:
- Deleting files is only allowed within the current workspace directory.

- Deleting files under the shared base directory (`~/.aipack-base/`) is not allowed.

#### Arguments

- `path: string`  
  The path to the file to delete, relative to the workspace root.

#### Returns

- `boolean`  
  `true` if a file was deleted, `false` if the file did not exist.

#### Example

```lua
local removed = aip.file.delete("logs/app.log")
if removed then
  print("Removed logs/app.log")
else
  print("No file to remove")
end
```

#### Error

Returns an error (Lua table `{ error: string }`) if:
- The path attempts to delete outside the allowed workspace directory.

- The target is in the `.aipack-base` folder (always forbidden).

- The file cannot be deleted due to permissions or other I/O errors.

- The operation requires a workspace context, but none is found.

### aip.file.ensure_exists

Ensure a file exists at the given path, optionally creating it or adding content if empty.

```lua
-- API Signature
aip.file.ensure_exists(path: string, content?: string, options?: {content_when_empty?: boolean}): FileInfo
```

Checks if the file exists. If not, creates it with `content`. If it exists and `options.content_when_empty` is true and the file is empty (or whitespace only), writes `content`. Intended for files, not directories.

#### Arguments

- `path: string`: The path relative to the workspace root.
- `content?: string` (optional): Content to write. Defaults to empty string.
- `options?: table` (optional):
  - `content_when_empty?: boolean` (optional): If true, the `content` will be written to the file if the file is empty (or only contains whitespace). Defaults to `false`.

#### Returns

- `[FileInfo](#fileinfo)`: Metadata ([FileInfo](#fileinfo)) about the file.

#### Example

```lua
-- Create config if needed, with default content
local default_config = "-- Default Settings --\nenabled=true"
local meta = aip.file.ensure_exists("config/settings.lua", default_config)
print("Ensured file:", meta.path)

-- Ensure log file exists, don't overwrite
aip.file.ensure_exists("logs/activity.log")

-- Add placeholder only if file is currently empty
aip.file.ensure_exists("src/module.lua", "-- TODO", {content_when_empty = true})
```

#### Error

Returns an error (Lua table `{ error: string }`) on creation/write failure, permission issues, or metadata retrieval failure.

### aip.file.exists

Checks if the specified path exists (file or directory).

```lua
-- API Signature
aip.file.exists(path: string): boolean
```

Checks if the file or directory specified by `path` exists. The path is resolved relative to the workspace root.
Supports relative paths, absolute paths, and pack references (`ns@pack/...`).

#### Arguments

- `path: string`: The path string to check for existence. Can be relative, absolute, or a pack reference.

#### Returns

- `boolean`: Returns `true` if a file or directory exists at the specified path, `false` otherwise.

#### Example

```lua
if aip.file.exists("README.md") then
  print("README.md exists.")
end

if aip.file.exists("ns@pack/main.aip") then
  print("Pack main agent exists.")
end
```

#### Error


Returns an error (Lua table `{ error: string }`) if the path string cannot be resolved (e.g., invalid pack reference, invalid path format).

```ts
{
  error: string // Error message
}
```

### aip.file.list

List file metadata ([FileInfo](#fileinfo)) matching glob patterns.

```lua
-- API Signature
aip.file.list(
  include_globs: string | string[],
  options?: {
    base_dir?: string,
    absolute?: boolean,
    with_meta?: boolean
  }
): FileInfo[]
```

Finds files matching `include_globs` within `base_dir` (or workspace) and returns a list of [FileInfo](#fileinfo) objects (metadata only, no content).

#### Arguments

- `include_globs: string | string[]`: Glob pattern(s). Pack references supported.
  Note: Common build/dependency folders (e.g., `target/`, `node_modules/`, `.build/`, `__pycache__/`) are excluded by default unless explicitly matched by `include_globs`.
- `options?: table` (optional):
  - `base_dir?: string` (optional): Base directory for globs. Defaults to workspace. Pack refs supported.
  - `absolute?: boolean` (optional): If `true`, the `path` in the returned [FileInfo](#fileinfo) objects will be absolute.
    If `false` (default), the `path` will be relative to the `base_dir`. If a path resolves outside the `base_dir`
    (e.g., using `../` in globs), it will be returned as an absolute path even if `absolute` is false.
  - `with_meta?: boolean` (optional): If `false`, the function will skip fetching detailed metadata
    (`ctime`, `mtime`, `size`) for each file, potentially improving performance
    if only the path information is needed. Defaults to `true`.
  - `ctime` is creation time, `mtime` is last modification time (from the file system), both in epoch micro

#### Returns

- `[FileInfo](#fileinfo)[]`: A Lua list of [FileInfo](#fileinfo) tables. Empty if no matches.

#### Example

```lua
-- List all Markdown files in the 'docs' directory (relative paths)
local doc_files = aip.file.list("*.md", { base_dir = "docs" })
for _, file in ipairs(doc_files) do
  print(file.path) -- e.g., "guide.md", "api.md"
end

-- List all '.aip' files in a specific pack (absolute paths, no detailed meta)
local agent_files = aip.file.list("**/*.aip", {
  base_dir = "ns@pack/",
  absolute = true,
  with_meta = false
})
for _, file in ipairs(agent_files) do
  print(file.path) -- e.g., "/path/to/workspace/.aipack/ns/pack/agent1.aip"
end

-- List text and config files from the workspace root
local config_files = aip.file.list({"*.txt", "*.config"})
for _, file in ipairs(config_files) do
  print(file.path, file.size) -- e.g., "notes.txt", 1024
end
```

#### Error

Returns an error (Lua table `{ error: string }`) on invalid arguments, resolution failure, glob matching error, or metadata retrieval error (if `with_meta=true`).

### aip.file.list_load

List and load files ([FileRecord](#filerecord)) matching glob patterns.

```lua
-- API Signature
aip.file.list_load(
  include_globs: string | string[],
  options?: {
    base_dir?: string,
    absolute?: boolean
  }
): FileRecord[]
```

Finds files matching `include_globs` patterns within the specified `base_dir` (or workspace root),
loads the content of each matching file, and returns a list of [FileRecord](#filerecord) objects.
Each [FileRecord](#filerecord) contains both metadata and the file content.

#### Arguments

- `include_globs: string | string[]` - A single glob pattern string or a Lua list (table) of glob pattern strings.
  Globs can include standard wildcards (`*`, `?`, `**`, `[]`). Pack references (e.g., `ns@pack/**/*.md`) are supported.
  Note: Common build/dependency folders (e.g., `target/`, `node_modules/`, `.build/`, `__pycache__/`) are excluded by default unless explicitly matched by `include_globs`.
- `options?: table` (optional) - A table containing options:
  - `base_dir?: string` (optional): The directory relative to which the `include_globs` are applied.
    Defaults to the workspace root. Pack references (e.g., `ns@pack/`) are supported.
  - `absolute?: boolean` (optional): If `true`, the paths used internally and potentially the `path` in the returned [FileRecord](#filerecord)
    objects will be absolute. If `false` (default), paths will generally be relative to the `base_dir`.
    Note: The exact path stored in [FileRecord](#filerecord).path depends on internal resolution logic, especially if paths resolve outside `base_dir`.

#### Returns

- `[FileRecord](#filerecord)[]`: A Lua list of [FileRecord](#filerecord) tables. Empty if no files match the globs.

#### Example

```lua
-- Load all Markdown files in the 'docs' directory
local doc_files = aip.file.list_load("*.md", { base_dir = "docs" })
for _, file in ipairs(doc_files) do
  print("--- File:", file.path, "---")
  print(file.content)
end

-- Load all '.aip' files in a specific pack
local agent_files = aip.file.list_load("**/*.aip", { base_dir = "ns@pack/" })
for _, file in ipairs(agent_files) do
  print("Agent Name:", file.stem)
end
```

#### Error

Returns an error (Lua table `{ error: string }`) on invalid arguments, resolution failure, glob matching error, or file read/metadata error.

### aip.file.first

Find the first file matching glob patterns and return its metadata ([FileInfo](#fileinfo)).

```lua
-- API Signature
aip.file.first(
  include_globs: string | string[],
  options?: {
    base_dir?: string,
    absolute?: boolean
  }
): FileInfo | nil
```

Searches for files matching the `include_globs` patterns within the specified `base_dir` (or workspace root).
It stops searching as soon as the first matching file is found and returns its [FileInfo](#fileinfo) object (metadata only, no content).
If no matching file is found, it returns `nil`.

#### Arguments

- `include_globs: string | string[]` - A single glob pattern string or a Lua list (table) of glob pattern strings.
  Globs can include standard wildcards (`*`, `?`, `**`, `[]`). Pack references (e.g., `ns@pack/**/*.md`) are supported.
  Note: Common build/dependency folders (e.g., `target/`, `node_modules/`, `.build/`, `__pycache__/`) are excluded by default unless explicitly matched by `include_globs`.
- `options?: table` (optional) - A table containing options:
  - `base_dir?: string` (optional): The directory relative to which the `include_globs` are applied.
    Defaults to the workspace root. Pack references (e.g., `ns@pack/`) are supported.
  - `absolute?: boolean` (optional): If `true`, the `path` in the returned [FileInfo](#fileinfo) object (if found) will be absolute.
    If `false` (default), the `path` will be relative to the `base_dir`. Similar to `aip.file.list`, paths outside `base_dir` become absolute.

#### Returns

- `[FileInfo](#fileinfo) | nil`: If a matching file is found, returns a [FileInfo](#fileinfo) table. If no matching file is found, returns `nil`.

#### Example

```lua
-- Find the first '.aip' file in a pack
local agent_meta = aip.file.first("*.aip", { base_dir = "ns@pack/" })
if agent_meta then
  print("Found agent:", agent_meta.path)
  -- To load its content:
  -- local agent_file = aip.file.load(agent_meta.path, { base_dir = "ns@pack/" })
  -- print(agent_file.content)
else
  print("No agent file found in pack.")
end

-- Find any config file in the root
local config_meta = aip.file.first({"*.toml", "*.yaml", "*.json"}, { base_dir = "." })
if config_meta then
  print("Config file:", config_meta.name)
end
```

#### Error

Returns an error (Lua table `{ error: string }`) on invalid arguments, resolution failure, error during search *before* first match, or metadata retrieval error for the first match.

### aip.file.info

Retrieves file metadata ([`FileInfo`](#fileinfo)) for the specified path.

```lua
-- API Signature
aip.file.info(path: string): FileInfo | nil
```

If the given `path` exists, this function returns a [`FileInfo`](#fileinfo) object
containing the file metadata (no content).  
If the path cannot be resolved or the file does not exist, it returns `nil`.

#### Arguments

- `path: string` – The file or directory path. Can be relative, absolute,
  or use pack references (`ns@pack/...`, `ns@pack$workspace/...`, etc.).

#### Returns

- `[FileInfo](#fileinfo) | nil`: Metadata for the file, or `nil` when not found.

#### Example

```lua
local file_info = aip.file.info("README.md")
if file_info then
  print("Size:", file_info.size)
end
```

#### Error

Returns an error only if the path cannot be resolved (invalid pack
reference, invalid format, …). If the path resolves successfully but the
file does not exist, the function simply returns `nil`.

### aip.file.stats

Calculates aggregate statistics for a set of files matching glob patterns.

```lua
-- API Signature
aip.file.stats(
  include_globs: string | string[] | nil,
  options?: {
    base_dir?: string,
    absolute?: boolean
  }
): FileStats | nil
```

Finds files matching the `include_globs` patterns within the specified `base_dir` (or workspace root)
and returns aggregate statistics about these files in a `FileStats` object.
If `include_globs` is `nil` or no files match the patterns, returns `nil`.

#### Arguments

- `include_globs: string | string[] | nil` - A single glob pattern string, a Lua list (table) of glob pattern strings, or `nil`.
  If `nil`, the function returns `nil`.
  Globs can include standard wildcards (`*`, `?`, `**`, `[]`). Pack references (e.g., `ns@pack/**/*.md`) are supported.
  Note: Common build/dependency folders (e.g., `target/`, `node_modules/`, `.build/`, `__pycache__/`) are excluded by default unless explicitly matched by `include_globs`.
- `options?: table` (optional) - A table containing options:
  - `base_dir?: string` (optional): The directory relative to which the `include_globs` are applied.
    Defaults to the workspace root. Pack references (e.g., `ns@pack/`) are supported.
  - `absolute?: boolean` (optional): Affects how files are resolved internally, but the statistics remain the same regardless.

#### Returns

- `[FileStats](#filestats)`: A [FileStats](#filestats) object containing aggregate statistics about the matching files.
- `nil` if `include_globs` is `nil`

If no files if ound a FileStats will all 0 will be returned.

#### Example

```lua
-- Get statistics for all Markdown files in the 'docs' directory
local stats = aip.file.stats("*.md", { base_dir = "docs" })
if stats then
  print("Number of files:", stats.number_of_files)
  print("Total size:", stats.total_size)
  print("First created:", stats.ctime_first)
  print("Last modified:", stats.mtime_last)
end

-- Get statistics for all '.aip' files in a specific pack
local agent_stats = aip.file.stats("**/*.aip", { base_dir = "ns@pack/" })
if agent_stats then
  print("Total agent files:", agent_stats.number_of_files)
end

-- Nil globs return nil
local nil_stats = aip.file.stats(nil)
print(nil_stats) -- Output: nil
```

#### Error

Returns an error if:
- `include_globs` is not a string, a list of strings, or `nil`.
- `base_dir` cannot be resolved (e.g., invalid pack reference).
- An error occurs during file system traversal or glob matching.

### aip.file.load_json

Load a file, parse its content as JSON, and return the corresponding Lua value.

```lua
-- API Signature
aip.file.load_json(path: string | nil): table | value | nil
```

Loads the file at `path` (relative to workspace), parses it as JSON, and converts it to a Lua value. Returns `nil` if `path` is `nil`.

#### Arguments

- `path: string | nil`: Path to the JSON file, relative to workspace root. If `nil`, returns `nil`.

#### Returns

- `table | value | nil`: Lua value representing the parsed JSON, or `nil`.

#### Example

```lua
-- Assuming 'config.json' contains {"port": 8080, "enabled": true}
local config = aip.file.load_json("config.json")
print(config.port) -- Output: 8080
```

#### Error

Returns an error (Lua table `{ error: string }`) if file not found/read, content is invalid JSON, or conversion fails.

### aip.file.load_toml

Load a file, parse its content as TOML, and return the corresponding Lua value.

```lua
-- API Signature
aip.file.load_toml(path: string): table | value
```

#### Arguments

- `path: string`: Path to the TOML file, relative to the workspace root.

#### Returns

- `table | value`: A Lua value representing the parsed TOML content.

#### Example

```lua
local config = aip.file.load_toml("Config.toml")
print(config.title)
print(config.owner.name)
```

#### Error

Returns an error (Lua table `{ error: string }`) if the file cannot be read, the TOML content is invalid, or the conversion to a Lua value fails.

### aip.file.load_yaml

Load a file, parse its content as YAML, and return the corresponding Lua list of documents.

```lua
-- API Signature
aip.file.load_yaml(path: string): list
```

Loads the content of the file specified by `path`, parses it as YAML (supporting multiple documents
separated by `---`), and converts the result into a Lua list of tables.
The path is resolved relative to the workspace root.

#### Arguments

- `path: string`: The path to the YAML file, relative to the workspace root.

#### Returns

- `list`: A Lua list (table indexed from 1) where each element corresponds to a parsed YAML document.

#### Example

```lua
-- Assuming 'data.yaml' contains:
-- name: Doc1
-- ---
-- name: Doc2

local docs = aip.file.load_yaml("data.yaml")
print(docs[1].name) -- Output: Doc1
print(docs[2].name) -- Output: Doc2
```

#### Error

Returns an error (Lua table `{ error: string }`) if:
- The file cannot be found or read.
- The file content is not valid YAML.
- The YAML value cannot be converted to a Lua value.

### aip.file.load_ndjson

Load a file containing newline-delimited JSON (NDJSON), parse each line, and return a Lua list (table) of the results.

```lua
-- API Signature
aip.file.load_ndjson(path: string | nil): object[] | nil
```

Loads the file at `path` (relative to workspace), parses each non-empty line as JSON, and returns a Lua list of the parsed values. Empty lines are skipped. Returns `nil` if `path` is `nil`.

#### Arguments

- `path: string | nil`: Path to the NDJSON file, relative to workspace root. If `nil`, returns `nil`.

#### Returns

- `object[] | nil`: Lua list containing parsed values from each line, or `nil`.

#### Example

```lua
-- Assuming 'logs.ndjson' contains:
-- {"level": "info", "msg": "Started"}
-- {"level": "warn", "msg": "Low space"}
local logs = aip.file.load_ndjson("logs.ndjson")
print(#logs) -- Output: 2
print(logs[1].msg) -- Output: Started
```

#### Error

Returns an error (Lua table `{ error: string }`) if file not found/read, any line has invalid JSON, or conversion fails.

### aip.file.append_json_line

Convert a Lua value to a JSON string and append it as a new line to a file.

```lua
-- API Signature
aip.file.append_json_line(path: string, data: value)
```

Converts `data` to JSON and appends it, followed by a newline (`\n`), to the file at `path` (relative to workspace). Creates file/directories if needed.

#### Arguments

- `path: string`: Path to the target file, relative to workspace root.
- `data: value`: Lua data (table, string, number, boolean, nil) to append.

#### Returns

- `FileInfo`: Metadata ([FileInfo](#fileinfo)) about the file.

#### Example

```lua
aip.file.append_json_line("output.ndjson", {user = "test", score = 100})
-- Appends '{"score":100,"user":"test"}\n' to output.ndjson
```

#### Error

Returns an error (Lua table `{ error: string }`) on conversion/serialization failure, directory creation failure, or file write/permission error.

### aip.file.append_json_lines

Convert a Lua list (table) of values to JSON strings and append them as new lines to a file.

```lua
-- API Signature
aip.file.append_json_lines(path: string, data: list): FileInfo
```

Iterates through the `data` list, converts each element to JSON, and appends it followed by a newline (`\n`) to the file at `path` (relative to workspace). Creates file/directories if needed. Uses buffering.

#### Arguments

- `path: string`: Path to the target file, relative to workspace root.
- `data: list`: Lua list (table with sequential integer keys from 1) of values to append.

#### Returns

- `FileInfo`: Metadata ([FileInfo](#fileinfo)) about the file.

#### Example

```lua
local users = { {user = "alice"}, {user = "bob"} }
aip.file.append_json_lines("users.ndjson", users)
-- Appends '{"user":"alice"}\n{"user":"bob"}\n'
```

#### Error

Returns an error (Lua table `{ error: string }`) if `data` is not a list, conversion/serialization fails for any element, directory creation fails, or file write/permission error.

### aip.file.load_md_sections

Load markdown sections from a file, optionally filtering by specific heading names.

```lua
-- API Signature
aip.file.load_md_sections(
  path: string,
  headings?: string | string[]
): MdSection[]
```

Reads the markdown file at `path` (relative to workspace) and splits it into sections based on headings (`#`). Returns a list of [MdSection](#mdsection) objects. Optionally filters by exact heading `name` (case-sensitive, excluding `#`).

#### Arguments

- `path: string`: Path to the markdown file, relative to workspace root.
- `headings?: string | string[]` (optional): Heading name(s) to filter by.

#### Returns

- `MdSection[]`: A Lua list of [MdSection](#mdsection) tables. Includes content before the first heading if no filter applied. Empty if file empty or no matching sections.

#### Example

```lua
-- Load all sections
local all_sections = aip.file.load_md_sections("doc/readme.md")

-- Load only the "Summary" section
local summary_section = aip.file.load_md_sections("doc/readme.md", "Summary")

-- Load "Summary" and "Conclusion" sections
local sections = aip.file.load_md_sections("doc/readme.md", {"Summary", "Conclusion"})
```

#### Error

Returns an error (Lua table `{ error: string }`) if file not found/read, `headings` invalid, or parsing/conversion error.

### aip.file.load_md_split_first

Splits a markdown file into three parts based on the *first* heading encountered.

```lua
-- API Signature
aip.file.load_md_split_first(path: string): {before: string, first: MdSection, after: string}
```

Reads the file at `path` (relative to workspace) and divides it into: content before the first heading (`before`), the first heading section (`first`), and content from the second heading onwards (`after`).

#### Arguments

- `path: string`: Path to the markdown file, relative to workspace root.

#### Returns

- `table`: A table containing the three parts:
  ```ts
  {
    before: string,       // Content before first heading.
    first: MdSection,     // The first [MdSection](#mdsection) (default if no headings).
    after: string         // Content from second heading onwards.
  }
  ```

#### Example

```lua
local split = aip.file.load_md_split_first("doc/structure.md")
print("--- BEFORE ---")
print(split.before)
print("--- FIRST Heading Name ---")
print(split.first.heading.name)
print("--- AFTER ---")
print(split.after)
```

#### Error

Returns an error (Lua table `{ error: string }`) if file not found/read, or parsing/conversion error.

### aip.file.load_csv_headers

Load a CSV file and return its header row as a list of strings.

```lua
-- API Signature
aip.file.load_csv_headers(path: string): string[]
```

Loads the CSV file at `path` (relative to workspace), parses its header row, and returns the headers as a Lua list of strings.

#### Arguments

- `path: string`: Path to the CSV file, relative to workspace root.

#### Returns

- `string[]`: A Lua list of strings containing the header names.

#### Example

```lua
local headers = aip.file.load_csv_headers("data/example.csv")
for i, header in ipairs(headers) do
  print(i, header)
end
```

#### Error

Returns an error (Lua table `{ error: string }`) if:
- The path cannot be resolved
- The file cannot be found or read
- CSV parsing fails

### aip.file.load_csv

Load a CSV file and return its headers and all rows as string arrays.

```lua
-- API Signature
aip.file.load_csv(path: string, options?: CsvOptions): {headers: string[], rows: string[][]}
```

Loads the CSV file at `path` (relative to workspace), parses it according to the provided options, and returns both headers and rows.

#### Arguments

- `path: string`: Path to the CSV file, relative to workspace root.
- `options?: CsvOptions` (optional): CSV parse options. Only `has_header` is honored by this API (defaults to `true`), which controls whether the first row is treated as headers and excluded from `rows`.

#### Returns

- `[CsvContent](#csvcontent)`: Matches the [CsvContent](#csvcontent) structure (same as `aip.file.load_csv`), including the `_type = "CsvContent"` marker alongside the `headers` and `rows` fields.

#### Example

```lua
local result = aip.file.load_csv("data/example.csv") -- defaults to has_header = true
print("Headers:", table.concat(result.headers, ", "))
for _, row in ipairs(result.rows) do
  print(table.concat(row, " | "))
end

-- Load CSV without headers
local result_no_headers = aip.file.load_csv("data/data-only.csv", {has_header = false})
```

#### Error

Returns an error (Lua table `{ error: string }`) if:
- The path cannot be resolved
- The file cannot be found or read
- CSV parsing fails

### aip.file.save_as_csv

Save data as CSV file (overwrite).

```lua
-- API Signature
aip.file.save_as_csv(
  path: string,
  data: any[][] | { headers?: string[], rows?: any[][] },
  options?: CsvOptions
): FileInfo
```

Writes `data` to the CSV file at `path`.

#### Arguments

- `path: string`: Path to the CSV file, relative to the workspace root.
- `data: any[][] | { headers?: string[], rows?: any[][] }`: The data to save. Can be:
    - A matrix (`any[][]`). If `options.has_header` is true, the first row is treated as headers.
    - A structured table `{ headers?: string[], rows?: any[][] }`. Supports defining headers only, rows only, or both.
- `options?: [CsvOptions](#csvoptions)` (optional): CSV write options. `header_labels` are used to map internal keys to output labels. `skip_header_row` can suppress header emission.

#### Returns

- `[FileInfo](#fileinfo)`: Metadata ([FileInfo](#fileinfo)) about the created CSV file.

#### Example

```lua
local data = {
    {"name", "age"},
    {"Alice", 30}
}
aip.file.save_as_csv("output/users.csv", data, { has_header = true })
```

#### Error

Returns an error (Lua table `{ error: string }`) on write failure, path restriction, or serialization issues.

### aip.file.save_records_as_csv

Save a list of record objects (tables with keys) to CSV (overwrite).

```lua
-- API Signature
aip.file.save_records_as_csv(
  path: string,
  records: table[],
  header_keys: string[],
  options?: CsvOptions
): FileInfo
```

Writes `records` to the CSV file at `path`, aligning values based on `header_keys`.

#### Arguments

- `path: string`: Path to the CSV file, relative to the workspace root.
- `records: table[]`: A list of Lua tables/objects (records).
- `header_keys: string[]`: Defines the column order and specifies which keys to extract from `records`.
- `options?: [CsvOptions](#csvoptions)` (optional): CSV write options. `header_labels` can map internal `header_keys` to output column names.

#### Returns

- `[FileInfo](#fileinfo)`: Metadata ([FileInfo](#fileinfo)) about the created CSV file.

#### Example

```lua
local users = {
    { id = 1, full_name = "Alice" },
    { id = 2, full_name = "Bob" }
}
local keys = {"id", "full_name"}
aip.file.save_records_as_csv("output/user_data.csv", users, keys)

-- Example with header labels mapping internal keys to external labels
local labeled_keys = {"id", "full_name"}
local opts = {
    header_labels = {
        id = "User ID",
        full_name = "Name"
    }
}
aip.file.save_records_as_csv("output/labeled_users.csv", users, labeled_keys, opts)
```

#### Error

Returns an error (Lua table `{ error: string }`) on write failure, missing keys in records, or serialization issues.

### aip.file.append_csv_rows

Appends multiple data rows (matrix `any[][]`) to a CSV file, creating the file if it doesn't exist.

```lua
-- API Signature
aip.file.append_csv_rows(
  path: string,
  value_lists: any[][],
  options?: CsvOptions
): FileInfo
```

This function focuses purely on appending data rows. Options related to automatic header writing (`has_header`, `header_labels`) are ignored.

#### Arguments

- `path: string`: Path to the CSV file, relative to the workspace root.
- `value_lists: any[][]`: List of lists of values (`any[][]`) to append. Inner values are converted to CSV strings (tables -> JSON).
- `options?: [CsvOptions](#csvoptions)` (optional): CSV write options (e.g., `delimiter`, `quote`, `escape`).

#### Returns

- `[FileInfo](#fileinfo)`: Metadata ([FileInfo](#fileinfo)) about the file.

#### Example

```lua
local rows_to_append = {
    {"2025-01-01", "Start"},
    {"2025-01-02", "Stop"}
}
aip.file.append_csv_rows("logs/activity.csv", rows_to_append)
```

#### Error

Returns an error (Lua table `{ error: string }`) on write failure or serialization issues.

### aip.file.append_csv_row

Appends a single data row (list of values) to a CSV file, creating the file if it doesn't exist.

```lua
-- API Signature
aip.file.append_csv_row(
  path: string,
  values: any[],
  options?: CsvOptions
): FileInfo
```

This function focuses purely on appending a single data row. Headers should be managed separately via `aip.file.append_csv_headers`.

#### Arguments

- `path: string`: Path to the CSV file, relative to the workspace root.
- `values: any[]`: A single list of values to append as a row. Inner values are converted to CSV strings (tables -> JSON).

#### Returns

- `[FileInfo](#fileinfo)`: Metadata ([FileInfo](#fileinfo)) about the file.

#### Example

```lua
aip.file.append_csv_row("logs/simple.csv", {"Data", 123, true})
```

#### Error

Returns an error (Lua table `{ error: string }`) on write failure or serialization issues.

### aip.file.save_html_to_md

Loads an HTML file, converts its content to Markdown, and saves it.

```lua
-- API Signature
aip.file.save_html_to_md(
  html_path: string,
  dest?: string | table
): FileInfo
```

Loads the HTML file at `html_path` (relative to workspace), converts its content to Markdown, and saves the result. The destination can be specified as a string path or a table of options ([DestOptions](#destoptions)).

#### Arguments

- `html_path: string`
  Path to the source HTML file, relative to the workspace root.

- `dest?: string | table (optional)`
  Destination path or options table:

  - `string`
    Path to save the `.md` file (relative or absolute).
  - `table` ([DestOptions](#destoptions)):
      - `base_dir?: string`: Base directory for resolving the destination.
      - `file_name?: string`: Custom file name for the Markdown output.
      - `suffix?: string`: Suffix appended to the source file stem before `.md`.

#### Returns

- `[FileInfo](#fileinfo)`
  Metadata ([FileInfo](#fileinfo)) about the created Markdown file.

#### Example

```lua
-- Default (replaces .html with .md):
aip.file.save_html_to_md("docs/page.html")
-- Result: docs/page.md

-- Using a custom string path:
aip.file.save_html_to_md("docs/page.html", "out/custom.md")

-- Using options table:
aip.file.save_html_to_md("docs/page.html", {
  base_dir = "output",
  suffix = "_v2",
})
-- Assuming source was 'docs/page.html', result might be 'output/page_v2.md'
```

#### Error


Returns an error (Lua table `{ error: string }`) if file I/O, parsing, conversion, or destination resolution fails.

### aip.file.save_html_to_slim

Loads an HTML file, "slims" its content (removes scripts, styles, comments, etc.), and saves the slimmed HTML.

```lua
-- API Signature
aip.file.save_html_to_slim(
  html_path: string,
  dest?: string | table
): FileInfo
```

Loads the HTML file at `html_path` (relative to workspace), removes non-content elements, and saves the cleaned HTML. The destination can be specified as a string path or a table of options ([DestOptions](#destoptions)).

#### Arguments

- `html_path: string`
  Path to the source HTML file, relative to the workspace root.

- `dest?: string | table (optional)`
  Destination path or options table ([DestOptions](#destoptions)) for the output `.html` file:

  - `nil`: Saves as `[original_name]-slim.html` in the same directory.
  - `string`: Path to save the slimmed `.html` file (relative or absolute).
  - `table` ([DestOptions](#destoptions)):
      - `base_dir?: string`: Base directory for resolving the destination. If provided without `file_name` or `suffix`, the output will be `[original_name].html` in this directory.
      - `file_name?: string`: Custom file name for the slimmed HTML output.
      - `suffix?: string`: Suffix appended to the source file stem (e.g., `_slimmed`).
      - `slim?: boolean`: If true (default), slims HTML before saving; if false, saves the original HTML content without slimming.

#### Returns

- `[FileInfo](#fileinfo)`
  Metadata ([FileInfo](#fileinfo)) about the created slimmed HTML file.

#### Example

```lua
-- Default (saves as original-slim.html):
aip.file.save_html_to_slim("web/page.html")
-- Result: web/page-slim.html

-- Using a custom string path:
aip.file.save_html_to_slim("web/page.html", "output/slim_page.html")

-- Using options table (base_dir, uses original name):
aip.file.save_html_to_slim("web/page.html", { base_dir = "slim_output" })
-- Assuming source was 'web/page.html', result might be 'slim_output/page.html'

-- Using options table (suffix):
aip.file.save_html_to_slim("web/page.html", { suffix = "_light" })
-- Assuming source was 'web/page.html', result might be 'web/page_light.html'

-- Using options table (no slimming):
aip.file.save_html_to_slim("web/page.html", { slim = false })
-- Result: web/page.html (original content saved)
```

#### Error


Returns an error (Lua table `{ error: string }`) if file I/O, slimming, or destination resolution fails.


### aip.file.load_html_as_slim

Loads an HTML file, "slims" its content (removes scripts, styles, comments, etc.), and returns the slimmed HTML string.

```lua
-- API Signature
aip.file.load_html_as_slim(html_path: string): string
```

#### Arguments

- `html_path: string`
  Path to the source HTML file, relative to the workspace root.

#### Returns

- `string`
  The slimmed HTML content.

#### Example

```lua
local slim = aip.file.load_html_as_slim("web/page.html")
-- For example, ensure scripts are removed:
print(string.find(slim, "<script") == nil)
```

#### Error

Returns an error (Lua table `{ error: string }`) if the HTML file cannot be found/read or if slimming fails.


### aip.file.load_html_as_md

Loads an HTML file, optionally "trims" (slims) its content, converts it to Markdown, and returns the Markdown string.

```lua
-- API Signature
aip.file.load_html_as_md(
  html_path: string,
  options?: { trim?: boolean } -- default true. When true, slim HTML before converting to Markdown.
): string
```

#### Arguments

- `html_path: string`
  Path to the source HTML file, relative to the workspace root.

- `options?: table`
  - `trim?: boolean` (default: true)
    When `true`, trims/slims the HTML (removes scripts, styles, comments, etc.) before conversion.
    Note: For compatibility, `slim` can also be used instead of `trim`.

#### Returns

- `string`
  The Markdown content converted from the (optionally slimmed) HTML.

#### Example

```lua
-- Default (slims first, then converts)
local md1 = aip.file.load_html_as_md("docs/page.html")

-- No slimming before conversion
local md2 = aip.file.load_html_as_md("docs/page.html", { trim = false })
```

#### Error

Returns an error (Lua table `{ error: string }`) if the HTML file cannot be found/read, if slimming fails (when enabled), or if the conversion to Markdown fails.


### aip.file.save_docx_to_md

Loads a DOCX file, converts its content to Markdown, and saves it.

```lua
-- API Signature
aip.file.save_docx_to_md(
  docx_path: string,
  dest?: string | table
): FileInfo
```

#### Arguments

- `docx_path: string`
  Path to the source DOCX file, relative to the workspace root.

- `dest?: string | table (optional)`
  Destination path or options table ([DestOptions](#destoptions)):

  - `string`
    Path to save the `.md` file (relative or absolute).
  - `table` ([DestOptions](#destoptions)):
      - `base_dir?: string`: Base directory for resolving the destination.
      - `file_name?: string`: Custom file name for the Markdown output.
      - `suffix?: string`: Suffix appended to the source file stem before `.md`.

#### Returns

- `[FileInfo](#fileinfo)`
  Metadata ([FileInfo](#fileinfo)) about the created Markdown file.

#### Example

```lua
-- Default (replaces .docx with .md):
aip.file.save_docx_to_md("docs/spec.docx")
-- Result: docs/spec.md

-- Using a custom string path:
aip.file.save_docx_to_md("docs/spec.docx", "out/spec.md")

-- Using options table:
aip.file.save_docx_to_md("docs/spec.docx", {
  base_dir = "output",
  suffix = "_v2",
})
-- Assuming source was 'docs/spec.docx', result might be 'output/spec_v2.md'
```

#### Error


Returns an error (Lua table `{ error: string }`) if file I/O, parsing/conversion, or destination resolution fails.


### aip.file.line_spans

Returns the byte spans for each line in a text file.

```lua
-- API Signature
aip.file.line_spans(path: string): [start: number, end: number][]
```

Given a file path, computes the start and end byte offsets for every line.

#### Arguments

- `path: string`
  Path to the source file (relative, absolute, or pack-ref supported).

#### Returns

- `[start: number, end: number]>[]
  A Lua list of two-item arrays where `span[1]` is the start byte and `span[2]` is the end byte for each line.

#### Example

```lua
local spans = aip.file.line_spans("logs/app.log")
for i, s in ipairs(spans) do
  print(i, s[1], s[2])
end
```

#### Error

Returns an error (Lua table `{ error: string }`) if the path cannot be resolved or the file cannot be read.


### aip.file.csv_row_spans

Returns the byte spans for each CSV row in a file.

```lua
-- API Signature
aip.file.csv_row_spans(path: string): [start: number, end: number][]
```

Parses the file as CSV and returns byte spans for each row (one span per CSV record).

#### Arguments

- `path: string`
  Path to the CSV file (relative, absolute, or pack-ref supported).

#### Returns

- `[start: number, end: number]>[]
  A Lua list of two-item arrays where `row[1]` is the start byte and `row[2]` is the end byte for each CSV row.

#### Example

```lua
local rows = aip.file.csv_row_spans("data/sample.csv")
-- Read the first row bytes as text:
if #rows > 0 then
  local first_row = rows[1]
  local text = aip.file.read_span("data/sample.csv", first_row[1], first_row[2])
  print(text)
end
```

#### Error

Returns an error (Lua table `{ error: string }`) if the path cannot be resolved or the file cannot be read/parsed as CSV.


### aip.file.read_span

Reads and returns the file substring between the given byte offsets.

```lua
-- API Signature
aip.file.read_span(path: string, start: number, end: number): string
```

#### Arguments

- `path: string`
  Path to the source file (relative, absolute, or pack-ref supported).

- `start: number`
  Start byte offset (non-negative).

- `end: number`
  End byte offset (non-negative, must be greater than or equal to `start`).

#### Returns

- `string`
  The substring of the file content between the given byte offsets.

#### Example

```lua
local spans = aip.file.line_spans("README.md")
-- Print the first line by span:
if #spans > 0 then
  local s = spans[1]
  local line = aip.file.read_span("README.md", s[1], s[2])
  print(line)
end
```

#### Error

Returns an error (Lua table `{ error: string }`) if `start`/`end` are invalid, the path cannot be resolved, or the file cannot be read.
