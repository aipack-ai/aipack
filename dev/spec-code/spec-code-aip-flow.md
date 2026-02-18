# AIPack Flow Specification

This document describes the `aip.flow` module and how it controls the execution lifecycle of an Agent.

## Overview

The `aip.flow` module provides functions that return "Directive Tables". These tables are recognized by the Agent Executor to alter the normal sequential flow of stages.

When a directive is returned:
1. It immediately interrupts the current stage.
2. It propagates back to the Agent Executor.
3. The Executor performs the requested action (Skip, Redo, etc.).

## Directives

### `aip.flow.skip(reason?)`

Instructs the executor to stop processing the current task and move to the next one.
- **BeforeAll**: Skips the entire run.
- **Data**: Skips the current input item.
- **Output**: Skips finishing the current task (output is recorded as a skip).

### `aip.flow.redo()`

Instructs the executor to finish the current active tasks and then restart the entire agent execution from the beginning.
- This is useful for self-correcting agents or when the agent needs to reload its own code/configuration.

### `aip.flow.before_all_response(data)`

Used in `before_all` to override inputs or options for the run.

### `aip.flow.data_response(data)`
Used in `data` to override the input or options for a single task.

## Flow Interruption Logic

### Stage-based behavior

- **BeforeAll**: If `skip` or `redo` is returned, the `Data` and `Ai` stages are never entered.
- **Data**: If `skip` or `redo` is returned for a specific input, the `Ai` and `Output` stages for that input are bypassed.
- **Output**: If `redo` is returned, the task is marked completed, and the redo flag is raised for the overall run.
- **AfterAll**: If `redo` is returned, the overall run is marked for redo.

### Propagation
Directives are wrapped in an `_aipack_` internal table structure:

```json
{
  "_aipack_": {
    "kind": "Redo"
  }
}
```

The runtime detects these patterns in the values returned by Lua scripts and executes the corresponding logic.

