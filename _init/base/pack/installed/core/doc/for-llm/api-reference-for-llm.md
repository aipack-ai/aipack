# AIPack Agent API Reference for LLMs

This document summarizes the structure, execution flow, and complete Lua API of an AIPack agent (`.aip` file) to enable autonomous creation and modification of agents.

## 1. Agent Execution Flow

An AIPack agent is a multi-stage Markdown file where stages define pre-processing, prompt templating (Handlebars), AI interaction, and post-processing (Lua).

| Stage           | Language       | Runs Per        | Injected Variables (Scope)                                      | Purpose                                                              |
|-----------------|----------------|-----------------|-----------------------------------------------------------------|----------------------------------------------------------------------|
| `# Options`     | **TOML (Markdown block)** | Once            | N/A                                                             | **Stage 0 (Config Step)**: Agent-specific configuration.             |
| `# Before All`  | **Lua (Markdown block)**  | Once            | `aip`, `CTX`, `inputs`                                          | **Stage 1**: Global setup, filtering `inputs`.                       |
| `# Data`        | **Lua (Markdown block)**  | Per Input       | `aip`, `CTX`, `input`, `before_all`                             | **Stage 2**: Per-input data gathering and flow control.              |
| `# System`      | **Handlebars** | Per Input       | `input`, `data`, `before_all`                                   | **Stage 3**: System prompt template.                                 |
| `# Instruction` | **Handlebars** | Per Input       | `input`, `data`, `before_all`                                   | **Stage 3**: User instruction prompt template.                       |
| `# Assistant`   | **Handlebars** | Per Input       | `input`, `data`, `before_all`                                   | **Stage 3**: Optional specialized prompt priming.                    |
| `# Output`      | **Lua (Markdown block)**  | Per Processed Input | `aip`, `CTX`, `input`, `data`, `before_all`, `ai_response`      | **Stage 4**: Process AI response and perform side effects.           |
| `# After All`   | **Lua (Markdown block)**  | Once            | `aip`, `CTX`, `inputs`, `outputs`, `before_all`                 | **Stage 5**: Final cleanup and aggregation.                          |

**Injected Variables Detail:**

| Variable      | Type            | Description                                                                 |
|---------------|-----------------|-----------------------------------------------------------------------------|
| `aip`         | `AipModule`     | The AIPack API module, containing utility functions (see section 4).         |
| `CTX`         | `Context`       | Contextual constants (paths, session UIDs). See section 5.                   |
| `inputs`      | `any[]`         | List of initial or modified inputs (`# Before All`, `# After All`).           |
| `input`       | `any`           | Current input item (e.g., `string` or `FileInfo`).                          |
| `before_all`  | `any`           | Data returned from `# Before All`.                                          |
| `data`        | `any`           | Data returned from `# Data`.                                                |
| `ai_response` | `AiResponse`    | AI result object (`# Output` only). See section 3.                          |

## 2. Flow Control (`aip.flow`)

Functions used as the return value in `# Before All` or `# Data` to control the agent pipeline.

```typescript
type BeforeAllData = {
  inputs?: any[],      // New list of inputs, replaces original inputs.
  options?: AgentOptions, // Overrides global options for the run (model, concurrency, etc.).
  before_all?: any,    // Data passed to subsequent stages.
  [key: string]: any,  // Arbitrary data fields allowed.
}

type DataData = {
  input?: any | nil,   // New input for this cycle. If nil, original input is used.
  data?: any | nil,    // Data passed to prompt/output stages.
  options?: AgentOptions, // Overrides options for this specific cycle.
  attachments?: Attachment | Attachment[] // Attachments for this cycle (e.g., images).
  [key: string]: any,  // Arbitrary data fields allowed.
}

/** Customizes execution flow at the 'Before All' stage (use as return value). */
aip.flow.before_all_response(data: BeforeAllData): table

/** Customizes execution flow at the 'Data' stage (use as return value). */
aip.flow.data_response(data: DataData): table

/** Skips processing the current input cycle (use as return value in # Data). */
aip.flow.skip(reason?: string): table
```

## 3. Core Data Types

### AiResponse

The result object available in the `# Output` stage.

```typescript
type AiResponse = {
  content?: string,          // The final text response from the AI.
  info: string,             // Formatted string capturing usage, price, model, duration.
  model_name: string,       // e.g., `gpt-5-mini`
  adapter_kind: string,     // e.g., `openai`
  usage: {
    prompt_tokens: number,
    completion_tokens: number
  },
  price_usd?: number,       // Approximate price in USD, if available.
  duration_sec: number,     // Duration in seconds (with millisecond precision).
  reasoning_content?: string, // Reasoning content, if available.
}
```

### FileInfo & FileRecord

File metadata and content structures.

```typescript
type FileInfo = {
  path: string,    // Relative or absolute path
  dir: string,     // Parent directory of the path
  name: string,    // File name with extension
  stem: string,    // File name without extension
  ext: string,     // File extension
  ctime?: number,  // Creation timestamp (microseconds since epoch)
  mtime?: number,  // Modification timestamp (microseconds)
  size?: number    // File size in bytes
}

type FileRecord = FileInfo & {
  content: string  // The text content of the file
}

type FileStats = {
  _type: "FileStats",
  total_size: number,      // Total size of all matched files in bytes
  number_of_files: number, // Number of files matched
  ctime_first: number,     // Oldest creation time (epoch microseconds)
  ctime_last: number,      // Newest creation time (epoch microseconds)
  mtime_first: number,     // Oldest modification time (epoch microseconds)
  mtime_last: number       // Newest modification time (epoch microseconds)
}
```

### Options Types

```typescript
type SaveOptions = {
  trim_start?: boolean,
  trim_end?: boolean,
  single_trailing_newline?: boolean
}

type ChangesInfo = {
  changed_count: number,     // Number of successful change count
  failed_changes?: FailChange[] // List of failed changes, if any
}

type FailChange = {
  search: string,
  replace: string,
  reason: string
}

type DestOptions = {
  base_dir?: string,
  file_name?: string,
  suffix?: string,
  slim?: boolean
}

type AgentOptions = {
  model?: string,
  temperature?: number,
  top_p?: number,
  input_concurrency?: number,
  model_aliases?: { [key: string]: string }
}
```

### Attachments

Used in `DataData` to attach files for multimodal models.

```typescript
type Attachment = {
  file_source: string,   // Local file path to the attachment
  file_name?: string,    // Optional custom file name for display
  title?: string         // Optional title/description for the attachment
}

type Attachments = Attachment | Attachment[] // List or single attachment
```

### WebResponse

Result structure from `aip.web.get`/`post`.

```typescript
type WebResponse = {
  success: boolean,        // true if HTTP status code is 2xx
  status: number,          // HTTP status code
  url: string,             // The final URL requested
  content: string | table, // Response body (table if JSON and parse=true)
  content_type?: string,
  headers?: { [key: string]: string | string[] },
  error?: string           // Error message if request failed or non-2xx status
}

type WebOptions = {
  user_agent?: string | boolean,
  headers?: table,
  redirect_limit?: number,
  parse?: boolean          // Attempt JSON parsing if Content-Type is 'application/json' (default false)
}
```

### CSV/Data Types

```typescript
type CsvContent = {
  _type: "CsvContent",
  headers: string[],
  rows: string[][]
}

type CsvOptions = {
  delimiter?: string,
  quote?: string,
  escape?: string,
  trim_fields?: boolean,
  has_header?: boolean,
  header_labels?: { [string]: string },
  skip_empty_lines?: boolean,
  comment?: string,
  skip_header_row?: boolean
}

type YamlDocs = any[] // List of parsed YAML documents

type Marker = {
  label: string,
  content: string
}
```

### Markdown Types

```typescript
type MdSection = {
  content: string,    // Full content of the section
  heading_raw: string,      // The raw heading line with trailing newline (empty string if no heading)
  heading_content: string,  // The raw heading line without trailing newline (empty string if no heading)
  heading_level: number,    // Heading level (1-6), or 0 if no heading
  heading_name: string      // Extracted and trimmed heading name (empty string if no heading)
}

type MdBlock = {
  content: string,     // Content inside the block (excluding fence lines)
  lang?: string,        // Language identifier (e.g., "rust")
}

type MdRef = {
  _type: "MdRef",
  target: string,       // URL, file path, or anchor
  text: string | nil,
  inline: boolean,      // True if prefixed with '![' (image)
  kind: string          // "Anchor" | "File" | "Url"
}
```

### RunAgentResponse & Other Utility Types

```typescript
type RunAgentResponse = {
  outputs: any[],   // List of values returned by each # Output stage for each input.
  after_all: any    // The value returned by the # After All stage (or nil).
}
```

```typescript
type TagElem = {
  tag: string,       // The tag name (e.g., "FILE")
  attrs?: table,     // Key/value map of attributes from the opening tag
  content: string    // The content between the tags
}

type CmdResponse = {
  stdout: string,  // Standard output
  stderr: string,  // Standard error
  exit:   number   // Exit code (0 usually success)
}
```

## 4. Lua Semantics & API Reference

### 4.1 nil vs. null

AIPack introduces a global `null` sentinel to bridge the gap between Lua's `nil` and JSON/SQL nulls.

- **`nil` (Lua Native)**: Use for "no value." In arrays (sequential tables), a `nil` will stop `ipairs` iteration. In objects, setting a property to `nil` deletes the key.
- **`null` (AIPack Sentinel)**: Use for "null value." It is preserved in arrays and does not stop `ipairs`. It is preserved in JSON serialization. Also available as `Null` and `NULL`.

**Recommendation:** In Lua, `nil` and the `null` sentinel are different types. Always use the global helpers (`is_null`, `nil_if_null`, `value_or`) to check or handle these values, as standard Lua comparison (e.g., `val == nil`) will not detect the `null` sentinel.

**Global Helpers:**

- `is_null(v)`: returns `true` if `v` is `nil` or `null`.
- `nil_if_null(v)`: returns `nil` if `v` is `nil` or `null`, otherwise `v`.
- `value_or(v, alt)`: returns `alt` if `v` is `nil` or `null`, otherwise `v`.

### 4.2 API (`aip.*`) Reference

**General Rules:**
- All functions return an error table `{ error: string }` on failure, unless otherwise specified (like `aip.path.parent` returning `nil`).
- Paths starting with `~` are user home. `ns@pack/` are pack references. Relative paths resolve to workspace root.
- `null` is a global sentinel for missing values (behaves like JSON null). Native Lua `nil` erases properties and stops `ipairs`.
- Build/dependency folders (e.g., `target/`, `node_modules/`) are excluded from file lists by default.
- **Stage Content Formatting:** Content for code-based stages (Lua and TOML) must be enclosed within triple-backtick Markdown code blocks. For example, use ```lua ... ``` for script stages and ```toml ... ``` for the options stage.

### aip.file - File System Operations

```typescript
aip.file.load(rel_path: string, options?: {base_dir: string}): FileRecord // base_dir can use pack references (ns@pack/).
aip.file.save(rel_path: string, content: string, options?: SaveOptions): FileInfo // SaveOptions: trim_start, trim_end, single_trailing_newline.
aip.file.copy(src_path: string, dest_path: string, options?: {overwrite?: boolean}): FileInfo // Workspace restricted. Default overwrite: false.
aip.file.move(src_path: string, dest_path: string, options?: {overwrite?: boolean}): FileInfo // Workspace restricted. Default overwrite: false.
aip.file.append(rel_path: string, content: string): FileInfo // Creates file/dirs if missing.
aip.file.delete(path: string): boolean // Allowed ONLY within workspace; forbidden in .aipack-base/.
aip.file.ensure_exists(path: string, content?: string, options?: {content_when_empty?: boolean}): FileInfo // content_when_empty: writes content if file exists but is whitespace-only.
aip.file.exists(path: string): boolean // Supports pack refs and relative/absolute paths.
aip.file.list(include_globs: string | string[], options?: {base_dir?: string, absolute?: boolean, with_meta?: boolean}): FileInfo[] // absolute: paths in result will be absolute (default false, but absolute if outside base_dir). with_meta: includes ctime, mtime, size (default true). Heavy dirs (target/, node_modules/) excluded unless explicitly matched.
aip.file.list_load(include_globs: string | string[], options?: {base_dir?: string, absolute?: boolean}): FileRecord[] // Loads content for all matching files.
aip.file.first(include_globs: string | string[], options?: {base_dir?: string, absolute?: boolean}): FileInfo | nil // Returns first matching file metadata.
aip.file.info(path: string): FileInfo | nil // Returns metadata or nil if not found.
aip.file.stats(include_globs: string | string[] | nil, options?: {base_dir?: string, absolute?: boolean}): FileStats | nil // Returns nil if globs is nil.
aip.file.load_json(path: string | nil): table | value | nil // Supports jsonc (comments and trailing commas).
aip.file.load_ndjson(path: string | nil): object[] | nil // Parses newline-delimited JSON.
aip.file.load_toml(path: string): table | value // Parses TOML file.
aip.file.load_yaml(path: string): list // Returns a list of documents.
aip.file.append_json_line(path: string, data: value): FileInfo // Serializes to JSON line.
aip.file.append_json_lines(path: string, data: list): FileInfo // Appends list as multiple JSON lines.
aip.file.save_changes(path: string, changes: string): FileInfo // Saves udiff-style changes.
aip.file.load_md_sections(path: string, headings?: string | string[]): MdSection[] // Filter by heading name(s).
aip.file.load_md_split_first(path: string): {before: string, first: MdSection, after: string} // Splits by first '#' heading.
aip.file.load_csv_headers(path: string): string[] // Returns header row only.
aip.file.load_csv(path: string, options?: CsvOptions): CsvContent // has_header default: true.
aip.file.save_as_csv(path: string, data: any[][] | {headers?: string[], rows?: any[][]}, options?: CsvOptions): FileInfo // Overwrites as CSV.
aip.file.save_records_as_csv(path: string, records: table[], header_keys: string[], options?: CsvOptions): FileInfo // record field selection via header_keys.
aip.file.append_csv_rows(path: string, value_lists: any[][], options?: CsvOptions): FileInfo // Simple data append (ignores has_header option).
aip.file.append_csv_row(path: string, values: any[], options?: CsvOptions): FileInfo // Single row append.
aip.file.save_html_to_md(html_path: string, dest?: string | table): FileInfo // Converts and saves HTML as Markdown.
aip.file.save_html_to_slim(html_path: string, dest?: string | table): FileInfo // Removes non-content tags. dest default: [stem]-slim.html.
aip.file.load_html_as_slim(html_path: string): string // Returns slimmed HTML string.
aip.file.load_html_as_md(html_path: string, options?: { trim?: boolean }): string // trim default: true (slims before conversion).
aip.file.save_docx_to_md(docx_path: string, dest?: string | table): FileInfo // Converts .docx to Markdown.
aip.file.load_docx_as_md(docx_path: string): string // Returns content as Markdown.
aip.file.line_spans(path: string): [start: number, end: number][] // Byte offsets for lines.
aip.file.csv_row_spans(path: string): [start: number, end: number][] // Byte offsets for CSV records.
aip.file.read_span(path: string, start: number, end: number): string // Reads file substring by byte offsets.
aip.file.hash_sha256(path: string): string // Hex encoding.
aip.file.hash_sha256_b64(path: string): string // Base64 encoding.
aip.file.hash_sha256_b64u(path: string): string // URL-safe Base64 (no padding).
aip.file.hash_sha256_b58u(path: string): string // Base58 encoding.
aip.file.hash_sha512(path: string): string // Hex encoding.
aip.file.hash_sha512_b64(path: string): string // Base64 encoding.
aip.file.hash_sha512_b64u(path: string): string // URL-safe Base64 (no padding).
aip.file.hash_sha512_b58u(path: string): string // Base58 encoding.
aip.file.hash_blake3(path: string): string // Hex encoding.
aip.file.hash_blake3_b64(path: string): string // Base64 encoding.
aip.file.hash_blake3_b64u(path: string): string // URL-safe Base64 (no padding).
aip.file.hash_blake3_b58u(path: string): string // Base58 encoding.
```

### aip.editor - Editor Integration

```typescript
aip.editor.open_file(path: string): { editor: string } | nil
```

### aip.path - Path Manipulation

```typescript
aip.path.split(path: string): (parent: string, filename: string) // Splits into dir and file parts.
aip.path.resolve(path: string): string // Normalizes relative paths, pack refs, and home tildes.
aip.path.exists(path: string): boolean // Check file/dir existence.
aip.path.is_file(path: string): boolean // True if existing file.
aip.path.is_dir(path: string): boolean // True if existing directory.
aip.path.diff(file_path: string, base_path: string): string // Relative path from base to file.
aip.path.parent(path: string): string | nil // Returns parent dir or nil.
aip.path.matches_glob(path: string | nil, globs: string | string[]): boolean | nil // Returns nil if path is nil.
aip.path.join(base: string, ...parts: string | string[]): string // Parts are concatenated into one string first, then joined to base with separator. Use table for path separation.
aip.path.parse(path: string | nil): FileInfo | nil
```

### aip.text - String Manipulation

```typescript
aip.text.escape_decode(content: string | nil): string | nil
aip.text.escape_decode_if_needed(content: string | nil): string | nil
aip.text.split_first(content: string | nil, sep: string): (string | nil, string | nil)
aip.text.split_last(content: string | nil, sep: string): (string | nil, string | nil)
aip.text.remove_first_line(content: string | nil): string | nil
aip.text.remove_first_lines(content: string | nil, n: number): string | nil
aip.text.remove_last_line(content: string | nil): string | nil
aip.text.remove_last_lines(content: string | nil, n: number): string | nil
aip.text.trim(content: string | nil): string | nil
aip.text.trim_start(content: string | nil): string | nil
aip.text.trim_end(content: string | nil): string | nil
aip.text.truncate(content: string | nil, max_len: number, ellipsis?: string): string | nil
aip.text.replace_markers(content: string | nil, new_sections: list): string | nil
aip.text.ensure(content: string | nil, {prefix?: string, suffix?: string}): string | nil // Adds prefix/suffix only if missing.
aip.text.ensure_single_trailing_newline(content: string | nil): string | nil
aip.text.format_size(bytes: integer | nil, lowest_size_unit?: "B" | "KB" | "MB" | "GB"): string | nil // lowest_size_unit defaults to "B".
aip.text.extract_line_blocks(content: string | nil, options: {starts_with: string, extrude?: "content", first?: number}): (string[] | nil, string | nil)
aip.text.split_first_line(content: string | nil, sep: string): (string | nil, string | nil)
aip.text.split_last_line(content: string | nil, sep: string): (string | nil, string | nil)
```

### aip.tag - Custom Tag Extraction

```typescript
aip.tag.extract(content: string, tag_names: string | string[], options?: {extrude?: "content"}): TagElem[] | (TagElem[], string)
aip.tag.extract_as_map(content: string, tag_names: string | string[], options?: {extrude?: "content"}): table | (table, string)
aip.tag.extract_as_multi_map(content: string, tag_names: string | string[], options?: {extrude?: "content"}): table | (table, string)
```

### aip.md - Markdown Processing

```typescript
aip.md.extract_blocks(md_content: string): MdBlock[] // Extracts all fenced code blocks.
aip.md.extract_blocks(md_content: string, lang: string): MdBlock[]
aip.md.extract_blocks(md_content: string, {lang?: string, extrude: "content"}): (MdBlock[], string) // extrude: returns content outside blocks as 2nd value.
aip.md.extract_meta(md_content: string | nil): (table | nil, string | nil) // Returns (nil, nil) if md_content is nil.
aip.md.outer_block_content_or_raw(md_content: string): string
aip.md.extract_refs(md_content: string | nil): MdRef[] // Returns empty list if md_content is nil.
```

### aip.json - JSON Helpers

```typescript
aip.json.parse(content: string | nil): table | value | nil // Supports jsonc (comments and trailing commas).
aip.json.parse_ndjson(content: string | nil): object[] | nil // Parses newline-delimited JSON.
aip.json.stringify(content: table): string // Compact single-line JSON.
aip.json.stringify_pretty(content: table): string // Pretty-printed (2 spaces).
```

### aip.toml - TOML Helpers

```typescript
aip.toml.parse(content: string): table
aip.toml.stringify(content: table): string
```

### aip.yaml - YAML Helpers

```typescript
aip.yaml.parse(content: string | nil): table[] | nil
aip.yaml.stringify(content: any): string
aip.yaml.stringify_multi_docs(content: table): string
```

### aip.web - HTTP Requests & URL

```typescript
aip.web.get(url: string, options?: WebOptions): WebResponse // Default User-Agent is 'aipack'.
aip.web.post(url: string, data: string | table, options?: WebOptions): WebResponse // Default User-Agent is 'aipack'.
aip.web.parse_url(url: string | nil): table | nil
aip.web.resolve_href(href: string | nil, base_url: string): string | nil
```

### aip.uuid - UUID Generation

```typescript
aip.uuid.new(): string // Alias for new_v4().
aip.uuid.new_v4(): string
aip.uuid.new_v7(): string
aip.uuid.new_v4_b64(): string
aip.uuid.new_v4_b64u(): string
aip.uuid.new_v4_b58(): string
aip.uuid.new_v7_b64(): string
aip.uuid.new_v7_b64u(): string
aip.uuid.new_v7_b58(): string
aip.uuid.to_time_epoch_ms(value: string | nil): integer | nil
```

### aip.hash - Hashing

```typescript
aip.hash.sha256(input: string): string
aip.hash.sha256_b58(input: string): string
aip.hash.sha256_b64(input: string): string
aip.hash.sha256_b64u(input: string): string
aip.hash.sha512(input: string): string
aip.hash.sha512_b58(input: string): string
aip.hash.sha512_b64(input: string): string
aip.hash.sha512_b64u(input: string): string
aip.hash.blake3(input: string): string
aip.hash.blake3_b58(input: string): string
aip.hash.blake3_b64(input: string): string
aip.hash.blake3_b64u(input: string): string
```

### aip.udiffx - Multi-File Changes

```typescript
aip.udiffx.apply_file_changes(content: string, base_dir?: string, options?: {extrude?: "content"}): ChangesInfo | (ChangesInfo, string) // Applies <FILE_CHANGES> envelope. base_dir defaults to workspace. If options provided, base_dir MUST be explicitly passed (can be nil).
aip.udiffx.load_files_context(include_globs: string | string[], options?: {base_dir?: string, absolute?: boolean}): string | nil
aip.udiffx.file_changes_instruction(): string
```

### aip.time - Time Utilities

```typescript
aip.time.now_iso_utc(): string
aip.time.now_iso_local(): string
aip.time.now_iso_utc_micro(): string
aip.time.now_iso_local_micro(): string
aip.time.now_utc_micro(): integer
aip.time.today_utc(): string
aip.time.today_local(): string
aip.time.today_iso_utc(): string
aip.time.today_iso_local(): string
aip.time.weekday_utc(): string
aip.time.weekday_local(): string
aip.time.local_tz_id(): string
```

### aip.lua - Lua Helpers

```typescript
aip.lua.dump(value: any): string // Stringifies Lua value for debugging.
aip.lua.merge(target: table, ...objs: table | nil): table // Shallow merge into target (mutates target in-place, returns target). nil/null are ignored. target cannot be nil/null.
aip.lua.merge_deep(target: table, ...objs: table | nil): table // Deep merge into target (mutates target in-place, return targets as well). nil/null are ignored. target cannot be nil/null.
```

### aip.pdf - PDF Utilities

```typescript
aip.pdf.page_count(path: string): number
aip.pdf.split_pages(path: string, dest_dir?: string): FileInfo[] // dest_dir default: [stem]/ in source dir.
```

### aip.csv - CSV Parsing and Formatting

```typescript
aip.csv.parse_row(row: string, options?: CsvOptions): string[] // Parses single CSV line.
aip.csv.parse(content: string, options?: CsvOptions): CsvContent // Parses full CSV string.
aip.csv.values_to_row(values: any[], options?: CsvOptions): string // Encodes list of values to CSV line.
aip.csv.value_lists_to_rows(value_lists: any[][], options?: CsvOptions): string[] // Encodes matrix to CSV lines.
```

### aip.hbs - Handlebars Rendering

```typescript
aip.hbs.render(content: string, data: any): string | {error: string} // Renders Handlebars template with Lua data.
```

### aip.agent - Agent Chaining

```typescript
aip.agent.run(agent_name: string, options?: {input?: any, inputs?: any[], options?: table, agent_base_dir?: string}): any
aip.agent.extract_options(value: any): table | nil
```

### aip.run & aip.task - Metadata/Pinning

```typescript
// aip.run (Requires CTX.RUN_UID). Global for the entire agent run.
aip.run.set_label(label: string)
aip.run.pin(iden: string, content: string | {label?: string, content: string})
aip.run.pin(iden: string, priority: number, content: string | {label?: string, content: string})

// aip.task (Requires CTX.RUN_UID and CTX.TASK_UID). Specific to current input task.
aip.task.set_label(label: string)
aip.task.pin(iden: string, content: string | {label?: string, content: string})
aip.task.pin(iden: string, priority: number, content: string | {label?: string, content: string})
```

### aip.cmd - System Commands

```typescript
aip.cmd.exec(cmd_name: string, args?: string | string[]): CmdResponse | {error: string, stdout?: string, stderr?: string, exit?: number} // args can be single string or list of strings.
```

### aip.semver - Semantic Versioning

```typescript
aip.semver.compare(version1: string, operator: string, version2: string): boolean | {error: string}
aip.semver.parse(version: string): {major: number, minor: number, patch: number, prerelease: string | nil, build: string | nil} | {error: string}
aip.semver.is_prerelease(version: string): boolean | {error: string}
aip.semver.valid(version: string): boolean
```

### aip.rust - Rust Code Processing

```typescript
aip.rust.prune_to_declarations(code: string): string | {error: string} // Removes function bodies (replacing with { ... }).
```

### aip.html - HTML Processing

```typescript
aip.html.slim(html_content: string): string | {error: string}
aip.html.select(html_content: string, selectors: string | string[]): Elem[]
aip.html.to_md(html_content: string): string | {error: string} // HTML to Markdown conversion.
```

### aip.git - Git Operations

```typescript
aip.git.restore(path: string): string | {error: string, stdout?: string, stderr?: string, exit?: number} // git restore <path>.
```

### aip.code - Code Utilities

```typescript
aip.code.comment_line(lang_ext: string, comment_content: string): string | {error: string}
```

### aip.shape - Record Shaping Utilities

```typescript
aip.shape.to_record(names: string[], values: any[]): table // Map names to values.
aip.shape.to_records(names: string[], rows: any[][]): object[] // Matrix to list of records.
aip.shape.record_to_values(record: table, names?: string[]): any[] // If names omitted, sorted alphabetically. Missing keys become NA.
aip.shape.records_to_value_lists(records: object[], names: string[]): any[][] // Records to matrix. Uses null sentinel for missing keys.
aip.shape.columnar_to_records(cols: { [string]: any[] }): object[] // Column-oriented to row-oriented.
aip.shape.records_to_columnar(recs: object[]): { [string]: any[] } // Row-oriented to column-oriented. Intersection of keys only.
aip.shape.select_keys(rec: table, keys: string[]): table
aip.shape.omit_keys(rec: table, keys: string[]): table
aip.shape.remove_keys(rec: table, keys: string[]): integer
aip.shape.extract_keys(rec: table, keys: string[]): table
```

## 5. Context Constants (`CTX`)

Injected into all Lua stages, providing execution environment information. All paths are absolute.

| Key                            | Description                                                               |
|--------------------------------|---------------------------------------------------------------------------|
| CTX.WORKSPACE_DIR              | Absolute path to the workspace directory (parent of `.aipack/`).          |
| CTX.BASE_AIPACK_DIR            | Absolute path to the user's base AIPack directory (`~/.aipack-base`).     |
| CTX.AGENT_NAME                 | Name or path used to invoke the agent (e.g., `my_pack/my-agent`).         |
| CTX.AGENT_FILE_PATH            | Absolute path to the resolved agent `.aip` file.                          |
| CTX.AGENT_FILE_DIR             | Absolute path to the directory containing the agent file.                 |
| CTX.TMP_DIR                    | Temporary directory for this session (`.aipack/.sessions/_uid_/tmp`).     |
| CTX.SESSION_UID                | The Session Unique ID.                                                    |
| CTX.RUN_UID                    | The Run Unique ID.                                                        |
| CTX.RUN_NUM                    | 1-based sequence number of the current agent run in the session.          |
| CTX.TASK_UID                   | The Task Unique ID (only available during per-input stages: `# Data`, `# Output`). |
| CTX.TASK_NUM                   | 1-based sequence number of the current task in the run.                   |
| CTX.PACK_IDENTITY              | Pack identity (`namespace@name`) (nil if not run via pack reference).     |
| CTX.PACK_WORKSPACE_SUPPORT_DIR | Workspace support directory for the pack (if applicable).                 |
| CTX.PACK_BASE_SUPPORT_DIR      | Base support directory for the pack (if applicable).                      |
