## aip.uuid

The `aip.uuid` module exposes functions for generating various UUIDs and converting timestamped UUIDs.

### Functions Summary

```lua
aip.uuid.new(): string
aip.uuid.new_v4(): string
aip.uuid.new_v7(): string
aip.uuid.new_v4_b64(): string
aip.uuid.new_v4_b64u(): string
aip.uuid.new_v4_b58(): string
aip.uuid.new_v7_b64(): string
aip.uuid.new_v7_b64u(): string
aip.uuid.new_v7_b58(): string
aip.uuid.to_time_epoch_ms(value: string | nil): integer | nil
```

### aip.uuid.new

Generates a new UUID version 4. This is an alias for `aip.uuid.new_v4()`.

```lua
-- API Signature
aip.uuid.new(): string
```

#### Returns

`string`: The generated UUIDv4 as a string (e.g., "f47ac10b-58cc-4372-a567-0e02b2c3d479").

#### Example

```lua
local id = aip.uuid.new()
print(id)
```

### aip.uuid.new_v4

Generates a new UUID version 4.

```lua
-- API Signature
aip.uuid.new_v4(): string
```

#### Returns

`string`: The generated UUIDv4 as a string (e.g., "f47ac10b-58cc-4372-a567-0e02b2c3d479").

#### Example

```lua
local id_v4 = aip.uuid.new_v4()
print(id_v4)
```

### aip.uuid.new_v7

Generates a new UUID version 7 (time-ordered).

```lua
-- API Signature
aip.uuid.new_v7(): string
```

#### Returns

`string`: The generated UUIDv7 as a string.

#### Example

```lua
local id_v7 = aip.uuid.new_v7()
print(id_v7)
```

### aip.uuid.new_v4_b64

Generates a new UUID version 4 and encodes it using standard Base64.

```lua
-- API Signature
aip.uuid.new_v4_b64(): string
```

#### Returns

`string`: The Base64 encoded UUIDv4 string.

#### Example

```lua
local id_v4_b64 = aip.uuid.new_v4_b64()
print(id_v4_b64)
```

### aip.uuid.new_v4_b64u

Generates a new UUID version 4 and encodes it using URL-safe Base64 without padding.

```lua
-- API Signature
aip.uuid.new_v4_b64u(): string
```

#### Returns

`string`: The URL-safe Base64 encoded (no padding) UUIDv4 string.

#### Example

```lua
local id_v4_b64u = aip.uuid.new_v4_b64u()
print(id_v4_b64u)
```

### aip.uuid.new_v4_b58

Generates a new UUID version 4 and encodes it using Base58.

```lua
-- API Signature
aip.uuid.new_v4_b58(): string
```

#### Returns

`string`: The Base58 encoded UUIDv4 string.

#### Example

```lua
local id_v4_b58 = aip.uuid.new_v4_b58()
print(id_v4_b58)
```

### aip.uuid.new_v7_b64

Generates a new UUID version 7 and encodes it using standard Base64.

```lua
-- API Signature
aip.uuid.new_v7_b64(): string
```

#### Returns

`string`: The Base64 encoded UUIDv7 string.

#### Example

```lua
local id_v7_b64 = aip.uuid.new_v7_b64()
print(id_v7_b64)
```

### aip.uuid.new_v7_b64u

Generates a new UUID version 7 and encodes it using URL-safe Base64 without padding.

```lua
-- API Signature
aip.uuid.new_v7_b64u(): string
```

#### Returns

`string`: The URL-safe Base64 encoded (no padding) UUIDv7 string.

#### Example

```lua
local id_v7_b64u = aip.uuid.new_v7_b64u()
print(id_v7_b64u)
```

### aip.uuid.new_v7_b58

Generates a new UUID version 7 and encodes it using Base58.

```lua
-- API Signature
aip.uuid.new_v7_b58(): string
```

#### Returns

`string`: The Base58 encoded UUIDv7 string.

#### Example

```lua
local id_v7_b58 = aip.uuid.new_v7_b58()
print(id_v7_b58)
```

### aip.uuid.to_time_epoch_ms

Converts a timestamped UUID string (V1, V6, V7) to milliseconds since Unix epoch.
Returns `nil` if the input is `nil`, not a valid UUID string, or if the UUID type
does not contain an extractable timestamp (e.g., V4).

```lua
-- API Signature
aip.uuid.to_time_epoch_ms(value: string | nil): integer | nil
```

#### Arguments

- `value: string | nil`: The UUID string or `nil`.

#### Returns

`integer | nil`: Milliseconds since Unix epoch, or `nil`.

#### Example

```lua
local v7_uuid_str = aip.uuid.new_v7()
local millis = aip.uuid.to_time_epoch_ms(v7_uuid_str)
if millis then
  print("Timestamp in ms: " .. millis)
else
  print("Could not extract timestamp.")
end

local v4_uuid_str = aip.uuid.new_v4()
local millis_v4 = aip.uuid.to_time_epoch_ms(v4_uuid_str)
-- millis_v4 will be nil

local invalid_millis = aip.uuid.to_time_epoch_ms("not-a-uuid")
-- invalid_millis will be nil

local nil_millis = aip.uuid.to_time_epoch_ms(nil)
-- nil_millis will be nil
```