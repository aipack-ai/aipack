## Common Types

Common data structures returned by or used in API functions.

### FileRecord

Represents a file with its metadata and content. Returned by `aip.file.load` and `aip.file.list_load`.

```ts
{
  _type: "FileRecord",
  path : string,    // Relative or absolute path used to load/find the file
  dir: string,      // Parent directory of the path (empty if no parent)
  name : string,    // File name with extension (e.g., "main.rs")
  stem : string,    // File name without extension (e.g., "main")
  ext  : string,    // File extension (e.g., "rs")

  ctime?: number,   // Creation timestamp (microseconds since epoch), optional
  ctime?: number,   // Modification timestamp (microseconds), optional
  size?: number,    // File size in bytes, optional

  content: string   // The text content of the file
}
```

### FileInfo

Represents file metadata without content. Returned by `aip.file.list`, `aip.file.first`, `aip.file.ensure_exists`, `aip.file.save_html_to_md`, and `aip.file.save_html_to_slim`.

```ts
{
  _type: "FileInfo",
  path : string,     // Relative or absolute path
  dir: string,       // Parent directory of the path
  name : string,     // File name with extension
  stem : string,     // File name without extension
  ext  : string,     // File extension

  ctime?: number,    // Creation timestamp (microseconds), optional (if with_meta=true for list)
  mtime?: number,    // Modification timestamp (microseconds), optional (if with_meta=true for list)
  size?: number      // File size in bytes, optional (if with_meta=true for list)
}
```

### FileStats

Aggregated statistics for a collection of files. Returned by `aip.file.stats`.

```ts
{
  _type: "FileStats",
  total_size: number,      // Total size of all matched files in bytes
  number_of_files: number, // Number of files matched
  ctime_first: number,     // Creation timestamp of the oldest file (microseconds since epoch)
  ctime_last: number,      // Creation timestamp of the newest file (microseconds since epoch)
  mtime_first: number,     // Modification timestamp of the oldest file (microseconds since epoch)
  mtime_last: number       // Modification timestamp of the newest file (microseconds since epoch)
}
```

### SaveOptions

Options table used for configuring content pre-processing in `aip.file.save`.

```ts
{
  trim_start?: boolean,            // If true, remove leading whitespace (default false).
  trim_end?: boolean,              // If true, remove trailing whitespace (default false).
  single_trailing_newline?: boolean // If true, ensure content ends with exactly one '\n' (default false).
}
```

### DestOptions

Options table used for specifying the destination path in functions like `aip.file.save_html_to_md` and `aip.file.save_html_to_slim`.

```ts
{
  base_dir?: string,  // Base directory for resolving the destination
  file_name?: string, // Custom file name for the output
  suffix?: string,    // Suffix appended to the source file stem
  slim?: boolean      // Whether to slim the HTML content (default false)
}
```

### CsvContent

Represents the table returned by `aip.file.load_csv`, which includes the `_type = "CsvContent"` marker, the parsed `headers` (empty when no header row was requested), and the `rows` matrix.

```ts
{
  _type: "CsvContent",
  headers: string[],
  rows: string[][]
}
```

### CsvOptions

Options table used for configuring CSV parsing behavior in functions like `aip.csv.parse` and `aip.file.load_csv`.

```ts
{
  delimiter?: string,         // Column delimiter, default ","
  quote?: string,             // Quote character, default "\""
  escape?: string,            // Escape character, default "\""
  trim_fields?: boolean,      // Whether to trim whitespace from fields, default false
  has_header?: boolean,       // Whether the first row is a header, default false (true for aip.file.load_csv),
  header_labels?: { [string]: string }, // Map { key: label } for renaming headers/keys (parsing and writing),
  skip_empty_lines?: boolean, // Whether to skip empty lines, default true,
  comment?: string,           // Comment character prefix (e.g., "#"), optional
  skip_header_row?: boolean   // Writing only: Suppress header emission even if headers are available (default false).
}
```

When an option expecting a character is given a multi-character string, only the first byte is used.

### MdSection

Represents a section of a Markdown document, potentially associated with a heading. Returned by `aip.file.load_md_sections` and `aip.file.load_md_split_first`.

```ts
{
  _type: "MdSection",
  content: string,    // Full content of the section (including heading line and sub-sections)
  heading_raw: string,      // The raw heading line with trailing newline (empty string if no heading)
  heading_content: string,  // The raw heading line without trailing newline (empty string if no heading)
  heading_level: number,    // Heading level (1-6), or 0 if no heading
  heading_name: string      // Extracted and trimmed heading name (empty string if no heading)
}
```

### MdBlock

Represents a fenced block (usually code) in Markdown. Returned by `aip.md.extract_blocks`.

```ts
{
  _type: "MdBlock",
  content: string,     // Content inside the block (excluding fence lines)
  lang?: string        // Language identifier (e.g., "rust", "lua"), optional
}
```

### MdRef

A parsed Markdown inline reference (link or image). Returned by `aip.md.extract_refs`.

```ts
{
  _type: "MdRef",       // Type identifier
  target: string,       // URL, file path, or in-document anchor
  text: string | nil,   // Content inside the brackets (nil if empty)
  inline: boolean,      // True if prefixed with '![' (image)
  kind: string          // "Anchor" | "File" | "Url"
}
```

### TagElem

Represents a block defined by start and end tags, like `<TAG>content</TAG>`. Returned by `aip.tag.extract`.

```ts
{
  tag: string,       // The tag name (e.g., "FILE", "DATA")
  attrs?: table,     // Key/value map of attributes from the opening tag, optional
  content: string    // The content between the opening and closing tags
}
```

### ApplyChangesStatus

Represents the overall result of applying multi-file changes. Returned by `aip.udiffx.apply_file_changes`.

```ts
{
  success: boolean,        // true if all directives were applied successfully
  total_count: number,     // Total number of directives found
  success_count: number,   // Number of successful directives
  fail_count: number,      // Number of failed directives
  items: ApplyChangesItem[] // List of results for each directive
}
```

### ApplyChangesItem

Represents the result for a single directive in a multi-file change operation.

```ts
{
  file_path: string,       // Path of the affected file
  kind: string,            // One of "New", "Patch", "Rename", "Delete", or "Fail"
  success: boolean,        // true if this directive succeeded
  error_msg?: string       // Error details if success is false
}
```

### WebResponse

Represents the result of an HTTP request made by `aip.web.get` or `aip.web.post`.

```ts
{
  success: boolean,   // true if HTTP status code is 2xx, false otherwise
  status: number,     // HTTP status code (e.g., 200, 404, 500)
  url: string,        // The final URL requested (after redirects)
  content: string | table, // Response body. Decoded to a Lua table if Content-Type is application/json AND WebOptions.parse was true, otherwise a string.
  content_type?: string, // The value of the Content-Type header, if present
  headers?: table,      // Lua table of response headers { header_name: string | string[] }
  error?: string      // Error message if success is false or if request initiation failed
}
```

### WebOptions

Options table used for configuring HTTP requests in `aip.web` functions.

```ts
{
  user_agent?: string | boolean,    // If boolean true, sets 'aipack' UA (aip.web.UA_AIPACK). If false, prevents setting UA. If string, sets as-is (can use aip.web.UA_BROWSER). Takes precedence over 'User-Agent' in headers. Defaults to 'aipack' if omitted and 'User-Agent' is missing from headers.
  headers?: table,                  // { header_name: string | string[] }
  redirect_limit?: number,          // Number of redirects to follow (default 5)
  parse?: boolean                   // If true, attempts to parse JSON response body if Content-Type is 'application/json'. Content in WebResponse will be a Lua table if successful, otherwise a string (defaults to false).
}
```

### CmdResponse

Represents the result of executing a system command via `aip.cmd.exec`.

```ts
{
  stdout: string,  // Standard output captured from the command
  stderr: string,  // Standard error captured from the command
  exit:   number   // Exit code returned by the command (0 usually indicates success)
}
```

### AgentOptions

Configuration options for an agent. Used in `aip.flow.before_all_response` and `aip.flow.data_response` to override settings for a run or a specific cycle.

```ts
{
  model?: string,
  temperature?: number,
  top_p?: number,
  input_concurrency?: number,
  model_aliases?: { [key: string]: string }
}
```

### Attachments

Represents a collection of file attachments that can be attached to a prompt. Used in `aip.flow.data_response` to attach images, PDFs, or other binary files to the AI request.

```ts
// Can be provided as:
// - A single Attachment object
// - A list of Attachment objects
// - null or empty object (no attachments)

type Attachment = {
  file_source: string,   // Local file path to the attachment
  file_name?: string,    // Optional custom file name for display
  title?: string         // Optional title/description for the attachment
}

type Attachments = Attachment[]
```

#### Example

```lua
return aip.flow.data_response({
  data = data,
  attachments = {
    { file_source = "images/screenshot.png", title = "UI Screenshot" },
    { file_source = "docs/spec.pdf", file_name = "specification.pdf" }
  }
})

-- Or a single attachment
return aip.flow.data_response({
  data = data,
  attachments = { file_source = "diagram.png" }
})
```

### YamlDocs

Represents a list of parsed YAML documents. Returned by `aip.file.load_yaml` and `aip.yaml.parse`.

```ts
type YamlDocs = any[] // List of parsed YAML documents
```

### Marker

Represents a simple labeled content component.

```ts
{
  label: string,
  content: string
}
```

## CTX

All Lua scripts get the `CTX` table in scope, providing context about the current execution environment.

| Key                      | Example Value                                                            | Description                                                       |
|--------------------------|--------------------------------------------------------------------------|-------------------------------------------------------------------|
| CTX.WORKSPACE_DIR        | `/Users/dev/my-project`                                                  | Absolute path to the workspace directory (containing `.aipack/`). |
| CTX.WORKSPACE_AIPack_DIR | `/Users/dev/my-project/.aipack`                                          | Absolute path to the `.aipack/` directory in the workspace.       |
| CTX.BASE_AIPACK_DIR      | `/Users/dev/.aipack-base`                                                | Absolute path to the user's base AIPack directory.                |
| CTX.AGENT_NAME           | `my_pack/my-agent` or `path/to/my-agent.aip`                             | The name or path used to invoke the agent.                        |
| CTX.AGENT_FILE_PATH      | `/Users/home/john/.aipack-base/pack/installed/acme/my_pack/my-agent.aip` | Absolute path to the resolved agent `.aip` file.                  |
| CTX.AGENT_FILE_DIR       | `/Users/home/john/.aipack-base/pack/installed/acme/my_pack`              | Absolute path to the directory containing the agent file.         |
| CTX.AGENT_FILE_NAME      | `my-agent.aip`                                                           | The base name of the my-agent file.                               |
| CTX.AGENT_FILE_STEM      | `my-agent`                                                               | The base name of the agent file without extension.                |
| CTX.TMP_DIR              | `.aipack/.session/0196adbf-b792-7070-a5be-eec26698c065/tmp`              | The tmp dir for this session (all redos in same session)          |
| CTX.SESSION_UID          | `0196adbf-b792-7070-a5be-eec26698c065`                                   | The Session Unique ID for this CLI Session                        |
| CTX.RUN_UID              | `0196adbf-b792-7070-a5be-ddc33698c065`                                   | The Run Unique ID                                                 |
| CTX.RUN_NUM              | `1`                                                                      | 1-based sequence number of the current agent run in the session.  |
| CTX.TASK_UID             | `0196adbf-b792-7070-a5be-aac55698c065`                                   | The Task Unique ID (when in a task stage)                         |
| CTX.TASK_NUM             | `5`                                                                      | 1-based sequence number of the current task in the run.           |



When running a pack. (when no packs, those will be all nil)

For `aip run acme@my_pack/my-agent`

| Key                            | Example Value                                             | Description                                                                       |
|--------------------------------|-----------------------------------------------------------|-----------------------------------------------------------------------------------|
| CTX.PACK_IDENTITY              | `acme@my_pack`                                            | Pack identity (namespace@name) (nil if not run via pack ref).                     |
| CTX.PACK_NAMESPACE             | `acme`                                                    | Namespace of the pack (nil if not run via pack reference).                        |
| CTX.PACK_NAME                  | `my_pack`                                                 | Name of the pack (nil if not run via pack reference).                             |
| CTX.PACK_REF                   | `acme@my_pack/my-agent`                                   | (Nil if not a pack) Full pack reference used (nil if not run via pack reference). |
| CTX.PACK_WORKSPACE_SUPPORT_DIR | `/Users/dev/my-project/.aipack/support/pack/acme/my_pack` | Workspace-specific support directory for this agent (if applicable).              |
| CTX.PACK_BASE_SUPPORT_DIR      | `/Users/home/john/.aipack-base/support/pack/acme/my_pack` | Base support directory for this agent (if applicable).                            |


- All paths are absolute and normalized for the OS.
- `CTX.PACK...` fields are `nil` if the agent was invoked directly via its file path rather than a pack reference (e.g., `aip run my-agent.aip`).
- The `AGENT_NAME` reflects how the agent was called, while `AGENT_FILE_PATH` is the fully resolved location.
