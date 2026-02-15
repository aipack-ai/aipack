## aip.flow

Functions for controlling the AIPack agent execution flow from within script blocks (`before_all`, `data`).

### Functions Summary

```lua
aip.flow.before_all_response(data: BeforeAllData) -> table

aip.flow.data_response(data: DataData) -> table

aip.flow.skip(reason?: string): table
```

These functions return special tables that instruct the agent executor how to proceed. They should be the return value of the script block.

### aip.flow.before_all_response

Customizes execution flow at the 'Before All' stage (in `before_all` script block).

```lua
-- API Signature
aip.flow.before_all_response(data: BeforeAllData) -> table
```

This function is typically called within the `before_all` block of an agent script
to override the default behavior of passing all initial inputs to the agent.

#### Arguments

- `data: table` - A table defining the new inputs and options for the agent execution cycle.
  ```ts
  type BeforeAllData = {
    inputs?:  any[],        // Optional. A list of new inputs to use for the agent run cycle. Overrides initial inputs.
    options?: AgentOptions, // Optional. Partial AgentOptions to override for this run.
    before_all?: any,       // Optional. The before_all data that can be access via before_all...
  } & any // Can also include other arbitrary data fields if needed.
  ```
  related types: [AgentOptions](#agentoptions)

#### Example

```lua
local result = aip.flow.before_all_response({
  inputs = {"processed_input_1", "processed_input_2"},
  options = {
    model = "gemini-2.5-flash",
    input_concurrency = 3
  },
  before_all = {some_data = "hello world" } -- Arbitrary data is allowed
})
-- The agent executor will process this result table.
```

#### Error

This function does not directly return any errors. Errors might occur during the creation of lua table.

### aip.flow.data_response

Customizes execution flow at the 'Data' stage for a single input (in `data` script block).

```lua
-- API Signature
aip.flow.data_response(data: DataData) -> table
```

This function is typically called within the `data` block of an agent script.
It allows overriding the input and/or options for the current input cycle,
or returning additional arbitrary data.

#### Arguments

- `data: table` - A table defining the new input, options, and/or other data for the current cycle.
  ```ts
  type DataData = {
    input?: any | nil,         // Optional. The new input to use for this cycle. If nil, the original input is used.
    data?: any | nil,          // Optional. Data that will be available in the next stage. Same as returning a simple data.
    options?: AgentOptions,    // Optional. Partial AgentOptions to override for this cycle.
    attachments?: Attachments  // Optional. Allows to attach images and pdf to the prompt. 
  } & any // Can also include other arbitrary data fields (e.g., computed values, flags)
  ```
  related types: [AgentOptions](#agentoptions), [Attachments](#attachments)

#### Example

```lua
-- Use a transformed input and override the model for this cycle
return aip.flow.data_response({
  data  = data,              -- The data that would have been returned
  input = transformed_input,
  options = { model = "gpt-5-mini" },
})
-- The agent executor will process this result table.
```

#### Error

This function does not directly return any errors. Errors might occur during the creation of lua table.

related types: [AgentOptions](#agentoptions), [Attachments](#attachments)

### aip.flow.skip

Skips processing the current input cycle (in `data` script block).

```lua
-- API Signature
aip.flow.skip(reason?: string): table
```

This function is typically called within the `data` block of an agent script
to instruct AIPack to skip processing the current input value and move to the next one.

#### Arguments

- `reason: string (optional)`: An optional string providing the reason for skipping the input cycle.
  This reason might be logged or displayed depending on the AIPack execution context.

#### Example

```lua
-- Skip processing if the input is nil or empty
if input == nil or input == "" then
  return aip.flow.skip("Input is empty")
end
-- Continue processing the input if not skipped
-- ... rest of data block ...
```

#### Error

This function does not directly return any errors. Errors might occur during the creation of lua table.
