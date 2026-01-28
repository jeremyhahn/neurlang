//! File System Wrappers
//!
//! Safe file system operations using std::fs.

use std::fs;
use std::path::Path;
use std::time::SystemTime;

use super::{OwnedBuffer, WrapperCategory, WrapperError, WrapperRegistry, WrapperResult};

// =============================================================================
// Reading Files
// =============================================================================

/// Read entire file contents as bytes
pub fn read_file(path: &OwnedBuffer) -> WrapperResult<OwnedBuffer> {
    let path_str = path.as_str()?;
    let contents = fs::read(path_str)?;
    Ok(OwnedBuffer::from_vec(contents))
}

/// Read file as UTF-8 string
pub fn read_file_string(path: &OwnedBuffer) -> WrapperResult<OwnedBuffer> {
    let path_str = path.as_str()?;
    let contents = fs::read_to_string(path_str)?;
    Ok(OwnedBuffer::from_string(contents))
}

/// Read file lines as vector of strings
pub fn read_file_lines(path: &OwnedBuffer) -> WrapperResult<Vec<OwnedBuffer>> {
    let path_str = path.as_str()?;
    let contents = fs::read_to_string(path_str)?;
    Ok(contents.lines().map(OwnedBuffer::from_str).collect())
}

// =============================================================================
// Writing Files
// =============================================================================

/// Write bytes to file (creates or overwrites)
pub fn write_file(path: &OwnedBuffer, contents: &OwnedBuffer) -> WrapperResult<()> {
    let path_str = path.as_str()?;
    fs::write(path_str, contents.as_slice())?;
    Ok(())
}

/// Append bytes to file
pub fn append_file(path: &OwnedBuffer, contents: &OwnedBuffer) -> WrapperResult<()> {
    use std::fs::OpenOptions;
    use std::io::Write;

    let path_str = path.as_str()?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path_str)?;
    file.write_all(contents.as_slice())?;
    Ok(())
}

/// Write lines to file
pub fn write_file_lines(path: &OwnedBuffer, lines: &[OwnedBuffer]) -> WrapperResult<()> {
    let path_str = path.as_str()?;
    let contents: String = lines
        .iter()
        .filter_map(|line| line.as_str().ok())
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(path_str, contents)?;
    Ok(())
}

// =============================================================================
// File Information
// =============================================================================

/// Check if path exists
pub fn exists(path: &OwnedBuffer) -> bool {
    path.as_str()
        .map(|p| Path::new(p).exists())
        .unwrap_or(false)
}

/// Check if path is a file
pub fn is_file(path: &OwnedBuffer) -> bool {
    path.as_str()
        .map(|p| Path::new(p).is_file())
        .unwrap_or(false)
}

/// Check if path is a directory
pub fn is_dir(path: &OwnedBuffer) -> bool {
    path.as_str()
        .map(|p| Path::new(p).is_dir())
        .unwrap_or(false)
}

/// Get file size in bytes
pub fn file_size(path: &OwnedBuffer) -> WrapperResult<u64> {
    let path_str = path.as_str()?;
    let metadata = fs::metadata(path_str)?;
    Ok(metadata.len())
}

/// Get file modification time (Unix timestamp in seconds)
pub fn file_modified(path: &OwnedBuffer) -> WrapperResult<u64> {
    let path_str = path.as_str()?;
    let metadata = fs::metadata(path_str)?;
    let modified = metadata.modified()?;
    let duration = modified
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|e| WrapperError::IoError(e.to_string()))?;
    Ok(duration.as_secs())
}

// =============================================================================
// Directory Operations
// =============================================================================

/// List directory contents (file names only)
pub fn list_dir(path: &OwnedBuffer) -> WrapperResult<Vec<OwnedBuffer>> {
    let path_str = path.as_str()?;
    let entries = fs::read_dir(path_str)?;

    let mut names = Vec::new();
    for entry in entries {
        let entry = entry?;
        if let Some(name) = entry.file_name().to_str() {
            names.push(OwnedBuffer::from_str(name));
        }
    }

    Ok(names)
}

/// Create directory (and parents if needed)
pub fn create_dir(path: &OwnedBuffer) -> WrapperResult<()> {
    let path_str = path.as_str()?;
    fs::create_dir_all(path_str)?;
    Ok(())
}

/// Create directory without parents
pub fn create_dir_single(path: &OwnedBuffer) -> WrapperResult<()> {
    let path_str = path.as_str()?;
    fs::create_dir(path_str)?;
    Ok(())
}

/// Remove empty directory
pub fn remove_dir(path: &OwnedBuffer) -> WrapperResult<()> {
    let path_str = path.as_str()?;
    fs::remove_dir(path_str)?;
    Ok(())
}

/// Remove directory and all contents
pub fn remove_dir_all(path: &OwnedBuffer) -> WrapperResult<()> {
    let path_str = path.as_str()?;
    fs::remove_dir_all(path_str)?;
    Ok(())
}

// =============================================================================
// File Operations
// =============================================================================

/// Remove file
pub fn remove_file(path: &OwnedBuffer) -> WrapperResult<()> {
    let path_str = path.as_str()?;
    fs::remove_file(path_str)?;
    Ok(())
}

/// Copy file
pub fn copy_file(src: &OwnedBuffer, dst: &OwnedBuffer) -> WrapperResult<u64> {
    let src_str = src.as_str()?;
    let dst_str = dst.as_str()?;
    let bytes = fs::copy(src_str, dst_str)?;
    Ok(bytes)
}

/// Move/rename file
pub fn move_file(src: &OwnedBuffer, dst: &OwnedBuffer) -> WrapperResult<()> {
    let src_str = src.as_str()?;
    let dst_str = dst.as_str()?;
    fs::rename(src_str, dst_str)?;
    Ok(())
}

// =============================================================================
// Registration
// =============================================================================

/// Register all filesystem wrappers with the registry
pub fn register(registry: &mut WrapperRegistry) {
    // Reading
    registry.register_wrapper(
        "read_file",
        "Read entire file contents as bytes",
        WrapperCategory::FileSystem,
        1,
        &["read", "load", "file", "open", "get file"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg("No path provided".to_string()));
            }
            read_file(&args[0])
        },
    );

    registry.register_wrapper(
        "read_file_string",
        "Read file as UTF-8 string",
        WrapperCategory::FileSystem,
        1,
        &["read text", "read string", "load text"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg("No path provided".to_string()));
            }
            read_file_string(&args[0])
        },
    );

    // Writing
    registry.register_wrapper(
        "write_file",
        "Write bytes to file (creates or overwrites)",
        WrapperCategory::FileSystem,
        2,
        &["write", "save", "store", "put file"],
        |args| {
            if args.len() < 2 {
                return Err(WrapperError::InvalidArg(
                    "Need path and contents".to_string(),
                ));
            }
            write_file(&args[0], &args[1])?;
            Ok(OwnedBuffer::new())
        },
    );

    registry.register_wrapper(
        "append_file",
        "Append bytes to file",
        WrapperCategory::FileSystem,
        2,
        &["append", "add to file"],
        |args| {
            if args.len() < 2 {
                return Err(WrapperError::InvalidArg(
                    "Need path and contents".to_string(),
                ));
            }
            append_file(&args[0], &args[1])?;
            Ok(OwnedBuffer::new())
        },
    );

    // File info
    registry.register_wrapper(
        "file_exists",
        "Check if path exists",
        WrapperCategory::FileSystem,
        1,
        &["exists", "is present", "check file"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg("No path provided".to_string()));
            }
            let result = if exists(&args[0]) { 1u8 } else { 0u8 };
            Ok(OwnedBuffer::from_slice(&[result]))
        },
    );

    registry.register_wrapper(
        "is_file",
        "Check if path is a file",
        WrapperCategory::FileSystem,
        1,
        &["is file", "is regular file"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg("No path provided".to_string()));
            }
            let result = if is_file(&args[0]) { 1u8 } else { 0u8 };
            Ok(OwnedBuffer::from_slice(&[result]))
        },
    );

    registry.register_wrapper(
        "is_dir",
        "Check if path is a directory",
        WrapperCategory::FileSystem,
        1,
        &["is directory", "is dir", "is folder"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg("No path provided".to_string()));
            }
            let result = if is_dir(&args[0]) { 1u8 } else { 0u8 };
            Ok(OwnedBuffer::from_slice(&[result]))
        },
    );

    registry.register_wrapper(
        "file_size",
        "Get file size in bytes",
        WrapperCategory::FileSystem,
        1,
        &["size", "file size", "length"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg("No path provided".to_string()));
            }
            let size = file_size(&args[0])?;
            Ok(OwnedBuffer::from_vec(size.to_le_bytes().to_vec()))
        },
    );

    // Directory operations
    registry.register_wrapper(
        "list_dir",
        "List directory contents",
        WrapperCategory::FileSystem,
        1,
        &["list", "ls", "dir", "readdir", "directory"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg("No path provided".to_string()));
            }
            let entries = list_dir(&args[0])?;
            // Return as newline-separated names
            let result: String = entries
                .iter()
                .filter_map(|e| e.as_str().ok())
                .collect::<Vec<_>>()
                .join("\n");
            Ok(OwnedBuffer::from_string(result))
        },
    );

    registry.register_wrapper(
        "create_dir",
        "Create directory (and parents)",
        WrapperCategory::FileSystem,
        1,
        &["mkdir", "create directory", "make dir"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg("No path provided".to_string()));
            }
            create_dir(&args[0])?;
            Ok(OwnedBuffer::new())
        },
    );

    registry.register_wrapper(
        "remove_file",
        "Remove file",
        WrapperCategory::FileSystem,
        1,
        &["delete", "remove", "rm", "unlink", "erase"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg("No path provided".to_string()));
            }
            remove_file(&args[0])?;
            Ok(OwnedBuffer::new())
        },
    );

    registry.register_wrapper(
        "remove_dir",
        "Remove empty directory",
        WrapperCategory::FileSystem,
        1,
        &["rmdir", "remove directory", "delete dir"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg("No path provided".to_string()));
            }
            remove_dir(&args[0])?;
            Ok(OwnedBuffer::new())
        },
    );

    registry.register_wrapper(
        "copy_file",
        "Copy file",
        WrapperCategory::FileSystem,
        2,
        &["copy", "cp", "duplicate"],
        |args| {
            if args.len() < 2 {
                return Err(WrapperError::InvalidArg(
                    "Need source and destination".to_string(),
                ));
            }
            let bytes = copy_file(&args[0], &args[1])?;
            Ok(OwnedBuffer::from_vec(bytes.to_le_bytes().to_vec()))
        },
    );

    registry.register_wrapper(
        "move_file",
        "Move/rename file",
        WrapperCategory::FileSystem,
        2,
        &["move", "rename", "mv"],
        |args| {
            if args.len() < 2 {
                return Err(WrapperError::InvalidArg(
                    "Need source and destination".to_string(),
                ));
            }
            move_file(&args[0], &args[1])?;
            Ok(OwnedBuffer::new())
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_read_write_file() {
        let path = OwnedBuffer::from_str("/tmp/neurlang_test_rw.txt");
        let content = OwnedBuffer::from_str("Hello, World!");

        write_file(&path, &content).unwrap();
        let read = read_file(&path).unwrap();

        assert_eq!(content, read);

        // Cleanup
        fs::remove_file("/tmp/neurlang_test_rw.txt").ok();
    }

    #[test]
    fn test_append_file() {
        let path = OwnedBuffer::from_str("/tmp/neurlang_test_append.txt");
        let content1 = OwnedBuffer::from_str("Hello");
        let content2 = OwnedBuffer::from_str(", World!");

        write_file(&path, &content1).unwrap();
        append_file(&path, &content2).unwrap();

        let read = read_file(&path).unwrap();
        assert_eq!(read.as_str().unwrap(), "Hello, World!");

        // Cleanup
        fs::remove_file("/tmp/neurlang_test_append.txt").ok();
    }

    #[test]
    fn test_exists() {
        let exists_path = OwnedBuffer::from_str("/tmp");
        let not_exists = OwnedBuffer::from_str("/nonexistent_path_xyz");

        assert!(exists(&exists_path));
        assert!(!exists(&not_exists));
    }

    #[test]
    fn test_is_file_is_dir() {
        let dir_path = OwnedBuffer::from_str("/tmp");
        let file_path = OwnedBuffer::from_str("/tmp/neurlang_test_isfile.txt");

        // Create a test file
        write_file(&file_path, &OwnedBuffer::from_str("test")).unwrap();

        assert!(is_dir(&dir_path));
        assert!(!is_file(&dir_path));
        assert!(is_file(&file_path));
        assert!(!is_dir(&file_path));

        // Cleanup
        fs::remove_file("/tmp/neurlang_test_isfile.txt").ok();
    }

    #[test]
    fn test_file_size() {
        let path = OwnedBuffer::from_str("/tmp/neurlang_test_size.txt");
        let content = OwnedBuffer::from_str("12345");

        write_file(&path, &content).unwrap();
        let size = file_size(&path).unwrap();

        assert_eq!(size, 5);

        // Cleanup
        fs::remove_file("/tmp/neurlang_test_size.txt").ok();
    }

    #[test]
    fn test_create_and_list_dir() {
        let dir_path = OwnedBuffer::from_str("/tmp/neurlang_test_dir");

        // Create directory
        create_dir(&dir_path).unwrap();

        // Create some files
        write_file(
            &OwnedBuffer::from_str("/tmp/neurlang_test_dir/file1.txt"),
            &OwnedBuffer::from_str("content1"),
        )
        .unwrap();
        write_file(
            &OwnedBuffer::from_str("/tmp/neurlang_test_dir/file2.txt"),
            &OwnedBuffer::from_str("content2"),
        )
        .unwrap();

        // List directory
        let entries = list_dir(&dir_path).unwrap();
        assert!(entries.len() >= 2);

        // Cleanup
        fs::remove_dir_all("/tmp/neurlang_test_dir").ok();
    }

    #[test]
    fn test_copy_file() {
        let src = OwnedBuffer::from_str("/tmp/neurlang_test_copy_src.txt");
        let dst = OwnedBuffer::from_str("/tmp/neurlang_test_copy_dst.txt");
        let content = OwnedBuffer::from_str("Copy me!");

        write_file(&src, &content).unwrap();
        copy_file(&src, &dst).unwrap();

        let dst_content = read_file(&dst).unwrap();
        assert_eq!(content, dst_content);

        // Cleanup
        fs::remove_file("/tmp/neurlang_test_copy_src.txt").ok();
        fs::remove_file("/tmp/neurlang_test_copy_dst.txt").ok();
    }
}
