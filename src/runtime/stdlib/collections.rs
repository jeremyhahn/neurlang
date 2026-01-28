//! Collection data structures: Vec and HashMap
//!
//! Uses a handle-based approach to manage Rust objects from IR code.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use crate::runtime::{ExtCategory, ExtError, ExtensionRegistry};

// =============================================================================
// Handle Management
// =============================================================================

/// Global handle counter
static NEXT_HANDLE: AtomicU64 = AtomicU64::new(1);

// =============================================================================
// Vec Storage
// =============================================================================

lazy_static::lazy_static! {
    /// Global storage for Vec instances
    /// Key: handle, Value: Vec<u64>
    pub static ref VEC_STORAGE: RwLock<HashMap<u64, Vec<u64>>> = RwLock::new(HashMap::new());
}

/// Generate a handle (exported for use by other modules)
pub fn next_handle() -> u64 {
    NEXT_HANDLE.fetch_add(1, Ordering::Relaxed)
}

/// Create a new Vec and return its handle
fn vec_new_impl() -> u64 {
    let handle = next_handle();
    let mut storage = VEC_STORAGE.write().unwrap();
    storage.insert(handle, Vec::new());
    handle
}

/// Create a new Vec with capacity and return its handle
fn vec_with_capacity_impl(capacity: usize) -> u64 {
    let handle = next_handle();
    let mut storage = VEC_STORAGE.write().unwrap();
    storage.insert(handle, Vec::with_capacity(capacity));
    handle
}

/// Push a value to a Vec
fn vec_push_impl(handle: u64, value: u64) -> Result<(), ExtError> {
    let mut storage = VEC_STORAGE.write().unwrap();
    let vec = storage
        .get_mut(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid vec handle: {}", handle)))?;
    vec.push(value);
    Ok(())
}

/// Pop a value from a Vec
fn vec_pop_impl(handle: u64) -> Result<Option<u64>, ExtError> {
    let mut storage = VEC_STORAGE.write().unwrap();
    let vec = storage
        .get_mut(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid vec handle: {}", handle)))?;
    Ok(vec.pop())
}

/// Get a value from a Vec by index
fn vec_get_impl(handle: u64, index: usize) -> Result<Option<u64>, ExtError> {
    let storage = VEC_STORAGE.read().unwrap();
    let vec = storage
        .get(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid vec handle: {}", handle)))?;
    Ok(vec.get(index).copied())
}

/// Set a value in a Vec by index
fn vec_set_impl(handle: u64, index: usize, value: u64) -> Result<(), ExtError> {
    let mut storage = VEC_STORAGE.write().unwrap();
    let vec = storage
        .get_mut(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid vec handle: {}", handle)))?;
    if index >= vec.len() {
        return Err(ExtError::BoundsViolation {
            offset: index,
            len: 1,
            cap_len: vec.len(),
        });
    }
    vec[index] = value;
    Ok(())
}

/// Get the length of a Vec
fn vec_len_impl(handle: u64) -> Result<usize, ExtError> {
    let storage = VEC_STORAGE.read().unwrap();
    let vec = storage
        .get(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid vec handle: {}", handle)))?;
    Ok(vec.len())
}

/// Get the capacity of a Vec
fn vec_capacity_impl(handle: u64) -> Result<usize, ExtError> {
    let storage = VEC_STORAGE.read().unwrap();
    let vec = storage
        .get(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid vec handle: {}", handle)))?;
    Ok(vec.capacity())
}

/// Clear a Vec
fn vec_clear_impl(handle: u64) -> Result<(), ExtError> {
    let mut storage = VEC_STORAGE.write().unwrap();
    let vec = storage
        .get_mut(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid vec handle: {}", handle)))?;
    vec.clear();
    Ok(())
}

/// Free a Vec
fn vec_free_impl(handle: u64) -> Result<(), ExtError> {
    let mut storage = VEC_STORAGE.write().unwrap();
    storage
        .remove(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid vec handle: {}", handle)))?;
    Ok(())
}

/// Insert a value at an index
fn vec_insert_impl(handle: u64, index: usize, value: u64) -> Result<(), ExtError> {
    let mut storage = VEC_STORAGE.write().unwrap();
    let vec = storage
        .get_mut(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid vec handle: {}", handle)))?;
    if index > vec.len() {
        return Err(ExtError::BoundsViolation {
            offset: index,
            len: 0,
            cap_len: vec.len(),
        });
    }
    vec.insert(index, value);
    Ok(())
}

/// Remove a value at an index and return it
fn vec_remove_impl(handle: u64, index: usize) -> Result<u64, ExtError> {
    let mut storage = VEC_STORAGE.write().unwrap();
    let vec = storage
        .get_mut(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid vec handle: {}", handle)))?;
    if index >= vec.len() {
        return Err(ExtError::BoundsViolation {
            offset: index,
            len: 1,
            cap_len: vec.len(),
        });
    }
    Ok(vec.remove(index))
}

// =============================================================================
// HashMap Storage
// =============================================================================

lazy_static::lazy_static! {
    /// Global storage for HashMap instances
    /// Key: handle, Value: HashMap<u64, u64>
    static ref HASHMAP_STORAGE: RwLock<HashMap<u64, HashMap<u64, u64>>> = RwLock::new(HashMap::new());
}

/// Create a new HashMap and return its handle
fn hashmap_new_impl() -> u64 {
    let handle = next_handle();
    let mut storage = HASHMAP_STORAGE.write().unwrap();
    storage.insert(handle, HashMap::new());
    handle
}

/// Insert a key-value pair into a HashMap
fn hashmap_insert_impl(handle: u64, key: u64, value: u64) -> Result<Option<u64>, ExtError> {
    let mut storage = HASHMAP_STORAGE.write().unwrap();
    let map = storage
        .get_mut(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid hashmap handle: {}", handle)))?;
    Ok(map.insert(key, value))
}

/// Get a value from a HashMap by key
fn hashmap_get_impl(handle: u64, key: u64) -> Result<Option<u64>, ExtError> {
    let storage = HASHMAP_STORAGE.read().unwrap();
    let map = storage
        .get(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid hashmap handle: {}", handle)))?;
    Ok(map.get(&key).copied())
}

/// Remove a key-value pair from a HashMap
fn hashmap_remove_impl(handle: u64, key: u64) -> Result<Option<u64>, ExtError> {
    let mut storage = HASHMAP_STORAGE.write().unwrap();
    let map = storage
        .get_mut(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid hashmap handle: {}", handle)))?;
    Ok(map.remove(&key))
}

/// Check if a HashMap contains a key
fn hashmap_contains_impl(handle: u64, key: u64) -> Result<bool, ExtError> {
    let storage = HASHMAP_STORAGE.read().unwrap();
    let map = storage
        .get(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid hashmap handle: {}", handle)))?;
    Ok(map.contains_key(&key))
}

/// Get the length of a HashMap
fn hashmap_len_impl(handle: u64) -> Result<usize, ExtError> {
    let storage = HASHMAP_STORAGE.read().unwrap();
    let map = storage
        .get(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid hashmap handle: {}", handle)))?;
    Ok(map.len())
}

/// Clear a HashMap
fn hashmap_clear_impl(handle: u64) -> Result<(), ExtError> {
    let mut storage = HASHMAP_STORAGE.write().unwrap();
    let map = storage
        .get_mut(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid hashmap handle: {}", handle)))?;
    map.clear();
    Ok(())
}

/// Free a HashMap
fn hashmap_free_impl(handle: u64) -> Result<(), ExtError> {
    let mut storage = HASHMAP_STORAGE.write().unwrap();
    storage
        .remove(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid hashmap handle: {}", handle)))?;
    Ok(())
}

/// Get all keys from a HashMap as a Vec (returns Vec handle)
fn hashmap_keys_impl(handle: u64) -> Result<u64, ExtError> {
    let storage = HASHMAP_STORAGE.read().unwrap();
    let map = storage
        .get(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid hashmap handle: {}", handle)))?;

    let keys: Vec<u64> = map.keys().copied().collect();
    let vec_handle = next_handle();

    drop(storage); // Release read lock before acquiring write lock

    let mut vec_storage = VEC_STORAGE.write().unwrap();
    vec_storage.insert(vec_handle, keys);

    Ok(vec_handle)
}

/// Get all values from a HashMap as a Vec (returns Vec handle)
fn hashmap_values_impl(handle: u64) -> Result<u64, ExtError> {
    let storage = HASHMAP_STORAGE.read().unwrap();
    let map = storage
        .get(&handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid hashmap handle: {}", handle)))?;

    let values: Vec<u64> = map.values().copied().collect();
    let vec_handle = next_handle();

    drop(storage);

    let mut vec_storage = VEC_STORAGE.write().unwrap();
    vec_storage.insert(vec_handle, values);

    Ok(vec_handle)
}

// =============================================================================
// Extension Registration
// =============================================================================

/// Register Vec extensions
pub fn register_vec_extensions(registry: &mut ExtensionRegistry) {
    // vec_new() -> handle
    registry.register(
        "vec_new",
        "Create a new empty Vec. Returns handle.",
        0,
        true,
        ExtCategory::Collections,
        Arc::new(|_args, outputs| {
            let handle = vec_new_impl();
            outputs[0] = handle;
            Ok(handle as i64)
        }),
    );

    // vec_with_capacity(capacity) -> handle
    registry.register(
        "vec_with_capacity",
        "Create a new Vec with specified capacity. Args: capacity. Returns handle.",
        1,
        true,
        ExtCategory::Collections,
        Arc::new(|args, outputs| {
            let capacity = args[0] as usize;
            let handle = vec_with_capacity_impl(capacity);
            outputs[0] = handle;
            Ok(handle as i64)
        }),
    );

    // vec_push(handle, value) -> 0
    registry.register(
        "vec_push",
        "Push a value onto a Vec. Args: handle, value. Returns 0 on success.",
        2,
        true,
        ExtCategory::Collections,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let value = args[1];
            vec_push_impl(handle, value)?;
            outputs[0] = 0;
            Ok(0)
        }),
    );

    // vec_pop(handle) -> value or -1 if empty
    registry.register(
        "vec_pop",
        "Pop a value from a Vec. Args: handle. Returns value or -1 if empty.",
        1,
        true,
        ExtCategory::Collections,
        Arc::new(|args, outputs| {
            let handle = args[0];
            match vec_pop_impl(handle)? {
                Some(value) => {
                    outputs[0] = value;
                    Ok(value as i64)
                }
                None => {
                    outputs[0] = u64::MAX; // -1 as unsigned
                    Ok(-1)
                }
            }
        }),
    );

    // vec_get(handle, index) -> value or -1 if out of bounds
    registry.register(
        "vec_get",
        "Get a value from a Vec by index. Args: handle, index. Returns value or -1 if out of bounds.",
        2,
        true,
        ExtCategory::Collections,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let index = args[1] as usize;
            match vec_get_impl(handle, index)? {
                Some(value) => {
                    outputs[0] = value;
                    Ok(value as i64)
                }
                None => {
                    outputs[0] = u64::MAX;
                    Ok(-1)
                }
            }
        }),
    );

    // vec_set(handle, index, value) -> 0 or error
    registry.register(
        "vec_set",
        "Set a value in a Vec by index. Args: handle, index, value. Returns 0 on success.",
        3,
        true,
        ExtCategory::Collections,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let index = args[1] as usize;
            let value = args[2];
            vec_set_impl(handle, index, value)?;
            outputs[0] = 0;
            Ok(0)
        }),
    );

    // vec_len(handle) -> length
    registry.register(
        "vec_len",
        "Get the length of a Vec. Args: handle. Returns length.",
        1,
        true,
        ExtCategory::Collections,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let len = vec_len_impl(handle)?;
            outputs[0] = len as u64;
            Ok(len as i64)
        }),
    );

    // vec_capacity(handle) -> capacity
    registry.register(
        "vec_capacity",
        "Get the capacity of a Vec. Args: handle. Returns capacity.",
        1,
        true,
        ExtCategory::Collections,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let cap = vec_capacity_impl(handle)?;
            outputs[0] = cap as u64;
            Ok(cap as i64)
        }),
    );

    // vec_clear(handle) -> 0
    registry.register(
        "vec_clear",
        "Clear all elements from a Vec. Args: handle. Returns 0.",
        1,
        true,
        ExtCategory::Collections,
        Arc::new(|args, outputs| {
            let handle = args[0];
            vec_clear_impl(handle)?;
            outputs[0] = 0;
            Ok(0)
        }),
    );

    // vec_free(handle) -> 0
    registry.register(
        "vec_free",
        "Free a Vec and release its memory. Args: handle. Returns 0.",
        1,
        true,
        ExtCategory::Collections,
        Arc::new(|args, outputs| {
            let handle = args[0];
            vec_free_impl(handle)?;
            outputs[0] = 0;
            Ok(0)
        }),
    );

    // vec_insert(handle, index, value) -> 0
    registry.register(
        "vec_insert",
        "Insert a value at an index. Args: handle, index, value. Returns 0.",
        3,
        true,
        ExtCategory::Collections,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let index = args[1] as usize;
            let value = args[2];
            vec_insert_impl(handle, index, value)?;
            outputs[0] = 0;
            Ok(0)
        }),
    );

    // vec_remove(handle, index) -> value
    registry.register(
        "vec_remove",
        "Remove and return a value at an index. Args: handle, index. Returns removed value.",
        2,
        true,
        ExtCategory::Collections,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let index = args[1] as usize;
            let value = vec_remove_impl(handle, index)?;
            outputs[0] = value;
            Ok(value as i64)
        }),
    );
}

/// Register HashMap extensions
pub fn register_hashmap_extensions(registry: &mut ExtensionRegistry) {
    // hashmap_new() -> handle
    registry.register(
        "hashmap_new",
        "Create a new empty HashMap. Returns handle.",
        0,
        true,
        ExtCategory::Collections,
        Arc::new(|_args, outputs| {
            let handle = hashmap_new_impl();
            outputs[0] = handle;
            Ok(handle as i64)
        }),
    );

    // hashmap_insert(handle, key, value) -> old_value or -1
    registry.register(
        "hashmap_insert",
        "Insert a key-value pair. Args: handle, key, value. Returns old value or -1 if new.",
        3,
        true,
        ExtCategory::Collections,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let key = args[1];
            let value = args[2];
            match hashmap_insert_impl(handle, key, value)? {
                Some(old) => {
                    outputs[0] = old;
                    Ok(old as i64)
                }
                None => {
                    outputs[0] = u64::MAX;
                    Ok(-1)
                }
            }
        }),
    );

    // hashmap_get(handle, key) -> value or -1
    registry.register(
        "hashmap_get",
        "Get a value by key. Args: handle, key. Returns value or -1 if not found.",
        2,
        true,
        ExtCategory::Collections,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let key = args[1];
            match hashmap_get_impl(handle, key)? {
                Some(value) => {
                    outputs[0] = value;
                    Ok(value as i64)
                }
                None => {
                    outputs[0] = u64::MAX;
                    Ok(-1)
                }
            }
        }),
    );

    // hashmap_remove(handle, key) -> old_value or -1
    registry.register(
        "hashmap_remove",
        "Remove a key-value pair. Args: handle, key. Returns old value or -1 if not found.",
        2,
        true,
        ExtCategory::Collections,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let key = args[1];
            match hashmap_remove_impl(handle, key)? {
                Some(value) => {
                    outputs[0] = value;
                    Ok(value as i64)
                }
                None => {
                    outputs[0] = u64::MAX;
                    Ok(-1)
                }
            }
        }),
    );

    // hashmap_contains(handle, key) -> 1 or 0
    registry.register(
        "hashmap_contains",
        "Check if key exists. Args: handle, key. Returns 1 if exists, 0 otherwise.",
        2,
        true,
        ExtCategory::Collections,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let key = args[1];
            let contains = hashmap_contains_impl(handle, key)?;
            outputs[0] = if contains { 1 } else { 0 };
            Ok(if contains { 1 } else { 0 })
        }),
    );

    // hashmap_len(handle) -> length
    registry.register(
        "hashmap_len",
        "Get the number of entries. Args: handle. Returns length.",
        1,
        true,
        ExtCategory::Collections,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let len = hashmap_len_impl(handle)?;
            outputs[0] = len as u64;
            Ok(len as i64)
        }),
    );

    // hashmap_clear(handle) -> 0
    registry.register(
        "hashmap_clear",
        "Remove all entries. Args: handle. Returns 0.",
        1,
        true,
        ExtCategory::Collections,
        Arc::new(|args, outputs| {
            let handle = args[0];
            hashmap_clear_impl(handle)?;
            outputs[0] = 0;
            Ok(0)
        }),
    );

    // hashmap_free(handle) -> 0
    registry.register(
        "hashmap_free",
        "Free a HashMap. Args: handle. Returns 0.",
        1,
        true,
        ExtCategory::Collections,
        Arc::new(|args, outputs| {
            let handle = args[0];
            hashmap_free_impl(handle)?;
            outputs[0] = 0;
            Ok(0)
        }),
    );

    // hashmap_keys(handle) -> vec_handle
    registry.register(
        "hashmap_keys",
        "Get all keys as a Vec. Args: handle. Returns Vec handle.",
        1,
        true,
        ExtCategory::Collections,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let vec_handle = hashmap_keys_impl(handle)?;
            outputs[0] = vec_handle;
            Ok(vec_handle as i64)
        }),
    );

    // hashmap_values(handle) -> vec_handle
    registry.register(
        "hashmap_values",
        "Get all values as a Vec. Args: handle. Returns Vec handle.",
        1,
        true,
        ExtCategory::Collections,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let vec_handle = hashmap_values_impl(handle)?;
            outputs[0] = vec_handle;
            Ok(vec_handle as i64)
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec_operations() {
        // Create vec
        let handle = vec_new_impl();
        assert!(handle > 0);

        // Push values
        vec_push_impl(handle, 10).unwrap();
        vec_push_impl(handle, 20).unwrap();
        vec_push_impl(handle, 30).unwrap();

        // Check length
        assert_eq!(vec_len_impl(handle).unwrap(), 3);

        // Get values
        assert_eq!(vec_get_impl(handle, 0).unwrap(), Some(10));
        assert_eq!(vec_get_impl(handle, 1).unwrap(), Some(20));
        assert_eq!(vec_get_impl(handle, 2).unwrap(), Some(30));
        assert_eq!(vec_get_impl(handle, 3).unwrap(), None);

        // Set value
        vec_set_impl(handle, 1, 25).unwrap();
        assert_eq!(vec_get_impl(handle, 1).unwrap(), Some(25));

        // Pop
        assert_eq!(vec_pop_impl(handle).unwrap(), Some(30));
        assert_eq!(vec_len_impl(handle).unwrap(), 2);

        // Clear
        vec_clear_impl(handle).unwrap();
        assert_eq!(vec_len_impl(handle).unwrap(), 0);

        // Free
        vec_free_impl(handle).unwrap();
        assert!(vec_len_impl(handle).is_err());
    }

    #[test]
    fn test_hashmap_operations() {
        // Create hashmap
        let handle = hashmap_new_impl();
        assert!(handle > 0);

        // Insert values
        assert_eq!(hashmap_insert_impl(handle, 1, 100).unwrap(), None);
        assert_eq!(hashmap_insert_impl(handle, 2, 200).unwrap(), None);
        assert_eq!(hashmap_insert_impl(handle, 1, 150).unwrap(), Some(100)); // Replace

        // Check length
        assert_eq!(hashmap_len_impl(handle).unwrap(), 2);

        // Get values
        assert_eq!(hashmap_get_impl(handle, 1).unwrap(), Some(150));
        assert_eq!(hashmap_get_impl(handle, 2).unwrap(), Some(200));
        assert_eq!(hashmap_get_impl(handle, 3).unwrap(), None);

        // Contains
        assert!(hashmap_contains_impl(handle, 1).unwrap());
        assert!(!hashmap_contains_impl(handle, 3).unwrap());

        // Remove
        assert_eq!(hashmap_remove_impl(handle, 1).unwrap(), Some(150));
        assert_eq!(hashmap_len_impl(handle).unwrap(), 1);

        // Free
        hashmap_free_impl(handle).unwrap();
        assert!(hashmap_len_impl(handle).is_err());
    }
}
