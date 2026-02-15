## aip.run

Functions for recording pins and updating run metadata (attached to the overall run).

### Functions Summary

```lua
aip.run.set_label(label: string)
aip.run.pin(iden: string, content: any)
aip.run.pin(iden: string, priority: number, content: any)
```

### aip.run.set_label

Sets a new human-readable label for the current run.

```lua
-- API Signature
aip.run.set_label(label: string)
```

#### Arguments

- `label: string`: The new label string for the run.

#### Returns

- Nothing. This function records the update as a side effect.

#### Example

```lua
aip.run.set_label("Run for project cleanup")
```

#### Error

Returns an error (Lua table `{ error: string }`) if called outside a run context or if arguments are invalid.

### aip.run.pin

Creates a pin attached to the current run. Requires that CTX.RUN_UID is available.

```lua
-- API Signatures
aip.run.pin(iden: string, content: string | {label?: string, content: string})
aip.run.pin(iden: string, priority: number, content: string | {label?: string, content: string})
```

Records a pin for the current run. When the optional priority is provided, it will be stored along with the pin.

#### Arguments

- `iden: string`
  Identifier (name) for this pin.

- `priority: number (optional)`
  Optional numeric priority to associate with the pin.

- `content: string | {label?: string, content: string}`
  The content to associate with the pin. This can be:
  - A simple string value (or other primitive value convertible to a string) to be stored as content.
  - A structured table `{label?: string, content: string}` to provide a display label and content.

#### Returns

- Nothing. This function records the pin as a side effect.

#### Example

```lua
-- Simple pin (no priority)
aip.run.pin("summary", "Run started successfully")

-- Pin with priority
aip.run.pin("quality-score", 0.85, { score = 0.85, rationale = "good coverage" })
```

#### Error

Returns an error (Lua table `{ error: string }`) if there is no run context (no `CTX.RUN_UID`) or if arguments are invalid.
