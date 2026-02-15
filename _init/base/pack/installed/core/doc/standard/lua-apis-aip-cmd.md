## aip.cmd

Functions for executing system commands.

### Functions Summary

```lua
aip.cmd.exec(cmd_name: string, args?: string | string[]): CmdResponse | {error: string, stdout?: string, stderr?: string, exit?: number}
```

### aip.cmd.exec

Execute a system command with optional arguments.

```lua
-- API Signature
aip.cmd.exec(cmd_name: string, args?: string | string[]): CmdResponse | {error: string, stdout?: string, stderr?: string, exit?: number}
```

Executes the command using the system shell. On Windows, wraps with `cmd /C`.

#### Arguments

- `cmd_name: string`: Command name or path.
- `args?: string | string[]` (optional): Arguments as a single string or list of strings.

#### Returns

- `CmdResponse`: A [CmdResponse](#cmdresponse) table with stdout, stderr, and exit code, even if the exit code is non-zero.

#### Example

```lua
-- Single string argument
local r1 = aip.cmd.exec("echo", "hello world")
print("stdout:", r1.stdout) -- Output: hello world\n (or similar)
print("exit:", r1.exit)   -- Output: 0

-- Table of arguments
local r2 = aip.cmd.exec("ls", {"-l", "-a", "nonexistent"})
print("stderr:", r2.stderr) -- Output: ls: nonexistent: No such file... (or similar)
print("exit:", r2.exit)   -- Output: non-zero exit code

-- Example of potential error return (e.g., command not found)
local r3 = aip.cmd.exec("nonexistent_command")
if type(r3) == "table" and r3.error then
  print("Execution Error:", r3.error)
end
```

#### Error

Returns an error (Lua table `{ error: string, stdout?: string, stderr?: string, exit?: number }`) only if the process *fails to start* (e.g., command not found, permission issue). Non-zero exit codes from the command itself are captured in the [CmdResponse](#cmdresponse) and do not cause a Lua error by default.
