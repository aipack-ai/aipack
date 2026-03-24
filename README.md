<div align="center">

<a href="https://crates.io/crates/aipack"><img src="https://img.shields.io/crates/v/aipack.svg" /></a>
<a href="https://github.com/aipack-ai/aipack"><img alt="Static Badge" src="https://img.shields.io/badge/GitHub-Repo?color=%23336699"></a>
<a href="https://www.youtube.com/watch?v=SioUg_N9HS0"><img alt="Static Badge" src="https://img.shields.io/badge/AIPack_Introduction_-Video?style=flat&logo=youtube&color=%23ff0000"></a>

</div>

# AIPack - Run, Build, and Share AI Packs

An open-source agentic runtime to run, build, and share AI Packs.

- **Simple & Powerful**, 1 Agent = 1 multi-stage Markdown file with built-in **concurrency**, **Map-Reduce**, and all APIs on a single [aip-doc page](https://aipack.ai/doc/lua-apis).

- **Light & Lean**, No bloat, **< 20MB**, **single executable**, **ZERO dependencies**.

- **Efficient**, Engine written in **Rust** with a lightweight and efficient embedded Lua script. All `aip` functions are implemented in Rust.

- **Multi-AI**, Supports all major AI **providers and models** (OpenAI, Google, Anthropic, xAI, Ollama, Groq, Fireworks.ai, ...) at the native layer for the major ones. For example, it can use Gemini models with zero thinking budget.

- **Local or Cloud**, Runs locally, is completely IDE-agnostic, or runs in the cloud, on a server or serverless.

**BIG UPDATE 0.8.x WITH NEW TUI**

### Check out:

- https://aipack.ai for more information and links, [AIPack News & Blog Posts](https://news.aipack.ai/archive)

- [Getting Started Video Tutorial](https://news.aipack.ai/p/aipack-tutorial-from-hello-world)

- **[AIPack Lab Repo](https://github.com/aipack-ai/aipack-lab)** for some cool examples.

- [AIPack Substack at news.aipack.ai](https://news.aipack.ai)

<img alt="Static Badge" src="https://img.shields.io/badge/AIPack_VIDEOS_-Video?style=flat&logo=youtube&color=%23ff0000">

- [Video: AIPack Concept Video](https://www.youtube.com/watch?v=SioUg_N9HS0&list=PL7r-PXl6ZPcDBodiiTdUeCmUwsYFyDzGt)
- [Video: Build Agent Getting Started Video Tutorial](https://news.aipack.ai/p/aipack-tutorial-from-hello-world)
- [Video: Production Coding With pro@coder AI PACK](https://news.aipack.ai/p/production-coding-example-with-procoder)
- [Video: Example generate doc from code](https://news.aipack.ai/p/procoder-ai-pack-demo-generate-doc)
- [Video: pro@coder with pro@rust10x Rust best practices](https://www.youtube.com/watch?v=rIAoSf4TWho&list=PL7r-PXl6ZPcBcLsBdBABOFUuLziNyigq)
- [Video: AIPack Playlist](https://www.youtube.com/playlist?list=PL7r-PXl6ZPcDBodiiTdUeCmUwsYFyDzGt)

## Quick Start

### Install From Binaries

Mac, Linux, and Windows on ARM and x86 platforms are supported. See below for binary installation instructions.

_(More info at [aipack.ai/doc/install](https://aipack.ai/doc/install))_

#### Mac

```sh
# Mac ARM / Apple Silicon
curl -O https://repo.aipack.ai/aip-dist/stable/latest/aarch64-apple-darwin/aip.tar.gz && \
        tar -xvf aip.tar.gz && \
        ./aip self setup

# Mac x86
curl -O https://repo.aipack.ai/aip-dist/stable/latest/x86_64-apple-darwin/aip.tar.gz && \
        tar -xvf aip.tar.gz && \
        ./aip self setup        
```


#### Linux

```sh
# Linux x86
curl -O https://repo.aipack.ai/aip-dist/stable/latest/x86_64-unknown-linux-gnu/aip.tar.gz && \
        tar -xvf aip.tar.gz && \
        ./aip self setup

# Linux ARM
curl -O https://repo.aipack.ai/aip-dist/stable/latest/aarch64-unknown-linux-gnu/aip.tar.gz && \
        tar -xvf aip.tar.gz && \
        ./aip self setup
```

#### Windows

```sh
# Windows x86
Invoke-WebRequest -Uri "https://repo.aipack.ai/aip-dist/stable/latest/x86_64-pc-windows-msvc/aip.tar.gz" -OutFile "aip.tar.gz"
tar -xvf aip.tar.gz
.\aip.exe self setup
     

# Windows ARM
Invoke-WebRequest -Uri "https://repo.aipack.ai/aip-dist/stable/latest/aarch64-pc-windows-msvc/aip.tar.gz" -OutFile "aip.tar.gz"
tar -xvf aip.tar.gz
.\aip.exe self setup
```


### Install from source

For now, installation requires building directly from source with Rust. It works on all major OSes.

- Install Rust: https://www.rust-lang.org/tools/install
- For now, install with `cargo install aipack`

#### Build docker

No images are published yet, but you can build a docker image locally.

```sh
docker build -t aipack:latest -f docker/Dockerfile .

# There is an alias you can copy or modify into your own ~/.bashrc or ~/.zshrc
source docker/docker-alias

aip check-keys
```

### Run

```sh
# In the terminal, navigate to your project
cd /path/to/my/project/

# Initialize workspace .aipack/ and ~/.aipack-base
aip init

# Make sure to export the desired API key (no spaces around `=`; Unix convention)
export OPENAI_API_KEY="sk...."
export ANTHROPIC_API_KEY="...."
export GEMINI_API_KEY="..."
# For more keys, see below

# Check the keys you've set up
aip check-keys

# To proofread your README.md (namespace: demo, pack_name: proof):
aip run demo@proof -f ./README.md

# You can just use @pack_name if there is no other pack with that name
aip run @proof -f ./README.md

```

### Thanks to

- **[Stephane Philipakis](https://github.com/sphilipakis)**, a key [aipack](https://aipack.ai) collaborator.
- [savi0ur](https://github.com/pratraut) for [PR #67 - ^ run - Add 'allow_run_on_task_fail' option](https://github.com/aipack-ai/aipack/pull/67)
- [Diaa Kasem](https://github.com/diaakasem) for `--non-interactive`/`--ni` (Now, in `v0.7.x` `-s` or `--single-shot`) mode ([#28](https://github.com/aipack-ai/aipack/pull/28))


### `pro@coder`

- You can install `pro@coder` with `aip install pro@coder`, and then
- Run it with `aip run pro@coder` or `aip run @coder` if you don't have any other `@coder` pack in a different namespace.

This is the agent I use every day for my production coding.

**IMPORTANT 1**: Make sure everything is committed before use, especially while you are learning AIPack.

**IMPORTANT 2**: Make sure you have your **API\_KEY** set as an environment variable. On macOS, there is experimental keychain support.

```
OPENAI_API_KEY
ANTHROPIC_API_KEY
GEMINI_API_KEY
FIREWORKS_API_KEY
TOGETHER_API_KEY
NEBIUS_API_KEY
XAI_API_KEY
DEEPSEEK_API_KEY
GROQ_API_KEY
COHERE_API_KEY
```

## More Info

- Website: https://aipack.ai

- [AIPack Overview Video](https://www.youtube.com/watch?v=SioUg_N9HS0)

- [Preview 'devai' intro video for v0.5](https://www.youtube.com/watch?v=b3LJcNkhkH4&list=PL7r-PXl6ZPcBcLsBdBABOFUuLziNyigqj)

- Built on top of the [Rust genai library](https://crates.io/crates/genai), which supports many top AI providers and models (OpenAI, Anthropic, Gemini, DeepSeek, Groq, Ollama, xAI, and Cohere).

- See full [CHANGELOG](CHANGELOG.md)

## How it works

- **One Agent** == **One Markdown**
    - An `.aip` agent file is just a **Markdown file** with sections for each stage of the agent's processing.
    - See below for all the [possible stages](#multi-stage).
- `aip run demo@proof -f "./*.md"`
    - will run the installed agent file `main.aip` in the
    - pack named `proof`
    - namespace `demo`
    - agent file `main.aip`
    - Full path `~/.aipack-base/pack/installed/demo/proof/main.aip`
    - You can pass input to your agent using:
        - `-f "path/with/optional/**/glob.*" -f "README.md"` (the Lua code will receive a `{path = .., name =..}` FileInfo-like structure as input)
        - `-i "some string" -i "another input"` (the Lua code will receive these strings as input)
    - Each input triggers one run of the agent.
- `aip run some/path/to/agent`
    - If the path ends with `.aip`, it's treated as a direct file run.
    - If there is no `.aip` extension, then:
        - `.../agent.aip` will be executed if it exists.
        - or `.../agent/main.aip` will be executed if it exists.
- **AIPack** agents are simple `.aip` files that can be placed anywhere on disk.
    - e.g., `aipack run ./my-path/to/my-agent.aip ...`
- **Multi AI Provider / Models** - **AIPack** uses [genai](https://crates.io/crates/genai) and therefore supports providers like OpenAI, Anthropic, Gemini, Groq, Ollama, Cohere, and more.
- **Lua** is used for all scripting (thanks to the great [mlua](https://crates.io/crates/mlua) crate).
- **Handlebars** is used for all prompt templating (thanks to the great Rust native [handlebars](https://crates.io/crates/handlebars) crate).

### Multi Stage

A single **AIPack** file may comprise any of the following stages.

| Stage           | Language       | Description                                                                                                                      |
|-----------------|----------------|----------------------------------------------------------------------------------------------------------------------------------|
| `# Before All`  | **Lua**        | Reshape/generate inputs and add global data to the command scope (the "map" part of map/reduce).                                 |
| `# Data`        | **Lua**        | Gather additional data per input and return it for the next stages.                                                              |
| `# System`      | **Handlebars** | Customize the system prompt using data from `# Before All` and `# Data`.                                                         |
| `# Instruction` | **Handlebars** | Customize the instruction prompt using data from `# Before All` and `# Data`.                                                    |
| `# Assistant`   | **Handlebars** | Optional for special customizations, such as the "Jedi Mind Trick."                                                              |
| `# Output`      | **Lua**        | Processes the `ai_response` from the LLM. If not defined, `ai_response.content` is output to the terminal.                       |
| `# After All`   | **Lua**        | Called with `inputs` and `outputs` for post-processing after all input processing is complete (the "reduce" part of map/reduce). |

- `# Before All` and `# After All` act like the **map** and **reduce** steps, running before and after the main input processing loop, respectively.

[more info on stages](_init/base/pack/installed/core/doc/README.md#complete-stages-description)

## [aipack doc](_init/base/pack/installed/core/doc/README.md)

See the AIPack documentation at **[core/doc/README.md](_init/base/pack/installed/core/doc/README.md)** (with the [Lua modules doc](_init/base/pack/installed/core/doc/lua.md)).

You can also run the `ask-aipack` agent.

```sh
# IMPORTANT: Make sure you have the `OPENAI_API_KEY` (or the key for your desired model) set in your environment.
aip run core@ask-aipack
# The prompt file will be at `.aipack/.prompt/core@ask-aipack/ask-prompt.md`
```
