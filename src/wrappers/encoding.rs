//! Encoding Wrappers
//!
//! Binary-to-text encoding and decoding operations.
//!
//! # Supported Encodings
//!
//! - **Base64** - Binary in text (email, JSON, configs)
//! - **Base64 URL-safe** - URLs, filenames
//! - **Hex** - Debugging, hashes, simple encoding
//! - **URL encoding** - Query strings, form data

use base64::{engine::general_purpose, Engine as _};

use super::{OwnedBuffer, WrapperCategory, WrapperError, WrapperRegistry, WrapperResult};

// =============================================================================
// Base64
// =============================================================================

/// Encode data as base64
pub fn base64_encode(input: &OwnedBuffer) -> OwnedBuffer {
    let encoded = general_purpose::STANDARD.encode(input.as_slice());
    OwnedBuffer::from_string(encoded)
}

/// Decode base64 to binary
pub fn base64_decode(input: &OwnedBuffer) -> WrapperResult<OwnedBuffer> {
    let input_str = input
        .as_str()
        .map_err(|e| WrapperError::EncodingError(format!("Invalid UTF-8: {}", e)))?;

    let decoded = general_purpose::STANDARD
        .decode(input_str.trim())
        .map_err(|e| WrapperError::EncodingError(format!("Invalid base64: {}", e)))?;

    Ok(OwnedBuffer::from_vec(decoded))
}

/// Encode as URL-safe base64 (uses - and _ instead of + and /)
pub fn base64_encode_url(input: &OwnedBuffer) -> OwnedBuffer {
    let encoded = general_purpose::URL_SAFE.encode(input.as_slice());
    OwnedBuffer::from_string(encoded)
}

/// Decode URL-safe base64
pub fn base64_decode_url(input: &OwnedBuffer) -> WrapperResult<OwnedBuffer> {
    let input_str = input
        .as_str()
        .map_err(|e| WrapperError::EncodingError(format!("Invalid UTF-8: {}", e)))?;

    let decoded = general_purpose::URL_SAFE
        .decode(input_str.trim())
        .map_err(|e| WrapperError::EncodingError(format!("Invalid URL-safe base64: {}", e)))?;

    Ok(OwnedBuffer::from_vec(decoded))
}

/// Encode as base64 without padding
pub fn base64_encode_no_pad(input: &OwnedBuffer) -> OwnedBuffer {
    let encoded = general_purpose::STANDARD_NO_PAD.encode(input.as_slice());
    OwnedBuffer::from_string(encoded)
}

/// Decode base64 without padding
pub fn base64_decode_no_pad(input: &OwnedBuffer) -> WrapperResult<OwnedBuffer> {
    let input_str = input
        .as_str()
        .map_err(|e| WrapperError::EncodingError(format!("Invalid UTF-8: {}", e)))?;

    let decoded = general_purpose::STANDARD_NO_PAD
        .decode(input_str.trim())
        .map_err(|e| WrapperError::EncodingError(format!("Invalid base64: {}", e)))?;

    Ok(OwnedBuffer::from_vec(decoded))
}

// =============================================================================
// Hex
// =============================================================================

/// Encode data as lowercase hex
pub fn hex_encode(input: &OwnedBuffer) -> OwnedBuffer {
    let encoded = hex::encode(input.as_slice());
    OwnedBuffer::from_string(encoded)
}

/// Encode data as uppercase hex
pub fn hex_encode_upper(input: &OwnedBuffer) -> OwnedBuffer {
    let encoded = hex::encode_upper(input.as_slice());
    OwnedBuffer::from_string(encoded)
}

/// Decode hex to binary
pub fn hex_decode(input: &OwnedBuffer) -> WrapperResult<OwnedBuffer> {
    let input_str = input
        .as_str()
        .map_err(|e| WrapperError::EncodingError(format!("Invalid UTF-8: {}", e)))?;

    let decoded = hex::decode(input_str.trim())
        .map_err(|e| WrapperError::EncodingError(format!("Invalid hex: {}", e)))?;

    Ok(OwnedBuffer::from_vec(decoded))
}

// =============================================================================
// URL Encoding
// =============================================================================

/// URL-encode a string (percent encoding)
pub fn url_encode(input: &OwnedBuffer) -> OwnedBuffer {
    let input_str = match input.as_str() {
        Ok(s) => s,
        Err(_) => return OwnedBuffer::new(), // Can't URL-encode non-UTF8
    };

    let encoded: String = input_str
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '~' {
                c.to_string()
            } else if c == ' ' {
                "%20".to_string()
            } else {
                format!("%{:02X}", c as u32)
            }
        })
        .collect();

    OwnedBuffer::from_string(encoded)
}

/// URL-decode a string
pub fn url_decode(input: &OwnedBuffer) -> WrapperResult<OwnedBuffer> {
    let input_str = input
        .as_str()
        .map_err(|e| WrapperError::EncodingError(format!("Invalid UTF-8: {}", e)))?;

    let mut result = Vec::new();
    let mut chars = input_str.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if hex.len() == 2 {
                let byte = u8::from_str_radix(&hex, 16).map_err(|_| {
                    WrapperError::EncodingError(format!("Invalid percent encoding: %{}", hex))
                })?;
                result.push(byte);
            } else {
                return Err(WrapperError::EncodingError(
                    "Incomplete percent encoding".to_string(),
                ));
            }
        } else if c == '+' {
            result.push(b' ');
        } else {
            result.push(c as u8);
        }
    }

    Ok(OwnedBuffer::from_vec(result))
}

/// URL-encode for path segments (stricter encoding)
pub fn url_encode_path(input: &OwnedBuffer) -> OwnedBuffer {
    let input_str = match input.as_str() {
        Ok(s) => s,
        Err(_) => return OwnedBuffer::new(),
    };

    let encoded: String = input_str
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '~' {
                c.to_string()
            } else {
                format!("%{:02X}", c as u32)
            }
        })
        .collect();

    OwnedBuffer::from_string(encoded)
}

/// URL-encode for query parameters (encodes = and &)
pub fn url_encode_query(input: &OwnedBuffer) -> OwnedBuffer {
    let input_str = match input.as_str() {
        Ok(s) => s,
        Err(_) => return OwnedBuffer::new(),
    };

    let encoded: String = input_str
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '~' {
                c.to_string()
            } else if c == ' ' {
                "+".to_string()
            } else {
                format!("%{:02X}", c as u32)
            }
        })
        .collect();

    OwnedBuffer::from_string(encoded)
}

// =============================================================================
// Registration
// =============================================================================

/// Register all encoding wrappers with the registry
pub fn register(registry: &mut WrapperRegistry) {
    // Base64
    registry.register_wrapper(
        "base64_encode",
        "Encode data as base64",
        WrapperCategory::Encoding,
        1,
        &["base64", "b64", "encode base64", "to base64"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg("No input provided".to_string()));
            }
            Ok(base64_encode(&args[0]))
        },
    );

    registry.register_wrapper(
        "base64_decode",
        "Decode base64 to binary",
        WrapperCategory::Encoding,
        1,
        &["decode base64", "from base64", "unbase64"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg("No input provided".to_string()));
            }
            base64_decode(&args[0])
        },
    );

    registry.register_wrapper(
        "base64_encode_url",
        "Encode as URL-safe base64",
        WrapperCategory::Encoding,
        1,
        &["base64 url", "url safe base64", "base64url"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg("No input provided".to_string()));
            }
            Ok(base64_encode_url(&args[0]))
        },
    );

    registry.register_wrapper(
        "base64_decode_url",
        "Decode URL-safe base64",
        WrapperCategory::Encoding,
        1,
        &["decode base64 url", "from base64url"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg("No input provided".to_string()));
            }
            base64_decode_url(&args[0])
        },
    );

    // Hex
    registry.register_wrapper(
        "hex_encode",
        "Encode data as lowercase hex",
        WrapperCategory::Encoding,
        1,
        &["hex", "hexadecimal", "to hex", "encode hex"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg("No input provided".to_string()));
            }
            Ok(hex_encode(&args[0]))
        },
    );

    registry.register_wrapper(
        "hex_encode_upper",
        "Encode data as uppercase hex",
        WrapperCategory::Encoding,
        1,
        &["hex upper", "uppercase hex"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg("No input provided".to_string()));
            }
            Ok(hex_encode_upper(&args[0]))
        },
    );

    registry.register_wrapper(
        "hex_decode",
        "Decode hex to binary",
        WrapperCategory::Encoding,
        1,
        &["decode hex", "from hex", "unhex"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg("No input provided".to_string()));
            }
            hex_decode(&args[0])
        },
    );

    // URL encoding
    registry.register_wrapper(
        "url_encode",
        "URL-encode a string (percent encoding)",
        WrapperCategory::Encoding,
        1,
        &["url encode", "percent encode", "urlencode"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg("No input provided".to_string()));
            }
            Ok(url_encode(&args[0]))
        },
    );

    registry.register_wrapper(
        "url_decode",
        "URL-decode a string",
        WrapperCategory::Encoding,
        1,
        &["url decode", "percent decode", "urldecode"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg("No input provided".to_string()));
            }
            url_decode(&args[0])
        },
    );

    registry.register_wrapper(
        "url_encode_path",
        "URL-encode for path segments",
        WrapperCategory::Encoding,
        1,
        &["url encode path", "path encode"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg("No input provided".to_string()));
            }
            Ok(url_encode_path(&args[0]))
        },
    );

    registry.register_wrapper(
        "url_encode_query",
        "URL-encode for query parameters",
        WrapperCategory::Encoding,
        1,
        &["url encode query", "query encode", "form encode"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg("No input provided".to_string()));
            }
            Ok(url_encode_query(&args[0]))
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_roundtrip() {
        let original = OwnedBuffer::from_slice(&[0x00, 0x01, 0x02, 0xFF]);
        let encoded = base64_encode(&original);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_base64_known_value() {
        let input = OwnedBuffer::from_str("Hello");
        let encoded = base64_encode(&input);
        assert_eq!(encoded.as_str().unwrap(), "SGVsbG8=");
    }

    #[test]
    fn test_base64_url_safe() {
        let data = OwnedBuffer::from_slice(&[0xFB, 0xFF]);
        let standard = base64_encode(&data);
        let url_safe = base64_encode_url(&data);

        // Standard has + and /
        assert!(
            standard.as_str().unwrap().contains('+') || standard.as_str().unwrap().contains('/')
        );
        // URL-safe uses - and _
        assert!(!url_safe.as_str().unwrap().contains('+'));
        assert!(!url_safe.as_str().unwrap().contains('/'));
    }

    #[test]
    fn test_hex_roundtrip() {
        let original = OwnedBuffer::from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);
        let encoded = hex_encode(&original);
        assert_eq!(encoded.as_str().unwrap(), "deadbeef");

        let decoded = hex_decode(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_hex_upper() {
        let data = OwnedBuffer::from_slice(&[0xAB, 0xCD]);
        let upper = hex_encode_upper(&data);
        assert_eq!(upper.as_str().unwrap(), "ABCD");
    }

    #[test]
    fn test_url_encode() {
        let input = OwnedBuffer::from_str("hello world");
        let encoded = url_encode(&input);
        assert_eq!(encoded.as_str().unwrap(), "hello%20world");
    }

    #[test]
    fn test_url_decode() {
        let input = OwnedBuffer::from_str("hello%20world");
        let decoded = url_decode(&input).unwrap();
        assert_eq!(decoded.as_str().unwrap(), "hello world");
    }

    #[test]
    fn test_url_roundtrip() {
        let original = OwnedBuffer::from_str("name=John Doe&city=New York");
        let encoded = url_encode(&original);
        let decoded = url_decode(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_url_encode_query() {
        let input = OwnedBuffer::from_str("hello world");
        let encoded = url_encode_query(&input);
        // Query encoding uses + for space
        assert_eq!(encoded.as_str().unwrap(), "hello+world");
    }
}
