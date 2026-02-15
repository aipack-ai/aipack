## aip.editor

Functions for opening files in external editors (VSCode, Zed, Vim, etc.) using auto-detection of the user's preferred editor.

### Functions Summary

```lua
aip.editor.open_file(path: string): { editor: string } | nil
```

### aip.editor.open_file

Opens the specified file in the auto-detected editor.

```lua
-- API Signature
aip.editor.open_file(path: string): { editor: string } | nil
```

Attempts to detect the preferred editor using environment variables (`VISUAL`, `EDITOR`, `TERM_PROGRAM`, `ZED_TERM`) and opens the file.

#### Arguments

- `path: string`: The path to the file to open. Supports relative paths, absolute paths, and pack references (e.g., `ns@pack/file.txt`).

#### Returns

- `table | nil`: A table `{ editor = "..." }` if successful, where "editor" is the command name of the detected editor (e.g., "code", "zed", "nvim"). Returns `nil` if no editor is detected or the command fails to start.

#### Example

```lua
-- Open the current input file in the editor
aip.editor.open_file(input.path)

-- Open a specific file
aip.editor.open_file("README.md")
```

#### Error

Returns an error (Lua table `{ error: string }`) if the path cannot be resolved.
