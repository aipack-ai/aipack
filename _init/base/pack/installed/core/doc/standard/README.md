## KEY CONCEPT - ONE Markdown, ONE Agent, Multi Stages

The main **aipack** concept is to minimize the friction of creating and running an agent while providing maximum control over how those agents run and maximizing iteration speed to mature them quickly.

- **One Agent** == **One Markdown**
    - (i.e., `my-agent.aip`; a `.aip` file is a Markdown file with multi-stage sections described below)
- **Multi AI Providers / Models**: Supports OpenAI, Anthropic, Gemini, Groq, Ollama, Cohere, and more to come.
- **Lua** for Scripting.
- **Handlebars** for prompt templating.
- **Multi-Stage** process, where ALL steps are optional.

| Stage           | Language       | Description                                                                                                |
|-----------------|----------------|------------------------------------------------------------------------------------------------------------|
| `# Options`     | **TOML**       | **Stage 0 (Config Step)**: Define agent-specific options (model, concurrency, etc.).                       |
| `# Before All`  | **Lua**        | **Stage 1**: Reshape/generate inputs and add command global data to scope (the "map" of the map/reduce).   |
| `# Data`        | **Lua**        | **Stage 2**: Gather additional data per input and return it for the next stages.                           |
| `# System`      | **Handlebars** | **Stage 3**: Customize the system prompt with the `input`, `data`, and `before_all` data.                  |
| `# Instruction` | **Handlebars** | **Stage 3**: Customize the user instruction prompt with the `input`, `data`, and `before_all` data.        |
| `# Assistant`   | **Handlebars** | **Stage 3**: Optional for special customizations, such as the "Jedi Mind Trick."                           |
| `# Output`      | **Lua**        | **Stage 4**: Processes the `ai_response` from the LLM.                                                     |
| `# After All`   | **Lua**        | **Stage 5**: Called after all inputs are completed for final processing.                                   |

> **Notes:**
> - Stage names are case insensitive. 
> - `# Instruction` is aliased to `# User` and `# Inst`
> - `# Assistant` is aliased to `# Model`, `# Mind Trick` or `# Jedi Trick`

```sh
# Run the ./first-agent.aip on the README.md file
aip run ./my-agents/first-agent.aip -f "./README.md"
```

`./my-agents/my-first-agent.aip`

````md
# Data

```lua
-- `aip run -f src/**/*.rs`
-- NOTE: Each file will be an input, so it can run concurrently (as defined by input_concurrency options)
-- It has access to `input` and `before_all` and can fetch more data, returning it for the next stage.
-- Input originates from the command line (when using -f, this will be a FileInfo object).

local file_record = aip.file.load(input.path)
return {
    file_content = file_record.content
}
```

# Instruction

This is a Handlebars section where we can include the data generated above. For example:

Here is the content of the file `{{input.path}}` to proofread:

```{{input.ext}}
{{data.file_content}}
```

- Correct the English in the content above.
- Do not modify it if it is grammatically correct.

# Output

```lua
-- This section allows processing of `ai_response`.
-- It has access to `input`, `data`, `before_all`, and `ai_response`.

-- Example: remove an optional leading markdown block if present.
local content = aip.md.outer_block_content_or_raw(ai_response.content)
-- Example: ensure single trailing newline for code output.
content = aip.text.ensure_single_ending_newline(content)
-- More processing....

-- Save the processed content back to the original file path.
-- `input.path` contains the path derived from the `-f` flag.
aip.file.save(input.path, content)

-- This string will be printed in the terminal if returned.
return "File saved: " .. input.path

```

````

- See [Complete Stages Description](#complete-stages-description) for more details on the stages.
- See [Lua doc](lua.md) for more details on the available Lua modules and functions.

## More Details

### `aip` command line doc

Here is the overall command 

```
Command Agent runner to accelerate production coding with genai.

Usage: aip <COMMAND>

Commands:
  init        Initialize the workspace `.aipack/` and the base `~/.aipack-base/` aipack directories
  init-base   Init the ~/.aipack-base only with force update
  run         Executes the AIPack agent using `aip run demo@craft/code`, or an agent file `aip run path/to/agent.aip`.
              
              Example usage:
              ```sh
              # Run the demo@craft/code AIP agent
              aip run demo@craft/code
              
              # Run the demo@proof main.aip agent and provide a file as input
              aip run demo@proof -f ./README.md
              
              # Run a direct agent file from the local directory
              aip run some/agent.aip
              ```
  list        Create a new agent from a built-in template Disabled for now List the available aipacks `aip run list` or `aip run list demo@`
  pack        Pack a directory into a .aipack file
  install     Install an aipack file
  check-keys  Check available API keys in the environment
  self        Manage the aip CLI itself
  help        Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```
#### `aip run ...` documentation 

Executes the AIPack agent using `aip run demo@craft/code`, or an agent file `aip run path/to/agent.aip`.

Example usage:

```sh
# Run a direct agent file from the local directory
aip run some/agent.aip

# Gives two inputs
aip run some/agent.aip -i "input 1" -i "input 2"

# Use -f to give file or globs (each matched file will be a input)
aip run some/agent.aip -f "src/**/*.js"

# Run the demo@craft/code AIP agent
aip run demo@craft/code

# Run the demo@proof main.aip agent and provide a single file as input
aip run demo@proof -f ./README.md

```

Usage: aip run [OPTIONS] <CMD_AGENT_NAME>

Arguments:
  <CMD_AGENT_NAME>  The name of the agent, which can be:
                    - A AIP pack reference:
                    `aip run demo@proof`
                    - Or a direct file:
                    `aip run path/to/agent.aip`

Options:
  -i, --input <ON_INPUTS>    Optional input, allowing multiple input NOTE: CANNOT be combined with -f/--on-files
  -f, --on-files <ON_FILES>  Optional file parameter, allowing multiple files NOTE: CANNOT be combined with -i/--input
  -w, --watch                Optional watch flag
  -v, --verbose              Verbose mode
  -o, --open                 Attempt to open the agent file (for now use VSCode code command)
      --dry <DRY_MODE>       Dry mode, takes either 'req' or 'res' [possible values: req, res]
  -s, --single-shot          Single Shot execution (e.g., non-interactive). (Was the `--ni` or `--non-interactive` in v0.6.x)
  -h, --help                 Print help

### Install From Git

Install a pack directly from a Git repository. Use an explicit `#` selector
when the pack is located in a repository subdirectory:

```sh
aip install git://git.example.com/team/example-pack.git
aip install git://git.example.com/team/example-pack.git#path/to/pack_dir
aip install git+ssh://git@github.com/owner/example-pack.git#path/to/pack_dir
```

The repository portion before `#` is cloned. Without a selector, the cloned
repository root must contain `pack.toml`. With a selector, the selected
directory must contain `pack.toml`. The manifest's `namespace` and `name`
fields determine the installed path under
`~/.aipack-base/pack/installed/<namespace>/<name>`.

Pack paths are selected only with an explicit `#` selector. A path after `.git`
without `#` is not inferred as a pack path. Ordinary `http://` and `https://`
links continue to be treated as downloadable `.aipack` archives.

### Tips

**aipack** is built on top of the [genai crate](https://crates.io/crates/genai) and therefore supports all major AI Providers and Models (OpenAI, Anthropic, Gemini, Ollama, Groq, Cohere).

You can customize the model and concurrency in `.aip/config.toml`.

New `.aip/` file structure with the `.aip` file extension. See [.aip/ folder structure](#aipack-folder-structure).

**TIP 1**: In VSCode or your editor, map the `*.aip` extension to `markdown` to benefit from markdown highlighting. `aipack` agent files are standard Markdown files.

**TIP 2**: Make sure to commit your changes before running this command so that overwritten files can be easily reverted.

_P.S. If possible, please refrain from publishing `aipack-custom` type crates on crates.io, as this might be more confusing than helpful. However, feel free to fork and code as you wish._

## Complete Stages Description

Here is a full description of the complete flow:

- First, an agent receives zero or more inputs.
    - Inputs can be given through the command line via:
        - `-i` or `--input` to specify one input (multiple `-i`/`--input` flags can be used). The input type is typically a string.
        - `-f some_glob` which creates one input per matched file, with the input structured as a [FileInfo object](lua.md#filemeta) `{path, name, stem, ext, ...}`.
    - Then the following stages occur (all are optional):
- **Stage 0**: `# Options` (toml block) (optional - Config Step)
    - This section allows defining agent-specific configuration using TOML.
    - Supported keys: `model`, `input_concurrency`, and `model_aliases`.
    - These settings take precedence over the workspace `.aipack/config.toml` and the base `~/.aipack-base/config.toml`.
- **Stage 1**: `# Before All` (lua block) (optional)
    - The `lua` block has the following in scope:
        - `inputs`: A list of all inputs provided to the agent run (or `nil` if no inputs).
        - `aip`: The [AIPack Lua API module](lua-apis).
        - `CTX`: Contextual [constants](lua.md#ctx) (paths, agent info).
    - It can return:
        - Nothing.
        - Data that will be available as `before_all` in subsequent stages (e.g., `return { some = "data" }`).
        - Overridden inputs and/or agent options using `return aip.flow.before_all_response({ inputs = {...}, options = {...}, before_all = {...} })`. See [aip.flow.before_all_response](lua-apis#aipflowbefore_all_response).
- **Stage 2**: `# Data` (lua block) (optional)
    - This stage runs *for each input* item.
    - The `lua` block receives the following variables in scope:
        - `input`: The current input item (string, [FileInfo](lua.md#filemeta), or value from `# Before All`).
        - `before_all`: Data returned from the `# Before All` stage (or `nil`).
        - `aip`: The [AIPack Lua API module](lua-apis).
        - `CTX`: Contextual [constants](lua.md#ctx).
    - It can return:
        - Data that will be available as `data` in subsequent stages for this input.
        - A special flow control object using `aip.flow.data_response({ data = ..., input = ..., options = ...})` to modify the input or options for this cycle. See [aip.flow.data_response](lua-apis#aipflowdata_response).
        - A skip instruction using `aip.flow.skip("reason")` to skip processing this input. See [aip.flow.skip](lua-apis#aipflowskip).
- **Stage 3**: Prompt Stages (`# System`, `# Instruction`, `# Assistant`) (handlebars templates) (optional)
    - The content of these sections is rendered via Handlebars with the following variables in scope:
        - `input`: The current input item (potentially modified by `# Data`).
        - `data`: Data returned by the `# Data` stage for this input (or `nil`).
        - `before_all`: Data returned by the `# Before All` stage (or `nil`).
    - The rendered content forms the prompt sent to the AI model.
- **Stage 4**: `# Output` (lua block) (optional)
    - This stage runs *for each input* that was processed by the AI (i.e., not skipped).
    - The `lua` block receives the following scope:
        - `input`: The current input item.
        - `data`: Data returned by the `# Data` stage for this input (or `nil`).
        - `before_all`: Data returned by the `# Before All` stage (or `nil`).
        - `ai_response`: Contains the AI's response. See [AiResponse](lua-apis#ai-response).
            - `.content`: The text content of the response.
            - `.model_name`: The name of the model used.
        - `aip`: The [AIPack Lua API module](lua-apis).
        - `CTX`: Contextual [constants](lua.md#ctx).
    - It can return data, which will be captured as the `output` for this input item in the `# After All` stage. If not specified, the raw `ai_response.content` is printed to the terminal.
- **Stage 5**: `# After All` (lua block) (optional)
    - This stage runs once after all inputs have been processed.
    - The `lua` block receives the following scope:
        - `inputs`: The final list of all inputs processed (potentially modified by `# Before All`).
        - `outputs`: A list containing the return value from the `# Output` stage for each corresponding input (or `nil` if an input was skipped or `# Output` returned nothing).
        - `before_all`: Data returned by the `# Before All` stage (or `nil`).
        - `aip`: The [AIPack Lua API module](lua-apis).
        - `CTX`: Contextual [constants](lua.md#ctx).
        - Note: The `inputs` and `outputs` arrays are kept in sync; `outputs[i]` corresponds to `inputs[i]`.
    - It can return data, which is currently discarded but might be used in the future.

## Usage

Example: `aip run proof-rs-comments -f "./src/main.rs"`

(You can also use any glob pattern, like `-f "./src/**/*.rs"`)

- This command initializes the `.aip/defaults` folder if necessary, containing the "Command Agent Markdown" file `proof-rs-comments.aip` (see [.aip/defaults/proof-comments.aip](./_init/agents/proof-comments.aip)), and runs it using `genai` as follows:
    - `-f "./src/**/*.rs"`: The `-f` argument takes a glob pattern. An "input" record (a [FileInfo object](lua.md#filemeta)) is created for each matching file.
    - `# Data`: Contains a ```lua``` block executed for each `input`.
        - Provides Lua utility functions (`aip.*`) for tasks like listing files, loading content (`aip.file.load`), etc. The results are returned as `data` for use in prompt stages.
    - Prompt Stages (`# System`, `# Instruction`, `# Assistant`): Handlebars template sections with access to `input`, `data`, and `before_all`.
        - The rendered content is sent to the AI.
    - `# Output`: Executes another ```lua``` block with access to `input`, `data`, `before_all`, and `ai_response`.
        - This stage can save modified files (`aip.file.save`), create new ones, or perform other actions based on the AI response.
- By default, this runs using the `gpt-5-mini` model and looks for the `OPENAI_API_KEY` environment variable.
- It supports all AI providers compatible with the [genai crate](https://crates.io/crates/genai).
    - Environment variable names per provider: `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `COHERE_API_KEY`, `GEMINI_API_KEY`, `GROQ_API_KEY`, `XAI_API_KEY`, `DEEPSEEK_API_KEY`.
    - On macOS, if an environment variable is not found, the tool may prompt to get/save the key from the keychain under the `aipack` group.

## `aip` command arguments

```sh
# Creates/updates the .aip/ settings folder (not strictly required, automatically runs on "run")
aip init

## -- Running a Pre-Installed Pack

# Executes the pre-installed `proof` AI Pack from the `core` namespace (~/.aipack-base/pack/installed/core/)
# on any file matching `src/**/*.rs` (these files become 'inputs').
aip run core@proof-rs -f "src/**/*.rs"

# Verbose mode prints details like the prompt sent, the AI response, and # Output return values to the console.
aip run core@proof-rs -f "src/**/*.rs" --verbose

# Performs a dry run up to the request stage. Use with -v to print the rendered instruction.
# Does not call the AI or run the # Output stage.
aip run core@proof-rs -f "src/**/*.rs" -v --dry req

# Performs a dry run including the AI call. Use with -v to print the rendered instruction and AI response.
# Does not run the # Output stage.
aip run core@proof-rs -f "src/**/*.rs" -v --dry res

## -- Running the ask-aipack

# Creates a prompt file for interactive editing (press 'r' in the terminal to re-run).
aip run core@ask-aipack

## -- Running your own agent
aip run path/to/my-agent.aip

# Happy coding!
```

- `aip init`: Initializes the current directory as an `aipack` workspace by creating the `.aip/` folder. Optionally creates the `~/.aipack-base` folder with base AIPack resource files if it doesn't exist.

- `aip run`: Runs an agent file (`.aip`) or a pre-installed pack.
    - The first argument is the agent file path or the pack name (e.g., `namespace@pack/agent`).
    - `-i <string>`: Provides a single string input. Can be used multiple times.
    - `-f <path_or_glob>`: Specifies input files using a path or glob pattern. Creates one input ([FileInfo](lua.md#filemeta)) per matched file. Can be used multiple times.
    - `--verbose` (`-v`): Prints detailed information to the console, including rendered prompts, AI responses, and `# Output` stage return values (if string-like).
    - `--dry req`: Performs a dry run, executing only the `# Before All`, `# Data`, and template rendering stages (`# System`, `# Instruction`, `# Assistant`). Use with `--verbose` to see the rendered prompt content. Does not call the AI.
    - `--dry res`: Performs a dry run including the AI call. It executes stages up to and including the AI interaction but skips the `# Output` and `# After All` stages. Use with `--verbose` to see the prompt sent and the AI response received.

- `aip init-base`: Updates the base resource folder at `~/.aipack-base`.

- `aip pack <folder_path>`: Creates an AI pack file (e.g., `my@pack-v0-1.0.aipack`) from the specified folder.

- `aip install <path/to/pack.aipack>`: Installs an AI pack from a local `.aipack` file.

- `aip install <url>`: Installs an AI pack from a URL (e.g., `https://cool-aipacks/my-aipack.aipack`).

- `aip install <pack_name>`: Installs a published AI pack from `aipack.ai` (e.g., `pro@coder`). Currently limited availability, planned to open later.

- `aip list`: Lists installed packs.

- `aip check-keys`: Checks for available AI provider API keys.

## `aipack` folder structure

(Updated in version `0.7.x` - migration handled automatically)

- `.aipack/` - The root folder for `aipack` configuration and custom files within your project (workspace).
    - `config.toml` - Workspace-specific configuration (model, concurrency, etc.).
    - `custom/` - User-defined agents and support files (Lua modules, templates).
        - `agent/` - Custom agents (`.aip` files).
        - `lua/` - Custom Lua modules (`*.lua`) usable via `require`.
        - `template/` - Custom templates (e.g., for `aip new`).
    - `support/` - Workspace-level support files generated or needed by packs.
        - `pack/`
            - `<pack-name>/`
                - `<agent-name>/` - Support files specific to an installed pack agent.
- `~/.aipack-base/` - The base directory for globally installed resources.
    - `config.toml` - Base configuration (API keys location, default model if not set in workspace).
    - `pack/`
        - `installed/`
            - `<namespace>/`
                - `<pack-name>/` - Installed pack files (agents, lua, templates).
    - `support/` - Global support files generated or needed by packs.
        - `pack/`
            - `<pack-name>/`
                - `<agent-name>/` - Global support files specific to an installed pack agent.

## Example of a Command Agent File

See the example agent file within this README, or check installed agents like `.aipack-base/pack/installed/core/proof-rs-comments/agent.aip`.

## Config

On first run (`aip run`, `aip init`, etc.), `.aipack/config.toml` and `~/.aipack-base/config.toml` files will be created if they don't exist.

### Options Precedence

AIPack uses a cascading merge strategy for agent options (such as `model` and `input_concurrency`). Settings defined at a more specific level override those at a more general level:

1.  **Lua Flow Overrides**: `aip.flow.data_response({options = ...})` or `aip.flow.before_all_response({options = ...})` (Highest precedence).
2.  **Agent Options Stage**: The `# Options` TOML block within the `.aip` file.
3.  **Workspace Config**: The project-specific `.aipack/config.toml` file.
4.  **Base Config**: The global `~/.aipack-base/config.toml` file (Lowest precedence).

**Workspace Config (`.aipack/config.toml`) Example:**
```toml
# Default options for agents run within this workspace.
# Can be overridden by agent-specific meta or `aip.flow` functions.
[default_options]
# Required model identifier (any model supported by the Rust genai crate).
model = "gpt-5-mini"

# Optional concurrency setting for processing inputs in parallel. Defaults to 1.
# Increasing this can speed up processing, especially with remote AI services.
input_concurrency = 4

# Aliases for model names
[model_aliases]
# main = "gpt-5"
# mini = "gpt-5-mini"
# fast = "gpt-5-nano"
```

**Base Config (`~/.aipack-base/config.toml`) Example:**
```toml
# Base configuration affecting all workspaces unless overridden locally.
[default_options]
# Default model if not set in workspace or agent.
# model = "gpt-5-mini"

# Where to store/retrieve API keys (e.g., "keyring", "env", "config")
# api_key_source = "keyring"

# Optional API keys can be stored directly here (less secure)
#[api_keys]
#OPENAI_API_KEY = "sk-..."
#ANTHROPIC_API_KEY = "..."
```

## Security

AIPack implements several safeguards to protect your system and data.

### File System Restrictions

To prevent unauthorized access, file operations are restricted by boundaries:

- **Write Restrictions**: The `aip.file.save` and related functions only allow writing within the active workspace or the shared `~/.aipack-base/` directory.
- **Deletion Restrictions**: The `aip.file.delete` function is more restrictive, only allowing deletions within the workspace. It specifically forbids deleting any files in the `.aipack-base` folder.

### API Key Security

AIPack never stores API keys inside agent files. Keys are managed through external sources:

- **Environment Variables**: The simplest way to provide API keys is via environment variables. AIPack supports: `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `GEMINI_API_KEY`, `FIREWORKS_API_KEY`, `TOGETHER_API_KEY`, `NEBIUS_API_KEY`, `XAI_API_KEY`, `DEEPSEEK_API_KEY`, `GROQ_API_KEY`, and `COHERE_API_KEY`.
- **System Keychain**: (not active for now) On macOS, AIPack can securely store and retrieve keys from the system keychain if the environment variable is missing. This ensures sensitive credentials remain outside of version-controlled files.
