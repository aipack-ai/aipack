# API Documentation

The `aip` top module provides a comprehensive set of functions for interacting with files, paths, text, markdown, JSON, web services, Lua value inspection, agent control flow, command execution, semantic versioning, Handlebars templating, code formatting, Git, Rust code processing, and HTML processing within the AIPack environment.

[Getting Started Video Tutorial](https://news.aipack.ai/p/aipack-tutorial-from-hello-world)

[AIPack Lab Repo](https://github.com/aipack-ai/aipack-lab)

#### The available submodules are:

- [`aip.file`](#aipfile): File system operations (load, save, list, append, JSON/MD/HTML handling).
- [`aip.editor`](#aipeditor): Functions for opening files in external editors.
- [`aip.path`](#aippath): Path manipulation and checking (split, resolve, exists, diff, parent).
- [`aip.text`](#aiptext): Text processing utilities (trim, split, split lines, replace, truncate, escape, ensure).
- [`aip.tag`](#aiptag): Custom tag block extraction (e.g., `<TAG>...</TAG>`).
- [`aip.md`](#aipmd): Markdown processing (extract blocks, extract metadata).
- [`aip.json`](#aipjson): JSON parsing and stringification.
- [`aip.toml`](#aiptoml): TOML parsing and stringification helpers.
- [`aip.yaml`](#aipyaml): YAML parsing and stringification.
- [`aip.web`](#aipweb): HTTP requests (GET, POST), URL parsing and resolution.
- [`aip.uuid`](#aipuuid): UUID generation and conversion.
- [`aip.hash`](#aiphash): Hashing utilities (SHA256, SHA512, Blake3) with various encodings.
- [`aip.lua`](#aiplua): Some lua helpers (for now only `.dump(data)`).
- [`aip.agent`](#aipagent): Running other AIPack agents.
- [`aip.run`](#aiprun): Run-level helpers (set label, attach pins to the current run).
- [`aip.task`](#aiptask): Task-level helpers (set label, attach pins to the current task).
- [`aip.flow`](#aipflow): Controlling agent execution flow.
- [`aip.cmd`](#aipcmd): Executing system commands.
- [`aip.semver`](#aipsemver): Semantic versioning operations.
- [`aip.rust`](#aiprust): Rust code specific processing.
- [`aip.html`](#aiphtml): HTML processing utilities.
- [`aip.git`](#aipgit): Basic Git operations.
- [`aip.hbs`](#aiphbs): Handlebars template rendering.
- [`aip.code`](#aipcode): Code commenting utilities.
- [`aip.time`](#aiptime): Time and date utilities (now, parse/format, epoch conversions).
- [`aip.shape`](#aipshape): Record shaping utilities (rows and columns, key selection/extraction).
- [`aip.csv`](#aipcsv): CSV parsing and processing utilities.
- [`aip.pdf`](#aippdf): PDF file utilities (page count, split pages).
- [`aip.udiffx`](#aipudiffx): Applying multi-file changes (New, Patch, Rename, Delete).

#### File Path supported

AIPack supports several types of file paths:

| Type                   | Example                                  | Notes                                                                                                   |
|------------------------|------------------------------------------|---------------------------------------------------------------------------------------------------------|
| Relative               | `some/file.txt`                          | Relative to the workspace directory                                                                     |
| Absolute               | `/absolute/path/file.txt`                | Absolute path (`C:/` on Windows)                                                                        |
| Pack Ref               | `my_org@my_pack/path/file.txt`           | Finds the closest pack (in custom workspace, custom base, or install base) and uses this as a directory |
| Home Tilde             | `~/path/to/file.txt`                     | User home directory; `~` is replaced by the home directory (or `/` if no home directory is found)       |
| Session TMP            | `$tmp/some/file.txt`                     | Located in `.aipack/.sessions/_uid_/` within the workspace; unique per session (until command stops)    |
| Workspace Pack Support | `my_org@my_pack$workspace/some/file.txt` | Maps to `.aipack/support/pack/my_org/my_pack/some/file.txt` in the workspace                            |
| Base Pack Support      | `my_org@my_pack$base/some/file.txt`      | Maps to `.aipack-base/support/pack/my_org/my_pack/some/file.txt` in the base directory                  |

Important notes:

- The workspace directory is the parent directory of the `.aipack/` folder. Like `.git` or `.vscode`, the `.aipack/` folder marks a directory as the workspace.
- The pack support directory suffixes `$workspace` and `$base` must appear immediately after the pack name.
- These paths can be used in globs (e.g., `aip.file.list({"~/path/to/**/*.md", "pro@rust10x/guide/**/*.md"})`).



#### Common Data Types:

- [`FileInfo`](#fileinfo) (for `aip.file..`) (FileInfo + `.content`)
- [`FileRecord`](#filerecord) (for `aip.file..`)
- [`FileStats`](#filestats) (for `aip.file..`)
- [`WebResponse`](#webresponse) (for `aip.web..`)
- [`WebOptions`](#weboptions) (for `aip.web..`)
- [`MdSection`](#mdsection) (for `aip.md..`)
- [`MdBlock`](#mdblock) (for `aip.md..`)
- [`MdRef`](#mdref) (for `aip.md..`)
- [`TagElem`](#tagelem) (for `aip.tag..`)
- [`ApplyChangesStatus`](#applychangesstatus) (for `aip.udiffx..`)
- [`CmdResponse`](#cmdresponse) (for `aip.cmd..`)
- [`DestOptions`](#destoptions) (for `aip.file.save_...to_...(src_path, dest))`)
- [`SaveOptions`](#saveoptions) (for `aip.file.save(...)`)
- [`CsvOptions`](#csvoptions) (for `aip.csv..` and `aip.file..csv..`)
- [`CsvContent`](#csvcontent) (for `aip.file.load_csv`)
- [`YamlDocs`](#yamldocs) (for `aip.file.load_yaml` and `aip.yaml.parse`)
- [`Marker`](#marker) (for `aip.task.pin` and `aip.run.pin`)


#### AI Response

An `ai_response` variable will be injected into the scope in the `# Output` Lua code block if an instruction was given and an AI request occurred (otherwise, it will be `nil`).

```ts
{
  // The final text response from the AI, if available.
  content?: string,
  // A formatted string capturing essential details like usage, price, model, and duration of the request, using the fields below.
  info: string,
  // e.g., `gpt-5-mini`
  model_name: string,
  // e.g., `openai`
  adapter_kind: AdapterKind,
  // Token usage details.
  usage: {
    prompt_tokens: number,
    completion_tokens: number
  },
  // The approximate price in USD, if available.
  price_usd?: number,
  // Duration in seconds (with millisecond precision).
  duration_sec: number,
  // Reasoning content, if available (e.g., from deepseek or some groq models).
  reasoning_content?: string,
}
```

#### Global and Injected Variables:

- All stage Lua code blocks and required scripts receive a [`CTX`](#ctx) variable containing context information (e.g., `CTX.AGENT_NAME`, `CTX.TMP_DIR`, etc.).
- All stage Lua code blocks also receive `options`, which includes `.model` and `.input_concurrency`.
- `# Before All` stage Lua code blocks receive `inputs` (can be `nil` if no inputs are given).
- `# Data` stage Lua code blocks receive `input` (can be `nil` if no input is provided) and the return value from the `# Before All` stage (`before_all`, which can be `nil`).
- `# Output` stage Lua code blocks receive `input`, `ai_response` (can be `nil`), `data` (the return value from the `# Data` stage), and `before_all`.
- `# After All` stage Lua code blocks receive `outputs` (return values from each `# Output` stage), `inputs`, and `before_all`.

**NOTE**

> All of the type documentation is noted in "TypeScript style" as it is a common and concise type notation for scripting languages and works well to express Lua types.
>       However, it is important to note that there is no TypeScript support, just standard Lua. For example, Lua properties are delimited with `=` and not `:',
>       and arrays and dictionaries are denoted with `{ }`.


#### nil vs. null

Lua has the `nil` keyword which partially acts like a common `null` but not exactly.

 For that reason, AIPack also adds the global concept of `null` (also aliased as `Null` and `NULL`) that behaves closer to JSON, JS, SQL, and other nulls.

Here are the key differences:

**`nil`**

- Native to `Lua`
- Means no value or property does not exist.
    - Limitation: We have no way to know if the property had a "null" value or just was not there.
- When put in an array `{"one", "two", nil, "four"}` this will actually stop the iterator (i.e., `ipairs`) on the first nil. For example:
```lua
local values = {"one", "two", nil, "four"}
for i, v in ipairs(values) do
    print("" .. i .. ": " .. v)
end
-- Will print: "1: one", "2: two"
-- NOTE: We won't see `4: four` because the iterator stop at first nil
```

**`null`**

 Added by `AIPack` to all Lua contexts, with keyword `null`. (It is also aliased as `Null` and `NULL` for convenience).
- Behaves like a JavaScript null, and can be used in variables, property values, and array items
- In Array
```lua
local values = {"one", "two", null, "four"}
for i, v in ipairs(values) do
    print("" .. i .. ": " .. v)
end
-- Will print: "1: one", "2: two", "3: null", "4: four"
```
- Works in objects as well, and when converting to JSON
```lua
local contact = {
    name: "Jen",
    home: null,
    phone: nil, -- NOTE the lua "nil" here
    title: "Director"
}
print(contact)
-- Will print: {home = null,name = "Jen",title = "Director"}
-- Note: "home" is present with null, but phone is not

local contact_json = aip.json.stringify(contact)
print(contact_json)
-- Will print: {"home":null,"name":"Jen","title":"Director"}
-- Note: Similar to the lua print.
```

**When to use `nil` vs. `null`**

- Use `null`
    - In arrays, use `null` over `nil` since the Lua `nil` will have some unexpected side effect, it stops iterators
    - In object property values when wanting to keep the property name when the value is null. Using `nil`, the property will be virtually "erased."
- Use `nil`
    - When using simple variable initialization, e.g., `local origin_path = nil`
    - In object property values when it's okay to not preserve the property name when it's nil. 

So, if you're still not sure, use `null` in arrays, and you can use either `nil` or `null` in other scenarios.

**Important Comparison Note**

In Lua, `nil` and the `null` sentinel are different types. A simple `val == nil` check will return `false` if `val` is `null`. Always use the [Global Null Helpers](#global-null-helpers) (`is_null`, `nil_if_null`, `value_or`) to safely compare or handle these values.


### Global Null Helpers

AIPack provides several global functions to safely handle both `nil` and `null` values.

**`is_null(value)`**

Returns `true` if the `value` is either `nil` or the `null` sentinel.

```lua
is_null(nil)   -- true
is_null(null)  -- true
is_null(false) -- false
```

**`nil_if_null(value)`**

Returns `nil` if the `value` is `null` or `nil`. Otherwise, returns the value itself. This is useful when calling functions that do not support the `null` sentinel.

```lua
local v = nil_if_null(null) -- v is nil
```

**`value_or(value, alt)`**

Returns `value` if it is not `nil` and not `null`. Otherwise, returns `alt`.

```lua
local name = value_or(input.name, "Anonymous")
```