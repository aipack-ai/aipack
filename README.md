<div align="center">

<a href="https://crates.io/crates/aipack"><img src="https://img.shields.io/crates/v/aipack.svg" /></a>
<a href="https://github.com/jeremychone/rust-aipack"><img alt="Static Badge" src="https://img.shields.io/badge/GitHub-Repo?color=%23336699"></a>
<a href="https://www.youtube.com/watch?v=SioUg_N9HS0"><img alt="Static Badge" src="https://img.shields.io/badge/AIPACK_Introduction_-Video?style=flat&logo=youtube&color=%23ff0000"></a>

</div>

# AIPACK - Run, Build, and Share AI Packs

Checkout the site: https://aipack.ai for more information and links.

Open-source Agentic Runtime to run, build, and share AI Packs.
- Supports **all** major AI providers and models.
- Efficient and small (**< 20MB**), with **zero** dependencies.
- Built in **Rust**, using Lua for embedded scripting (small and efficient).
- Runs locally, completely IDE-agnostic.
- Or in the cloud—server or serverless.

### Quick Start

**Install**

For now, the install requires building it directly from source via Rust. Works great on all OSes.

- Install Rust: https://www.rust-lang.org/tools/install  
- For now, install with `cargo install aipack`  

> **NOTE:** Ironically, while the binary is relatively small (<20MB with batteries included), the build process can take up quite a bit of space. However, Cargo should clean it up afterward.  
> Binaries and installation instructions will be available at https://aipack.ai

> DISCLAIMER: For now, v0.6.x, AIPACK works on **Linux & Mac**, and requires **WSL** on **Windows** 
>
> IMPORTANT: **Proper Windows support** is coming sometime in v0.6.x and **definitely by v0.7.x** (about **Mid / End of March**)

**Run**

```sh
# In terminal go to your projct
cd /path/to/my/project/

# initialize workspace .aipack/ and ~/.aipack-base
aip init

# Make sure to export the desired API key
export OPENAI_API_KEY    = "sk...."
export ANTHROPIC_API_KEY = "...."
export GEMINI_API_KEY    = "..."
# For more keys, see below

# To proof read your README.md (namespace: demo, pack_name: proof)
aip run demo@proof -f ./README.md

# You can just use @pack_name if there is no other packname of this name
aip run @proof -f ./README.md

# To do some code crafting (will create _craft-code.md)
aip run demo@craft/code

# Or create your .aip file (you can omit the .aip)
aip run path/to/file.aip

# This is good agent to run to ask questions about aipack
# Can even generate aipack code
aip run core@ask-aipack
# prompt file will be at `.aipack/.prompt/core@ask-aipack/ask-prompt.md`

```

**jc@coder**

- You can install `jc@coder` with `aip install jc@coder`, and then  
- Run it with `aip run jc@coder` or `aip run @coder` if you don't have any other `@coder` pack in a different namespace.  

This is the agent I use every day for my production coding.


**IMPORTANT 1**: Make sure everything is committed before usage (at least while you are learning about aipack).

**IMPORTANT 2**: Make sure to have your **API_KEY** in an environment variable (on Mac, there is an experimental keychain support)

```
OPENAI_API_KEY
ANTHROPIC_API_KEY
GEMINI_API_KEY
XAI_API_KEY
DEEPSEEK_API_KEY
GROQ_API_KEY
COHERE_API_KEY
```

### Info

- Website: https://aipack.ai

- [AIPack Overview Video](https://www.youtube.com/watch?v=SioUg_N9HS0)

- [Full intro video for v0.5 (still old devai name, but same concept)](https://www.youtube.com/watch?v=b3LJcNkhkH4&list=PL7r-PXl6ZPcBcLsBdBABOFUuLziNyigqj)

- Built on top of the [Rust genai library](https://crates.io/crates/genai), which supports all the top AI providers and models (OpenAI, Anthropic, Gemini, DeepSeek, Groq, Ollama, xAI, and Cohere).

- Top new features: (see full [CHANGELOG](CHANGELOG.md))
  - **2025-03-02 (v0.6.4) -  Fixes, and now support first repo pack `aip install jc@coder`**
  - **2025-02-28 (v0.6.3) - `aip pack ..`, `aip instal local...`, `ai_response.price_usd`, and more**
  - **2025-02-26 **(v0.6.0)** - BIG UPDATE - to **AIPACK**, now with pack support (`aip run demo@craft/code`)**
  - 2025-02-22 (v0.5.11) - Huge update with parametric agents, and coder (more info soon)
  - 2025-01-27 (v0.5.9) - Deepseek distill models support for Groq and Ollama (local)
  - 2025-01-23 (v0.5.7) - `aipack run craft/text` or `aipack run craft/code` (example of cool new agent module support)
  - 2025-01-06 (v0.5.4) - DeepSeek `deepseek-chat` support
  - 2024-12-08 (v0.5.1) - Added support for **xAI**

- **WINDOWS DISCLAIMER:**
    - This CLI uses a path scheme from Mac/Unix-like systems, which might not function correctly in the Windows `bat` command line.
    - Full Windows local path support is in development.
    - **RECOMMENDATION:** Use PowerShell or WSL on Windows. Please log issues if small changes can accommodate Windows PowerShell/WSL.

- Thanks to
  - **[Stephane Philipakis](https://github.com/sphilipakis)**, a **top** [aipack](https://aipack.ai) collaborator.
  - [David Horner](https://github.com/davehorner) for adding Windows support for Open Agent (with VSCode) ([#30](https://github.com/jeremychone/rust-aipack/pull/30))
  - [Diaa Kasem](https://github.com/diaakasem) for `--non-interactive`/`--ni` mode ([#28](https://github.com/jeremychone/rust-aipack/pull/28))

### How it works

- **One Agent** == **One Markdown**
    - A `.aip` Agent file is just a **Markdown File** with sections for each stage of the agent processing.
    - See below for all the [possible stages](#multi-stage).
- `aip run demo@proof -f "./*.md"`
  - will run the installed Agent file `main.aip` in the
  - pack name `proof`
  - namespace `demo`
  - agent file `main.aip`
  - Full path `~/.aipack-base/pack/installed/demo/proof/main.aip`
  - You can pass input to your agent with
    - `-f "path/with/optional/**/glob.*" -f "README.md` (then the lua code will get a `{path = .., name =..}` FileMeta type of structure as input)
    -  `-i "some string" -i "another input"` (then the lua code will get those strings as input)
    - Each input will be one run of the agent.
- `aip run some/path/to/agent`
  - can end with `.aip` in this case direct file run
  - if no `.aip` extension, then,
    - `...agent.aip` will be executed if exists
    - or `...agent/main.aip` will be executed if exists
- **aipack** agents are simple `.aip` files that can be placed anywhere on disk.
  - e.g., `aipack run ./my-path/to/my-agent.aipack ...`
- **Multi AI Provider / Models** - **aipack** uses [genai](https://crates.io/crates/genai) and therefore supports OpenAI, Anthropic, Gemini, Groq, Ollama, Cohere, and more to come.
- **Lua** is used for all scripting (thanks to the great [mlua](https://crates.io/crates/mlua) crate).
- **Handlebars** is used for all prompt templating (thanks to the great Rust native [handlebars](https://crates.io/crates/handlebars) crate).

### Multi Stage

A single **aipack** file may comprise any of the following stages.

| Stage           | Language       | Description                                                                                                |
|-----------------|----------------|------------------------------------------------------------------------------------------------------------|
| `# Before All`  | **Lua**        | Reshape/generate inputs and add command global data to scope (the "map" of the map/reduce capability).    |
| `# Data`        | **Lua**        | Gather additional data per input and return it for the next stages.                                       |
| `# System`      | **Handlebars** | Customize the prompt with the `data` and `before_all` data.                                                |
| `# Instruction` | **Handlebars** | Customize the prompt with the `data` and `before_all` data.                                                |
| `# Assistant`   | **Handlebars** | Optional for special customizations, such as the "Jedi Mind Trick."                                        |
| `# Output`      | **Lua**        | Processes the `ai_response` from the LLM. Otherwise, `ai_response.content` will be output to the terminal. |
| `# After All`   | **Lua**        | Called with `inputs` and `outputs` for post-processing after all inputs are completed.                     |

- `# Before All` / `# After All` can be considered as the **map**/**reduce** of the agent, and these will be run before and after the input processing.

[more info on stages](_init/base/doc/README.md#complete-stages-description)

## [aipack doc](_init/doc/README.md)

See the aipack documentation at **[_init/doc/README.md](_init/base/doc/README.md)** (With [Lua modules doc](_init/base/doc/lua.md))

You can also run the `ask-aipack` agent.

```sh
# IMPORTANT - Make sure you have the `OPENAI_API_KEY` or the key of your model in your environment
aip run core@ask-aipack
# prompt file will be at `.aipack/.prompt/core@ask-aipack/ask-prompt.md`
```
