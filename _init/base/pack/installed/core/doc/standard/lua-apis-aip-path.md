## aip.path

Functions for path manipulation, checking, and resolution within the AIPack workspace.

### Functions Summary

```lua
aip.path.split(path: string): (parent: string, filename: string)

aip.path.resolve(path: string): string

aip.path.exists(path: string): boolean

aip.path.is_file(path: string): boolean

aip.path.is_dir(path: string): boolean

aip.path.diff(file_path: string, base_path: string): string

aip.path.parent(path: string): string | nil

aip.path.matches_glob(path: string | nil, globs: string | string[]): boolean | nil

aip.path.join(base: string, ...parts: string | string[]): string

aip.path.parse(path: string | nil): FileInfo | nil
```

> Note: Paths are typically relative to the workspace directory unless otherwise specified or resolved using pack references.

### aip.path.parse

Parses a path string and returns a [FileInfo](#fileinfo) table representation of its components.

```lua
-- API Signature
aip.path.parse(path: string | nil): FileInfo | nil
```

Parses the given path string into a structured [FileInfo](#fileinfo) containing components like `dir`, `name`, `stem`, `ext`, etc., without checking file existence or metadata.

#### Arguments

- `path: string | nil`: The path string to parse. If `nil`, the function returns `nil`.

#### Returns

- `FileInfo | nil`: A [FileInfo](#fileinfo) table representation of the parsed path components if `path` is a string. Returns `nil` if the input `path` was `nil`. Note that `ctime`, `mtime`, and `size` fields will be `nil` as this function only parses the string, it does not access the filesystem.

#### Example

```lua
local parsed = aip.path.parse("some/folder/file.txt")
-- parsed will be similar to { path = "some/folder/file.txt", dir = "some/folder", name = "file.txt", stem = "file", ext = "txt", ctime = nil, ... }
print(parsed.name) -- Output: "file.txt"

local nil_result = aip.path.parse(nil)
-- nil_result will be nil
```

#### Error


Returns an error (Lua table `{ error: string }`) if the path string is provided but is invalid and cannot be parsed into a valid SPath object.

```ts
{
  error: string // Error message
}
```

### aip.path.split

Split path into parent directory and filename.

```lua
-- API Signature
aip.path.split(path: string): (parent: string, filename: string)
```

Splits the given path into its parent directory and filename components.

#### Arguments

- `path: string`: The path to split.

#### Returns

- `parent: string`: The parent directory path (empty string if no parent).
- `filename: string`: The filename component (empty string if no filename).

#### Example

```lua
local parent, filename = aip.path.split("folder/file.txt")
print(parent)   -- Output: "folder"
print(filename) -- Output: "file.txt"

local parent, filename = aip.path.split("justafile.md")
print(parent)   -- Output: ""
print(filename) -- Output: "justafile.md"
```

#### Error

Does not typically error.

### aip.path.join

Joins a base path with one or more path segments.

```lua
-- API Signature
aip.path.join(base: string, ...parts: string | string[]): string
```

Constructs a new path by appending processed segments from `...parts` to the `base` path.
Each argument in `...parts` is first converted to a string:
- String arguments are used as-is.
- List (table) arguments have their string items joined by `/`.
These resulting strings are then concatenated together. Finally, this single concatenated string
is joined with `base` using system-appropriate path logic (which also normalizes separators like `//` to `/`).


#### Arguments

- `base: string`: The initial base path.
- `...parts: string | string[]` (variadic): One or more path segments to process and append. Each part can be a single string or a Lua list (table) of strings.

#### Returns

- `string`: A new string representing the combined and normalized path.

#### Example

```lua
-- Example 1: Basic join
print(aip.path.join("dir1/", "file1.txt"))             -- Output: "dir1/file1.txt"
print(aip.path.join("dir1", "file1.txt"))              -- Output: "dir1/file1.txt"

-- Example 2: Joining with a list (table)
print(aip.path.join("dir1/", {"subdir", "file2.txt"})) -- Output: "dir1/subdir/file2.txt"

-- Example 3: Multiple string arguments
-- Segments are concatenated, then joined to base.
print(aip.path.join("dir1/", "subdir/", "file3.txt"))  -- Output: "dir1/subdir/file3.txt"
print(aip.path.join("dir1/", "subdir", "file3.txt"))   -- Output: "dir1/subdirfile3.txt"

-- Example 4: Mixed arguments (strings and lists)
-- Lists are pre-joined with '/', then all resulting strings are concatenated, then joined to base.
print(aip.path.join("root/", {"user", "docs"}, "projectA", {"report", "final.pdf"}))
-- Output: "root/user/docsprojectAreport/final.pdf"

-- Example 5: Normalization
print(aip.path.join("", {"my-dir//", "///file.txt"}))  -- Output: "my-dir/file.txt"
print(aip.path.join("a", "b", "c"))                     -- Output: "a/bc"
print(aip.path.join("a/", "b/", "c/"))                  -- Output: "a/b/c/"
```

#### Error

Returns an error (Lua table `{ error: string }`) if any of the `parts` arguments cannot be converted to a string or a list of strings (e.g., passing a boolean or a function).


### aip.path.resolve

Resolves and normalizes a path relative to the workspace or pack structure.

```lua
-- API Signature
aip.path.resolve(path: string): string
```

Resolves relative paths (`.`, `..`), absolute paths, and pack references (`ns@pack/`, `ns@pack$base/`, `ns@pack$workspace/`) to a normalized, typically absolute, path.

#### Arguments

- `path: string`: The path string to resolve.

#### Returns

- `string`: The resolved and normalized path.

#### Example

```lua
local resolved_path = aip.path.resolve("./agent-script/../agent.aip")
-- Output: /path/to/workspace/agent.aip (example)

local resolved_pack_path = aip.path.resolve("ns@pack/some/file.txt")
-- Output: /path/to/aipack-base/packs/ns/pack/some/file.txt (example)
```

#### Error

Returns an error (Lua table `{ error: string }`) if the path cannot be resolved (e.g., invalid pack reference, invalid format).

### aip.path.exists

Checks if the specified path exists (file or directory).

```lua
-- API Signature
aip.path.exists(path: string): boolean
```

Checks existence after resolving the path relative to the workspace or pack structure.

#### Arguments

- `path: string`: The path string to check.

#### Returns

- `boolean`: `true` if exists, `false` otherwise.

#### Example

```lua
if aip.path.exists("README.md") then print("Exists") end
if aip.path.exists("ns@pack/main.aip") then print("Pack agent exists") end
```

#### Error

Returns an error (Lua table `{ error: string }`) if the path cannot be resolved.

### aip.path.is_file

Checks if the specified path points to an existing file.

```lua
-- API Signature
aip.path.is_file(path: string): boolean
```

Checks after resolving the path relative to the workspace or pack structure.

#### Arguments

- `path: string`: The path string to check.

#### Returns

- `boolean`: `true` if it's an existing file, `false` otherwise.

#### Example

```lua
if aip.path.is_file("config.toml") then print("Is a file") end
```

#### Error

Returns an error (Lua table `{ error: string }`) if the path cannot be resolved.

### aip.path.is_dir

Checks if the specified path points to an existing directory.

```lua
-- API Signature
aip.path.is_dir(path: string): boolean
```

Checks after resolving the path relative to the workspace or pack structure.

#### Arguments

- `path: string`: The path string to check.

#### Returns

- `boolean`: `true` if it's an existing directory, `false` otherwise.

#### Example

```lua
if aip.path.is_dir("src/") then print("Is a directory") end
```

#### Error

Returns an error (Lua table `{ error: string }`) if the path cannot be resolved.

### aip.path.diff

Computes the relative path from `base_path` to `file_path`.

```lua
-- API Signature
aip.path.diff(file_path: string, base_path: string): string
```

Calculates the relative path string that navigates from `base_path` to `file_path`.

#### Arguments

- `file_path: string`: The target path.
- `base_path: string`: The starting path.

#### Returns

- `string`: The relative path. Empty if paths are the same or cannot be computed (e.g., different drives).

#### Example

```lua
print(aip.path.diff("/a/b/c/file.txt", "/a/b/")) -- Output: "c/file.txt"
print(aip.path.diff("/a/b/", "/a/b/c/file.txt")) -- Output: "../.."
print(aip.path.diff("folder/file.txt", "folder")) -- Output: "file.txt"
```

#### Error

Returns an error (Lua table `{ error: string }`) if paths are invalid.

### aip.path.parent

Returns the parent directory path of the specified path.

```lua
-- API Signature
aip.path.parent(path: string): string | nil
```

Gets the parent directory component.

#### Arguments

- `path: string`: The path string.

#### Returns

- `string | nil`: The parent directory path, or `nil` if no parent (e.g., for ".", "/", "C:/").

#### Example

```lua
print(aip.path.parent("some/path/file.txt")) -- Output: "some/path"
print(aip.path.parent("."))                  -- Output: nil
```

#### Error

Does not typically error.


### aip.path.matches_glob

Checks if a path matches one or more glob patterns.

```lua
-- API Signature
aip.path.matches_glob(path: string | nil, globs: string | string[]): boolean | nil
```

Determines whether the provided `path` matches any of the glob patterns given
in `globs`. The function returns `nil` when `path` is `nil`.  
If `globs` is an empty string or an empty list, the result is `false`.

#### Arguments

- `path: string | nil`  
  The path to test. If `nil`, the function returns `nil`.

- `globs: string | string[]`  
  A single glob pattern string or a Lua list of pattern strings.  
  Standard wildcards (`*`, `?`, `**`, `[]`) are supported.

#### Returns

- `boolean | nil`  
  `true`  – when the `path` matches at least one pattern.  
  `false` – when it matches none.  
  `nil`   – when the supplied `path` was `nil`.

#### Example

```lua
-- Single pattern
print(aip.path.matches_glob("src/main.rs", "**/*.rs"))            -- true

-- Multiple patterns
print(aip.path.matches_glob("README.md", {"*.md", "*.txt"}))      -- true

-- No match
print(aip.path.matches_glob("image.png", {"*.jpg", "*.gif"}))     -- false

-- Nil path
print(aip.path.matches_glob(nil, "*.rs"))                         -- nil
```

#### Error

Returns an error (Lua table `{ error: string }`) if `globs` is not a string
or a list of strings.
