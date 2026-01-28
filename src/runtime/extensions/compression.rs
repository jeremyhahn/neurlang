//! Compression Extensions
//!
//! Safe compression and decompression using proven Rust crates.
//!
//! # Supported Algorithms
//! - **zlib/deflate** (flate2) - General purpose, wide compatibility
//! - **gzip** (flate2) - HTTP compression, file archives
//! - **lz4** - Very fast, lower ratio
//! - **zstd** - Best ratio, fast

use std::io::{Read, Write};
use std::sync::Arc;

use flate2::read::{GzDecoder, ZlibDecoder};
use flate2::write::{GzEncoder, ZlibEncoder};
use flate2::Compression;

use super::buffer::{HandleManager, OwnedBuffer};
use super::{ext_ids, ExtCategory, ExtError, ExtensionRegistry};

// =============================================================================
// Zlib
// =============================================================================

fn zlib_compress_impl(input: &[u8], level: u32) -> Result<Vec<u8>, ExtError> {
    let level = level.min(9);
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(level));
    encoder
        .write_all(input)
        .map_err(|e| ExtError::ExtensionError(format!("Compression error: {}", e)))?;
    encoder
        .finish()
        .map_err(|e| ExtError::ExtensionError(format!("Compression error: {}", e)))
}

fn zlib_decompress_impl(input: &[u8]) -> Result<Vec<u8>, ExtError> {
    let mut decoder = ZlibDecoder::new(input);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| ExtError::ExtensionError(format!("Decompression error: {}", e)))?;
    Ok(decompressed)
}

// =============================================================================
// Gzip
// =============================================================================

fn gzip_compress_impl(input: &[u8], level: u32) -> Result<Vec<u8>, ExtError> {
    let level = level.min(9);
    let mut encoder = GzEncoder::new(Vec::new(), Compression::new(level));
    encoder
        .write_all(input)
        .map_err(|e| ExtError::ExtensionError(format!("Compression error: {}", e)))?;
    encoder
        .finish()
        .map_err(|e| ExtError::ExtensionError(format!("Compression error: {}", e)))
}

fn gzip_decompress_impl(input: &[u8]) -> Result<Vec<u8>, ExtError> {
    let mut decoder = GzDecoder::new(input);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| ExtError::ExtensionError(format!("Decompression error: {}", e)))?;
    Ok(decompressed)
}

// =============================================================================
// LZ4
// =============================================================================

fn lz4_compress_impl(input: &[u8]) -> Result<Vec<u8>, ExtError> {
    lz4::block::compress(input, None, false)
        .map_err(|e| ExtError::ExtensionError(format!("LZ4 compression error: {}", e)))
}

fn lz4_decompress_impl(input: &[u8], max_size: usize) -> Result<Vec<u8>, ExtError> {
    lz4::block::decompress(input, Some(max_size as i32))
        .map_err(|e| ExtError::ExtensionError(format!("LZ4 decompression error: {}", e)))
}

// =============================================================================
// Zstandard
// =============================================================================

fn zstd_compress_impl(input: &[u8], level: i32) -> Result<Vec<u8>, ExtError> {
    let level = level.clamp(1, 22);
    zstd::encode_all(input, level)
        .map_err(|e| ExtError::ExtensionError(format!("Zstd compression error: {}", e)))
}

fn zstd_decompress_impl(input: &[u8]) -> Result<Vec<u8>, ExtError> {
    zstd::decode_all(input)
        .map_err(|e| ExtError::ExtensionError(format!("Zstd decompression error: {}", e)))
}

// =============================================================================
// Extension Registration
// =============================================================================

pub fn register_compression(registry: &mut ExtensionRegistry) {
    // Zlib compress
    registry.register_with_id(
        ext_ids::ZLIB_COMPRESS,
        "zlib_compress",
        "Compress data using zlib. Args: buffer_handle, level. Returns buffer_handle.",
        2,
        true,
        ExtCategory::Compression,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let level = args[1] as u32;

            let input = HandleManager::get(handle)
                .ok_or_else(|| ExtError::ExtensionError("Invalid buffer handle".to_string()))?;

            let compressed = zlib_compress_impl(input.as_slice(), level)?;
            let result_handle = HandleManager::store(OwnedBuffer::from_vec(compressed));

            outputs[0] = result_handle;
            Ok(result_handle as i64)
        }),
    );

    // Zlib decompress
    registry.register_with_id(
        ext_ids::ZLIB_DECOMPRESS,
        "zlib_decompress",
        "Decompress zlib data. Args: buffer_handle. Returns buffer_handle.",
        1,
        true,
        ExtCategory::Compression,
        Arc::new(|args, outputs| {
            let handle = args[0];

            let input = HandleManager::get(handle)
                .ok_or_else(|| ExtError::ExtensionError("Invalid buffer handle".to_string()))?;

            let decompressed = zlib_decompress_impl(input.as_slice())?;
            let result_handle = HandleManager::store(OwnedBuffer::from_vec(decompressed));

            outputs[0] = result_handle;
            Ok(result_handle as i64)
        }),
    );

    // Gzip compress
    registry.register_with_id(
        ext_ids::GZIP_COMPRESS,
        "gzip_compress",
        "Compress data using gzip. Args: buffer_handle, level. Returns buffer_handle.",
        2,
        true,
        ExtCategory::Compression,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let level = args[1] as u32;

            let input = HandleManager::get(handle)
                .ok_or_else(|| ExtError::ExtensionError("Invalid buffer handle".to_string()))?;

            let compressed = gzip_compress_impl(input.as_slice(), level)?;
            let result_handle = HandleManager::store(OwnedBuffer::from_vec(compressed));

            outputs[0] = result_handle;
            Ok(result_handle as i64)
        }),
    );

    // Gzip decompress
    registry.register_with_id(
        ext_ids::GZIP_DECOMPRESS,
        "gzip_decompress",
        "Decompress gzip data. Args: buffer_handle. Returns buffer_handle.",
        1,
        true,
        ExtCategory::Compression,
        Arc::new(|args, outputs| {
            let handle = args[0];

            let input = HandleManager::get(handle)
                .ok_or_else(|| ExtError::ExtensionError("Invalid buffer handle".to_string()))?;

            let decompressed = gzip_decompress_impl(input.as_slice())?;
            let result_handle = HandleManager::store(OwnedBuffer::from_vec(decompressed));

            outputs[0] = result_handle;
            Ok(result_handle as i64)
        }),
    );

    // LZ4 compress
    registry.register_with_id(
        ext_ids::LZ4_COMPRESS,
        "lz4_compress",
        "Compress data using LZ4. Args: buffer_handle. Returns buffer_handle.",
        1,
        true,
        ExtCategory::Compression,
        Arc::new(|args, outputs| {
            let handle = args[0];

            let input = HandleManager::get(handle)
                .ok_or_else(|| ExtError::ExtensionError("Invalid buffer handle".to_string()))?;

            let compressed = lz4_compress_impl(input.as_slice())?;
            let result_handle = HandleManager::store(OwnedBuffer::from_vec(compressed));

            outputs[0] = result_handle;
            Ok(result_handle as i64)
        }),
    );

    // LZ4 decompress
    registry.register_with_id(
        ext_ids::LZ4_DECOMPRESS,
        "lz4_decompress",
        "Decompress LZ4 data. Args: buffer_handle, max_size. Returns buffer_handle.",
        2,
        true,
        ExtCategory::Compression,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let max_size = args[1] as usize;

            let input = HandleManager::get(handle)
                .ok_or_else(|| ExtError::ExtensionError("Invalid buffer handle".to_string()))?;

            let decompressed = lz4_decompress_impl(input.as_slice(), max_size)?;
            let result_handle = HandleManager::store(OwnedBuffer::from_vec(decompressed));

            outputs[0] = result_handle;
            Ok(result_handle as i64)
        }),
    );

    // Zstd compress
    registry.register_with_id(
        ext_ids::ZSTD_COMPRESS,
        "zstd_compress",
        "Compress data using Zstandard. Args: buffer_handle, level. Returns buffer_handle.",
        2,
        true,
        ExtCategory::Compression,
        Arc::new(|args, outputs| {
            let handle = args[0];
            let level = args[1] as i32;

            let input = HandleManager::get(handle)
                .ok_or_else(|| ExtError::ExtensionError("Invalid buffer handle".to_string()))?;

            let compressed = zstd_compress_impl(input.as_slice(), level)?;
            let result_handle = HandleManager::store(OwnedBuffer::from_vec(compressed));

            outputs[0] = result_handle;
            Ok(result_handle as i64)
        }),
    );

    // Zstd decompress
    registry.register_with_id(
        ext_ids::ZSTD_DECOMPRESS,
        "zstd_decompress",
        "Decompress Zstandard data. Args: buffer_handle. Returns buffer_handle.",
        1,
        true,
        ExtCategory::Compression,
        Arc::new(|args, outputs| {
            let handle = args[0];

            let input = HandleManager::get(handle)
                .ok_or_else(|| ExtError::ExtensionError("Invalid buffer handle".to_string()))?;

            let decompressed = zstd_decompress_impl(input.as_slice())?;
            let result_handle = HandleManager::store(OwnedBuffer::from_vec(decompressed));

            outputs[0] = result_handle;
            Ok(result_handle as i64)
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zlib_roundtrip() {
        let original = b"Hello, World! ".repeat(100);
        let compressed = zlib_compress_impl(&original, 6).unwrap();
        let decompressed = zlib_decompress_impl(&compressed).unwrap();
        assert_eq!(original.as_slice(), decompressed.as_slice());
        assert!(compressed.len() < original.len());
    }

    #[test]
    fn test_gzip_roundtrip() {
        let original = b"Test data for gzip compression";
        let compressed = gzip_compress_impl(original, 6).unwrap();
        let decompressed = gzip_decompress_impl(&compressed).unwrap();
        assert_eq!(original.as_slice(), decompressed.as_slice());
    }
}
