`.` minor | `-` Fix | `+` Addition | `^` improvement | `!` Change | `*` important | `>` Refactor

## 2026-05-06 [v0.8.24](https://github.com/jeremychone/rust-devai/compare/v0.8.23...v0.8.24)

- `-` tui - Fix overview link zones
- `^` tui - Auto-switch to Overview tab if no tasks after 500ms
- `.` genai -  with Bedrock support
- `.` udiffx - dependencies alignment, emphasis file_append

## 2026-04-29 [v0.8.23](https://github.com/jeremychone/rust-devai/compare/v0.8.22...v0.8.23)

- `.` MODEL - `gpt` alias to `gpt-5.5`
- `.` MODEL - deepseek ai - deepseek-v4-flash/pro pricing and aliases for dflash, dpro
- `.` MODEL - fireworks - pricing update deepseek v4-pro, kimi 2.6, gpt-5.5-pro
- `^` tui - expand task completion header width, align task cache info with duration, and show full task header for nested runs
- `^` term - restore tmux title on quit
- `-` model/pin - fix run pin lookup returning task pins with same iden
- `.` udiffx - 0.1.41 - hunk error cause concise

## 2026-04-24 [v0.8.22](https://github.com/jeremychone/rust-devai/compare/v0.8.21...v0.8.22)

- `^` NEW API `aip.zip` - `aip.zip.create`, `aip.zip.extract`, and `aip.zip.list` 
- `+` `aip.file` - add workspace-safe `ensure_dir` directory creation
- `^` TUI - Big TUI perf improvement when many (3k+ tasks)
- `^` TUI - Expand only selected top run
- `!` flow redo rename redo API to `redo_run`
- `^` udiffx - include `error_hunks`, ignore no-op hunks, normalize single-hunk errors, and more resilience
- `^` perf - use batch create for task and related entities
- `^` tui - detect multi-dot js config file paths in help text
- `>` internal - data/model/task event cleanup
- `.` Model pricing / model aliases - gpt 5.4, opus 4.7 
- `.` Model pricing - gpt 5.5 (not aliased yet as not available as API) 

## 2026-03-25 [v0.8.21](https://github.com/jeremychone/rust-devai/compare/v0.8.20...v0.8.21)

- `+` unpack - add unpack command specification
- `^` aip.udiffx - include error_hunks data
- `^` tui - detect multi-dot js config file paths in help text
- `.` docs - update udiffx `ApplyChangesItem` error_hunks serialization docs
- `.` doc/udiffx - update ApplyChangesStatus and error_hunks

## 2026-03-19 [v0.8.20](https://github.com/jeremychone/rust-devai/compare/v0.8.19...v0.8.20)

- `^` model - gpt-5.4 mini/nano pricing update and aliases
- `^` model suffix - add support for `-xhigh` and `-max` (with property fallback)
- `-` tui text_helpers - Fix file path highlighting for folder-less filenames, add `~` support, and fix false positives
- `^` udiffx
  - add new `~` patch command, and fix error when partial hunks fail
  - fix wrong hunk order in patches
  - add `FILE_COPY` directive
  - fully ignore surround-only hunks (no operations hunks), add `error_hunks` in report, and apply possible hunks per file
  - handle wrapper text (e.g. `*** End Patch`) and ignore unactionable hunks
  - update to `0.6.0-beta.5`, add Anthropic adaptive thinking and `-max`
- `>` memory optimization - ucontent - Now pins share common content.
- `>` genai - Now genai points GPT-5 to the OpenAI Responses API endpoint

## 2026-03-08 [v0.8.19](https://github.com/jeremychone/rust-devai/compare/v0.8.18...v0.8.19)

- `^` udiffx - crlf handling, lot of resiliency tuning in file patch
- `&` udiffx - file_patch fix when append with file_append directive (simpler, more accurate when just appending)
- `^` aip.file - code clean, enforce file write on all file_append
- `>` file write manager - now have ArcSwap and auto cleanup. Memory optimization
- `.` pricing / aliases - gpt 5.4

## 2026-03-05 [v0.8.18](https://github.com/jeremychone/rust-devai/compare/v0.8.17...v0.8.18)

- `^` API - FileRecord & FileInfo now have `.is_likely_text`
- `^` aip file append - now fully concurrent proof (atomic file append per file)
- `^` run - report task errors when allow_run_on_task_fail is enabled (#68)
- `^` Model Pricing & Alias default updated - gpt 5.3, gemini-3.1-flash-lite-preview
- `^` udiffx - 0.1.19 - white line drifting
- `.` genai 0.6.0-beta.3
- `+` aip_path - Add aip.path.sort_by_globs Lua function with tests
- `.` lua engine - include pack ref base dir when present as well as the file agent dir (for lua includes)

## 2026-02-23 - [v0.8.17](https://github.com/jeremychone/rust-devai/compare/v0.8.16...v0.8.17)

- `+` aip.udiffx - add `aip.udiffx.load_files_context(..)`, `aip.udiffx.file_changes_instruction()`
- `-` fix nested task/run freeze on core limit
- `^` udiffx - more resilient patches
- `^` run - Add 'allow_run_on_task_fail' option to control run behavior on task failure (#67) (thanks to [savi0ur](https://github.com/pratraut))
- `.` aip.flow.redo() - first pass at the redo (not official API yet)
- `.` tui - Infra for config popup (disabled for now)
- `.` tui - Update spec-code-tui.md to match current TUI implementation
- `.` doc-for-llm - format
- `.` udiffx - v0.1.17 (more resilience)
- `.` config-default & pricing - sonnet-4-6, gemini 3.1
- `.` doc - update doc-for-llm

## 2026-02-06 - [v0.8.16](https://github.com/jeremychone/rust-devai/compare/v0.8.15...v0.8.16)

- `^` Anthropic - add `opus-4-6 `pricing and default for `opus` alias

## 2026-02-04 - [v0.8.15](https://github.com/jeremychone/rust-devai/compare/v0.8.14...v0.8.15)

- `+` aip.file - Add `.copy(..)` and `.move(..)`
- `^` api.lua.merge.. - now accept Value::Nil and Value::NULL as objs (will be ignored)
- `!` Now using lua Value::NULL (removed custom NullSentinel) and now is_null(v), value_or(v,alt),  and nil_if_null(v)
- `^` agent - Add .input support to aip.agent.run options
- `^` aip.agent.run - Add agent_base_dir option to aip.agent.run
- `>` aip.agent.run - refactoring to type RunAgentOptions
- `-` aip.agent.run - fix error reporting issue when fail during prep run (find agent)
- `>` types - add AgentRunResponse into types::
- `^` core@doc - update for-llm
- `.` config - update codex to gpt-5.2-codex
- `.` doc - Update documentation with missing and corrected types and fix MdSection typo
- `.` add first dev/agents/docaify
- `>` code clean - remove all .unwrap() (should not have been)
- `-` fix concurrency db transaction issue


## 2026-01-19 - [v0.8.14](https://github.com/jeremychone/rust-devai/compare/v0.8.13...v0.8.14)

- `^` udiffx - many updates (more resilient)
- `^` install - Skip installation if pack is already up to date and add --force flag
- `^` init-base - now update built-in packs only if needed
- `^` packer - Implement safer cleanup during pack installation using trash
- `-` error - make sure that error in 'aip install' quit the term
- `>` support/files - Refactor safer deletes to use simple-fs trash API

## 2026-01-14 - [v0.8.13](https://github.com/jeremychone/rust-devai/compare/v0.8.12...v0.8.13)

- `^` aip.udiffx - update dependency. More resilient uniffied patch matching.

## 2026-01-05 - [v0.8.12](https://github.com/jeremychone/rust-devai/compare/v0.8.11...v0.8.12)

- `+` NEW API - aip.file - Implement `aip.file.load_yaml`, `aip.yaml.parse/stringify` for Lua
- `+` NEW API - `aip.udiffx.apply_file_changes`
- `+` NEW API - Add aip.editor.open_file Lua function (with terminal specified editor)
- `+` tui - File Path click to open in editor (with new editor support logic)
- `^` tui - Add Shift+Arrow support for scroll to top/bottom (for term that supports mod on shift)
- `^` tui - Add keyboard scroll support (Up/Down arrows) via normalized ActionEvent
- `+` cli - Add `aip create-gitignore` to create a default/best practice Add --force option to create-gitignore command
- `+` cli - `aip run ...` pack auto install when not present
- `^` cli - run - now -o use the editor logic to open file
- `^` CTX - added CTX.RUN_NUM and CTX.TASK_NUM
- `^` term - tmux - set agent name as pane name (not the window name)
- `^` term - add support for wezterm (set title)
- `^` doc - Add # Options stage documentation and precedence rules
- `^` doc - Added readme-for-llm.md for concise AIP framework overview
- `-` agent - demo@proof - fix when content is empty
- `-` list files - correctly ignore .git/.., node_modules/ in globs (by default)
- `.` list files - add `.aipack/` as the default folder excludes (for when list **/*, can be put expclictly to include)
- `.` demo@proof - disable the open vscode
- `.` config-default - flash to flash-3-preview
- `.` pricing - add gemini-3-flash pricing

## 2025-12-13 - [v0.8.11](https://github.com/jeremychone/rust-devai/compare/v0.8.10...v0.8.11)

- `!` init-base - now delete the builtin installed packs before installing (for consistency, use to trash for recovery)
- `!` aip.file.delete - now use `to_trash` to enable recovery.
- `+` aip.pdf - `.page_count(path)` and `.split_pages(path, dest_dir?)`
- `^` model - support the -none suffix now (mostly for >= gpt 5.1
- `^` tui - update prompt/compl layout with ellipsis
- `*` core@doc - now has the for-llm/ and standard/ structure
- `-` aip.file.list - fix issue when listting `**/..` and before was a non existent start folder glob

## 2025-11-28 - [v0.8.10](https://github.com/jeremychone/rust-devai/compare/v0.8.9...v0.8.10)

- `+` BIG ONE - file attachment support - on a `aip.flow.data_response(..., {attchments: ...})`
- `+` aip.task.set_label - new api to set task label
- `+` aip.runs.set_label - new api to set run label
- `+` aip.md.extract_refs - news api to extract markdown ref/links
- `-` self update on nix - fix has_aip_in_path function

## 2025-11-25 - [v0.8.9](https://github.com/jeremychone/rust-devai/compare/v0.8.8...v0.8.9)

- `^` aip.lua.merge - support multi object merge and merge_deep (& merge on first target object now)
- `^` Opus-4-5 - Added support for new reasonning effort config
- `^` Opus-4-5 - Updated pricing
- `^` gemini 3 - Use new `thinkingLevel` when gemini-3.. for `-low` and `-high`

## 2025-11-23 - [v0.8.8](https://github.com/jeremychone/rust-devai/compare/v0.8.7...v0.8.8)

- `!` api change - csv - now returns `{headers?, rows}` (was `{headers, content}`)
- `!` aip.file.load_spans - return tuple spans `[start, end]` rather than `{start: number, end: number}`
- `+` NEW API - `aip.lua.merge` and `aip.lua.merge_deep`
- `+` NEW API - `aip.file` .. `.save_as_csv`, `.append_csv_row`, `.append_csv_rows`
- `+` NEW API - `aip.csv.value_lists_to_rows`, `aip.csv.values_to_row`
- `+` NEW API - `aip.shape.records_to_value_lists`
- `+` NEW API - `aip.toml.parse`, `aip.file.load_toml`, `aip.toml.stringify`
- `+` NEW API Argument - Add `CsvOptions` to `aip.csv.values_to_row` and `value_lists_to_rows`
- `^` `aip.file.apply_changes` - now return two values `(FileInfo, ChangesInfo)`.
- `+` tui - display model pricing while AI running
- `^` pricing - gemini 3, openai gpt 5.1, fireworks pricing update
- `-` tui - runs nav view - fix issue when clicking nav on no item, last item get selected

## 2025-10-25 - [v0.8.7](https://github.com/jeremychone/rust-devai/compare/v0.8.5...v0.8.7)

- `^` aip.file.list - big speed boost and special folder support for `target/`, `.build/`, `node_modules/` and `__pycache__/` (but have explicit globs)
- `+` aip.web - `.get` and `.post` now support `.headers`, `.user_agent`, and `redirect_limit` in options, and return `.headers` in WebResponse
- `!` aip.web.post - and `...get` now accept `{parse = true}` as an option to parse JSON content when the content type is JSON (otherwise returns a string)
- `^` aip.web - added WebOptions (user_agent, headers, redirect_limit)
- `-` self setup - Unix - create `.zshenv` or `.bashrc` if the user selects Y and no aip is in the path
- `.` windows - prevent potential end term debug print
- `.` config-default - added haiku 4.5 to default aliases
- `.` pricing - updated Anthropic pricing with haiku 4.5

## 2025-10-13 - [v0.8.5](https://github.com/jeremychone/rust-devai/compare/v0.8.4...v0.8.5)

- `^` aip.tag.extract - Add support for tag attributes

## 2025-10-12 - [v0.8.4](https://github.com/jeremychone/rust-devai/compare/v0.8.3...v0.8.4)

- `+` New API - aip.tag - `aip.tag.extract(content, tag_names, options?)`, `.extact_as_map`, `.extact_as_multi_map`
- `+` tui - add 'x' to cancel run (without quitting)
- `^` tui - added prompt size display in task view while AI processing
- `^` tui - enable copy-to-clipboard click for task_view AI section
- `^` tui - ai step now show provider model name when different
- `^` tui - action view clickable (e.g., `[r] Replay`, `[x] Cancel Run`, ...)
- `^` aip.file.save - add new options for trim_start, trim_end, single_trailing_newline
- `^` tui - added cumulative task durations for all run summary bar
- `-` windows - check the crlf issue of apply_changes (is no match, then normalize to lf)
- `-` tui - fix AI state display when canceled
- `!` aip.text - renamed to 'ensure_single_trailing_newline' from 'ensure_single_ending_newline'
- `.` lua.html.select - now element content are trimmed, and .attrs is present only when not empty
- `.` lua file - error when globs starts with ../
- `^` aip.file.list - now always get/return meta (size, ctime, mtime) when found

## 2025-10-03 - [v0.8.3](https://github.com/jeremychone/rust-devai/compare/v0.8.2...v0.8.3)

- `.` pricing - update pricing for gemini..-latest
- `-` lua_value_to_serde_value - made that LightUserData get json seralized as null
- `-` windows - fix Y setup prompt
- `.` config - update the flash/lite aliases with -latest
- `.` config-default - update the gpt mini/nano names
- `+` model aliases - add support for reasoning suffixes, -low, -high, ..
- `^` anthropic models - add support for reasoning (genai 0.4.1)
- `.` doc - lua-apis fix for site render

## 2025-09-29 - [v0.8.2](https://github.com/jeremychone/rust-devai/compare/v0.8.1...v0.8.2)

- `!` globs - now does not ignore target/ and node_modules/
- `+` gpt-5-codex - Add transparent support for OpenAI gpt-5-codex model
- `+` api.csv... - add parse_row(row, opts), parse(content,opts)
- `+` aip.file...spans - new aip.file.line_spans/csv_spans(path)
- `^` aip.file.load_csv - add CsvOptions
- `.` aip.csv.parse - return to match {headers, content}
- `.` openai - add support for gpt-5-codex (pricing & genai 0.4.0-alpha.17)
- `.` init - update config-default claude aliases

## 2025-08-31 - [v0.8.1](https://github.com/jeremychone/rust-devai/compare/v0.8.0...v0.8.1)

- `+` **NEW API** - New `aip.shape...` apis to reshape data.
- `^` **NEW API** - add aip.time.local_tz_id() e.g., 'America/Los_Angeles'
- `+` **NEW LUA** - Added support for `null`
- `!` demo packs - update `demo@proof` (and remove the demo@craft)
- `^` perf - implement batch create task for TUI perf with 500+ tasks
- `^` task limit (for TUI) - increase default max select limit to 12,000 (from 300)
- `.` config - Update to latest gpt and gemini models
- `+` **NEW API** - aip.time.today_utc/local return weekday + today_iso, aip.time.weekday_utc/local
- `^` tui - modifier + up should not trigger copy to clipboard
- `-` tui - Many fixes
- `-` aip.agent.extract_options - fix input_concurrency typo
- `.` aip - remove end "happy coding" exit message

## 2025-08-14 - [v0.8.0](https://github.com/jeremychone/rust-devai/compare/v0.7.20...v0.8.0)

- **TUI BY DEFAULT** (`--old-term` for the old terminal, and still `-s` for single shot)
- `+` aip.file.load_html_as_md
- `+` aip.file.load_html_as_slim
- `+` aip.time.now\_.. - new time APIs
- `^` aip.file.save_html_to_md - add the {slim: bool}, slim: true by default
- `.` file_save_html_to_md - slim by default
- `+` aip.file.delete - only in workspace dir
- `.` lua-api - fix markdown format

## 2025-08-12 - [v0.7.20](https://github.com/jeremychone/rust-devai/compare/v0.7.19...v0.7.20)

- `+` BIG ONE, tui, Copy to clipboard sections
- `+` aip.file.load_docx_as_md(...)
- `+` aip.file.save_docx_to_md, first pass
- `^` aip.file.list.., now sort by globs (end-weighted)
- `.` pricing, update Anthropic, GPT
- `.` tui, timed popup view gets removed even if user/data event
- `.` genai to 0.4.0-alpha-12 for minimal support for GPT-5...
- `.` many TUI tune-ups
- `.` config default, TOML, GPT-5 only

## 2025-08-08 - [v0.7.19](https://github.com/jeremychone/rust-devai/compare/v0.7.18...v0.7.19)

- `^` pricing - add gpt-5, gpt-5-mini, gpt-5-nano pricing
- `^` pricing - add GPT-OSS models (Fireworks) and include in config-default.toml
- `^` pricing - update Together AI and OpenAI GPT-5 pricing; refresh Anthropic and Groq; update pricing-all.json and pricing/data.rs
- `.` pricing - update Fireworks GPT-OSS
- `^` `aip.text.format_size` - add `{trim = true}`
- `^` inputs - FileInfo and FileRecord now default \_display to path when not provided
- `^` types - implement custom serializer to include \_type metadata, types like FileInfo, FileRecord,
- `.` lua-apis - update aip.run/task pin APIs
- `.` keys - add Nebius API key to known systems

## 2025-08-04 - [v0.7.18](https://github.com/jeremychone/rust-devai/compare/v0.7.17...v0.7.18)

- `!` base config tomls - now have `~/.aipack-base/config-default.toml` and `~/.aipack-base/config-user.toml`
  - Old `~/.aipack-base/config.toml` will be renamed to `~/.aipack-base/config-deprecated.toml` and won't be used (can be deleted)
  - Default model aliases updated.
- `+` NEW Lua API - `aip.file.info(file_path): FileInfo` to get the FileInfo of a give file.
- `+` NEW Lua API - `aip.file.stats(globs): FileStats` (`{total_size, number_of_files, ctime_first, ...}`)
- `+` NEW genai provider - Now support fireworks.ai (and its pricing)
  - namespaced like `fireworks::glm-4p5` or full fireworks name like `accounts/fireworks/models/glm-4p5`
- `+` NEW genai provider - Now support together.ai (no pricing yet).
  - namespaced like `together::together::Qwen/Qwen3-235B-A22B-Instruct-2507-tput`
- `^` `aip.text.format_size` - add lowest_unit support with `..format_size(2344333, "MB")`
- `-` sys error - implement sys error when tui and error happen before first run
- `-` tui - fix tasks 'no current tasks' state issue when changing sub run
- `-` tui - fix agent run cost not displaying when no sub agents
- `-` model pin - fix the pin run save concurrency issue

## 2025-07-21 - [v0.7.17](https://github.com/jeremychone/rust-devai/compare/v0.7.16...v0.7.17)

- `+` pin - added `aip.run.pin(..)` and `aip.task.pin(..)` with text and first Marker Universal Component `{label, content}`
- `+` tui - nested runs (sub agents)
- `+` aip.text - add aip.text.format_size(..) for formatting bytes (fixed 9 chars format)
- `+` lua - add aip.path.matches_glob(...)
- `^` json - added default support for jsonc (except for newline json)
- `^` lua - aip.file.save_change ..json return FileInfo
- `^` lua - aip.file save, append now returns FileInfo
- `^` tui - overview - add legend to list mode
- `^` tui - run overview - added tasks legend
- `-` before all - allow to set inputs to empty array with inputs = {} on the before_all_response
- `-` lua - fix error reporting (put back the line with error which did not match with mlua 0.11.1)
- `-` run agent - Fix `aip.agent.run(..)`
- `-` tui - fix overview link zones
- `!` ctx.session_uid now (before ctx.session)
- `.` tui - update headers display when no task/model/cost (still need to do parent agent totals)
- `.` tui - overview - tasks legend back at the bottom of list/grid
- `.` tui - now task tab stay selected
- `.` tui - run tab overview when no tasks
- `.` tui - show_runs true by default
- `.` tui sys info memory - toggle on shift + m
- `.` lua print - now when multiple args, join on \n and not \t
- `.` aip json - now empty content ia Value::Nil (does not throw error)
- `.` tui - fix task label prefix
- `.` lua_engine - refactor print to print object now (aip lua dump)
- `.` tui - overview tasks mode - fancy toggle next logic
- `.` tui - truncate model name, queue (rather than waiting)
- `.` tui - run nav view - fix run icon to show run past state

## 2025-07-23 - [v0.7.16](https://github.com/jeremychone/rust-devai/compare/v0.7.15...v0.7.16)

- `-` tui - fix tui crash when resize to very small height
- `-` fix issue when first run is skipped at before all with instruction (used to never show task(s) again)
- `.` turn off debug trace when xp-tui
- `.` core@doc - update # Options (from legacy config)

## 2025-07-23 - [v0.7.15](https://github.com/jeremychone/rust-devai/compare/v0.7.14...v0.7.15)

- `+` **BIG ONE**: New Terminal UI with the `--xp-tui` flag
  - Example: `aip run pro@coder --xp-tui`
  - or `aip run my-agent.aip --xp-tui`
  - This will be the default UI in version `0.8.0`
  - Running without `--xp-tui` will use the previous UI
- `.` Other fixes, pricing & model update (Kimi 2 on groq)

## 2025-06-23 - [v0.7.14](https://github.com/jeremychone/rust-devai/compare/v0.7.13...v0.7.14)

- `-` pricing - fix gemini 2.5 lite pricing (to reflect new pricing)
- `.` update config.toml with gemini 2.5

## 2025-06-12 - [v0.7.13](https://github.com/jeremychone/rust-devai/compare/v0.7.12...v0.7.13)

- `+` `aip.file_file_hash_...` for sha256, sha512, and blake3 (hex, b64, b64u, and b58 encoding)
- `^` doc - lua-apis for the file.hash*... and aip.file.hash*...
- `^` pricing - update new OpenAI o3 pricing
- `.` all 'y' input are now case insensitive and match 'yes' as well
- `.` init - config update to latest gemini 2.5 pro 06-05

## 2025-05-31 - [v0.7.12](https://github.com/jeremychone/rust-devai/compare/v0.7.11...v0.7.12)

- `+` Added `aip.hash` - [See doc](https://aipack.ai/doc/lua-apis#aipuuid) e.g., `aip.hash.blak3_b58(some_text_content)` - Added `aip.hash` lua utilities for sha256, sha512, blake3, with hex, b64, b64u, base 58 serialization
- `+` Added `aip.uuid` - [See doc](https://aipack.ai/doc/lua-apis#aiphash) e.g., `aip.hash.new() -- v4`, `aip.hash.new_v7_b58() -- v7 in base 58`
- `-` aip.file.load - fix base_path issue (was causing issue with pro@coder when base_dir was not empty)

## 2025-05-26 - [v0.7.11](https://github.com/jeremychone/rust-devai/compare/v0.7.10...v0.7.11)

- `+` BIG ONE - on mac/lin now `aip self update` fully update binary!
- `+` Now support `~/` for home dir, and will normalize path with `~/` (to limit Personal Information in prompt)
- `+` `aip.file.save_changes` - big one, now support applying SEARCH/REPLACE pairs as well as whole content (use in `pro@coder` experimental flag)
- `+` `aip.text.split_.._line` - added `split_first_line` and `split_last_line` (more bullet proof way to split a file from delim)
- `+` `aip.text.split_last`
- `+` AgentOptions - added top_p for agent options
- `^` `aip.web.parse_url` - add page_url (no fragment, no query)
- `^` add `aip.file.exits(path)` (same as `aip.path.exists`) make it work with pack ref (when no exists)
- `^` doc - update `core@doc/lua-apis.md` more path information
- `^` AiResponse.info - add temperature when set
- `-` update/fix pricing
- `-` fix gemini cached computation (from genai 0.3.3)
- `.` rename to FileInfo (from FileMeta) - Should not change aips
- `.` update init config.tomls
- `.` craft@text - add for no long dash (more)

## 2025-05-17 - [v0.7.10](https://github.com/jeremychone/rust-devai/compare/v0.7.9...v0.7.10)

- `+` meta block / parametric prompt - added support for json and yaml (on top of toml)
- `!` soft deprecation - now `config.toml` has `[options]` (rather than `[default_options]`). Both still supported during transition.
- `.` message - fix print issue when pressing R while already running

## 2025-05-16 - [v0.7.9](https://github.com/jeremychone/rust-devai/compare/v0.7.7...v0.7.9)

- `+` aip.path.parse - parse a file and return a FileInfo (without size, modified, ...)
- `+` aip.web.resolve_href
- `+` aip.web.parse_url - parse a URL string into a url struct
- `^` pricing - update gemini caching pricing
- `!` aip.text - now accept content strings to be nil (will return nil)
- `+` aip.html.select - added select(html_content, selectors)
- `+` aip.file.join - add back join, simpler and more ergonomic
- `^` lua-apis - add aip.agent.extract_options
- `+` aip.agent.extract_options - from a lua table, only the properties that are agent options
- `^` aip.md.extract_meta - allow content to be nil
- `!` path.join - fully deprecate join (removed). Just add '/' regardless of OS, '\' will be normalized
- `^` error - make better error display
- `-` (v0.7.8) - Fix install issue on nixes.

## 2025-05-08 - [v0.7.7](https://github.com/jeremychone/rust-devai/compare/v0.7.6...v0.7.7)

- `+` NEW session & tmp dir - Added `CTX.SESSION`, `CTX.TMP_DIR`, and `$tmp/some/path.txt` alias
- `+` NEW Base and Workspace support directory for packs with `ns@pack_name$base/some/file.txt` and `ns@pack_name$workspace/some/file.txt`
  - C`TX.PACK_BASE_SUPPORT_DIR` and `CTX.PACK_WORKSPACE_SUPPORT_DIR`
- `+` NEW `aip.lua.file`
  - Added `aip.file.save_html_to_slim(html_file_path, dest?)`,
  - Added `aip.file.save_html_to_md(html_file_path, dest?)`,
  - Added `aip.file.append_json_line(path, data)`
  - Added `aip.file.append_json_lines(path, [data])`
  - Added `aip.file.load_ndjson(path)`
  - Added `aip.file.load_json`
  - `FileInfo` and `FileRecord` now have created, updated, size metadata
- `^` `aip.json`
  - Now `aip.json.stringify(data)` stringify to single line.
  - Added `aip.json.stringify_pretty(data)`
- `+` `aip.html`
  - Added `aip.html.to_md(html_content)`
  - Now `aip.html.slim(full_html)` (from deprecated `.prune_to_content(full_html)`)
- `-` test - fix test_pricing_pricer...
- `.` \_init config.toml update with flash-prev-zero
- `.` Gemini - Added support for `-zero`, `-low`, `-medium`, `-high` suffixes for reasoning budget

## 2025-04-26 - [v0.7.6](https://github.com/jeremychone/rust-devai/compare/v0.7.5...v0.7.6)

- `-` gemini pricing fix (update to genai 0.2.3 for normalized gemini usage)
- `-` pricing - fix price calculation to correctly compute prompt price when cache tokens

## 2025-04-20 - [v0.7.5](https://github.com/jeremychone/rust-devai/compare/v0.7.4...v0.7.5)

- `-` pricing calculation - Fix pricing calculation when cache tokens.

## 2025-04-19 - [v0.7.4](https://github.com/jeremychone/rust-devai/compare/v0.7.3...v0.7.4)

- `^` Windows x86 binary now 64bit (https://aipack.ai)
- `^` gemini 2.5\* - Support reasoning tokens,and update pricing calculation
- `.` config.toml - update the config tomls with new models

## 2025-04-16 - [v0.7.3](https://github.com/jeremychone/rust-devai/compare/v0.7.2...v0.7.3)

- `.` Added support and pricing for OpenAI `o4*` and `o3` models
- `^` `aip self update` - now check online version

## 2025-04-15 - [v0.7.2](https://github.com/jeremychone/rust-devai/compare/v0.7.0...v0.7.2)

- `.` `aip self update` - fix self update to print messsage (rather to do incomplete update)
- `.` pricing - add xai/grok pricing
- `.` update the base config.toml model aliases with gpt-4.1...
- `!` pack - keep the version as is in filename (no more replacing the `.` by `-`)
- `.` error - remove the default display {self}, was creating infinite looop
- `.` pricing - add pricing for the openai gpt 4.1 family
- `^` core@doc - add doc for aip.web.\*
- `.` update setup messages
- `-` aip.file.list_load - fix bug that make the function hang

## 2025-04-13 - [v0.7.0](https://github.com/jeremychone/rust-devai/compare/v0.6.18...v0.7.0)

- `*` BIG ONE - WINDOWS SUPPORT - x86 & ARM
- `!` --single-shot - For single shot run, use `-s` or `--single-shot` (rather than --non-interactive)
- `.` self update - first pass at self update (just point to install)
- `^` error display - First pass at 'De-uglifying' error message
- `!` API KEY - disable the mac keychain storage for now
- `.` lua cmd.exec - make it cross platform(ish), by wrapping the cmd with /C on windows

## 2025-04-10 - [v0.6.18](https://github.com/jeremychone/rust-devai/compare/v0.6.17...v0.6.18)

- `.` update to simple-fs 0.6.0-rc.4
- `.` zip - for common zip (for pack ..) compress with Deflated (most standard)
- `.` build.rs - use zstd for asset zip/unzip

## 2025-04-08 - [v0.6.17](https://github.com/jeremychone/rust-devai/compare/v0.6.16...v0.6.17)

- `+` self setup - Added setup support that init ~/.apack-base/bin/aip and aip-env
- `>` tui - work on the Prompt hub event
- `.` pricing - update pricing-all.json and pricing/data.rs for gemini 2.5 pro
- `.` more fixes & refactors

## 2025-03-29 - [v0.6.16](https://github.com/jeremychone/rust-devai/compare/v0.6.15...v0.6.16)

- `.` run input - change the way model print and ai_response.info to include provider model name
- `.` craft/text - tune the prompt to not echo content tag and follow better user instruction

## 2025-03-28 - [v0.6.15](https://github.com/jeremychone/rust-devai/compare/v0.6.14...v0.6.15)

- `*` install, pack, list - now does not init workspace
- `+` cli - add 'aip check-keys'
- `>` dir_context - lot of internal refactoring
- `.` doc update extract_line_blocks rust doc
- `.` clean jc@coder to pro@coder
- `.` doc - first pass at updated core/doc/README.md
- `.` init-base - fix print
- `.` demo@craft/text - fix the ==== issue

## 2025-03-25 - [v0.6.14](https://github.com/jeremychone/rust-devai/compare/v0.6.13...v0.6.14)

- `+` agent - add aip.flow.data_response({input?, data?, options?}) - Now can override model, input at Data stage
- `^` doc lua - add aip.agent, aip.flow
- `^` ask-aipack - update to take core@doc, and tune prompt
- `*` doc - move aipack do to core@doc
- `!` aip.agent.run - now agent path is relative to the caller agent
- `^` agent prompt part - now have 'input' in hbs context
- `^` demo@text - tune prompt, and prep to allow agent options meta
- `-` agent parse - fix issues when prompt part has a code block with level one heading
- `^` models-pricing - update data and agent

## 2025-03-21 - [v0.6.13](https://github.com/jeremychone/rust-devai/compare/v0.6.12...v0.6.13)

- `^` @ask-aipack - Fixes and improvements
- `^` run - now init-base if aipack version is not the same

## 2025-03-20 - [v0.6.12](https://github.com/jeremychone/rust-devai/compare/v0.6.11...v0.6.12)

- `+` NEW - Now can **call agent within agents** `aip.agent.run(agent_name, {inputs?, options?})`
- `-` agent - fix parse issue with not matching backticks on prompt sections
- `.` doc - Relatively big update of the lua.md & lua-api.md (in `~/aipack-base/doc/`)
- `-` run - fix input_concurrency in before_all getting ignored
- `>` refactor - Runtime & Executor
- `>` refactor aip_modules code layout
- `^` error msg - Enhance parse_prompt_part_options error message
- `.` update test_lua_semver_compare_basic

## 2025-03-16 - [v0.6.11](https://github.com/jeremychone/rust-devai/compare/v0.6.10...v0.6.11)

- `-` fix html5ever 0.29.2 yank compile issue
- `^` PromptPart options now evaluated at hbs render time
- `.` update version to 0.6.11-WIP

## 2025-03-14 - [v0.6.10](https://github.com/jeremychone/rust-devai/compare/v0.6.9...v0.6.10)

- `-` compile issue - fix compile issue from html5ever patch update and markup5ever_rcdom
- `>` prompt_part - prep work to support dynamic (at hbs render time) prompt part options
- `^` agent - now support four backticks (as well as the three) for the toml/lua code blocks

## 2025-03-12 - [v0.6.9](https://github.com/jeremychone/rust-devai/compare/v0.6.8...v0.6.9)

- `!` Lua module rename - Now `aip...` rather than `utils...` (backward compatibility preserved)
- `!` Lua module rename - Now `aip.flow` rather than `aipack.` (backward compatibility preserved)
- `-` Fix price_it (when pricing was missing for claude sonnet)
- `+` BIG ONE - Now supports `namespace@pack_name/some/**/*.*` for all `aip.file..` load, list, and such.
- `^` Lua - `file.save` now has a workspace guard (cannot save outside of the workspace for now). Might allow force flag later.
- `*` FIX v0.6.8 compile error ([#56 aipack install fail (v0.6.8)](https://github.com/aipack-ai/aipack/issues/56)).

## 2025-03-10 - [v0.6.8](https://github.com/jeremychone/rust-devai/compare/v0.6.7...v0.6.8)

BIG ERROR - This won't compile; some files are missing. Use v0.6.9.

- `+` agent - added support for Prompt PartOptions, e.g., cache = true
- `*` First pass at Anthropic Caching support
- `-` negative glob support e.g. `context_globs = ["src/**/*.rs", "!src/**/mod.rs"]`
- `^` find_agent - now support symlink for pack dirs
- `.` int - base config model name update.

## 2025-03-06 - [v0.6.7](https://github.com/jeremychone/rust-devai/compare/v0.6.6...v0.6.7)

- `!` lua - added the 'utils' to the 'aip' (which might be the new name for base aip utils). For now, they just alias to the same utilities set
- `.` @craft/text - minor prompt update
- `+` lua - add aip.semver.compare and more
- `.` add CTX.AIPACK_VERSION
- `.` .aip format - removed legacy config, added 'user' as alias of 'instruction'
- `>` refactor tui
- `^` install - installing pack check version now to make sure greater than.
- `>` refactors - tui, printers, tests_installer, tests_packer

## 2025-03-04 - [v0.6.6](https://github.com/jeremychone/rust-devai/compare/v0.6.5...v0.6.6)

- `.` Minor cleanup and update AIPACK resource links and text.

## 2025-03-02 - [v0.6.5](https://github.com/jeremychone/rust-devai/compare/v0.6.4...v0.6.5)

- `!` now pack `jc@coder` is not preinstalled. Install it with `aip install jc@coder`
- `.` refators and fixes

## 2025-03-01 - [v0.6.4](https://github.com/jeremychone/rust-devai/compare/v0.6.3...v0.6.4)

- `^` demo & jc agents - change default prompt file to be under .aipack/.prompt/namespace@pack_name/...-prompt.md
- `-` (#53) core@ask-aipack - first pass at fixing the `aip run core@ask-aipack`
- `^` lua - file.list.. - added {absolute} option
- `-` init-base - fix no print issue

## 2025-02-28 - [v0.6.3](https://github.com/jeremychone/rust-devai/compare/v0.6.2...v0.6.3)

- `+` **pricing** - first pass at adding pricing. Now, when available, `ai_response.price_usd` and added in `ai_response.info`
- `+` **install** - Now can do `aip install path/to/file.aipack`
- `>` major internal refactor - pack, packer (and first wire for aip install)

## 2025-02-28 - [v0.6.2](https://github.com/jeremychone/rust-devai/compare/v0.6.1...v0.6.2)

- `-` @coder - normalize coder to use four backtics for code block
- `-` jc@coder - fix the 6 backticks to be 4, which is the correct standard (for extract_blocks and extract_sections)
- `+` pack - template generation
- `+` pack - first pass at pack dir `aip pack some/path/to/dir

## 2025-02-27 - [v0.6.1](https://github.com/jeremychone/rust-devai/compare/v0.6.0...v0.6.1)

- `!` workspace - do not add .aipack/pack/custom on init anymore (still part of pack resolution)
- `-` aipbase - fix core/ask-aipack/

## 2025-02-26 - **v0.6.0** **BIG CHANGE - now AIPACK**

**BIG CHANGE - now AIPACK with agent packs `aip run namespace@pack_name`**

- **same codebase**, **same feature set**, **same licenses (MIT or Apache)**

- But now **ai packs centric**, which is going to bring huge value for the users and community.

- See [README.md](README.md)

## 2025-02-23 - [v0.5.12](https://github.com/jeremychone/rust-devai/compare/v0.5.11...v0.5.12)

- `*` readme - NOTICE about AIPACK migration
- `.` rust - update to 2024 edition, rust-version 1.85
- `^` lua - aip.text.extract_line_blocks error handling when options.starts_with is missing
- `^` agent - coder - fine tune prompt & move the initial doc below the ====

## 2025-02-22 - [v0.5.11](https://github.com/jeremychone/rust-devai/compare/v0.5.9...v0.5.11)

- `+` Parametric Agents with support for `#!meta` prompt code blocks
- `+` `coder` agent
- ... many more

## 2025-01-27 - [v0.5.9](https://github.com/jeremychone/rust-devai/compare/v0.5.8...v0.5.9)

- `^` groq - update genai to 0.1.19 for Groq deepseek-r1-distill-llama-70b

## 2025-01-23 - [v0.5.8](https://github.com/jeremychone/rust-devai/compare/v0.5.7...v0.5.8)

- `^` genai - use genai v0.1.18 for local and remote deepseek support

## 2025-01-23 - [v0.5.7](https://github.com/jeremychone/rust-devai/compare/v0.5.6...v0.5.7)

- `-` (#24) fix - Compile - does not compile in non macos

## 2025-01-20 - [v0.5.6](https://github.com/jeremychone/rust-devai/compare/v0.5.4...v0.5.6)

IMPORTANT: Can't compile on non-Mac. See v0.5.7 for fix.

**v0.5.6**

- `-` init - fix issue when running without an devai (was hanging)

**v0.5.4**

- `+` NEW - agent - added the craft/[text,code] in default agents
- `+` NEW - agent module - added first support of `my_dir/my_agent.devai` support, `devai run my_dir/my_agent`
- `^` BIG - lua - big error reporting update (inline code line with issue)
- `-` FIX - init - fix to avoid recreating default .lua file on each init (when exists)
- `-` FIX - auth - made keyring only for mac (as it is supposed to be for now)
- `+` NEW - lua - add aip.text.split_first(content, sep)
- `-` lua - fix input not being 'nil' when it is not specified (now it is nil)
- `^` lua - functions optimization and fixes.
- `.` doc - fix doc/lua for CTX

## 2025-01-06 - `0.5.4`

- `+` NEW - ~/.devai-base/ - first pass (supports custom/agent and custom/lua)
- `+` NEW - lua - first pass at supporting 'require' with the '.devai/custom/lua' path added
- `!` CHANGE - remove devai new-solo
- `!` CHANGE - .devai/... name change, rename the folders to (for simplification)
  - e.g., Now `.devai/custom/agent` (before `.devai/custom/command-agent`)
- `.` init - do not create custom/new-template anymore
- `.` fix agent proof-comments
- `^` genai - updated to 0.1.17 with DeepSeek support
- `.` add in cargo.toml comment gemini-2.0-flash-exp
- `-` fix glob issue when relatively globs does not start with './'
- `.` update genai to 0.1.16
- `^` lua - override global lua print to print correctly
- `.` minor code refactor
- `.` lua_engine - minor refactor
- `.` clippy clean

## 2024-12-12 - `0.5.3`

Thanks to [Kees Jongenburger](https://github.com/keesj) for reporting

- `-` Fix critical bug - [#23 cli issue - devai init fails when the .devai directory does not exits](https://github.com/jeremychone/rust-devai/issues/23)

## 2024-12-11 - `0.5.2`

> NOTE - This version introduced a critical bug (when .devai/ did not exist).

         See [#23](https://github.com/jeremychone/rust-devai/issues/23)
         Use `0.5.3` and above

- `+` lua - add `aip.file.ensure_exists(path, optional_content)`
- `+` version - added `.devai/verion.txt` to force update doc on version change.
- `.` doc - remove ; in lua code
- `+` lua - add `aip.text.ensure(content, {prefix, suffix})`

## 2024-12-08 - `0.5.1`

- `+` Add xAI support (thanks to genai v0.1.15)
- `-` First fix for the keychain prompt
- `^` lua - load_md_sections now can take only the path (selecting all md sections)

## 2024-12-05 - `0.5.0`

- `*` BIG release with Lua and more. See [YouTube intro](https://www.youtube.com/watch?v=b3LJcNkhkH4&list=PL7r-PXl6ZPcBcLsBdBABOFUuLziNyigqj)
