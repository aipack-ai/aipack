## aip.agent

Functions for running other AIPack agents from within a Lua script.

### Functions Summary

```lua
aip.agent.run(agent_name: string, options?: table): any

aip.agent.extract_options(value: any): AgentOptions | nil
```

### aip.agent.run

Runs another agent and returns its response.

```lua
-- API Signature
aip.agent.run(agent_name: string, options?: table): any
```

Executes the agent specified by `agent_name`. The function waits for the called agent
to complete and returns its result. This allows for chaining agents together.

#### Arguments

- `agent_name: string`: The name of the agent to run. This can be a relative path
  (e.g., `"../my-other-agent.aip"`) or a fully qualified pack reference
  (e.g., `"my-ns@my-pack/feature/my-agent.aip"`). Relative paths are resolved
  from the directory of the calling agent.
- `options?: table`: An optional table containing input data and agent options.
  - `input?: any`: (since 0.8.15) A single input value of any type.
  - `inputs?: list`: A list of inputs for the agent. Each element can be a string, a [FileInfo](#fileinfo), a [FileRecord](#filerecord), or a structured table.
    Note: If both `input` and `inputs` are provided, `input` is prepended to the `inputs` list.
  - `options?: AgentOptions`: Agent-specific options. These options are passed directly to the called agent's
    execution environment and can override settings defined in the called agent's `.aip` file.
  - `agent_base_dir?: string`: (since 0.8.15) The base directory used to resolve relative agent paths.
    By default, it is the directory of the caller agent. If provided, it overrides the default (e.g.,
    using `CTX.WORKSPACE_DIR`). Note that pack references (e.g., `ns@pack/`) are still resolved to
    their pack path regardless of this base directory.

##### Input Examples:

```lua
-- Run an agent with a single string input
local response = aip.agent.run("agent-name", { inputs = {"hello"} })

-- Run an agent with multiple string inputs
local response = aip.agent.run("agent-name", { inputs = {"input1", "input2"} })

-- Run an agent with structured inputs (e.g., FileRecord)
local response = aip.agent.run("agent-name", {
  inputs = {
    { path = "file1.txt", content = "..." },
    { path = "file2.txt", content = "..." }
  }
})
```

### Returns

Returns a `RunAgentResponse` table containing the outputs from the agent's stages.

```ts
{
  outputs: any[],   // List of values returned by each # Output stage for each input.
  after_all: any    // The value returned by the # After All stage (or nil).
}
```

#### Error


Returns an error if:
- The `agent_name` is invalid or the agent file cannot be located/loaded.
- The options table contains invalid parameters.
- The execution of the called agent fails.
- An internal communication error occurs while waiting for the agent's result.

```ts
{
  error: string // Error message
}
```

### aip.agent.extract_options

Extracts relevant agent options from a given Lua value.

```lua
-- API Signature
aip.agent.extract_options(value: any): AgentOptions | nil
```

If the input `value` is a Lua table, this function creates a new [AgentOptions](#agentoptions) table 
and copies the following properties if they exist in the input table:

- `model`
- `model_aliases`
- `input_concurrency`
- `temperature`

Other properties are ignored. If the input `value` is `nil` or not a table,
the function returns `nil`.

#### Arguments

- `value: any`: The Lua value from which to extract options.

#### Returns

A new Lua table containing the extracted options, or `nil` if the input
was `nil` or not a table.
