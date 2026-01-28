# Encoding Module

Binary-to-text encoding and decoding operations.

## Overview

The encoding module provides safe encoding and decoding between binary data and text representations. Essential for data interchange, URLs, and human-readable formats.

## Supported Encodings

| Encoding | Expansion | Use Case |
|----------|-----------|----------|
| Base64 | 4:3 | Binary in text (email, JSON, configs) |
| Base64 URL-safe | 4:3 | URLs, filenames |
| Hex | 2:1 | Debugging, hashes, simple encoding |
| URL encoding | Variable | Query strings, form data |

## API Reference

### Base64

```rust
/// Encode data as base64
pub fn base64_encode(input: &OwnedBuffer) -> OwnedBuffer;

/// Decode base64 to binary
pub fn base64_decode(input: &OwnedBuffer) -> WrapperResult<OwnedBuffer>;

/// Encode as URL-safe base64 (uses - and _ instead of + and /)
pub fn base64_encode_url(input: &OwnedBuffer) -> OwnedBuffer;

/// Decode URL-safe base64
pub fn base64_decode_url(input: &OwnedBuffer) -> WrapperResult<OwnedBuffer>;
```

### Hex

```rust
/// Encode data as lowercase hex
pub fn hex_encode(input: &OwnedBuffer) -> OwnedBuffer;

/// Encode data as uppercase hex
pub fn hex_encode_upper(input: &OwnedBuffer) -> OwnedBuffer;

/// Decode hex to binary
pub fn hex_decode(input: &OwnedBuffer) -> WrapperResult<OwnedBuffer>;
```

### URL Encoding

```rust
/// URL-encode a string (percent encoding)
pub fn url_encode(input: &OwnedBuffer) -> OwnedBuffer;

/// URL-decode a string
pub fn url_decode(input: &OwnedBuffer) -> WrapperResult<OwnedBuffer>;

/// URL-encode for path segments (stricter)
pub fn url_encode_path(input: &OwnedBuffer) -> OwnedBuffer;

/// URL-encode for query parameters
pub fn url_encode_query(input: &OwnedBuffer) -> OwnedBuffer;
```

## Usage Examples

### Base64 Encoding

```rust
use neurlang::wrappers::{OwnedBuffer, encoding};

// Encode binary data
let binary = OwnedBuffer::from_slice(&[0x00, 0x01, 0x02, 0xFF]);
let encoded = encoding::base64_encode(&binary);
assert_eq!(encoded.as_str().unwrap(), "AAEC/w==");

// Decode back
let decoded = encoding::base64_decode(&encoded)?;
assert_eq!(decoded.as_slice(), &[0x00, 0x01, 0x02, 0xFF]);
```

### URL-Safe Base64

```rust
// Standard base64 has + and / which are problematic in URLs
let data = OwnedBuffer::from_slice(&[0xFB, 0xFF]);
let standard = encoding::base64_encode(&data);
assert_eq!(standard.as_str().unwrap(), "+/8=");

// URL-safe uses - and _ instead
let url_safe = encoding::base64_encode_url(&data);
assert_eq!(url_safe.as_str().unwrap(), "-_8=");
```

### Hex Encoding

```rust
// Encode
let data = OwnedBuffer::from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);
let hex = encoding::hex_encode(&data);
assert_eq!(hex.as_str().unwrap(), "deadbeef");

// Uppercase
let hex_upper = encoding::hex_encode_upper(&data);
assert_eq!(hex_upper.as_str().unwrap(), "DEADBEEF");

// Decode
let decoded = encoding::hex_decode(&hex)?;
assert_eq!(decoded.as_slice(), &[0xDE, 0xAD, 0xBE, 0xEF]);
```

### URL Encoding

```rust
// Encode special characters
let query = OwnedBuffer::from_str("name=John Doe&city=New York");
let encoded = encoding::url_encode(&query);
assert_eq!(encoded.as_str().unwrap(), "name%3DJohn%20Doe%26city%3DNew%20York");

// Decode
let decoded = encoding::url_decode(&encoded)?;
assert_eq!(decoded.as_str().unwrap(), "name=John Doe&city=New York");
```

## IR Assembly Usage

```asm
; Base64
ext.call r0, @"base64 encode", r1
ext.call r0, @"base64 decode", r1

; URL-safe base64
ext.call r0, @"base64 encode url", r1
ext.call r0, @"base64 decode url", r1

; Hex
ext.call r0, @"hex encode", r1
ext.call r0, @"hex decode", r1

; URL encoding
ext.call r0, @"url encode", r1
ext.call r0, @"url decode", r1
```

## RAG Keywords

| Intent | Resolves To |
|--------|-------------|
| "base64", "b64", "encode base64" | `base64_encode` |
| "decode base64", "from base64" | `base64_decode` |
| "hex", "hexadecimal", "to hex" | `hex_encode` |
| "from hex", "unhex" | `hex_decode` |
| "url encode", "percent encode", "urlencode" | `url_encode` |
| "url decode", "percent decode", "urldecode" | `url_decode` |

## Error Handling

```rust
// Base64 decode errors
match encoding::base64_decode(&input) {
    Ok(data) => { /* use data */ }
    Err(WrapperError::EncodingError(msg)) => {
        // Invalid base64 (wrong length, invalid chars)
        eprintln!("Invalid base64: {}", msg);
    }
}

// Hex decode errors
match encoding::hex_decode(&input) {
    Ok(data) => { /* use data */ }
    Err(WrapperError::EncodingError(msg)) => {
        // Odd length or invalid hex character
        eprintln!("Invalid hex: {}", msg);
    }
}

// URL decode errors
match encoding::url_decode(&input) {
    Ok(data) => { /* use data */ }
    Err(WrapperError::EncodingError(msg)) => {
        // Invalid percent encoding
        eprintln!("Invalid URL encoding: {}", msg);
    }
}
```

## Encoding Comparison

| Encoding | Input | Output | Overhead |
|----------|-------|--------|----------|
| Base64 | `[0x00, 0xFF]` | `AP8=` | 33% |
| Hex | `[0x00, 0xFF]` | `00ff` | 100% |
| URL | `hello world` | `hello%20world` | Variable |

### When to Use What

- **Base64**: Binary data in JSON, XML, email, configs
- **Base64 URL-safe**: Tokens in URLs, filenames
- **Hex**: Hash display, debugging, simple cases
- **URL encoding**: Query strings, form data

## Implementation Notes

### Character Sets

**Base64 Standard (RFC 4648)**
```
ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/
Padding: =
```

**Base64 URL-safe (RFC 4648)**
```
ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_
Padding: = (optional)
```

**Hex**
```
0123456789abcdef (lowercase)
0123456789ABCDEF (uppercase)
```

### URL Encoding Rules

Characters NOT encoded:
- Alphanumeric: `A-Z a-z 0-9`
- Unreserved: `- _ . ~`

Characters encoded as `%XX`:
- Space: `%20`
- Reserved: `! # $ & ' ( ) * + , / : ; = ? @ [ ]`
- Other special characters

## Dependencies

```toml
[dependencies]
base64 = "0.21"   # Base64 encoding
hex = "0.4"       # Hex encoding
# URL encoding uses Rust's standard percent-encoding
```

## See Also

- [Buffer Module](buffer.md) - OwnedBuffer type
- [Compression Module](compression.md) - Reduce data size before encoding
