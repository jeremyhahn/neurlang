//! JSON parsing and manipulation extensions
//!
//! Uses serde_json for reliable JSON handling.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use serde_json::{Map, Value};

use super::{ext_ids, ExtCategory, ExtError, ExtensionRegistry};

// =============================================================================
// Handle Management
// =============================================================================

static NEXT_JSON_HANDLE: AtomicU64 = AtomicU64::new(1);

fn next_handle() -> u64 {
    NEXT_JSON_HANDLE.fetch_add(1, Ordering::Relaxed)
}

// =============================================================================
// JSON Storage
// =============================================================================

lazy_static::lazy_static! {
    /// Global storage for JSON Value instances
    static ref JSON_STORAGE: RwLock<HashMap<u64, Value>> = RwLock::new(HashMap::new());
}

/// JSON type constants
pub mod json_type {
    pub const NULL: u64 = 0;
    pub const BOOL: u64 = 1;
    pub const NUMBER: u64 = 2;
    pub const STRING: u64 = 3;
    pub const ARRAY: u64 = 4;
    pub const OBJECT: u64 = 5;
}

/// Parse JSON from bytes, return handle
fn json_parse_impl(ptr: *const u8, len: usize) -> Result<u64, ExtError> {
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    let s = std::str::from_utf8(bytes)
        .map_err(|e| ExtError::ExtensionError(format!("Invalid UTF-8: {}", e)))?;

    let value: Value = serde_json::from_str(s)
        .map_err(|e| ExtError::ExtensionError(format!("JSON parse error: {}", e)))?;

    let handle = next_handle();
    let mut storage = JSON_STORAGE.write().unwrap();
    storage.insert(handle, value);
    Ok(handle)
}

/// Stringify JSON to buffer, return bytes written
fn json_stringify_impl(handle: u64, ptr: *mut u8, max_len: usize) -> Result<usize, ExtError> {
    let storage = JSON_STORAGE.read().unwrap();
    let value = storage
        .get(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid JSON handle: {}", handle)))?;

    let json_str = serde_json::to_string(value)
        .map_err(|e| ExtError::ExtensionError(format!("JSON stringify error: {}", e)))?;

    let len = std::cmp::min(json_str.len(), max_len);
    unsafe {
        std::ptr::copy_nonoverlapping(json_str.as_ptr(), ptr, len);
    }
    Ok(len)
}

/// Get a value from a JSON object by key, return new handle
fn json_get_impl(handle: u64, key_ptr: *const u8, key_len: usize) -> Result<u64, ExtError> {
    let key =
        unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(key_ptr, key_len)) };

    let storage = JSON_STORAGE.read().unwrap();
    let value = storage
        .get(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid JSON handle: {}", handle)))?;

    let result = match value {
        Value::Object(map) => map.get(key).cloned().unwrap_or(Value::Null),
        _ => return Err(ExtError::ExtensionError("Not a JSON object".to_string())),
    };

    let new_handle = next_handle();
    drop(storage);

    let mut storage = JSON_STORAGE.write().unwrap();
    storage.insert(new_handle, result);
    Ok(new_handle)
}

/// Set a value in a JSON object by key
fn json_set_impl(
    handle: u64,
    key_ptr: *const u8,
    key_len: usize,
    value_handle: u64,
) -> Result<(), ExtError> {
    let key =
        unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(key_ptr, key_len)) };

    let storage = JSON_STORAGE.read().unwrap();
    let value_to_set = storage
        .get(&value_handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid JSON handle: {}", value_handle)))?
        .clone();

    drop(storage);

    let mut storage = JSON_STORAGE.write().unwrap();
    let value = storage
        .get_mut(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid JSON handle: {}", handle)))?;

    match value {
        Value::Object(map) => {
            map.insert(key.to_string(), value_to_set);
            Ok(())
        }
        _ => Err(ExtError::ExtensionError("Not a JSON object".to_string())),
    }
}

/// Get the type of a JSON value
fn json_get_type_impl(handle: u64) -> Result<u64, ExtError> {
    let storage = JSON_STORAGE.read().unwrap();
    let value = storage
        .get(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid JSON handle: {}", handle)))?;

    Ok(match value {
        Value::Null => json_type::NULL,
        Value::Bool(_) => json_type::BOOL,
        Value::Number(_) => json_type::NUMBER,
        Value::String(_) => json_type::STRING,
        Value::Array(_) => json_type::ARRAY,
        Value::Object(_) => json_type::OBJECT,
    })
}

/// Get array length
fn json_array_len_impl(handle: u64) -> Result<usize, ExtError> {
    let storage = JSON_STORAGE.read().unwrap();
    let value = storage
        .get(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid JSON handle: {}", handle)))?;

    match value {
        Value::Array(arr) => Ok(arr.len()),
        _ => Err(ExtError::ExtensionError("Not a JSON array".to_string())),
    }
}

/// Get array element by index
fn json_array_get_impl(handle: u64, index: usize) -> Result<u64, ExtError> {
    let storage = JSON_STORAGE.read().unwrap();
    let value = storage
        .get(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid JSON handle: {}", handle)))?;

    let element = match value {
        Value::Array(arr) => arr.get(index).cloned().unwrap_or(Value::Null),
        _ => return Err(ExtError::ExtensionError("Not a JSON array".to_string())),
    };

    let new_handle = next_handle();
    drop(storage);

    let mut storage = JSON_STORAGE.write().unwrap();
    storage.insert(new_handle, element);
    Ok(new_handle)
}

/// Push element to JSON array
fn json_array_push_impl(handle: u64, element_handle: u64) -> Result<(), ExtError> {
    let storage = JSON_STORAGE.read().unwrap();
    let element = storage
        .get(&element_handle)
        .ok_or_else(|| {
            ExtError::ExtensionError(format!("Invalid JSON handle: {}", element_handle))
        })?
        .clone();

    drop(storage);

    let mut storage = JSON_STORAGE.write().unwrap();
    let value = storage
        .get_mut(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid JSON handle: {}", handle)))?;

    match value {
        Value::Array(arr) => {
            arr.push(element);
            Ok(())
        }
        _ => Err(ExtError::ExtensionError("Not a JSON array".to_string())),
    }
}

/// Get all keys of a JSON object as a Vec of string handles
fn json_object_keys_impl(handle: u64) -> Result<u64, ExtError> {
    let storage = JSON_STORAGE.read().unwrap();
    let value = storage
        .get(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid JSON handle: {}", handle)))?;

    let keys: Vec<String> = match value {
        Value::Object(map) => map.keys().cloned().collect(),
        _ => return Err(ExtError::ExtensionError("Not a JSON object".to_string())),
    };

    drop(storage);

    // Create string handles for each key
    let mut string_handles = Vec::with_capacity(keys.len());
    {
        let mut string_storage = super::strings::STRING_STORAGE.write().unwrap();
        for key in keys {
            let h = super::strings::next_handle();
            string_storage.insert(h, key);
            string_handles.push(h);
        }
    }

    // Create a Vec to hold the string handles
    let vec_handle = super::collections::next_handle();
    {
        let mut vec_storage = super::collections::VEC_STORAGE.write().unwrap();
        vec_storage.insert(vec_handle, string_handles);
    }

    Ok(vec_handle)
}

/// Free a JSON value
fn json_free_impl(handle: u64) -> Result<(), ExtError> {
    let mut storage = JSON_STORAGE.write().unwrap();
    storage
        .remove(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid JSON handle: {}", handle)))?;
    Ok(())
}

/// Create a new empty JSON object
fn json_new_object_impl() -> u64 {
    let handle = next_handle();
    let mut storage = JSON_STORAGE.write().unwrap();
    storage.insert(handle, Value::Object(Map::new()));
    handle
}

/// Create a new empty JSON array
fn json_new_array_impl() -> u64 {
    let handle = next_handle();
    let mut storage = JSON_STORAGE.write().unwrap();
    storage.insert(handle, Value::Array(Vec::new()));
    handle
}

// =============================================================================
// Extension Registration
// =============================================================================

pub fn register_json(registry: &mut ExtensionRegistry) {
    registry.register_with_id(
        ext_ids::JSON_PARSE,
        "json_parse",
        "Parse JSON from UTF-8 bytes. Args: ptr, len. Returns JSON handle.",
        2,
        true,
        ExtCategory::Json,
        Arc::new(|args, outputs| {
            let ptr = args[0] as *const u8;
            let len = args[1] as usize;
            let handle = json_parse_impl(ptr, len)?;
            outputs[0] = handle;
            Ok(handle as i64)
        }),
    );

    registry.register_with_id(
        ext_ids::JSON_STRINGIFY,
        "json_stringify",
        "Convert JSON to string. Args: handle, ptr, max_len. Returns bytes written.",
        3,
        true,
        ExtCategory::Json,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let ptr = args[1] as *mut u8;
            let max_len = args[2] as usize;
            let len = json_stringify_impl(handle, ptr, max_len)?;
            outputs[0] = len as u64;
            Ok(len as i64)
        }),
    );

    registry.register_with_id(
        ext_ids::JSON_GET,
        "json_get",
        "Get value from JSON object by key. Args: handle, key_ptr, key_len. Returns new handle.",
        3,
        true,
        ExtCategory::Json,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let key_ptr = args[1] as *const u8;
            let key_len = args[2] as usize;
            let new_handle = json_get_impl(handle, key_ptr, key_len)?;
            outputs[0] = new_handle;
            Ok(new_handle as i64)
        }),
    );

    registry.register_with_id(
        ext_ids::JSON_SET,
        "json_set",
        "Set value in JSON object. Args: handle, key_ptr, key_len, value_handle. Returns 0.",
        4,
        true,
        ExtCategory::Json,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let key_ptr = args[1] as *const u8;
            let key_len = args[2] as usize;
            let value_handle = args[3];
            json_set_impl(handle, key_ptr, key_len, value_handle)?;
            outputs[0] = 0;
            Ok(0)
        }),
    );

    registry.register_with_id(
        ext_ids::JSON_GET_TYPE,
        "json_get_type",
        "Get JSON value type. Args: handle. Returns type (0=null, 1=bool, 2=number, 3=string, 4=array, 5=object).",
        1,
        true,
        ExtCategory::Json,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let type_id = json_get_type_impl(handle)?;
            outputs[0] = type_id;
            Ok(type_id as i64)
        }),
    );

    registry.register_with_id(
        ext_ids::JSON_ARRAY_LEN,
        "json_array_len",
        "Get JSON array length. Args: handle. Returns length.",
        1,
        true,
        ExtCategory::Json,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let len = json_array_len_impl(handle)?;
            outputs[0] = len as u64;
            Ok(len as i64)
        }),
    );

    registry.register_with_id(
        ext_ids::JSON_ARRAY_GET,
        "json_array_get",
        "Get JSON array element. Args: handle, index. Returns new handle.",
        2,
        true,
        ExtCategory::Json,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let index = args[1] as usize;
            let new_handle = json_array_get_impl(handle, index)?;
            outputs[0] = new_handle;
            Ok(new_handle as i64)
        }),
    );

    registry.register_with_id(
        ext_ids::JSON_ARRAY_PUSH,
        "json_array_push",
        "Push element to JSON array. Args: handle, element_handle. Returns 0.",
        2,
        true,
        ExtCategory::Json,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let element = args[1];
            json_array_push_impl(handle, element)?;
            outputs[0] = 0;
            Ok(0)
        }),
    );

    registry.register_with_id(
        ext_ids::JSON_OBJECT_KEYS,
        "json_object_keys",
        "Get all keys of JSON object. Args: handle. Returns Vec handle of string handles.",
        1,
        true,
        ExtCategory::Json,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let vec_handle = json_object_keys_impl(handle)?;
            outputs[0] = vec_handle;
            Ok(vec_handle as i64)
        }),
    );

    registry.register_with_id(
        ext_ids::JSON_FREE,
        "json_free",
        "Free a JSON value. Args: handle. Returns 0.",
        1,
        true,
        ExtCategory::Json,
        Arc::new(|args, outputs| {
            let handle = args[0];
            json_free_impl(handle)?;
            outputs[0] = 0;
            Ok(0)
        }),
    );

    registry.register_with_id(
        ext_ids::JSON_NEW_OBJECT,
        "json_new_object",
        "Create a new empty JSON object. Returns handle.",
        0,
        true,
        ExtCategory::Json,
        Arc::new(|_args, outputs| {
            let handle = json_new_object_impl();
            outputs[0] = handle;
            Ok(handle as i64)
        }),
    );

    registry.register_with_id(
        ext_ids::JSON_NEW_ARRAY,
        "json_new_array",
        "Create a new empty JSON array. Returns handle.",
        0,
        true,
        ExtCategory::Json,
        Arc::new(|_args, outputs| {
            let handle = json_new_array_impl();
            outputs[0] = handle;
            Ok(handle as i64)
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_parse_object() {
        let json = r#"{"name": "test", "value": 42}"#;
        let handle = json_parse_impl(json.as_ptr(), json.len()).unwrap();
        assert_eq!(json_get_type_impl(handle).unwrap(), json_type::OBJECT);
        json_free_impl(handle).unwrap();
    }

    #[test]
    fn test_json_parse_array() {
        let json = r#"[1, 2, 3, 4, 5]"#;
        let handle = json_parse_impl(json.as_ptr(), json.len()).unwrap();
        assert_eq!(json_get_type_impl(handle).unwrap(), json_type::ARRAY);
        assert_eq!(json_array_len_impl(handle).unwrap(), 5);
        json_free_impl(handle).unwrap();
    }
}
