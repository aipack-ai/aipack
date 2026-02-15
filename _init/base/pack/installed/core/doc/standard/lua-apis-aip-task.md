## aip.task

Functions for recording pins and updating task metadata (attached to the current task of the current run).

### Functions Summary

```lua
aip.task.set_label(label: string)
aip.task.pin(iden: string, content: string | Marker)
aip.task.pin(iden: string, priority: number, content: string | Marker)
```

### aip.task.set_label

Sets a new human-readable label for the current task.

```lua
-- API Signature
aip.task.set_label(label: string)
```

#### Arguments

- `label: string`: The new label string for the task.

#### Returns

- Nothing. This function records the update as a side effect.

#### Example

```lua
aip.task.set_label("Task: Validate inputs")
```

#### Error

Returns an error (Lua table `{ error: string }`) if called outside a task context or if arguments are invalid.

### aip.task.pin

Creates a pin attached to the current task. Requires that both [CTX](#ctx).RUN_UID and [CTX](#ctx).TASK_UID are available (i.e., must be called during a task cycle, not in before_all or after_all).

```lua
-- API Signatures
aip.task.pin(iden: string, content: string | Marker)
aip.task.pin(iden: string, priority: number, content: string | Marker)
```

Records a pin for the current task. When the optional priority is provided, it will be stored along with the pin.

#### Arguments

- `iden: string`
  Identifier (name) for this pin.

- `priority: number (optional)`
  Optional numeric priority to associate with the pin.

- `content: string | Marker`
  The content to associate with the pin. This can be:
  - A simple string value (or other primitive value convertible to a string) to be stored as content.
  - A structured [Marker](#marker) to provide a display label and content.

#### Returns

- Nothing. This function records the pin as a side effect.

#### Example

```lua
-- Simple pin (no priority)
aip.task.pin("review", "Needs follow-up")

-- Pin with priority
aip.task.pin("checkpoint", 0.7, { step = 3, note = "after validation" })
```

#### Error

Returns an error (Lua table `{ error: string }`) if called outside a task context (no `CTX.TASK_UID`), if there is no run context, or if arguments are invalid.
