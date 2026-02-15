## aip.hash

The `aip.hash` module exposes functions for various hashing algorithms and encodings.

### Functions Summary

```lua
aip.hash.sha256(input: string): string
aip.hash.sha256_b58(input: string): string
aip.hash.sha256_b64(input: string): string
aip.hash.sha256_b64u(input: string): string
aip.hash.sha512(input: string): string
aip.hash.sha512_b58(input: string): string
aip.hash.sha512_b64(input: string): string
aip.hash.sha512_b64u(input: string): string
aip.hash.blake3(input: string): string
aip.hash.blake3_b58(input: string): string
aip.hash.blake3_b64(input: string): string
aip.hash.blake3_b64u(input: string): string
```

### aip.hash.sha256

Computes the SHA256 hash of the input string and returns it as a lowercase hex-encoded string.

```lua
-- API Signature
aip.hash.sha256(input: string): string
```

#### Arguments

- `input: string`: The string to hash.

#### Returns

`string`: The SHA256 hash, hex-encoded.

#### Example

```lua
local hex_hash = aip.hash.sha256("hello world")
-- hex_hash will be "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
print(hex_hash)
```

#### Error

This function does not typically error if the input is a string.

### aip.hash.sha256_b58

Computes the SHA256 hash of the input string and returns it as a Base58-encoded string.

```lua
-- API Signature
aip.hash.sha256_b58(input: string): string
```

#### Arguments

- `input: string`: The string to hash.

#### Returns

`string`: The SHA256 hash, Base58-encoded.

#### Example

```lua
local b58_hash = aip.hash.sha256_b58("hello world")
-- b58_hash will be "CnKqR4x6r4nAv2iGk8DrZSSWp7n3W9xKRj69eZysS272"
print(b58_hash)
```

#### Error

This function does not typically error if the input is a string.

### aip.hash.sha256_b64

Computes the SHA256 hash of the input string and returns it as a standard Base64-encoded string (RFC 4648).

```lua
-- API Signature
aip.hash.sha256_b64(input: string): string
```

#### Arguments

- `input: string`: The string to hash.

#### Returns

`string`: The SHA256 hash, standard Base64-encoded.

#### Example

```lua
local b64_hash = aip.hash.sha256_b64("hello world")
-- b64_hash will be "uU0nuZNNPgilLlLX2n2r+sSE7+N6U4DukIj3rOLvzek="
print(b64_hash)
```

#### Error

This function does not typically error if the input is a string.

### aip.hash.sha256_b64u

Computes the SHA256 hash of the input string and returns it as a URL-safe Base64-encoded string (RFC 4648, section 5), without padding.

```lua
-- API Signature
aip.hash.sha256_b64u(input: string): string
```

#### Arguments

- `input: string`: The string to hash.

#### Returns

`string`: The SHA256 hash, URL-safe Base64-encoded without padding.

#### Example

```lua
local b64u_hash = aip.hash.sha256_b64u("hello world")
-- b64u_hash will be "uU0nuZNNPgilLlLX2n2r-sSE7-N6U4DukIj3rOLvzek"
print(b64u_hash)
```

#### Error

This function does not typically error if the input is a string.

### aip.hash.sha512

Computes the SHA512 hash of the input string and returns it as a lowercase hex-encoded string.

```lua
-- API Signature
aip.hash.sha512(input: string): string
```

#### Arguments

- `input: string`: The string to hash.

#### Returns

`string`: The SHA512 hash, hex-encoded.

#### Example

```lua
local hex_hash = aip.hash.sha512("hello world")
-- hex_hash will be "309ecc489c12d6eb4cc40f50c902f2b4d0ed77ee511a7c7a9bcd3ca86d4cd86f989dd35bc5ff499670da34255b45b0cfd830e81f605dcf7dc5542e93ae9cd76f"
print(hex_hash)
```

#### Error

This function does not typically error if the input is a string.

### aip.hash.sha512_b58

Computes the SHA512 hash of the input string and returns it as a Base58-encoded string.

```lua
-- API Signature
aip.hash.sha512_b58(input: string): string
```

#### Arguments

- `input: string`: The string to hash.

#### Returns

`string`: The SHA512 hash, Base58-encoded.

#### Example

```lua
local b58_hash = aip.hash.sha512_b58("hello world")
-- b58_hash will be "yP4cqy7jmaRDzC2bmcGNZkuQb3VdftMk6YH7ynQ2Qw4zktKsyA9fk52xghNQNAdkpF9iFmFkKh2bNVG4kDWhsok"
print(b58_hash)
```

#### Error

This function does not typically error if the input is a string.

### aip.hash.sha512_b64

Computes the SHA512 hash of the input string and returns it as a standard Base64-encoded string (RFC 4648).

```lua
-- API Signature
aip.hash.sha512_b64(input: string): string
```

#### Arguments

- `input: string`: The string to hash.

#### Returns

`string`: The SHA512 hash, standard Base64-encoded.

#### Example

```lua
local b64_hash = aip.hash.sha512_b64("hello world")
-- b64_hash will be "MJ7MSJwS1utMxA9QyQLytNDtd+5RGnx6m808qG1M2G+YndNbxf9JlnDaNCVbRbDP2DDoH2Bdz33FVC6TrpzXbw=="
print(b64_hash)
```

#### Error

This function does not typically error if the input is a string.

### aip.hash.sha512_b64u

Computes the SHA512 hash of the input string and returns it as a URL-safe Base64-encoded string (RFC 4648, section 5), without padding.

```lua
-- API Signature
aip.hash.sha512_b64u(input: string): string
```

#### Arguments

- `input: string`: The string to hash.

#### Returns

`string`: The SHA512 hash, URL-safe Base64-encoded without padding.

#### Example

```lua
local b64u_hash = aip.hash.sha512_b64u("hello world")
-- b64u_hash will be "MJ7MSJwS1utMxA9QyQLytNDtd-5RGnx6m808qG1M2G-YndNbxf9JlnDaNCVbRbDP2DDoH2Bdz33FVC6TrpzXbw"
print(b64u_hash)
```

#### Error

This function does not typically error if the input is a string.

### aip.hash.blake3

Computes the Blake3 hash of the input string and returns it as a lowercase hex-encoded string.

```lua
-- API Signature
aip.hash.blake3(input: string): string
```

#### Arguments

- `input: string`: The string to hash.

#### Returns

`string`: The Blake3 hash, hex-encoded.

#### Example

```lua
local hex_hash = aip.hash.blake3("hello world")
-- hex_hash will be "d74981efa70a0c880b8d8c1985d075dbcbf679b99a5f9914e5aaf96b831a9e24"
print(hex_hash)
```

#### Error

This function does not typically error if the input is a string.

### aip.hash.blake3_b58

Computes the Blake3 hash of the input string and returns it as a Base58-encoded string.

```lua
-- API Signature
aip.hash.blake3_b58(input: string): string
```

#### Arguments

- `input: string`: The string to hash.

#### Returns

`string`: The Blake3 hash, Base58-encoded.

#### Example

```lua
local b58_hash = aip.hash.blake3_b58("hello world")
-- b58_hash will be "FVPfbg9bK7mj7jnaSRXhuVcVakkXcjMPgSwxmauUofYf"
print(b58_hash)
```

#### Error

This function does not typically error if the input is a string.

### aip.hash.blake3_b64

Computes the Blake3 hash of the input string and returns it as a standard Base64-encoded string (RFC 4648).

```lua
-- API Signature
aip.hash.blake3_b64(input: string): string
```

#### Arguments

- `input: string`: The string to hash.

#### Returns

`string`: The Blake3 hash, standard Base64-encoded.

#### Example

```lua
local b64_hash = aip.hash.blake3_b64("hello world")
-- b64_hash will be "10mB76cKDIgLjYwZhdB128v2ebmaX5kU5ar5a4ManiQ="
print(b64_hash)
```

#### Error

This function does not typically error if the input is a string.

### aip.hash.blake3_b64u

Computes the Blake3 hash of the input string and returns it as a URL-safe Base64-encoded string (RFC 4648, section 5), without padding.

```lua
-- API Signature
aip.hash.blake3_b64u(input: string): string
```

#### Arguments

- `input: string`: The string to hash.

#### Returns

`string`: The Blake3 hash, URL-safe Base64-encoded without padding.

#### Example

```lua
local b64u_hash = aip.hash.blake3_b64u("hello world")
-- b64u_hash will be "10mB76cKDIgLjYwZhdB128v2ebmaX5kU5ar5a4ManiQ"
print(b64u_hash)
```

#### Error

This function does not typically error if the input is a string.
