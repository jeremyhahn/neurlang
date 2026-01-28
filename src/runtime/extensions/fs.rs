//! File System Extensions
//!
//! Safe file system operations using std::fs.

use std::fs;
use std::path::Path;
use std::sync::Arc;

use super::buffer::{HandleManager, OwnedBuffer};
use super::{ext_ids, ExtCategory, ExtError, ExtensionRegistry};

// =============================================================================
// Reading Files
// =============================================================================

/// Read entire file contents as bytes
fn read_file_impl(path: &str) -> Result<Vec<u8>, ExtError> {
    fs::read(path).map_err(|e| ExtError::ExtensionError(format!("Failed to read file: {}", e)))
}

/// Read file as UTF-8 string
#[allow(dead_code)]
fn read_file_string_impl(path: &str) -> Result<String, ExtError> {
    fs::read_to_string(path)
        .map_err(|e| ExtError::ExtensionError(format!("Failed to read file: {}", e)))
}

// =============================================================================
// Writing Files
// =============================================================================

/// Write bytes to file (creates or overwrites)
fn write_file_impl(path: &str, contents: &[u8]) -> Result<(), ExtError> {
    fs::write(path, contents)
        .map_err(|e| ExtError::ExtensionError(format!("Failed to write file: {}", e)))
}

// =============================================================================
// File Information
// =============================================================================

/// Check if path exists
fn exists_impl(path: &str) -> bool {
    Path::new(path).exists()
}

/// Check if path is a file
#[allow(dead_code)]
fn is_file_impl(path: &str) -> bool {
    Path::new(path).is_file()
}

/// Check if path is a directory
#[allow(dead_code)]
fn is_dir_impl(path: &str) -> bool {
    Path::new(path).is_dir()
}

// =============================================================================
// Directory Operations
// =============================================================================

/// List directory contents (file names only)
fn list_dir_impl(path: &str) -> Result<Vec<String>, ExtError> {
    let entries = fs::read_dir(path)
        .map_err(|e| ExtError::ExtensionError(format!("Failed to read directory: {}", e)))?;

    let mut names = Vec::new();
    for entry in entries {
        let entry =
            entry.map_err(|e| ExtError::ExtensionError(format!("Failed to read entry: {}", e)))?;
        if let Some(name) = entry.file_name().to_str() {
            names.push(name.to_string());
        }
    }

    Ok(names)
}

/// Create directory (and parents if needed)
fn create_dir_impl(path: &str) -> Result<(), ExtError> {
    fs::create_dir_all(path)
        .map_err(|e| ExtError::ExtensionError(format!("Failed to create directory: {}", e)))
}

// =============================================================================
// File Operations
// =============================================================================

/// Remove file
fn remove_file_impl(path: &str) -> Result<(), ExtError> {
    fs::remove_file(path)
        .map_err(|e| ExtError::ExtensionError(format!("Failed to remove file: {}", e)))
}

// =============================================================================
// Extension Registration
// =============================================================================

pub fn register_fs(registry: &mut ExtensionRegistry) {
    // fs_read: Read file contents
    registry.register_with_id(
        ext_ids::FS_READ,
        "fs_read",
        "Read file contents. Args: path_handle. Returns buffer_handle.",
        1,
        true,
        ExtCategory::FileSystem,
        Arc::new(|args, outputs| {
            let path_handle = args[0];

            let path_buf = HandleManager::get(path_handle)
                .ok_or_else(|| ExtError::ExtensionError("Invalid path handle".to_string()))?;
            let path = path_buf
                .as_str()
                .map_err(|e| ExtError::ExtensionError(format!("Invalid UTF-8 in path: {}", e)))?;

            let contents = read_file_impl(path)?;
            let result_handle = HandleManager::store(OwnedBuffer::from_vec(contents));
            outputs[0] = result_handle;
            Ok(result_handle as i64)
        }),
    );

    // fs_write: Write to file
    registry.register_with_id(
        ext_ids::FS_WRITE,
        "fs_write",
        "Write to file. Args: path_handle, contents_handle. Returns 0 on success.",
        2,
        true,
        ExtCategory::FileSystem,
        Arc::new(|args, outputs| {
            let path_handle = args[0];
            let contents_handle = args[1];

            let path_buf = HandleManager::get(path_handle)
                .ok_or_else(|| ExtError::ExtensionError("Invalid path handle".to_string()))?;
            let contents_buf = HandleManager::get(contents_handle)
                .ok_or_else(|| ExtError::ExtensionError("Invalid contents handle".to_string()))?;

            let path = path_buf
                .as_str()
                .map_err(|e| ExtError::ExtensionError(format!("Invalid UTF-8 in path: {}", e)))?;

            write_file_impl(path, contents_buf.as_slice())?;
            outputs[0] = 0;
            Ok(0)
        }),
    );

    // fs_exists: Check if path exists
    registry.register_with_id(
        ext_ids::FS_EXISTS,
        "fs_exists",
        "Check if path exists. Args: path_handle. Returns 1 if exists, 0 otherwise.",
        1,
        true,
        ExtCategory::FileSystem,
        Arc::new(|args, outputs| {
            let path_handle = args[0];

            let path_buf = HandleManager::get(path_handle)
                .ok_or_else(|| ExtError::ExtensionError("Invalid path handle".to_string()))?;
            let path = path_buf
                .as_str()
                .map_err(|e| ExtError::ExtensionError(format!("Invalid UTF-8 in path: {}", e)))?;

            let result = if exists_impl(path) { 1i64 } else { 0i64 };
            outputs[0] = result as u64;
            Ok(result)
        }),
    );

    // fs_delete: Delete file
    registry.register_with_id(
        ext_ids::FS_DELETE,
        "fs_delete",
        "Delete file. Args: path_handle. Returns 0 on success.",
        1,
        true,
        ExtCategory::FileSystem,
        Arc::new(|args, outputs| {
            let path_handle = args[0];

            let path_buf = HandleManager::get(path_handle)
                .ok_or_else(|| ExtError::ExtensionError("Invalid path handle".to_string()))?;
            let path = path_buf
                .as_str()
                .map_err(|e| ExtError::ExtensionError(format!("Invalid UTF-8 in path: {}", e)))?;

            remove_file_impl(path)?;
            outputs[0] = 0;
            Ok(0)
        }),
    );

    // fs_mkdir: Create directory
    registry.register_with_id(
        ext_ids::FS_MKDIR,
        "fs_mkdir",
        "Create directory (and parents). Args: path_handle. Returns 0 on success.",
        1,
        true,
        ExtCategory::FileSystem,
        Arc::new(|args, outputs| {
            let path_handle = args[0];

            let path_buf = HandleManager::get(path_handle)
                .ok_or_else(|| ExtError::ExtensionError("Invalid path handle".to_string()))?;
            let path = path_buf
                .as_str()
                .map_err(|e| ExtError::ExtensionError(format!("Invalid UTF-8 in path: {}", e)))?;

            create_dir_impl(path)?;
            outputs[0] = 0;
            Ok(0)
        }),
    );

    // fs_list_dir: List directory contents
    registry.register_with_id(
        ext_ids::FS_LIST_DIR,
        "fs_list_dir",
        "List directory contents. Args: path_handle. Returns vec_handle of string handles.",
        1,
        true,
        ExtCategory::FileSystem,
        Arc::new(|args, outputs| {
            let path_handle = args[0];

            let path_buf = HandleManager::get(path_handle)
                .ok_or_else(|| ExtError::ExtensionError("Invalid path handle".to_string()))?;
            let path = path_buf
                .as_str()
                .map_err(|e| ExtError::ExtensionError(format!("Invalid UTF-8 in path: {}", e)))?;

            let names = list_dir_impl(path)?;

            // Store each name as a string handle
            let mut handles = Vec::with_capacity(names.len());
            for name in names {
                let h = super::strings::next_handle();
                super::strings::STRING_STORAGE
                    .write()
                    .unwrap()
                    .insert(h, name);
                handles.push(h);
            }

            // Store the vec of handles
            let vec_handle = super::collections::next_handle();
            super::collections::VEC_STORAGE
                .write()
                .unwrap()
                .insert(vec_handle, handles);

            outputs[0] = vec_handle;
            Ok(vec_handle as i64)
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_read_write_file() {
        let path = "/tmp/neurlang_ext_test_rw.txt";
        let content = b"Hello, World!";

        write_file_impl(path, content).unwrap();
        let read = read_file_impl(path).unwrap();
        assert_eq!(content.as_slice(), read.as_slice());

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_exists() {
        assert!(exists_impl("/tmp"));
        assert!(!exists_impl("/nonexistent_path_xyz_12345"));
    }

    #[test]
    fn test_is_file_is_dir() {
        assert!(is_dir_impl("/tmp"));
        assert!(!is_file_impl("/tmp"));
    }

    #[test]
    fn test_create_and_list_dir() {
        let dir_path = "/tmp/neurlang_ext_test_dir";

        // Create directory
        create_dir_impl(dir_path).unwrap();
        assert!(is_dir_impl(dir_path));

        // Create some files
        write_file_impl(&format!("{}/file1.txt", dir_path), b"content1").unwrap();
        write_file_impl(&format!("{}/file2.txt", dir_path), b"content2").unwrap();

        // List directory
        let entries = list_dir_impl(dir_path).unwrap();
        assert!(entries.len() >= 2);

        // Cleanup
        fs::remove_dir_all(dir_path).ok();
    }
}
