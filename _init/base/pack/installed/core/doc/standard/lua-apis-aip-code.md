## aip.code

Utility functions for code formatting and manipulation.

### Functions Summary

```lua
aip.code.comment_line(lang_ext: string, comment_content: string): string | {error: string}
```

### aip.code.comment_line

Creates a single comment line appropriate for a given language extension.

```lua
-- API Signature
aip.code.comment_line(lang_ext: string, comment_content: string): string | {error: string}
```

Formats `comment_content` as a single-line comment based on `lang_ext`.

#### Arguments

- `lang_ext: string`: File extension or language identifier (e.g., "rs", "lua", "py", "js", "css", "html"). Case-insensitive.
- `comment_content: string`: The text to put inside the comment.

#### Returns

- `string`: The formatted comment line (without trailing newline) on success.
- `{error: string}`: An error table on failure.

#### Example

```lua
print(aip.code.comment_line("rs", "TODO: Refactor"))  -- Output: // TODO: Refactor
print(aip.code.comment_line("py", "Add validation"))  -- Output: # Add validation
print(aip.code.comment_line("lua", "Fix this later")) -- Output: -- Fix this later
print(aip.code.comment_line("html", "Main content"))  -- Output: <!-- Main content -->
```

#### Error

Returns an error (Lua table `{ error: string }`) on conversion or formatting issues.
