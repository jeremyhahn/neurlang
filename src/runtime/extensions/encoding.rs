//! Encoding Extensions
//!
//! Binary-to-text encoding and decoding operations.
//!
//! # Supported Encodings
//! - **Base64** - Binary in text (email, JSON, configs)
//! - **Base64 URL-safe** - URLs, filenames
//! - **Hex** - Debugging, hashes, simple encoding
//! - **URL encoding** - Query strings, form data

use std::sync::Arc;

use base64::{engine::general_purpose, Engine as _};

use super::buffer::{HandleManager, OwnedBuffer};
use super::{ext_ids, ExtCategory, ExtError, ExtensionRegistry};

// =============================================================================
// Base64
// =============================================================================

fn base64_encode_impl(input: &[u8]) -> Vec<u8> {
    let encoded = general_purpose::STANDARD.encode(input);
    encoded.into_bytes()
}

fn base64_decode_impl(input: &[u8]) -> Result<Vec<u8>, ExtError> {
    let input_str = std::str::from_utf8(input)
        .map_err(|e| ExtError::ExtensionError(format!("Invalid UTF-8: {}", e)))?;

    general_purpose::STANDARD
        .decode(input_str.trim())
        .map_err(|e| ExtError::ExtensionError(format!("Invalid base64: {}", e)))
}

// =============================================================================
// Hex
// =============================================================================

fn hex_encode_impl(input: &[u8]) -> Vec<u8> {
    let encoded = hex::encode(input);
    encoded.into_bytes()
}

fn hex_decode_impl(input: &[u8]) -> Result<Vec<u8>, ExtError> {
    let input_str = std::str::from_utf8(input)
        .map_err(|e| ExtError::ExtensionError(format!("Invalid UTF-8: {}", e)))?;

    hex::decode(input_str.trim())
        .map_err(|e| ExtError::ExtensionError(format!("Invalid hex: {}", e)))
}

// =============================================================================
// URL Encoding
// =============================================================================

fn url_encode_impl(input: &[u8]) -> Result<Vec<u8>, ExtError> {
    let input_str = std::str::from_utf8(input)
        .map_err(|e| ExtError::ExtensionError(format!("Invalid UTF-8: {}", e)))?;

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

    Ok(encoded.into_bytes())
}

fn url_decode_impl(input: &[u8]) -> Result<Vec<u8>, ExtError> {
    let input_str = std::str::from_utf8(input)
        .map_err(|e| ExtError::ExtensionError(format!("Invalid UTF-8: {}", e)))?;

    let mut result = Vec::new();
    let mut chars = input_str.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if hex.len() == 2 {
                let byte = u8::from_str_radix(&hex, 16).map_err(|_| {
                    ExtError::ExtensionError(format!("Invalid percent encoding: %{}", hex))
                })?;
                result.push(byte);
            } else {
                return Err(ExtError::ExtensionError(
                    "Incomplete percent encoding".to_string(),
                ));
            }
        } else if c == '+' {
            result.push(b' ');
        } else {
            result.push(c as u8);
        }
    }

    Ok(result)
}

// =============================================================================
// Extension Registration
// =============================================================================

pub fn register_encoding(registry: &mut ExtensionRegistry) {
    // Base64 encode
    registry.register_with_id(
        ext_ids::BASE64_ENCODE,
        "base64_encode",
        "Encode data as base64. Args: buffer_handle. Returns buffer_handle.",
        1,
        true,
        ExtCategory::Encoding,
        Arc::new(|args, outputs| {
            let handle = args[0];

            let input = HandleManager::get(handle)
                .ok_or_else(|| ExtError::ExtensionError("Invalid buffer handle".to_string()))?;

            let encoded = base64_encode_impl(input.as_slice());
            let result_handle = HandleManager::store(OwnedBuffer::from_vec(encoded));

            outputs[0] = result_handle;
            Ok(result_handle as i64)
        }),
    );

    // Base64 decode
    registry.register_with_id(
        ext_ids::BASE64_DECODE,
        "base64_decode",
        "Decode base64 to binary. Args: buffer_handle. Returns buffer_handle.",
        1,
        true,
        ExtCategory::Encoding,
        Arc::new(|args, outputs| {
            let handle = args[0];

            let input = HandleManager::get(handle)
                .ok_or_else(|| ExtError::ExtensionError("Invalid buffer handle".to_string()))?;

            let decoded = base64_decode_impl(input.as_slice())?;
            let result_handle = HandleManager::store(OwnedBuffer::from_vec(decoded));

            outputs[0] = result_handle;
            Ok(result_handle as i64)
        }),
    );

    // Hex encode
    registry.register_with_id(
        ext_ids::HEX_ENCODE,
        "hex_encode",
        "Encode data as lowercase hex. Args: buffer_handle. Returns buffer_handle.",
        1,
        true,
        ExtCategory::Encoding,
        Arc::new(|args, outputs| {
            let handle = args[0];

            let input = HandleManager::get(handle)
                .ok_or_else(|| ExtError::ExtensionError("Invalid buffer handle".to_string()))?;

            let encoded = hex_encode_impl(input.as_slice());
            let result_handle = HandleManager::store(OwnedBuffer::from_vec(encoded));

            outputs[0] = result_handle;
            Ok(result_handle as i64)
        }),
    );

    // Hex decode
    registry.register_with_id(
        ext_ids::HEX_DECODE,
        "hex_decode",
        "Decode hex to binary. Args: buffer_handle. Returns buffer_handle.",
        1,
        true,
        ExtCategory::Encoding,
        Arc::new(|args, outputs| {
            let handle = args[0];

            let input = HandleManager::get(handle)
                .ok_or_else(|| ExtError::ExtensionError("Invalid buffer handle".to_string()))?;

            let decoded = hex_decode_impl(input.as_slice())?;
            let result_handle = HandleManager::store(OwnedBuffer::from_vec(decoded));

            outputs[0] = result_handle;
            Ok(result_handle as i64)
        }),
    );

    // URL encode
    registry.register_with_id(
        ext_ids::URL_ENCODE,
        "url_encode",
        "URL-encode a string. Args: buffer_handle. Returns buffer_handle.",
        1,
        true,
        ExtCategory::Encoding,
        Arc::new(|args, outputs| {
            let handle = args[0];

            let input = HandleManager::get(handle)
                .ok_or_else(|| ExtError::ExtensionError("Invalid buffer handle".to_string()))?;

            let encoded = url_encode_impl(input.as_slice())?;
            let result_handle = HandleManager::store(OwnedBuffer::from_vec(encoded));

            outputs[0] = result_handle;
            Ok(result_handle as i64)
        }),
    );

    // URL decode
    registry.register_with_id(
        ext_ids::URL_DECODE,
        "url_decode",
        "URL-decode a string. Args: buffer_handle. Returns buffer_handle.",
        1,
        true,
        ExtCategory::Encoding,
        Arc::new(|args, outputs| {
            let handle = args[0];

            let input = HandleManager::get(handle)
                .ok_or_else(|| ExtError::ExtensionError("Invalid buffer handle".to_string()))?;

            let decoded = url_decode_impl(input.as_slice())?;
            let result_handle = HandleManager::store(OwnedBuffer::from_vec(decoded));

            outputs[0] = result_handle;
            Ok(result_handle as i64)
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_roundtrip() {
        let original = vec![0x00, 0x01, 0x02, 0xFF];
        let encoded = base64_encode_impl(&original);
        let decoded = base64_decode_impl(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_hex_roundtrip() {
        let original = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let encoded = hex_encode_impl(&original);
        assert_eq!(std::str::from_utf8(&encoded).unwrap(), "deadbeef");
        let decoded = hex_decode_impl(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_url_roundtrip() {
        let original = b"hello world";
        let encoded = url_encode_impl(original).unwrap();
        assert_eq!(std::str::from_utf8(&encoded).unwrap(), "hello%20world");
        let decoded = url_decode_impl(&encoded).unwrap();
        assert_eq!(original.to_vec(), decoded);
    }
}
