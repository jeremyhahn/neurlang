# Compression Module

Compress and decompress data using various algorithms.

## Overview

The compression module provides safe, memory-efficient compression and decompression operations using proven Rust crates. All operations take `OwnedBuffer` inputs and return `OwnedBuffer` results.

## Supported Algorithms

| Algorithm | Crate | Speed | Ratio | Use Case |
|-----------|-------|-------|-------|----------|
| zlib/deflate | flate2 | Medium | Good | General purpose, wide compatibility |
| gzip | flate2 | Medium | Good | HTTP compression, file archives |
| lz4 | lz4 | Very Fast | Lower | Real-time compression, games |
| zstd | zstd | Fast | Excellent | Modern compression, best ratio |

## API Reference

### Zlib (Default)

```rust
/// Compress data using zlib (deflate)
pub fn compress(input: &OwnedBuffer) -> WrapperResult<OwnedBuffer>;

/// Decompress zlib-compressed data
pub fn decompress(input: &OwnedBuffer) -> WrapperResult<OwnedBuffer>;

/// Compress with specific compression level (0-9)
pub fn compress_level(input: &OwnedBuffer, level: u32) -> WrapperResult<OwnedBuffer>;
```

### Gzip

```rust
/// Compress data using gzip format
pub fn compress_gzip(input: &OwnedBuffer) -> WrapperResult<OwnedBuffer>;

/// Decompress gzip-compressed data
pub fn decompress_gzip(input: &OwnedBuffer) -> WrapperResult<OwnedBuffer>;
```

### LZ4

```rust
/// Compress data using LZ4 (fast compression)
pub fn compress_lz4(input: &OwnedBuffer) -> WrapperResult<OwnedBuffer>;

/// Decompress LZ4-compressed data
pub fn decompress_lz4(input: &OwnedBuffer) -> WrapperResult<OwnedBuffer>;
```

### Zstandard

```rust
/// Compress data using Zstandard (best ratio)
pub fn compress_zstd(input: &OwnedBuffer) -> WrapperResult<OwnedBuffer>;

/// Decompress Zstandard-compressed data
pub fn decompress_zstd(input: &OwnedBuffer) -> WrapperResult<OwnedBuffer>;

/// Compress with specific compression level (1-22)
pub fn compress_zstd_level(input: &OwnedBuffer, level: i32) -> WrapperResult<OwnedBuffer>;
```

## Usage Examples

### Basic Compression

```rust
use neurlang::wrappers::{OwnedBuffer, compression};

// Create some data
let original = OwnedBuffer::from_str("Hello, World! ".repeat(1000).as_str());
println!("Original: {} bytes", original.len());

// Compress
let compressed = compression::compress(&original)?;
println!("Compressed: {} bytes", compressed.len());

// Decompress
let restored = compression::decompress(&compressed)?;
assert_eq!(original, restored);
```

### Choosing an Algorithm

```rust
// For maximum compatibility (HTTP, legacy systems)
let compressed = compression::compress_gzip(&data)?;

// For speed (real-time, large volumes)
let compressed = compression::compress_lz4(&data)?;

// For best compression ratio (storage, archival)
let compressed = compression::compress_zstd(&data)?;
```

### Compression Levels

```rust
// Zlib: 0 (no compression) to 9 (maximum)
let fast = compression::compress_level(&data, 1)?;      // Fast, larger
let balanced = compression::compress_level(&data, 6)?;  // Default
let small = compression::compress_level(&data, 9)?;     // Slow, smaller

// Zstd: 1 to 22 (higher = slower but smaller)
let fast = compression::compress_zstd_level(&data, 1)?;
let balanced = compression::compress_zstd_level(&data, 3)?;  // Default
let small = compression::compress_zstd_level(&data, 19)?;
```

## IR Assembly Usage

```asm
; Compress data in r1, result in r0
ext.call r0, @"compress", r1

; Decompress
ext.call r0, @"decompress", r1

; Algorithm-specific
ext.call r0, @"compress gzip", r1
ext.call r0, @"compress lz4", r1
ext.call r0, @"compress zstd", r1

; With level
ext.call r0, @"compress zstd level", r1, 19
```

## RAG Keywords

The following keywords will resolve to compression operations:

| Intent | Resolves To |
|--------|-------------|
| "compress", "shrink", "deflate", "zip", "pack" | `compress` |
| "decompress", "expand", "inflate", "unzip", "unpack" | `decompress` |
| "gzip", "gz" | `compress_gzip` / `decompress_gzip` |
| "lz4", "fast compress" | `compress_lz4` / `decompress_lz4` |
| "zstd", "zstandard" | `compress_zstd` / `decompress_zstd` |

## Error Handling

```rust
use neurlang::wrappers::{WrapperError, compression};

match compression::decompress(&data) {
    Ok(decompressed) => {
        println!("Success: {} bytes", decompressed.len());
    }
    Err(WrapperError::CompressionError(msg)) => {
        // Invalid compressed data
        eprintln!("Decompression failed: {}", msg);
    }
    Err(e) => {
        eprintln!("Unexpected error: {}", e);
    }
}
```

Common errors:
- `CompressionError("invalid deflate stream")` - Data is not valid zlib
- `CompressionError("unexpected end of file")` - Truncated data
- `CompressionError("invalid gzip header")` - Not valid gzip format

## Performance Characteristics

### Compression Ratio (typical text)

| Algorithm | Ratio | Notes |
|-----------|-------|-------|
| zlib -9 | ~3:1 | Good for most data |
| gzip | ~3:1 | Same as zlib with header |
| lz4 | ~2:1 | Trades ratio for speed |
| zstd -19 | ~4:1 | Best ratio |

### Speed (approximate)

| Algorithm | Compress | Decompress |
|-----------|----------|------------|
| zlib -6 | ~50 MB/s | ~200 MB/s |
| gzip | ~50 MB/s | ~200 MB/s |
| lz4 | ~400 MB/s | ~2000 MB/s |
| zstd -3 | ~300 MB/s | ~800 MB/s |

## Implementation Notes

### Memory Safety

All compression operations:
- Allocate output buffers internally
- Never expose raw pointers
- Handle all error cases
- Clean up on failure

### Streaming (Future)

For very large files, streaming APIs may be added:

```rust
// Future API (not yet implemented)
let encoder = compression::StreamEncoder::new_zstd(output_file)?;
for chunk in input_chunks {
    encoder.write(&chunk)?;
}
encoder.finish()?;
```

## Dependencies

```toml
[dependencies]
flate2 = "1.0"    # zlib, gzip
lz4 = "1.24"      # LZ4
zstd = "0.13"     # Zstandard
```

## See Also

- [Buffer Module](buffer.md) - OwnedBuffer type
- [Encoding Module](encoding.md) - Base64 for binary-to-text
