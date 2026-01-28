# Buffer Module

Memory-safe buffer type for passing data between wrapper operations.

## Overview

The `OwnedBuffer` type is the foundation of the safe wrappers system. Unlike the existing `SafeBuffer` in `runtime::extensions` which uses raw pointers, `OwnedBuffer` owns its memory via `Vec<u8>`, making it safe to pass between operations without lifetime concerns.

## OwnedBuffer Type

### Properties

- **Owns its memory** - No dangling pointers
- **Bounds-checked access** - All operations check bounds
- **Automatically freed** - Dropped when out of scope
- **Efficiently passed** - Can be cloned or moved between wrappers

### Creation

```rust
// Empty buffer
let buf = OwnedBuffer::new();

// With capacity (avoids reallocation)
let buf = OwnedBuffer::with_capacity(1024);

// Filled with zeros
let buf = OwnedBuffer::zeroed(256);

// From existing data
let buf = OwnedBuffer::from_vec(vec![1, 2, 3]);
let buf = OwnedBuffer::from_slice(b"hello");
let buf = OwnedBuffer::from_str("hello");
let buf = OwnedBuffer::from_string(String::from("hello"));
```

### Access

```rust
// Length and capacity
buf.len()           // Number of bytes
buf.is_empty()      // True if len == 0
buf.capacity()      // Allocated capacity

// Read data
buf.as_slice()      // &[u8]
buf.as_str()        // Result<&str, Utf8Error>
buf.get(index)      // Option<u8>
buf.get_range(0, 5) // Option<&[u8]>

// Write data
buf.as_mut_slice()  // &mut [u8]
buf.set(0, 42)      // Set byte at index
buf.extend(b"more") // Append bytes
buf.clear()         // Empty the buffer
buf.resize(100, 0)  // Resize with fill value
buf.truncate(50)    // Shrink to length
```

### Conversion

```rust
// To owned types
buf.into_vec()      // Consume and return Vec<u8>
buf.to_string()     // Result<String, FromUtf8Error>

// Slicing (creates copies)
buf.slice(0, 10)    // OwnedBuffer of bytes 0-9
buf.split_at(5)     // (OwnedBuffer, OwnedBuffer)
```

### Traits

```rust
// From conversions
let buf: OwnedBuffer = vec![1, 2, 3].into();
let buf: OwnedBuffer = b"hello".as_slice().into();
let buf: OwnedBuffer = "hello".into();
let buf: OwnedBuffer = String::from("hello").into();

// AsRef/AsMut
fn process(data: impl AsRef<[u8]>) { ... }
process(&buf);  // Works!

// Deref to [u8]
let len = buf.len();    // Deref to slice
let first = buf[0];     // Index access
```

## Handle Management

For passing buffers across FFI boundaries (e.g., through the extension system), use `HandleManager`:

### Storing Buffers

```rust
use neurlang::wrappers::{OwnedBuffer, HandleManager, BufferHandle};

// Store a buffer and get a handle
let buf = OwnedBuffer::from_str("secret data");
let handle: BufferHandle = HandleManager::store(buf);

// Handle is just a u64 that can be passed through registers
assert!(handle > 0);
```

### Retrieving Buffers

```rust
// Get a copy of the buffer
let buf: Option<OwnedBuffer> = HandleManager::get(handle);

// Or use a closure to avoid copying
let len = HandleManager::with(handle, |buf| buf.len());

// Mutate in place
HandleManager::with_mut(handle, |buf| buf.extend(b"!"));
```

### Cleanup

```rust
// Remove and return the buffer
let buf = HandleManager::remove(handle);

// Check existence
if HandleManager::exists(handle) {
    // ...
}

// Clear all handles (for testing)
HandleManager::clear();
```

## Integration with Wrappers

All wrappers use `OwnedBuffer` for input and output:

```rust
// Compression example
pub fn compress(input: &OwnedBuffer) -> WrapperResult<OwnedBuffer> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(input.as_slice())?;
    let compressed = encoder.finish()?;
    Ok(OwnedBuffer::from_vec(compressed))
}

// Usage
let data = OwnedBuffer::from_str("Hello, World!");
let compressed = compress(&data)?;
let decompressed = decompress(&compressed)?;
assert_eq!(data, decompressed);
```

## Memory Safety Guarantees

### No Dangling Pointers

```rust
// OwnedBuffer owns its data
let buf = OwnedBuffer::from_str("hello");
let slice = buf.as_slice();  // Borrow
// slice is valid as long as buf exists
drop(buf);
// slice is now invalid - but Rust's borrow checker prevents this!
```

### Bounds Checking

```rust
let buf = OwnedBuffer::from_slice(b"hello");

// Safe access - returns Option
assert_eq!(buf.get(0), Some(b'h'));
assert_eq!(buf.get(100), None);  // Out of bounds returns None

// Range access
assert_eq!(buf.get_range(0, 5), Some(b"hello".as_slice()));
assert_eq!(buf.get_range(0, 100), None);  // Out of bounds
```

### No Data Races

```rust
// HandleManager uses RwLock internally
// Multiple readers OR single writer, never both
let handle = HandleManager::store(OwnedBuffer::from_str("data"));

// Thread 1: Read
std::thread::spawn(move || {
    HandleManager::with(handle, |buf| println!("{}", buf.len()));
});

// Thread 2: Also read (allowed!)
HandleManager::with(handle, |buf| println!("{:?}", buf.as_slice()));
```

## Performance Considerations

### Avoid Unnecessary Copies

```rust
// Bad: Multiple copies
let data = get_data();
let buf = OwnedBuffer::from_slice(&data);  // Copy 1
let result = process(&buf);
let output = result.into_vec();  // Move, no copy

// Good: Move semantics
let data = get_data();
let buf = OwnedBuffer::from_vec(data);  // Move, no copy
let result = process(&buf);
let output = result.into_vec();  // Move, no copy
```

### Pre-allocate When Size is Known

```rust
// Bad: Multiple reallocations
let mut buf = OwnedBuffer::new();
for chunk in chunks {
    buf.extend(chunk);  // May reallocate each time
}

// Good: Single allocation
let total_size: usize = chunks.iter().map(|c| c.len()).sum();
let mut buf = OwnedBuffer::with_capacity(total_size);
for chunk in chunks {
    buf.extend(chunk);  // No reallocation
}
```

### Use Handles for FFI

```rust
// When crossing FFI boundary, use handles instead of passing buffers
let handle = HandleManager::store(large_buffer);
// Pass handle (u64) through FFI
// Other side retrieves with HandleManager::get(handle)
```

## Error Handling

The buffer module integrates with `WrapperError`:

```rust
use neurlang::wrappers::{WrapperResult, WrapperError};

fn process_utf8(buf: &OwnedBuffer) -> WrapperResult<String> {
    buf.as_str()
        .map(|s| s.to_string())
        .map_err(|e| WrapperError::EncodingError(e.to_string()))
}
```

## Example: Complete Workflow

```rust
use neurlang::wrappers::{OwnedBuffer, HandleManager, compression};

// 1. Create input buffer
let input = OwnedBuffer::from_str("Hello, World! ".repeat(100).as_str());
println!("Original size: {}", input.len());

// 2. Compress
let compressed = compression::compress(&input)?;
println!("Compressed size: {}", compressed.len());

// 3. Store as handle for passing through system
let handle = HandleManager::store(compressed);

// 4. Later: retrieve and decompress
let compressed = HandleManager::remove(handle).unwrap();
let decompressed = compression::decompress(&compressed)?;

// 5. Verify
assert_eq!(input, decompressed);
println!("Roundtrip successful!");
```
