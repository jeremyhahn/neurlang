//! Compression Wrappers
//!
//! Safe compression and decompression using proven Rust crates.
//!
//! # Supported Algorithms
//!
//! - **zlib/deflate** (flate2) - General purpose, wide compatibility
//! - **gzip** (flate2) - HTTP compression, file archives
//! - **lz4** - Very fast, lower ratio
//! - **zstd** - Best ratio, fast

use std::io::{Read, Write};

use flate2::read::{GzDecoder, ZlibDecoder};
use flate2::write::{GzEncoder, ZlibEncoder};
use flate2::Compression;

use super::{OwnedBuffer, WrapperCategory, WrapperError, WrapperRegistry, WrapperResult};

// =============================================================================
// Zlib (Default)
// =============================================================================

/// Compress data using zlib (deflate)
pub fn compress(input: &OwnedBuffer) -> WrapperResult<OwnedBuffer> {
    compress_level(input, 6)
}

/// Compress with specific compression level (0-9)
pub fn compress_level(input: &OwnedBuffer, level: u32) -> WrapperResult<OwnedBuffer> {
    let level = level.min(9);
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(level));
    encoder
        .write_all(input.as_slice())
        .map_err(|e| WrapperError::CompressionError(e.to_string()))?;
    let compressed = encoder
        .finish()
        .map_err(|e| WrapperError::CompressionError(e.to_string()))?;
    Ok(OwnedBuffer::from_vec(compressed))
}

/// Decompress zlib-compressed data
pub fn decompress(input: &OwnedBuffer) -> WrapperResult<OwnedBuffer> {
    let mut decoder = ZlibDecoder::new(input.as_slice());
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| WrapperError::CompressionError(e.to_string()))?;
    Ok(OwnedBuffer::from_vec(decompressed))
}

// =============================================================================
// Gzip
// =============================================================================

/// Compress data using gzip format
pub fn compress_gzip(input: &OwnedBuffer) -> WrapperResult<OwnedBuffer> {
    compress_gzip_level(input, 6)
}

/// Compress with gzip and specific level (0-9)
pub fn compress_gzip_level(input: &OwnedBuffer, level: u32) -> WrapperResult<OwnedBuffer> {
    let level = level.min(9);
    let mut encoder = GzEncoder::new(Vec::new(), Compression::new(level));
    encoder
        .write_all(input.as_slice())
        .map_err(|e| WrapperError::CompressionError(e.to_string()))?;
    let compressed = encoder
        .finish()
        .map_err(|e| WrapperError::CompressionError(e.to_string()))?;
    Ok(OwnedBuffer::from_vec(compressed))
}

/// Decompress gzip-compressed data
pub fn decompress_gzip(input: &OwnedBuffer) -> WrapperResult<OwnedBuffer> {
    let mut decoder = GzDecoder::new(input.as_slice());
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| WrapperError::CompressionError(e.to_string()))?;
    Ok(OwnedBuffer::from_vec(decompressed))
}

// =============================================================================
// LZ4
// =============================================================================

/// Compress data using LZ4 (fast compression)
pub fn compress_lz4(input: &OwnedBuffer) -> WrapperResult<OwnedBuffer> {
    let compressed = lz4::block::compress(input.as_slice(), None, false)
        .map_err(|e| WrapperError::CompressionError(e.to_string()))?;
    Ok(OwnedBuffer::from_vec(compressed))
}

/// Decompress LZ4-compressed data
pub fn decompress_lz4(input: &OwnedBuffer, max_size: usize) -> WrapperResult<OwnedBuffer> {
    let decompressed = lz4::block::decompress(input.as_slice(), Some(max_size as i32))
        .map_err(|e| WrapperError::CompressionError(e.to_string()))?;
    Ok(OwnedBuffer::from_vec(decompressed))
}

// =============================================================================
// Zstandard
// =============================================================================

/// Compress data using Zstandard (best ratio)
pub fn compress_zstd(input: &OwnedBuffer) -> WrapperResult<OwnedBuffer> {
    compress_zstd_level(input, 3)
}

/// Compress with Zstandard and specific level (1-22)
pub fn compress_zstd_level(input: &OwnedBuffer, level: i32) -> WrapperResult<OwnedBuffer> {
    let level = level.clamp(1, 22);
    let compressed = zstd::encode_all(input.as_slice(), level)
        .map_err(|e| WrapperError::CompressionError(e.to_string()))?;
    Ok(OwnedBuffer::from_vec(compressed))
}

/// Decompress Zstandard-compressed data
pub fn decompress_zstd(input: &OwnedBuffer) -> WrapperResult<OwnedBuffer> {
    let decompressed = zstd::decode_all(input.as_slice())
        .map_err(|e| WrapperError::CompressionError(e.to_string()))?;
    Ok(OwnedBuffer::from_vec(decompressed))
}

// =============================================================================
// Registration
// =============================================================================

/// Register all compression wrappers with the registry
pub fn register(registry: &mut WrapperRegistry) {
    // Zlib
    registry.register_wrapper(
        "compress",
        "Compress data using zlib (deflate)",
        WrapperCategory::Compression,
        1,
        &["compress", "shrink", "deflate", "zlib"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg("No input provided".to_string()));
            }
            compress(&args[0])
        },
    );

    registry.register_wrapper(
        "decompress",
        "Decompress zlib-compressed data",
        WrapperCategory::Compression,
        1,
        &["decompress", "expand", "inflate", "unzlib"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg("No input provided".to_string()));
            }
            decompress(&args[0])
        },
    );

    // Gzip
    registry.register_wrapper(
        "compress_gzip",
        "Compress data using gzip format",
        WrapperCategory::Compression,
        1,
        &["gzip", "gz", "compress gzip"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg("No input provided".to_string()));
            }
            compress_gzip(&args[0])
        },
    );

    registry.register_wrapper(
        "decompress_gzip",
        "Decompress gzip-compressed data",
        WrapperCategory::Compression,
        1,
        &["gunzip", "ungzip", "decompress gzip"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg("No input provided".to_string()));
            }
            decompress_gzip(&args[0])
        },
    );

    // LZ4
    registry.register_wrapper(
        "compress_lz4",
        "Compress data using LZ4 (fast compression)",
        WrapperCategory::Compression,
        1,
        &["lz4", "fast compress", "compress lz4"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg("No input provided".to_string()));
            }
            compress_lz4(&args[0])
        },
    );

    registry.register_wrapper(
        "decompress_lz4",
        "Decompress LZ4-compressed data",
        WrapperCategory::Compression,
        2,
        &["unlz4", "decompress lz4"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg("No input provided".to_string()));
            }
            // Default max size of 64MB
            let max_size = if args.len() > 1 {
                // Parse max size from second arg if provided
                args[1]
                    .as_slice()
                    .iter()
                    .take(8)
                    .fold(0usize, |acc, &b| (acc << 8) | (b as usize))
            } else {
                64 * 1024 * 1024
            };
            decompress_lz4(&args[0], max_size)
        },
    );

    // Zstd
    registry.register_wrapper(
        "compress_zstd",
        "Compress data using Zstandard (best ratio)",
        WrapperCategory::Compression,
        1,
        &["zstd", "zstandard", "compress zstd"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg("No input provided".to_string()));
            }
            compress_zstd(&args[0])
        },
    );

    registry.register_wrapper(
        "decompress_zstd",
        "Decompress Zstandard-compressed data",
        WrapperCategory::Compression,
        1,
        &["unzstd", "decompress zstd", "decompress zstandard"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg("No input provided".to_string()));
            }
            decompress_zstd(&args[0])
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zlib_roundtrip() {
        let original = OwnedBuffer::from_str("Hello, World! ".repeat(100).as_str());
        let compressed = compress(&original).unwrap();
        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(original, decompressed);
        assert!(compressed.len() < original.len());
    }

    #[test]
    fn test_gzip_roundtrip() {
        let original = OwnedBuffer::from_str("Test data for gzip compression");
        let compressed = compress_gzip(&original).unwrap();
        let decompressed = decompress_gzip(&compressed).unwrap();
        assert_eq!(original, decompressed);
    }

    #[test]
    fn test_lz4_roundtrip() {
        let original = OwnedBuffer::from_str("LZ4 is fast! ".repeat(50).as_str());
        let compressed = compress_lz4(&original).unwrap();
        let decompressed = decompress_lz4(&compressed, 10 * 1024 * 1024).unwrap();
        assert_eq!(original, decompressed);
    }

    #[test]
    fn test_zstd_roundtrip() {
        let original = OwnedBuffer::from_str("Zstandard provides excellent compression ratio");
        let compressed = compress_zstd(&original).unwrap();
        let decompressed = decompress_zstd(&compressed).unwrap();
        assert_eq!(original, decompressed);
    }

    #[test]
    fn test_compression_levels() {
        let data = OwnedBuffer::from_str("A".repeat(1000).as_str());

        let fast = compress_level(&data, 1).unwrap();
        let best = compress_level(&data, 9).unwrap();

        // Best compression should be smaller or equal
        assert!(best.len() <= fast.len());
    }

    #[test]
    fn test_empty_input() {
        let empty = OwnedBuffer::new();
        let compressed = compress(&empty).unwrap();
        let decompressed = decompress(&compressed).unwrap();
        assert_eq!(empty, decompressed);
    }
}
