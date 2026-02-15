## aip.git

Functions for performing basic Git operations in the workspace.

### Functions Summary

```lua
aip.git.restore(path: string): string | {error: string, stdout?: string, stderr?: string, exit?: number}
```

### aip.git.restore

Executes `git restore <path>` in the workspace directory.

```lua
-- API Signature
aip.git.restore(path: string): string | {error: string, stdout?: string, stderr?: string, exit?: number}
```

Restores the specified file or directory path to its state from the Git index.

#### Arguments

- `path: string`: The file or directory path to restore (relative to workspace root).

#### Returns

- `string`: Standard output from the `git restore` command on success.
- `{error: string, stdout?: string, stderr?: string, exit?: number}`: An error table if the command fails (e.g., path not known to Git, non-zero exit code, stderr output). This error table is similar to a [CmdResponse](#cmdresponse) but includes an additional `error` field.

#### Example

```lua
-- Restore a modified file
local result = aip.git.restore("src/main.rs")
-- Check if result is an error table or the stdout string
if type(result) == "table" and result.error then
  print("Error restoring:", result.error)
  print("Stderr:", result.stderr) -- May contain git error message
else
  print("Restore stdout:", result)
end
```

#### Error

Returns an error (Lua table `{ error: string, stdout?: string, stderr?: string, exit?: number }`, similar to a [CmdResponse](#cmdresponse)) if the `git restore` command encounters an issue, such as the path not being known to Git, insufficient permissions, or the command returning a non-zero exit code with stderr output.
