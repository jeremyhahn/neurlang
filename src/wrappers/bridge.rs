//! Bridge Module
//!
//! Integrates the safe wrappers with the existing extension registry.
//! Uses HandleManager for safe buffer access - NO UNSAFE OPERATIONS.

use std::sync::Arc;

use super::compression;
use super::datetime;
use super::encoding;
use super::fs as fs_wrapper;
use super::regex as regex_wrapper;
use super::tls as tls_wrapper;
use super::x509 as x509_wrapper;
use super::WrapperError;
use crate::runtime::{ExtCategory, ExtError, ExtensionRegistry, HandleManager, OwnedBuffer};

// =============================================================================
// Helper Functions
// =============================================================================

/// Convert WrapperError to ExtError
fn wrapper_to_ext_error(e: WrapperError) -> ExtError {
    ExtError::ExtensionError(e.to_string())
}

/// Convert Utf8Error to ExtError (for string conversions)
fn utf8_to_ext_error(e: std::str::Utf8Error) -> ExtError {
    ExtError::ExtensionError(format!("UTF-8 encoding error: {}", e))
}

/// Get buffer from HandleManager (safe, no raw pointers)
fn get_buffer(handle: u64) -> Result<OwnedBuffer, ExtError> {
    HandleManager::get(handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid buffer handle: {}", handle)))
}

// =============================================================================
// Registration
// =============================================================================

/// Register all safe wrappers with the extension registry
pub fn register_wrappers(registry: &mut ExtensionRegistry) {
    register_compression_wrappers(registry);
    register_encoding_wrappers(registry);
    register_datetime_wrappers(registry);
    register_regex_wrappers(registry);
    register_fs_wrappers(registry);
    register_x509_wrappers(registry);
    register_tls_wrappers(registry);
}

// =============================================================================
// Compression Wrappers
// =============================================================================

fn register_compression_wrappers(registry: &mut ExtensionRegistry) {
    // zlib compress
    registry.register(
        "zlib_compress",
        "Compress data using zlib. Args: input_handle. Returns output_handle.",
        1,
        true,
        ExtCategory::Compression,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let input = get_buffer(args[0])?;
            let compressed = compression::compress(&input).map_err(wrapper_to_ext_error)?;
            outputs[0] = HandleManager::store(compressed);
            Ok(0)
        }),
    );

    // zlib decompress
    registry.register(
        "zlib_decompress",
        "Decompress zlib data. Args: input_handle. Returns output_handle.",
        1,
        true,
        ExtCategory::Compression,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let input = get_buffer(args[0])?;
            let decompressed = compression::decompress(&input).map_err(wrapper_to_ext_error)?;
            outputs[0] = HandleManager::store(decompressed);
            Ok(0)
        }),
    );

    // gzip compress
    registry.register(
        "gzip_compress",
        "Compress data using gzip. Args: input_handle. Returns output_handle.",
        1,
        true,
        ExtCategory::Compression,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let input = get_buffer(args[0])?;
            let compressed = compression::compress_gzip(&input).map_err(wrapper_to_ext_error)?;
            outputs[0] = HandleManager::store(compressed);
            Ok(0)
        }),
    );

    // gzip decompress
    registry.register(
        "gzip_decompress",
        "Decompress gzip data. Args: input_handle. Returns output_handle.",
        1,
        true,
        ExtCategory::Compression,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let input = get_buffer(args[0])?;
            let decompressed =
                compression::decompress_gzip(&input).map_err(wrapper_to_ext_error)?;
            outputs[0] = HandleManager::store(decompressed);
            Ok(0)
        }),
    );

    // lz4 compress
    registry.register(
        "lz4_compress",
        "Compress data using LZ4. Args: input_handle. Returns output_handle.",
        1,
        true,
        ExtCategory::Compression,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let input = get_buffer(args[0])?;
            let compressed = compression::compress_lz4(&input).map_err(wrapper_to_ext_error)?;
            outputs[0] = HandleManager::store(compressed);
            Ok(0)
        }),
    );

    // lz4 decompress
    registry.register(
        "lz4_decompress",
        "Decompress LZ4 data. Args: input_handle, max_size. Returns output_handle.",
        2,
        true,
        ExtCategory::Compression,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let input = get_buffer(args[0])?;
            let max_size = args[1] as usize;
            let decompressed =
                compression::decompress_lz4(&input, max_size).map_err(wrapper_to_ext_error)?;
            outputs[0] = HandleManager::store(decompressed);
            Ok(0)
        }),
    );

    // zstd compress
    registry.register(
        "zstd_compress",
        "Compress data using Zstandard. Args: input_handle. Returns output_handle.",
        1,
        true,
        ExtCategory::Compression,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let input = get_buffer(args[0])?;
            let compressed = compression::compress_zstd(&input).map_err(wrapper_to_ext_error)?;
            outputs[0] = HandleManager::store(compressed);
            Ok(0)
        }),
    );

    // zstd decompress
    registry.register(
        "zstd_decompress",
        "Decompress Zstandard data. Args: input_handle. Returns output_handle.",
        1,
        true,
        ExtCategory::Compression,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let input = get_buffer(args[0])?;
            let decompressed =
                compression::decompress_zstd(&input).map_err(wrapper_to_ext_error)?;
            outputs[0] = HandleManager::store(decompressed);
            Ok(0)
        }),
    );
}

// =============================================================================
// Encoding Wrappers
// =============================================================================

fn register_encoding_wrappers(registry: &mut ExtensionRegistry) {
    // base64 encode
    registry.register(
        "base64_encode",
        "Encode data as base64. Args: input_handle. Returns output_handle.",
        1,
        true,
        ExtCategory::Regex,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let input = get_buffer(args[0])?;
            let encoded = encoding::base64_encode(&input);
            outputs[0] = HandleManager::store(encoded);
            Ok(0)
        }),
    );

    // base64 decode
    registry.register(
        "base64_decode",
        "Decode base64 data. Args: input_handle. Returns output_handle.",
        1,
        true,
        ExtCategory::Regex,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let input = get_buffer(args[0])?;
            let decoded = encoding::base64_decode(&input).map_err(wrapper_to_ext_error)?;
            outputs[0] = HandleManager::store(decoded);
            Ok(0)
        }),
    );

    // hex encode
    registry.register(
        "hex_encode",
        "Encode data as hexadecimal. Args: input_handle. Returns output_handle.",
        1,
        true,
        ExtCategory::Regex,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let input = get_buffer(args[0])?;
            let encoded = encoding::hex_encode(&input);
            outputs[0] = HandleManager::store(encoded);
            Ok(0)
        }),
    );

    // hex decode
    registry.register(
        "hex_decode",
        "Decode hexadecimal data. Args: input_handle. Returns output_handle.",
        1,
        true,
        ExtCategory::Regex,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let input = get_buffer(args[0])?;
            let decoded = encoding::hex_decode(&input).map_err(wrapper_to_ext_error)?;
            outputs[0] = HandleManager::store(decoded);
            Ok(0)
        }),
    );

    // url encode
    registry.register(
        "url_encode",
        "URL-encode a string. Args: input_handle. Returns output_handle.",
        1,
        true,
        ExtCategory::Regex,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let input = get_buffer(args[0])?;
            let encoded = encoding::url_encode(&input);
            outputs[0] = HandleManager::store(encoded);
            Ok(0)
        }),
    );

    // url decode
    registry.register(
        "url_decode",
        "URL-decode a string. Args: input_handle. Returns output_handle.",
        1,
        true,
        ExtCategory::Regex,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let input = get_buffer(args[0])?;
            let decoded = encoding::url_decode(&input).map_err(wrapper_to_ext_error)?;
            outputs[0] = HandleManager::store(decoded);
            Ok(0)
        }),
    );
}

// =============================================================================
// DateTime Wrappers
// =============================================================================

fn register_datetime_wrappers(registry: &mut ExtensionRegistry) {
    // Get current UTC timestamp (milliseconds)
    registry.register(
        "datetime_now",
        "Get current UTC timestamp in milliseconds. No args. Returns timestamp.",
        0,
        true,
        ExtCategory::X509,
        Arc::new(|_args, outputs| {
            let ts = datetime::now();
            outputs[0] = ts as u64;
            Ok(ts)
        }),
    );

    // Get current UTC timestamp (seconds)
    registry.register(
        "datetime_now_secs",
        "Get current UTC timestamp in seconds. No args. Returns timestamp.",
        0,
        true,
        ExtCategory::X509,
        Arc::new(|_args, outputs| {
            let ts = datetime::now_secs();
            outputs[0] = ts as u64;
            Ok(ts)
        }),
    );

    // Format timestamp as ISO 8601
    registry.register(
        "datetime_format_iso",
        "Format timestamp as ISO 8601 string. Args: timestamp_ms. Returns output_handle.",
        1,
        true,
        ExtCategory::X509,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let timestamp_ms = args[0] as i64;
            let formatted = datetime::format_iso(timestamp_ms);
            outputs[0] = HandleManager::store(formatted);
            Ok(0)
        }),
    );

    // Get year from timestamp
    registry.register(
        "datetime_year",
        "Get year from timestamp. Args: timestamp_ms. Returns year.",
        1,
        true,
        ExtCategory::X509,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let timestamp_ms = args[0] as i64;
            let year = datetime::year(timestamp_ms);
            outputs[0] = year as u64;
            Ok(year as i64)
        }),
    );

    // Get month from timestamp
    registry.register(
        "datetime_month",
        "Get month (1-12) from timestamp. Args: timestamp_ms. Returns month.",
        1,
        true,
        ExtCategory::X509,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let timestamp_ms = args[0] as i64;
            let month = datetime::month(timestamp_ms);
            outputs[0] = month as u64;
            Ok(month as i64)
        }),
    );

    // Get day from timestamp
    registry.register(
        "datetime_day",
        "Get day of month from timestamp. Args: timestamp_ms. Returns day.",
        1,
        true,
        ExtCategory::X509,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let timestamp_ms = args[0] as i64;
            let day = datetime::day(timestamp_ms);
            outputs[0] = day as u64;
            Ok(day as i64)
        }),
    );

    // Add days to timestamp
    registry.register(
        "datetime_add_days",
        "Add days to timestamp. Args: timestamp_ms, days. Returns new timestamp.",
        2,
        true,
        ExtCategory::X509,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let timestamp_ms = args[0] as i64;
            let days = args[1] as i64;
            let result = datetime::add_days(timestamp_ms, days);
            outputs[0] = result as u64;
            Ok(result)
        }),
    );
}

// =============================================================================
// Regex Wrappers
// =============================================================================

fn register_regex_wrappers(registry: &mut ExtensionRegistry) {
    // Regex match
    registry.register(
        "regex_match",
        "Check if pattern matches input. Args: pattern_handle, input_handle. Returns 1 if match, 0 if not.",
        2,
        true,
        ExtCategory::Regex,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let pattern = get_buffer(args[0])?;
            let input = get_buffer(args[1])?;
            let matched = regex_wrapper::is_match(&pattern, &input).map_err(wrapper_to_ext_error)?;
            let result = if matched { 1i64 } else { 0i64 };
            outputs[0] = result as u64;
            Ok(result)
        }),
    );

    // Regex find
    registry.register(
        "regex_find",
        "Find first match. Args: pattern_handle, input_handle. Returns output_handle (empty if no match).",
        2,
        true,
        ExtCategory::Regex,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let pattern = get_buffer(args[0])?;
            let input = get_buffer(args[1])?;
            match regex_wrapper::find_text(&pattern, &input).map_err(wrapper_to_ext_error)? {
                Some(found) => {
                    outputs[0] = HandleManager::store(found);
                    Ok(1)
                }
                None => {
                    outputs[0] = HandleManager::store(OwnedBuffer::new());
                    Ok(0)
                }
            }
        }),
    );

    // Regex replace
    registry.register(
        "regex_replace",
        "Replace first match. Args: pattern_handle, input_handle, replacement_handle. Returns output_handle.",
        3,
        true,
        ExtCategory::Regex,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let pattern = get_buffer(args[0])?;
            let input = get_buffer(args[1])?;
            let replacement = get_buffer(args[2])?;
            let result = regex_wrapper::replace(&pattern, &input, &replacement).map_err(wrapper_to_ext_error)?;
            outputs[0] = HandleManager::store(result);
            Ok(0)
        }),
    );

    // Regex replace all
    registry.register(
        "regex_replace_all",
        "Replace all matches. Args: pattern_handle, input_handle, replacement_handle. Returns output_handle.",
        3,
        true,
        ExtCategory::Regex,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let pattern = get_buffer(args[0])?;
            let input = get_buffer(args[1])?;
            let replacement = get_buffer(args[2])?;
            let result = regex_wrapper::replace_all(&pattern, &input, &replacement).map_err(wrapper_to_ext_error)?;
            outputs[0] = HandleManager::store(result);
            Ok(0)
        }),
    );
}

// =============================================================================
// File System Wrappers
// =============================================================================

fn register_fs_wrappers(registry: &mut ExtensionRegistry) {
    // read_file
    registry.register(
        "fs_read_file",
        "Read entire file contents. Args: path_handle. Returns content_handle.",
        1,
        true,
        ExtCategory::FileSystem,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let path = get_buffer(args[0])?;
            let contents = fs_wrapper::read_file(&path).map_err(wrapper_to_ext_error)?;
            outputs[0] = HandleManager::store(contents);
            Ok(0)
        }),
    );

    // write_file
    registry.register(
        "fs_write_file",
        "Write data to file. Args: path_handle, data_handle. Returns 0 on success.",
        2,
        true,
        ExtCategory::FileSystem,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let path = get_buffer(args[0])?;
            let data = get_buffer(args[1])?;
            fs_wrapper::write_file(&path, &data).map_err(wrapper_to_ext_error)?;
            outputs[0] = 0;
            Ok(0)
        }),
    );

    // append_file
    registry.register(
        "fs_append_file",
        "Append data to file. Args: path_handle, data_handle. Returns 0 on success.",
        2,
        true,
        ExtCategory::FileSystem,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let path = get_buffer(args[0])?;
            let data = get_buffer(args[1])?;
            fs_wrapper::append_file(&path, &data).map_err(wrapper_to_ext_error)?;
            outputs[0] = 0;
            Ok(0)
        }),
    );

    // file_exists
    registry.register(
        "fs_exists",
        "Check if path exists. Args: path_handle. Returns 1 if exists, 0 if not.",
        1,
        true,
        ExtCategory::FileSystem,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let path = get_buffer(args[0])?;
            let exists = fs_wrapper::exists(&path);
            let result = if exists { 1i64 } else { 0i64 };
            outputs[0] = result as u64;
            Ok(result)
        }),
    );

    // is_file
    registry.register(
        "fs_is_file",
        "Check if path is a file. Args: path_handle. Returns 1 if file, 0 if not.",
        1,
        true,
        ExtCategory::FileSystem,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let path = get_buffer(args[0])?;
            let is_file = fs_wrapper::is_file(&path);
            let result = if is_file { 1i64 } else { 0i64 };
            outputs[0] = result as u64;
            Ok(result)
        }),
    );

    // is_dir
    registry.register(
        "fs_is_dir",
        "Check if path is a directory. Args: path_handle. Returns 1 if directory, 0 if not.",
        1,
        true,
        ExtCategory::FileSystem,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let path = get_buffer(args[0])?;
            let is_dir = fs_wrapper::is_dir(&path);
            let result = if is_dir { 1i64 } else { 0i64 };
            outputs[0] = result as u64;
            Ok(result)
        }),
    );

    // file_size
    registry.register(
        "fs_file_size",
        "Get file size in bytes. Args: path_handle. Returns size in bytes.",
        1,
        true,
        ExtCategory::FileSystem,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let path = get_buffer(args[0])?;
            let size = fs_wrapper::file_size(&path).map_err(wrapper_to_ext_error)?;
            outputs[0] = size;
            Ok(size as i64)
        }),
    );

    // list_dir
    registry.register(
        "fs_list_dir",
        "List directory contents (newline-separated). Args: path_handle. Returns output_handle.",
        1,
        true,
        ExtCategory::FileSystem,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let path = get_buffer(args[0])?;
            let entries = fs_wrapper::list_dir(&path).map_err(wrapper_to_ext_error)?;
            let result: String = entries
                .iter()
                .filter_map(|e| e.as_str().ok())
                .collect::<Vec<_>>()
                .join("\n");
            let result_buf = OwnedBuffer::from_string(result);
            outputs[0] = HandleManager::store(result_buf);
            Ok(0)
        }),
    );

    // create_dir
    registry.register(
        "fs_create_dir",
        "Create directory (and parents). Args: path_handle. Returns 0 on success.",
        1,
        true,
        ExtCategory::FileSystem,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let path = get_buffer(args[0])?;
            fs_wrapper::create_dir(&path).map_err(wrapper_to_ext_error)?;
            outputs[0] = 0;
            Ok(0)
        }),
    );

    // remove_file
    registry.register(
        "fs_remove_file",
        "Remove file. Args: path_handle. Returns 0 on success.",
        1,
        true,
        ExtCategory::FileSystem,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let path = get_buffer(args[0])?;
            fs_wrapper::remove_file(&path).map_err(wrapper_to_ext_error)?;
            outputs[0] = 0;
            Ok(0)
        }),
    );

    // remove_dir
    registry.register(
        "fs_remove_dir",
        "Remove empty directory. Args: path_handle. Returns 0 on success.",
        1,
        true,
        ExtCategory::FileSystem,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let path = get_buffer(args[0])?;
            fs_wrapper::remove_dir(&path).map_err(wrapper_to_ext_error)?;
            outputs[0] = 0;
            Ok(0)
        }),
    );

    // copy_file
    registry.register(
        "fs_copy_file",
        "Copy file. Args: src_handle, dst_handle. Returns bytes copied.",
        2,
        true,
        ExtCategory::FileSystem,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let src = get_buffer(args[0])?;
            let dst = get_buffer(args[1])?;
            let bytes = fs_wrapper::copy_file(&src, &dst).map_err(wrapper_to_ext_error)?;
            outputs[0] = bytes;
            Ok(bytes as i64)
        }),
    );

    // move_file
    registry.register(
        "fs_move_file",
        "Move/rename file. Args: src_handle, dst_handle. Returns 0 on success.",
        2,
        true,
        ExtCategory::FileSystem,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let src = get_buffer(args[0])?;
            let dst = get_buffer(args[1])?;
            fs_wrapper::move_file(&src, &dst).map_err(wrapper_to_ext_error)?;
            outputs[0] = 0;
            Ok(0)
        }),
    );
}

// =============================================================================
// X509 Certificate Wrappers
// =============================================================================

fn register_x509_wrappers(registry: &mut ExtensionRegistry) {
    // keypair_generate_ec
    registry.register(
        "x509_keypair_ec",
        "Generate an EC key pair. Args: curve_handle. Returns keypair_handle.",
        1,
        true,
        ExtCategory::Crypto,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let curve_buf = get_buffer(args[0])?;
            let curve = curve_buf.as_str().map_err(utf8_to_ext_error)?;
            let keypair = x509_wrapper::keypair_generate_ec(curve).map_err(wrapper_to_ext_error)?;
            outputs[0] = HandleManager::store(keypair);
            Ok(0)
        }),
    );

    // keypair_generate_ed25519
    registry.register(
        "x509_keypair_ed25519",
        "Generate an Ed25519 key pair. No args. Returns keypair_handle.",
        0,
        true,
        ExtCategory::Crypto,
        Arc::new(|_args: &[u64], outputs: &mut [u64]| {
            let keypair = x509_wrapper::keypair_generate_ed25519().map_err(wrapper_to_ext_error)?;
            outputs[0] = HandleManager::store(keypair);
            Ok(0)
        }),
    );

    // create_self_signed
    registry.register(
        "x509_create_self_signed",
        "Create self-signed certificate. Args: subject_handle, keypair_handle, days. Returns cert_handle.",
        3,
        true,
        ExtCategory::Crypto,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let subject_buf = get_buffer(args[0])?;
            let subject = subject_buf.as_str().map_err(utf8_to_ext_error)?;
            let keypair = get_buffer(args[1])?;
            let days = args[2] as u32;
            let cert = x509_wrapper::create_self_signed(subject, &keypair, days).map_err(wrapper_to_ext_error)?;
            outputs[0] = HandleManager::store(cert);
            Ok(0)
        }),
    );

    // create_ca
    registry.register(
        "x509_create_ca",
        "Create CA certificate. Args: subject_handle, keypair_handle, days. Returns cert_handle.",
        3,
        true,
        ExtCategory::Crypto,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let subject_buf = get_buffer(args[0])?;
            let subject = subject_buf.as_str().map_err(utf8_to_ext_error)?;
            let keypair = get_buffer(args[1])?;
            let days = args[2] as u32;
            let cert =
                x509_wrapper::create_ca(subject, &keypair, days).map_err(wrapper_to_ext_error)?;
            outputs[0] = HandleManager::store(cert);
            Ok(0)
        }),
    );

    // create_csr
    registry.register(
        "x509_create_csr",
        "Create Certificate Signing Request. Args: subject_handle, keypair_handle. Returns csr_handle.",
        2,
        true,
        ExtCategory::Crypto,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let subject_buf = get_buffer(args[0])?;
            let subject = subject_buf.as_str().map_err(utf8_to_ext_error)?;
            let keypair = get_buffer(args[1])?;
            let csr = x509_wrapper::create_csr(subject, &keypair).map_err(wrapper_to_ext_error)?;
            outputs[0] = HandleManager::store(csr);
            Ok(0)
        }),
    );

    // get_subject
    registry.register(
        "x509_get_subject",
        "Get certificate subject. Args: cert_handle. Returns subject_handle.",
        1,
        true,
        ExtCategory::Crypto,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let cert = get_buffer(args[0])?;
            let subject = x509_wrapper::get_subject(&cert).map_err(wrapper_to_ext_error)?;
            outputs[0] = HandleManager::store(subject);
            Ok(0)
        }),
    );

    // get_issuer
    registry.register(
        "x509_get_issuer",
        "Get certificate issuer. Args: cert_handle. Returns issuer_handle.",
        1,
        true,
        ExtCategory::Crypto,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let cert = get_buffer(args[0])?;
            let issuer = x509_wrapper::get_issuer(&cert).map_err(wrapper_to_ext_error)?;
            outputs[0] = HandleManager::store(issuer);
            Ok(0)
        }),
    );

    // get_expiry
    registry.register(
        "x509_get_expiry",
        "Get certificate expiry. Args: cert_handle. Returns expiry_handle.",
        1,
        true,
        ExtCategory::Crypto,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let cert = get_buffer(args[0])?;
            let expiry = x509_wrapper::get_expiry(&cert).map_err(wrapper_to_ext_error)?;
            outputs[0] = HandleManager::store(expiry);
            Ok(0)
        }),
    );

    // is_expired
    registry.register(
        "x509_is_expired",
        "Check if certificate is expired. Args: cert_handle. Returns 1 if expired, 0 if not.",
        1,
        true,
        ExtCategory::Crypto,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let cert = get_buffer(args[0])?;
            let expired = x509_wrapper::is_expired(&cert).map_err(wrapper_to_ext_error)?;
            let result = if expired { 1i64 } else { 0i64 };
            outputs[0] = result as u64;
            Ok(result)
        }),
    );

    // is_ca
    registry.register(
        "x509_is_ca",
        "Check if certificate is a CA. Args: cert_handle. Returns 1 if CA, 0 if not.",
        1,
        true,
        ExtCategory::Crypto,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let cert = get_buffer(args[0])?;
            let is_ca = x509_wrapper::is_ca(&cert).map_err(wrapper_to_ext_error)?;
            let result = if is_ca { 1i64 } else { 0i64 };
            outputs[0] = result as u64;
            Ok(result)
        }),
    );
}

// =============================================================================
// TLS Wrappers
// =============================================================================

fn register_tls_wrappers(registry: &mut ExtensionRegistry) {
    // https_get (simple HTTPS GET request)
    registry.register(
        "tls_https_get",
        "Perform HTTPS GET request. Args: url_handle. Returns response_handle.",
        1,
        true,
        ExtCategory::Tls,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let url_buf = get_buffer(args[0])?;
            let url = url_buf.as_str().map_err(utf8_to_ext_error)?;
            let response = tls_wrapper::https_get(url).map_err(wrapper_to_ext_error)?;
            outputs[0] = HandleManager::store(response);
            Ok(0)
        }),
    );

    // client_config (create TLS client config)
    registry.register(
        "tls_client_config",
        "Create TLS client config with Mozilla CA roots. No args. Returns 1 on success.",
        0,
        true,
        ExtCategory::Tls,
        Arc::new(|_args, outputs| {
            let _config = tls_wrapper::client_config().map_err(wrapper_to_ext_error)?;
            outputs[0] = 1;
            Ok(1)
        }),
    );
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to store data and get a handle
    fn store_buffer(data: &[u8]) -> u64 {
        HandleManager::store(OwnedBuffer::from_slice(data))
    }

    /// Helper to get data from a handle
    fn get_buffer_data(handle: u64) -> Vec<u8> {
        HandleManager::get(handle)
            .map(|b| b.as_slice().to_vec())
            .unwrap_or_default()
    }

    #[test]
    fn test_register_wrappers() {
        let mut registry = ExtensionRegistry::new();
        let initial_count = registry.list().len();

        register_wrappers(&mut registry);

        // Should have added new extensions
        assert!(registry.list().len() > initial_count);

        // Check compression extensions exist
        assert!(registry.get_by_name("zlib_compress").is_some());
        assert!(registry.get_by_name("zlib_decompress").is_some());

        // Check encoding extensions exist
        assert!(registry.get_by_name("base64_encode").is_some());
        assert!(registry.get_by_name("base64_decode").is_some());

        // Check datetime extensions exist
        assert!(registry.get_by_name("datetime_now").is_some());
        assert!(registry.get_by_name("datetime_year").is_some());

        // Check regex extensions exist
        assert!(registry.get_by_name("regex_match").is_some());
        assert!(registry.get_by_name("regex_replace").is_some());

        // Check filesystem extensions exist
        assert!(registry.get_by_name("fs_read_file").is_some());
        assert!(registry.get_by_name("fs_write_file").is_some());
        assert!(registry.get_by_name("fs_exists").is_some());
        assert!(registry.get_by_name("fs_list_dir").is_some());

        // Check X509 extensions exist
        assert!(registry.get_by_name("x509_keypair_ec").is_some());
        assert!(registry.get_by_name("x509_create_self_signed").is_some());
        assert!(registry.get_by_name("x509_get_subject").is_some());
        assert!(registry.get_by_name("x509_is_expired").is_some());

        // Check TLS extensions exist
        assert!(registry.get_by_name("tls_https_get").is_some());
        assert!(registry.get_by_name("tls_client_config").is_some());
    }

    #[test]
    fn test_compression_roundtrip() {
        let mut registry = ExtensionRegistry::new();
        register_wrappers(&mut registry);

        let input_data = b"Hello, World! This is a test for compression.";
        let input = store_buffer(input_data);
        let mut outputs = [0u64; 4];

        // Compress
        let compress_id = registry.get_id("zlib_compress").unwrap();
        let result = registry.call(compress_id, &[input], &mut outputs).unwrap();
        assert_eq!(result, 0);
        let compressed = outputs[0];

        // Decompress
        let decompress_id = registry.get_id("zlib_decompress").unwrap();
        let result = registry
            .call(decompress_id, &[compressed], &mut outputs)
            .unwrap();
        assert_eq!(result, 0);
        let decompressed_data = get_buffer_data(outputs[0]);

        // Verify
        assert_eq!(decompressed_data.as_slice(), input_data);

        // Cleanup
        HandleManager::remove(input);
        HandleManager::remove(compressed);
        HandleManager::remove(outputs[0]);
    }

    #[test]
    fn test_encoding_roundtrip() {
        let mut registry = ExtensionRegistry::new();
        register_wrappers(&mut registry);

        let input_data = b"Hello, Base64!";
        let input = store_buffer(input_data);
        let mut outputs = [0u64; 4];

        // Encode
        let encode_id = registry.get_id("base64_encode").unwrap();
        registry.call(encode_id, &[input], &mut outputs).unwrap();
        let encoded = outputs[0];

        // Decode
        let decode_id = registry.get_id("base64_decode").unwrap();
        registry.call(decode_id, &[encoded], &mut outputs).unwrap();
        let decoded_data = get_buffer_data(outputs[0]);

        // Verify
        assert_eq!(decoded_data.as_slice(), input_data);

        // Cleanup
        HandleManager::remove(input);
        HandleManager::remove(encoded);
        HandleManager::remove(outputs[0]);
    }

    #[test]
    fn test_datetime() {
        let mut registry = ExtensionRegistry::new();
        register_wrappers(&mut registry);

        let mut outputs = [0u64; 4];

        // Get current timestamp
        let now_id = registry.get_id("datetime_now").unwrap();
        let ts = registry.call(now_id, &[], &mut outputs).unwrap();

        // Should be a reasonable timestamp (after 2020)
        assert!(ts > 1577836800000); // Jan 1, 2020 in ms

        // Get year
        let year_id = registry.get_id("datetime_year").unwrap();
        let year = registry.call(year_id, &[ts as u64], &mut outputs).unwrap();
        assert!(year >= 2024);
    }

    #[test]
    fn test_regex() {
        let mut registry = ExtensionRegistry::new();
        register_wrappers(&mut registry);

        let pattern = store_buffer(br"\d+");
        let input = store_buffer(b"The answer is 42");
        let mut outputs = [0u64; 4];

        // Match
        let match_id = registry.get_id("regex_match").unwrap();
        let matched = registry
            .call(match_id, &[pattern, input], &mut outputs)
            .unwrap();
        assert_eq!(matched, 1);

        // Find
        let pattern2 = store_buffer(br"\d+");
        let input2 = store_buffer(b"The answer is 42");
        let find_id = registry.get_id("regex_find").unwrap();
        let found = registry
            .call(find_id, &[pattern2, input2], &mut outputs)
            .unwrap();
        assert_eq!(found, 1);
        let found_data = get_buffer_data(outputs[0]);
        assert_eq!(found_data.as_slice(), b"42");

        // Cleanup
        HandleManager::remove(pattern);
        HandleManager::remove(input);
        HandleManager::remove(pattern2);
        HandleManager::remove(input2);
        HandleManager::remove(outputs[0]);
    }

    #[test]
    fn test_filesystem() {
        let mut registry = ExtensionRegistry::new();
        register_wrappers(&mut registry);

        let mut outputs = [0u64; 4];

        // Test fs_exists on /tmp (should exist)
        let tmp_path = store_buffer(b"/tmp");
        let exists_id = registry.get_id("fs_exists").unwrap();
        let exists = registry.call(exists_id, &[tmp_path], &mut outputs).unwrap();
        assert_eq!(exists, 1);

        // Test fs_is_dir on /tmp (should be a directory)
        let tmp_path2 = store_buffer(b"/tmp");
        let is_dir_id = registry.get_id("fs_is_dir").unwrap();
        let is_dir = registry
            .call(is_dir_id, &[tmp_path2], &mut outputs)
            .unwrap();
        assert_eq!(is_dir, 1);

        // Test fs_is_file on /tmp (should not be a file)
        let tmp_path3 = store_buffer(b"/tmp");
        let is_file_id = registry.get_id("fs_is_file").unwrap();
        let is_file = registry
            .call(is_file_id, &[tmp_path3], &mut outputs)
            .unwrap();
        assert_eq!(is_file, 0);

        // Test write and read file
        let test_path = store_buffer(b"/tmp/neurlang_bridge_test.txt");
        let test_data = store_buffer(b"Hello from bridge test!");

        // Write
        let write_id = registry.get_id("fs_write_file").unwrap();
        let result = registry
            .call(write_id, &[test_path, test_data], &mut outputs)
            .unwrap();
        assert_eq!(result, 0);

        // Read
        let test_path2 = store_buffer(b"/tmp/neurlang_bridge_test.txt");
        let read_id = registry.get_id("fs_read_file").unwrap();
        registry.call(read_id, &[test_path2], &mut outputs).unwrap();
        let read_data = get_buffer_data(outputs[0]);
        assert_eq!(read_data.as_slice(), b"Hello from bridge test!");

        // Cleanup
        let test_path3 = store_buffer(b"/tmp/neurlang_bridge_test.txt");
        let remove_id = registry.get_id("fs_remove_file").unwrap();
        let _ = registry.call(remove_id, &[test_path3], &mut outputs);

        HandleManager::remove(tmp_path);
        HandleManager::remove(tmp_path2);
        HandleManager::remove(tmp_path3);
        HandleManager::remove(test_path);
        HandleManager::remove(test_path2);
        HandleManager::remove(test_path3);
        HandleManager::remove(test_data);
        HandleManager::remove(outputs[0]);
    }

    #[test]
    fn test_x509_keypair() {
        let mut registry = ExtensionRegistry::new();
        register_wrappers(&mut registry);

        let mut outputs = [0u64; 4];

        // Generate EC keypair
        let curve = store_buffer(b"P-256");
        let keypair_id = registry.get_id("x509_keypair_ec").unwrap();
        registry.call(keypair_id, &[curve], &mut outputs).unwrap();
        let keypair_data = get_buffer_data(outputs[0]);
        assert!(!keypair_data.is_empty());

        // Verify it's PEM formatted
        let keypair_str = std::str::from_utf8(&keypair_data).unwrap();
        assert!(keypair_str.contains("BEGIN PRIVATE KEY"));

        HandleManager::remove(curve);
        HandleManager::remove(outputs[0]);
    }

    #[test]
    fn test_x509_certificate() {
        let mut registry = ExtensionRegistry::new();
        register_wrappers(&mut registry);

        let mut outputs = [0u64; 4];

        // Generate keypair first
        let curve = store_buffer(b"P-256");
        let keypair_id = registry.get_id("x509_keypair_ec").unwrap();
        registry.call(keypair_id, &[curve], &mut outputs).unwrap();
        let keypair = outputs[0];

        // Create self-signed certificate
        let subject = store_buffer(b"CN=test.example.com");
        let days: u64 = 365;
        let cert_id = registry.get_id("x509_create_self_signed").unwrap();
        registry
            .call(cert_id, &[subject, keypair, days], &mut outputs)
            .unwrap();
        let cert = outputs[0];
        let cert_data = get_buffer_data(cert);

        // Verify it's PEM formatted
        let cert_str = std::str::from_utf8(&cert_data).unwrap();
        assert!(cert_str.contains("BEGIN CERTIFICATE"));

        // Get subject from certificate
        let get_subject_id = registry.get_id("x509_get_subject").unwrap();
        registry
            .call(get_subject_id, &[cert], &mut outputs)
            .unwrap();
        let subject_data = get_buffer_data(outputs[0]);
        let subject_str = std::str::from_utf8(&subject_data).unwrap();
        assert!(subject_str.contains("test.example.com"));

        // Check not expired
        let cert2 = store_buffer(&cert_data);
        let is_expired_id = registry.get_id("x509_is_expired").unwrap();
        let expired = registry
            .call(is_expired_id, &[cert2], &mut outputs)
            .unwrap();
        assert_eq!(expired, 0); // Not expired

        // Check not CA
        let cert3 = store_buffer(&cert_data);
        let is_ca_id = registry.get_id("x509_is_ca").unwrap();
        let is_ca = registry.call(is_ca_id, &[cert3], &mut outputs).unwrap();
        assert_eq!(is_ca, 0); // Not a CA

        // Cleanup
        HandleManager::remove(curve);
        HandleManager::remove(keypair);
        HandleManager::remove(subject);
        HandleManager::remove(cert);
        HandleManager::remove(cert2);
        HandleManager::remove(cert3);
    }

    #[test]
    fn test_tls_config() {
        let mut registry = ExtensionRegistry::new();
        register_wrappers(&mut registry);

        let mut outputs = [0u64; 4];

        // Create TLS client config
        let config_id = registry.get_id("tls_client_config").unwrap();
        let result = registry.call(config_id, &[], &mut outputs).unwrap();
        assert_eq!(result, 1); // Success
    }
}
