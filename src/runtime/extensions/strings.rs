//! String processing extensions
//!
//! Provides string manipulation operations using a handle-based approach.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use super::collections::VEC_STORAGE;
use super::{ext_ids, ExtCategory, ExtError, ExtensionRegistry};

// =============================================================================
// Handle Management
// =============================================================================

static NEXT_STRING_HANDLE: AtomicU64 = AtomicU64::new(1);

/// Generate a handle (exported for use by other modules)
pub fn next_handle() -> u64 {
    NEXT_STRING_HANDLE.fetch_add(1, Ordering::Relaxed)
}

// =============================================================================
// String Storage
// =============================================================================

lazy_static::lazy_static! {
    /// Global storage for String instances
    pub static ref STRING_STORAGE: RwLock<HashMap<u64, String>> = RwLock::new(HashMap::new());
}

/// Create a new empty String and return its handle
fn string_new_impl() -> u64 {
    let handle = next_handle();
    let mut storage = STRING_STORAGE.write().unwrap();
    storage.insert(handle, String::new());
    handle
}

/// Create a String from bytes and return its handle
fn string_from_bytes_impl(ptr: *const u8, len: usize) -> Result<u64, ExtError> {
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    let s = String::from_utf8(bytes.to_vec())
        .map_err(|e| ExtError::ExtensionError(format!("Invalid UTF-8: {}", e)))?;

    let handle = next_handle();
    let mut storage = STRING_STORAGE.write().unwrap();
    storage.insert(handle, s);
    Ok(handle)
}

/// Get the length of a String (in bytes)
fn string_len_impl(handle: u64) -> Result<usize, ExtError> {
    let storage = STRING_STORAGE.read().unwrap();
    let s = storage
        .get(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid string handle: {}", handle)))?;
    Ok(s.len())
}

/// Concatenate two strings, return new handle
fn string_concat_impl(handle1: u64, handle2: u64) -> Result<u64, ExtError> {
    let storage = STRING_STORAGE.read().unwrap();
    let s1 = storage
        .get(&handle1)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid string handle: {}", handle1)))?;
    let s2 = storage
        .get(&handle2)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid string handle: {}", handle2)))?;

    let result = format!("{}{}", s1, s2);
    let new_handle = next_handle();

    drop(storage);

    let mut storage = STRING_STORAGE.write().unwrap();
    storage.insert(new_handle, result);
    Ok(new_handle)
}

/// Get a substring, return new handle
fn string_substr_impl(handle: u64, start: usize, len: usize) -> Result<u64, ExtError> {
    let storage = STRING_STORAGE.read().unwrap();
    let s = storage
        .get(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid string handle: {}", handle)))?;

    if start > s.len() {
        return Err(ExtError::BoundsViolation {
            offset: start,
            len,
            cap_len: s.len(),
        });
    }

    let end = std::cmp::min(start + len, s.len());
    let substr = s[start..end].to_string();
    let new_handle = next_handle();

    drop(storage);

    let mut storage = STRING_STORAGE.write().unwrap();
    storage.insert(new_handle, substr);
    Ok(new_handle)
}

/// Find a substring, return index or -1
fn string_find_impl(handle: u64, needle_handle: u64) -> Result<i64, ExtError> {
    let storage = STRING_STORAGE.read().unwrap();
    let haystack = storage
        .get(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid string handle: {}", handle)))?;
    let needle = storage.get(&needle_handle).ok_or_else(|| {
        ExtError::ExtensionError(format!("Invalid string handle: {}", needle_handle))
    })?;

    match haystack.find(needle.as_str()) {
        Some(idx) => Ok(idx as i64),
        None => Ok(-1),
    }
}

/// Replace all occurrences of a pattern, return new handle
fn string_replace_impl(
    handle: u64,
    pattern_handle: u64,
    replacement_handle: u64,
) -> Result<u64, ExtError> {
    let storage = STRING_STORAGE.read().unwrap();
    let s = storage
        .get(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid string handle: {}", handle)))?;
    let pattern = storage.get(&pattern_handle).ok_or_else(|| {
        ExtError::ExtensionError(format!("Invalid string handle: {}", pattern_handle))
    })?;
    let replacement = storage.get(&replacement_handle).ok_or_else(|| {
        ExtError::ExtensionError(format!("Invalid string handle: {}", replacement_handle))
    })?;

    let result = s.replace(pattern.as_str(), replacement.as_str());
    let new_handle = next_handle();

    drop(storage);

    let mut storage = STRING_STORAGE.write().unwrap();
    storage.insert(new_handle, result);
    Ok(new_handle)
}

/// Split a string by delimiter, return Vec handle containing string handles
fn string_split_impl(handle: u64, delimiter_handle: u64) -> Result<u64, ExtError> {
    let storage = STRING_STORAGE.read().unwrap();
    let s = storage
        .get(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid string handle: {}", handle)))?;
    let delimiter = storage.get(&delimiter_handle).ok_or_else(|| {
        ExtError::ExtensionError(format!("Invalid string handle: {}", delimiter_handle))
    })?;

    let parts: Vec<String> = s.split(delimiter.as_str()).map(|s| s.to_string()).collect();

    drop(storage);

    // Create string handles for each part
    let mut string_handles = Vec::with_capacity(parts.len());
    {
        let mut storage = STRING_STORAGE.write().unwrap();
        for part in parts {
            let h = next_handle();
            storage.insert(h, part);
            string_handles.push(h);
        }
    }

    // Create a Vec to hold the string handles
    let vec_handle = super::collections::next_handle();
    {
        let mut vec_storage = VEC_STORAGE.write().unwrap();
        vec_storage.insert(vec_handle, string_handles);
    }

    Ok(vec_handle)
}

/// Trim whitespace from both ends, return new handle
fn string_trim_impl(handle: u64) -> Result<u64, ExtError> {
    let storage = STRING_STORAGE.read().unwrap();
    let s = storage
        .get(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid string handle: {}", handle)))?;

    let trimmed = s.trim().to_string();
    let new_handle = next_handle();

    drop(storage);

    let mut storage = STRING_STORAGE.write().unwrap();
    storage.insert(new_handle, trimmed);
    Ok(new_handle)
}

/// Convert to uppercase, return new handle
fn string_to_upper_impl(handle: u64) -> Result<u64, ExtError> {
    let storage = STRING_STORAGE.read().unwrap();
    let s = storage
        .get(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid string handle: {}", handle)))?;

    let upper = s.to_uppercase();
    let new_handle = next_handle();

    drop(storage);

    let mut storage = STRING_STORAGE.write().unwrap();
    storage.insert(new_handle, upper);
    Ok(new_handle)
}

/// Convert to lowercase, return new handle
fn string_to_lower_impl(handle: u64) -> Result<u64, ExtError> {
    let storage = STRING_STORAGE.read().unwrap();
    let s = storage
        .get(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid string handle: {}", handle)))?;

    let lower = s.to_lowercase();
    let new_handle = next_handle();

    drop(storage);

    let mut storage = STRING_STORAGE.write().unwrap();
    storage.insert(new_handle, lower);
    Ok(new_handle)
}

/// Check if string starts with prefix
fn string_starts_with_impl(handle: u64, prefix_handle: u64) -> Result<bool, ExtError> {
    let storage = STRING_STORAGE.read().unwrap();
    let s = storage
        .get(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid string handle: {}", handle)))?;
    let prefix = storage.get(&prefix_handle).ok_or_else(|| {
        ExtError::ExtensionError(format!("Invalid string handle: {}", prefix_handle))
    })?;

    Ok(s.starts_with(prefix.as_str()))
}

/// Check if string ends with suffix
fn string_ends_with_impl(handle: u64, suffix_handle: u64) -> Result<bool, ExtError> {
    let storage = STRING_STORAGE.read().unwrap();
    let s = storage
        .get(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid string handle: {}", handle)))?;
    let suffix = storage.get(&suffix_handle).ok_or_else(|| {
        ExtError::ExtensionError(format!("Invalid string handle: {}", suffix_handle))
    })?;

    Ok(s.ends_with(suffix.as_str()))
}

/// Copy string bytes to a buffer
fn string_to_bytes_impl(handle: u64, ptr: *mut u8, max_len: usize) -> Result<usize, ExtError> {
    let storage = STRING_STORAGE.read().unwrap();
    let s = storage
        .get(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid string handle: {}", handle)))?;

    let len = std::cmp::min(s.len(), max_len);
    unsafe {
        std::ptr::copy_nonoverlapping(s.as_ptr(), ptr, len);
    }
    Ok(len)
}

/// Free a String
fn string_free_impl(handle: u64) -> Result<(), ExtError> {
    let mut storage = STRING_STORAGE.write().unwrap();
    storage
        .remove(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid string handle: {}", handle)))?;
    Ok(())
}

/// Parse string as integer
fn string_parse_int_impl(handle: u64) -> Result<i64, ExtError> {
    let storage = STRING_STORAGE.read().unwrap();
    let s = storage
        .get(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid string handle: {}", handle)))?;

    s.trim()
        .parse::<i64>()
        .map_err(|e| ExtError::ExtensionError(format!("Parse error: {}", e)))
}

/// Parse string as float
fn string_parse_float_impl(handle: u64) -> Result<f64, ExtError> {
    let storage = STRING_STORAGE.read().unwrap();
    let s = storage
        .get(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid string handle: {}", handle)))?;

    s.trim()
        .parse::<f64>()
        .map_err(|e| ExtError::ExtensionError(format!("Parse error: {}", e)))
}

/// Create string from integer
fn string_from_int_impl(value: i64) -> u64 {
    let s = value.to_string();
    let handle = next_handle();
    let mut storage = STRING_STORAGE.write().unwrap();
    storage.insert(handle, s);
    handle
}

/// Create string from float
fn string_from_float_impl(value: f64) -> u64 {
    let s = value.to_string();
    let handle = next_handle();
    let mut storage = STRING_STORAGE.write().unwrap();
    storage.insert(handle, s);
    handle
}

// =============================================================================
// Extension Registration
// =============================================================================

pub fn register_strings(registry: &mut ExtensionRegistry) {
    registry.register_with_id(
        ext_ids::STRING_NEW,
        "string_new",
        "Create a new empty string. Returns handle.",
        0,
        true,
        ExtCategory::Strings,
        Arc::new(|_args, outputs| {
            let handle = string_new_impl();
            outputs[0] = handle;
            Ok(handle as i64)
        }),
    );

    registry.register_with_id(
        ext_ids::STRING_FROM_BYTES,
        "string_from_bytes",
        "Create a string from UTF-8 bytes. Args: ptr, len. Returns handle.",
        2,
        true,
        ExtCategory::Strings,
        Arc::new(|args, outputs| {
            let ptr = args[0] as *const u8;
            let len = args[1] as usize;
            let handle = string_from_bytes_impl(ptr, len)?;
            outputs[0] = handle;
            Ok(handle as i64)
        }),
    );

    registry.register_with_id(
        ext_ids::STRING_LEN,
        "string_len",
        "Get string length in bytes. Args: handle. Returns length.",
        1,
        true,
        ExtCategory::Strings,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let len = string_len_impl(handle)?;
            outputs[0] = len as u64;
            Ok(len as i64)
        }),
    );

    registry.register_with_id(
        ext_ids::STRING_CONCAT,
        "string_concat",
        "Concatenate two strings. Args: handle1, handle2. Returns new handle.",
        2,
        true,
        ExtCategory::Strings,
        Arc::new(|args, outputs| {
            let h1 = args[0];
            let h2 = args[1];
            let handle = string_concat_impl(h1, h2)?;
            outputs[0] = handle;
            Ok(handle as i64)
        }),
    );

    registry.register_with_id(
        ext_ids::STRING_SUBSTR,
        "string_substr",
        "Get a substring. Args: handle, start, len. Returns new handle.",
        3,
        true,
        ExtCategory::Strings,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let start = args[1] as usize;
            let len = args[2] as usize;
            let new_handle = string_substr_impl(handle, start, len)?;
            outputs[0] = new_handle;
            Ok(new_handle as i64)
        }),
    );

    registry.register_with_id(
        ext_ids::STRING_FIND,
        "string_find",
        "Find a substring. Args: haystack_handle, needle_handle. Returns index or -1.",
        2,
        true,
        ExtCategory::Strings,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let needle = args[1];
            let result = string_find_impl(handle, needle)?;
            outputs[0] = result as u64;
            Ok(result)
        }),
    );

    registry.register_with_id(
        ext_ids::STRING_REPLACE,
        "string_replace",
        "Replace all occurrences. Args: handle, pattern_handle, replacement_handle. Returns new handle.",
        3,
        true,
        ExtCategory::Strings,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let pattern = args[1];
            let replacement = args[2];
            let new_handle = string_replace_impl(handle, pattern, replacement)?;
            outputs[0] = new_handle;
            Ok(new_handle as i64)
        }),
    );

    registry.register_with_id(
        ext_ids::STRING_SPLIT,
        "string_split",
        "Split string by delimiter. Args: handle, delimiter_handle. Returns Vec handle.",
        2,
        true,
        ExtCategory::Strings,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let delimiter = args[1];
            let vec_handle = string_split_impl(handle, delimiter)?;
            outputs[0] = vec_handle;
            Ok(vec_handle as i64)
        }),
    );

    registry.register_with_id(
        ext_ids::STRING_TRIM,
        "string_trim",
        "Trim whitespace from both ends. Args: handle. Returns new handle.",
        1,
        true,
        ExtCategory::Strings,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let new_handle = string_trim_impl(handle)?;
            outputs[0] = new_handle;
            Ok(new_handle as i64)
        }),
    );

    registry.register_with_id(
        ext_ids::STRING_TO_UPPER,
        "string_to_upper",
        "Convert to uppercase. Args: handle. Returns new handle.",
        1,
        true,
        ExtCategory::Strings,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let new_handle = string_to_upper_impl(handle)?;
            outputs[0] = new_handle;
            Ok(new_handle as i64)
        }),
    );

    registry.register_with_id(
        ext_ids::STRING_TO_LOWER,
        "string_to_lower",
        "Convert to lowercase. Args: handle. Returns new handle.",
        1,
        true,
        ExtCategory::Strings,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let new_handle = string_to_lower_impl(handle)?;
            outputs[0] = new_handle;
            Ok(new_handle as i64)
        }),
    );

    registry.register_with_id(
        ext_ids::STRING_STARTS_WITH,
        "string_starts_with",
        "Check if string starts with prefix. Args: handle, prefix_handle. Returns 1 or 0.",
        2,
        true,
        ExtCategory::Strings,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let prefix = args[1];
            let result = string_starts_with_impl(handle, prefix)?;
            outputs[0] = if result { 1 } else { 0 };
            Ok(if result { 1 } else { 0 })
        }),
    );

    registry.register_with_id(
        ext_ids::STRING_ENDS_WITH,
        "string_ends_with",
        "Check if string ends with suffix. Args: handle, suffix_handle. Returns 1 or 0.",
        2,
        true,
        ExtCategory::Strings,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let suffix = args[1];
            let result = string_ends_with_impl(handle, suffix)?;
            outputs[0] = if result { 1 } else { 0 };
            Ok(if result { 1 } else { 0 })
        }),
    );

    registry.register_with_id(
        ext_ids::STRING_TO_BYTES,
        "string_to_bytes",
        "Copy string bytes to buffer. Args: handle, ptr, max_len. Returns bytes written.",
        3,
        true,
        ExtCategory::Strings,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let ptr = args[1] as *mut u8;
            let max_len = args[2] as usize;
            let len = string_to_bytes_impl(handle, ptr, max_len)?;
            outputs[0] = len as u64;
            Ok(len as i64)
        }),
    );

    registry.register_with_id(
        ext_ids::STRING_FREE,
        "string_free",
        "Free a string. Args: handle. Returns 0.",
        1,
        true,
        ExtCategory::Strings,
        Arc::new(|args, outputs| {
            let handle = args[0];
            string_free_impl(handle)?;
            outputs[0] = 0;
            Ok(0)
        }),
    );

    registry.register_with_id(
        ext_ids::STRING_PARSE_INT,
        "string_parse_int",
        "Parse string as integer. Args: handle. Returns parsed value.",
        1,
        true,
        ExtCategory::Strings,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let value = string_parse_int_impl(handle)?;
            outputs[0] = value as u64;
            Ok(value)
        }),
    );

    registry.register_with_id(
        ext_ids::STRING_PARSE_FLOAT,
        "string_parse_float",
        "Parse string as float. Args: handle. Returns float bits.",
        1,
        true,
        ExtCategory::Strings,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let value = string_parse_float_impl(handle)?;
            outputs[0] = value.to_bits();
            Ok(0)
        }),
    );

    registry.register_with_id(
        ext_ids::STRING_FROM_INT,
        "string_from_int",
        "Create string from integer. Args: value. Returns handle.",
        1,
        true,
        ExtCategory::Strings,
        Arc::new(|args, outputs| {
            let value = args[0] as i64;
            let handle = string_from_int_impl(value);
            outputs[0] = handle;
            Ok(handle as i64)
        }),
    );

    registry.register_with_id(
        ext_ids::STRING_FROM_FLOAT,
        "string_from_float",
        "Create string from float bits. Args: value_bits. Returns handle.",
        1,
        true,
        ExtCategory::Strings,
        Arc::new(|args, outputs| {
            let value = f64::from_bits(args[0]);
            let handle = string_from_float_impl(value);
            outputs[0] = handle;
            Ok(handle as i64)
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_operations() {
        let bytes = b"Hello, World!";
        let handle = string_from_bytes_impl(bytes.as_ptr(), bytes.len()).unwrap();
        assert_eq!(string_len_impl(handle).unwrap(), 13);
        string_free_impl(handle).unwrap();
    }
}
