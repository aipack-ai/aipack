## aip.semver

Functions for semantic versioning (SemVer 2.0.0) operations.

### Functions Summary

```lua
aip.semver.compare(version1: string, operator: string, version2: string): boolean | {error: string}

aip.semver.parse(version: string): {major: number, minor: number, patch: number, prerelease: string | nil, build: string | nil} | {error: string}

aip.semver.is_prerelease(version: string): boolean | {error: string}

aip.semver.valid(version: string): boolean
```

### aip.semver.compare

Compares two version strings using an operator.

```lua
-- API Signature
aip.semver.compare(version1: string, operator: string, version2: string): boolean | {error: string}
```

Compares versions according to SemVer rules (prerelease < release, build metadata ignored).

#### Arguments

- `version1: string`: First version string.
- `operator: string`: Comparison operator (`<`, `<=`, `=`, `==`, `>=`, `>`, `!=`, `~=`).
- `version2: string`: Second version string.

#### Returns

- `boolean`: `true` if the comparison holds, `false` otherwise.
- `{error: string}`: An error table on failure.

#### Example

```lua
print(aip.semver.compare("1.2.3", ">", "1.2.0"))     -- Output: true
print(aip.semver.compare("1.0.0-alpha", "<", "1.0.0")) -- Output: true
print(aip.semver.compare("1.0.0+build", "==", "1.0.0")) -- Output: true

local r = aip.semver.compare("abc", ">", "1.0.0")
if type(r) == "table" and r.error then
  print("Error:", r.error)
end
```

#### Error

Returns an error (Lua table `{ error: string }`) if operator is invalid or versions are not valid SemVer strings.

### aip.semver.parse

Parses a version string into its components.

```lua
-- API Signature
aip.semver.parse(version: string): {major: number, minor: number, patch: number, prerelease: string | nil, build: string | nil} | {error: string}
```

#### Arguments

- `version: string`: The SemVer string to parse.

#### Returns

- `table`: A table containing `major`, `minor`, `patch`, `prerelease` (string or nil), and `build` (string or nil) components.
- `{error: string}`: An error table on failure.

#### Example

```lua
local v = aip.semver.parse("1.2.3-beta.1+build.123")
print(v.major, v.minor, v.patch) -- Output: 1 2 3
print(v.prerelease)             -- Output: beta.1
print(v.build)                  -- Output: build.123

local r = aip.semver.parse("invalid")
if type(r) == "table" and r.error then
  print("Error:", r.error)
end
```

#### Error

Returns an error (Lua table `{ error: string }`) if `version` is not a valid SemVer string.

### aip.semver.is_prerelease

Checks if a version string has a prerelease component.

```lua
-- API Signature
aip.semver.is_prerelease(version: string): boolean | {error: string}
```

#### Arguments

- `version: string`: The SemVer string to check.

#### Returns

- `boolean`: `true` if it has a prerelease component (e.g., `-alpha`), `false` otherwise.
- `{error: string}`: An error table on failure.

#### Example

```lua
print(aip.semver.is_prerelease("1.2.3-beta"))      -- Output: true
print(aip.semver.is_prerelease("1.2.3"))         -- Output: false
print(aip.semver.is_prerelease("1.0.0+build")) -- Output: false

local r = aip.semver.is_prerelease("invalid")
if type(r) == "table" and r.error then
  print("Error:", r.error)
end
```

#### Error

Returns an error (Lua table `{ error: string }`) if `version` is not a valid SemVer string.

### aip.semver.valid

Checks if a string is a valid SemVer 2.0.0 version.

```lua
-- API Signature
aip.semver.valid(version: string): boolean
```

#### Arguments

- `version: string`: The string to validate.

#### Returns

- `boolean`: `true` if valid, `false` otherwise.

#### Example

```lua
print(aip.semver.valid("1.2.3"))          -- Output: true
print(aip.semver.valid("1.2.3-alpha.1"))   -- Output: true
print(aip.semver.valid("1.0"))           -- Output: false
print(aip.semver.valid("invalid"))       -- Output: false
```

#### Error

This function does not typically error, returning `false` for invalid formats.
