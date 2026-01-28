# File System Module

Safe file system operations.

## Overview

The file system module provides memory-safe wrappers around `std::fs` operations. All operations use `OwnedBuffer` for data transfer and return proper error types.

## API Reference

### Reading Files

```rust
/// Read entire file contents as bytes
pub fn read_file(path: &OwnedBuffer) -> WrapperResult<OwnedBuffer>;

/// Read file as UTF-8 string
pub fn read_file_string(path: &OwnedBuffer) -> WrapperResult<OwnedBuffer>;

/// Read file lines as vector of strings
pub fn read_file_lines(path: &OwnedBuffer) -> WrapperResult<Vec<OwnedBuffer>>;
```

### Writing Files

```rust
/// Write bytes to file (creates or overwrites)
pub fn write_file(path: &OwnedBuffer, contents: &OwnedBuffer) -> WrapperResult<()>;

/// Append bytes to file
pub fn append_file(path: &OwnedBuffer, contents: &OwnedBuffer) -> WrapperResult<()>;

/// Write lines to file
pub fn write_file_lines(path: &OwnedBuffer, lines: &[OwnedBuffer]) -> WrapperResult<()>;
```

### File Information

```rust
/// Check if path exists
pub fn exists(path: &OwnedBuffer) -> bool;

/// Check if path is a file
pub fn is_file(path: &OwnedBuffer) -> bool;

/// Check if path is a directory
pub fn is_dir(path: &OwnedBuffer) -> bool;

/// Get file size in bytes
pub fn file_size(path: &OwnedBuffer) -> WrapperResult<u64>;

/// Get file modification time (Unix timestamp)
pub fn file_modified(path: &OwnedBuffer) -> WrapperResult<u64>;
```

### Directory Operations

```rust
/// List directory contents (file names only)
pub fn list_dir(path: &OwnedBuffer) -> WrapperResult<Vec<OwnedBuffer>>;

/// Create directory (and parents if needed)
pub fn create_dir(path: &OwnedBuffer) -> WrapperResult<()>;

/// Create directory without parents
pub fn create_dir_single(path: &OwnedBuffer) -> WrapperResult<()>;

/// Remove empty directory
pub fn remove_dir(path: &OwnedBuffer) -> WrapperResult<()>;

/// Remove directory and all contents
pub fn remove_dir_all(path: &OwnedBuffer) -> WrapperResult<()>;
```

### File Operations

```rust
/// Remove file
pub fn remove_file(path: &OwnedBuffer) -> WrapperResult<()>;

/// Copy file
pub fn copy_file(src: &OwnedBuffer, dst: &OwnedBuffer) -> WrapperResult<u64>;

/// Move/rename file
pub fn move_file(src: &OwnedBuffer, dst: &OwnedBuffer) -> WrapperResult<()>;
```

## Usage Examples

### Reading and Writing

```rust
use neurlang::wrappers::{OwnedBuffer, fs};

// Write a file
let path = OwnedBuffer::from_str("/tmp/test.txt");
let content = OwnedBuffer::from_str("Hello, World!");
fs::write_file(&path, &content)?;

// Read it back
let read_content = fs::read_file(&path)?;
assert_eq!(content, read_content);

// Append
let more = OwnedBuffer::from_str("\nMore content");
fs::append_file(&path, &more)?;
```

### Working with Lines

```rust
// Write lines
let path = OwnedBuffer::from_str("/tmp/lines.txt");
let lines = vec![
    OwnedBuffer::from_str("Line 1"),
    OwnedBuffer::from_str("Line 2"),
    OwnedBuffer::from_str("Line 3"),
];
fs::write_file_lines(&path, &lines)?;

// Read lines back
let read_lines = fs::read_file_lines(&path)?;
assert_eq!(read_lines.len(), 3);
```

### File Checks

```rust
let path = OwnedBuffer::from_str("/tmp/test.txt");

if fs::exists(&path) {
    if fs::is_file(&path) {
        let size = fs::file_size(&path)?;
        println!("File size: {} bytes", size);
    } else if fs::is_dir(&path) {
        println!("It's a directory");
    }
}
```

### Directory Operations

```rust
// Create directory tree
let dir = OwnedBuffer::from_str("/tmp/my/nested/dir");
fs::create_dir(&dir)?;  // Creates all parent dirs

// List contents
let parent = OwnedBuffer::from_str("/tmp/my");
let entries = fs::list_dir(&parent)?;
for entry in entries {
    println!("  {}", entry.as_str().unwrap());
}

// Clean up
fs::remove_dir_all(&OwnedBuffer::from_str("/tmp/my"))?;
```

### Copy and Move

```rust
let src = OwnedBuffer::from_str("/tmp/original.txt");
let dst = OwnedBuffer::from_str("/tmp/copy.txt");

// Copy
let bytes_copied = fs::copy_file(&src, &dst)?;
println!("Copied {} bytes", bytes_copied);

// Move
let new_location = OwnedBuffer::from_str("/tmp/moved.txt");
fs::move_file(&dst, &new_location)?;
```

## IR Assembly Usage

```asm
; Read file
mov r1, path_ptr
ext.call r0, @"read file", r1

; Write file
mov r1, path_ptr
mov r2, content_ptr
ext.call r0, @"write file", r1, r2

; Check exists
mov r1, path_ptr
ext.call r0, @"file exists", r1
; r0 = 1 if exists, 0 if not

; List directory
mov r1, dir_ptr
ext.call r0, @"list dir", r1
; r0 = handle to vec of names

; Create directory
mov r1, dir_ptr
ext.call r0, @"create dir", r1

; Copy file
mov r1, src_ptr
mov r2, dst_ptr
ext.call r0, @"copy file", r1, r2
```

## RAG Keywords

| Intent | Resolves To |
|--------|-------------|
| "read file", "load file", "get file", "open" | `read_file` |
| "write file", "save file", "store file" | `write_file` |
| "append file", "add to file" | `append_file` |
| "file exists", "check file" | `exists` |
| "is file", "is regular file" | `is_file` |
| "is directory", "is dir", "is folder" | `is_dir` |
| "list directory", "ls", "dir", "readdir" | `list_dir` |
| "create directory", "mkdir", "make dir" | `create_dir` |
| "remove file", "delete file", "rm", "unlink" | `remove_file` |
| "remove directory", "rmdir", "delete dir" | `remove_dir` |
| "copy file", "cp" | `copy_file` |
| "move file", "rename file", "mv" | `move_file` |
| "file size", "size of file" | `file_size` |

## Error Handling

```rust
use neurlang::wrappers::{WrapperError, fs};

match fs::read_file(&path) {
    Ok(content) => {
        println!("Read {} bytes", content.len());
    }
    Err(WrapperError::IoError(msg)) => {
        // File not found, permission denied, etc.
        eprintln!("IO error: {}", msg);
    }
    Err(e) => {
        eprintln!("Unexpected error: {}", e);
    }
}
```

Common IO errors:
- `IoError("No such file or directory")` - Path doesn't exist
- `IoError("Permission denied")` - Insufficient permissions
- `IoError("Is a directory")` - Tried to read directory as file
- `IoError("Not a directory")` - Tried to list file as directory
- `IoError("Directory not empty")` - Can't remove non-empty dir

## Security Considerations

### Path Traversal

The module does NOT perform path sanitization. Callers should validate paths:

```rust
// Potentially dangerous - user input could contain ../
let user_path = get_user_input();

// Validate first
if user_path.contains("..") {
    return Err(WrapperError::InvalidArg("Path traversal not allowed"));
}

// Now safe to use
fs::read_file(&OwnedBuffer::from_str(&user_path))?;
```

### Permissions

File operations use the process's permissions. For sandboxed execution:
- Use the IO permissions system from `stencil::IOPermissions`
- Validate paths against allowed directories
- Consider using a chroot or container

## Performance Considerations

### Buffering

- `read_file` reads entire file into memory
- For very large files, consider streaming (future API)
- `write_file` writes atomically

### Directory Listing

- `list_dir` returns all entries at once
- For directories with many files, this may use significant memory
- Future: streaming directory iteration

## Implementation Notes

### Atomic Writes

`write_file` should ideally write to a temp file and rename:

```rust
// Implementation approach (simplified)
fn write_file(path: &str, content: &[u8]) -> Result<()> {
    let temp = format!("{}.tmp.{}", path, random_id());
    std::fs::write(&temp, content)?;
    std::fs::rename(&temp, path)?;  // Atomic on most filesystems
    Ok(())
}
```

### Path Handling

Paths are treated as UTF-8 strings. On systems with non-UTF-8 paths, this may fail. Future versions may support `OsString`.

## Dependencies

```toml
[dependencies]
# Uses std::fs, no additional dependencies
```

## See Also

- [Buffer Module](buffer.md) - OwnedBuffer type
- [Compression Module](compression.md) - Compress before writing
