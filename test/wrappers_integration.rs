//! Integration Tests for Wrappers Module
//!
//! Tests the complete wrappers system including:
//! - OwnedBuffer operations
//! - WrapperRegistry RAG search
//! - Compression roundtrips
//! - Encoding roundtrips
//! - Extension registry integration

use neurlang::wrappers::{OwnedBuffer, WrapperRegistry, WrapperCategory};
use neurlang::runtime::ExtensionRegistry;

// =============================================================================
// OwnedBuffer Tests
// =============================================================================

#[test]
fn test_owned_buffer_full_lifecycle() {
    // Create empty buffer
    let mut buf = OwnedBuffer::new();
    assert!(buf.is_empty());

    // Extend with data
    buf.extend(b"Hello, ");
    buf.extend(b"World!");
    assert_eq!(buf.len(), 13);
    assert_eq!(buf.as_str().unwrap(), "Hello, World!");

    // Split at position
    let (left, right) = buf.split_at(7);
    assert_eq!(left.as_str().unwrap(), "Hello, ");
    assert_eq!(right.as_str().unwrap(), "World!");
}

#[test]
fn test_owned_buffer_binary_data() {
    // Binary data with null bytes
    let binary = vec![0u8, 1, 2, 255, 0, 127];
    let buf = OwnedBuffer::from_vec(binary.clone());

    assert_eq!(buf.len(), 6);
    assert_eq!(buf.as_slice(), &binary[..]);

    // Should fail to convert to string
    assert!(buf.as_str().is_err());
}

// =============================================================================
// WrapperRegistry Tests
// =============================================================================

#[test]
fn test_wrapper_registry_registration_and_lookup() {
    let mut registry = WrapperRegistry::new();

    // Register a test wrapper
    let id = registry.register_wrapper(
        "test_wrapper",
        "A test wrapper for integration testing",
        WrapperCategory::Compression,
        1,
        &["test", "example", "demo"],
        |args| {
            if args.is_empty() {
                return Err(neurlang::wrappers::WrapperError::InvalidArg("No input".to_string()));
            }
            // Just return the input unchanged
            Ok(args[0].clone())
        },
    );

    // Should be able to look up by name
    assert_eq!(registry.get_id("test_wrapper"), Some(id));

    // Should be able to get info
    let info = registry.get(id).unwrap();
    assert_eq!(info.name, "test_wrapper");
    assert_eq!(info.category, WrapperCategory::Compression);
}

#[test]
fn test_wrapper_registry_rag_search() {
    let mut registry = WrapperRegistry::new();

    // Register compression wrappers
    neurlang::wrappers::compression::register(&mut registry);

    // Search by keyword
    let result = registry.search("compress");
    assert!(result.is_some());

    // Search by synonym
    let result = registry.search("shrink");
    assert!(result.is_some());

    // Search by related term
    let result = registry.search("zip");
    assert!(result.is_some());
}

#[test]
fn test_wrapper_registry_category_listing() {
    let mut registry = WrapperRegistry::new();

    // Register various wrappers
    neurlang::wrappers::compression::register(&mut registry);
    neurlang::wrappers::encoding::register(&mut registry);

    // List by category
    let compression_wrappers = registry.list_by_category(WrapperCategory::Compression);
    assert!(!compression_wrappers.is_empty());

    let encoding_wrappers = registry.list_by_category(WrapperCategory::Encoding);
    assert!(!encoding_wrappers.is_empty());
}

// =============================================================================
// Compression Integration Tests
// =============================================================================

#[test]
fn test_compression_all_algorithms() {
    use neurlang::wrappers::compression;

    let test_data = OwnedBuffer::from_str(
        "The quick brown fox jumps over the lazy dog. ".repeat(100).as_str()
    );

    // zlib
    {
        let compressed = compression::compress(&test_data).unwrap();
        let decompressed = compression::decompress(&compressed).unwrap();
        assert_eq!(test_data.as_slice(), decompressed.as_slice());
        assert!(compressed.len() < test_data.len());
    }

    // gzip
    {
        let compressed = compression::compress_gzip(&test_data).unwrap();
        let decompressed = compression::decompress_gzip(&compressed).unwrap();
        assert_eq!(test_data.as_slice(), decompressed.as_slice());
        assert!(compressed.len() < test_data.len());
    }

    // lz4
    {
        let compressed = compression::compress_lz4(&test_data).unwrap();
        let decompressed = compression::decompress_lz4(&compressed, test_data.len() * 2).unwrap();
        assert_eq!(test_data.as_slice(), decompressed.as_slice());
    }

    // zstd
    {
        let compressed = compression::compress_zstd(&test_data).unwrap();
        let decompressed = compression::decompress_zstd(&compressed).unwrap();
        assert_eq!(test_data.as_slice(), decompressed.as_slice());
        assert!(compressed.len() < test_data.len());
    }
}

#[test]
fn test_compression_empty_input() {
    use neurlang::wrappers::compression;

    let empty = OwnedBuffer::new();

    // All algorithms should handle empty input
    let _ = compression::compress(&empty).unwrap();
    let _ = compression::compress_gzip(&empty).unwrap();
    let _ = compression::compress_lz4(&empty).unwrap();
    let _ = compression::compress_zstd(&empty).unwrap();
}

// =============================================================================
// Encoding Integration Tests
// =============================================================================

#[test]
fn test_encoding_all_formats() {
    use neurlang::wrappers::encoding;

    let test_data = OwnedBuffer::from_str("Hello, World! 123 äöü");

    // Base64
    {
        let encoded = encoding::base64_encode(&test_data);
        let decoded = encoding::base64_decode(&encoded).unwrap();
        assert_eq!(test_data.as_slice(), decoded.as_slice());
    }

    // Hex
    {
        let encoded = encoding::hex_encode(&test_data);
        let decoded = encoding::hex_decode(&encoded).unwrap();
        assert_eq!(test_data.as_slice(), decoded.as_slice());
    }

    // URL encoding
    {
        let url_data = OwnedBuffer::from_str("hello world?foo=bar&baz=qux");
        let encoded = encoding::url_encode(&url_data);
        let decoded = encoding::url_decode(&encoded).unwrap();
        assert_eq!(url_data.as_slice(), decoded.as_slice());
    }
}

#[test]
fn test_encoding_binary_data() {
    use neurlang::wrappers::encoding;

    // Binary data with all byte values
    let mut binary = Vec::new();
    for i in 0..=255u8 {
        binary.push(i);
    }
    let test_data = OwnedBuffer::from_vec(binary);

    // Base64 handles binary
    {
        let encoded = encoding::base64_encode(&test_data);
        let decoded = encoding::base64_decode(&encoded).unwrap();
        assert_eq!(test_data.as_slice(), decoded.as_slice());
    }

    // Hex handles binary
    {
        let encoded = encoding::hex_encode(&test_data);
        let decoded = encoding::hex_decode(&encoded).unwrap();
        assert_eq!(test_data.as_slice(), decoded.as_slice());
    }
}

// =============================================================================
// DateTime Integration Tests
// =============================================================================

#[test]
fn test_datetime_operations() {
    use neurlang::wrappers::datetime;

    // Get current time
    let now = datetime::now();
    assert!(now > 0);

    // Format and parse roundtrip
    let iso = datetime::format_iso(now);
    let parsed = datetime::parse_iso(&iso).unwrap();
    // Allow 1 second tolerance due to formatting precision
    assert!((now - parsed).abs() < 1000);

    // Component extraction
    let year = datetime::year(now);
    let month = datetime::month(now);
    let day = datetime::day(now);

    assert!(year >= 2024);
    assert!((1..=12).contains(&month));
    assert!((1..=31).contains(&day));

    // Date arithmetic
    let tomorrow = datetime::add_days(now, 1);
    assert_eq!(datetime::diff_days(tomorrow, now), 1);
}

// =============================================================================
// Regex Integration Tests
// =============================================================================

#[test]
fn test_regex_operations() {
    use neurlang::wrappers::regex;

    let pattern = OwnedBuffer::from_str(r"\d+");
    let input = OwnedBuffer::from_str("There are 42 apples and 7 oranges");

    // Match
    assert!(regex::is_match(&pattern, &input).unwrap());

    // Find all
    let matches = regex::find_all_text(&pattern, &input).unwrap();
    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0].as_str().unwrap(), "42");
    assert_eq!(matches[1].as_str().unwrap(), "7");

    // Replace
    let replacement = OwnedBuffer::from_str("X");
    let replaced = regex::replace(&pattern, &input, &replacement).unwrap();
    assert_eq!(replaced.as_str().unwrap(), "There are X apples and 7 oranges");

    // Replace all
    let replaced_all = regex::replace_all(&pattern, &input, &replacement).unwrap();
    assert_eq!(replaced_all.as_str().unwrap(), "There are X apples and X oranges");
}

#[test]
fn test_regex_capture_groups() {
    use neurlang::wrappers::regex;

    let pattern = OwnedBuffer::from_str(r"(?P<name>\w+)@(?P<domain>\w+)\.(?P<tld>\w+)");
    let input = OwnedBuffer::from_str("Contact us at support@example.com");

    let named = regex::captures_named(&pattern, &input).unwrap();
    assert_eq!(named.len(), 3);

    // Check named captures
    let name = named.iter().find(|(k, _)| k == "name").unwrap();
    assert_eq!(name.1.as_str().unwrap(), "support");

    let domain = named.iter().find(|(k, _)| k == "domain").unwrap();
    assert_eq!(domain.1.as_str().unwrap(), "example");
}

// =============================================================================
// Extension Registry Integration Tests
// =============================================================================

#[test]
fn test_extension_registry_includes_wrappers() {
    let registry = ExtensionRegistry::new();

    // Wrappers should be automatically registered
    assert!(registry.get_by_name("zlib_compress").is_some());
    assert!(registry.get_by_name("zlib_decompress").is_some());
    assert!(registry.get_by_name("base64_encode").is_some());
    assert!(registry.get_by_name("base64_decode").is_some());
    assert!(registry.get_by_name("datetime_now").is_some());
    assert!(registry.get_by_name("regex_match").is_some());
}

#[test]
fn test_extension_registry_compression_call() {
    let registry = ExtensionRegistry::new();

    let input = b"Hello, World! This needs to be long enough to compress well. Let's add more text here.";
    let mut compressed = vec![0u8; 256];
    let mut decompressed = vec![0u8; 256];
    let mut outputs = [0u64; 4];

    // Compress via extension registry
    let compress_id = registry.get_id("zlib_compress").unwrap();
    let args = [
        input.as_ptr() as u64,
        input.len() as u64,
        compressed.as_mut_ptr() as u64,
        compressed.len() as u64,
    ];
    let compressed_len = registry.call(compress_id, &args, &mut outputs).unwrap() as usize;
    assert!(compressed_len > 0);

    // Decompress via extension registry
    let decompress_id = registry.get_id("zlib_decompress").unwrap();
    let args = [
        compressed.as_ptr() as u64,
        compressed_len as u64,
        decompressed.as_mut_ptr() as u64,
        decompressed.len() as u64,
    ];
    let decompressed_len = registry.call(decompress_id, &args, &mut outputs).unwrap() as usize;

    // Verify roundtrip
    assert_eq!(&decompressed[..decompressed_len], input);
}

#[test]
fn test_extension_registry_filesystem_call() {
    let registry = ExtensionRegistry::new();
    let mut outputs = [0u64; 4];

    // Test fs_exists on /tmp
    let tmp_path = b"/tmp";
    let exists_id = registry.get_id("fs_exists").unwrap();
    let args = [
        tmp_path.as_ptr() as u64,
        tmp_path.len() as u64,
    ];
    let exists = registry.call(exists_id, &args, &mut outputs).unwrap();
    assert_eq!(exists, 1); // /tmp exists

    // Test fs_is_dir on /tmp
    let is_dir_id = registry.get_id("fs_is_dir").unwrap();
    let is_dir = registry.call(is_dir_id, &args, &mut outputs).unwrap();
    assert_eq!(is_dir, 1); // /tmp is a directory

    // Write a test file
    let test_path = b"/tmp/neurlang_ext_registry_test.txt";
    let test_content = b"Extension registry filesystem test!";
    let write_id = registry.get_id("fs_write_file").unwrap();
    let args = [
        test_path.as_ptr() as u64,
        test_path.len() as u64,
        test_content.as_ptr() as u64,
        test_content.len() as u64,
    ];
    let result = registry.call(write_id, &args, &mut outputs).unwrap();
    assert_eq!(result, 0); // Success

    // Read it back
    let mut read_buf = vec![0u8; 256];
    let read_id = registry.get_id("fs_read_file").unwrap();
    let args = [
        test_path.as_ptr() as u64,
        test_path.len() as u64,
        read_buf.as_mut_ptr() as u64,
        read_buf.len() as u64,
    ];
    let read_len = registry.call(read_id, &args, &mut outputs).unwrap() as usize;
    assert_eq!(&read_buf[..read_len], test_content);

    // Get file size
    let size_id = registry.get_id("fs_file_size").unwrap();
    let args = [
        test_path.as_ptr() as u64,
        test_path.len() as u64,
    ];
    let size = registry.call(size_id, &args, &mut outputs).unwrap() as usize;
    assert_eq!(size, test_content.len());

    // Cleanup
    let remove_id = registry.get_id("fs_remove_file").unwrap();
    let _ = registry.call(remove_id, &args, &mut outputs);
}

#[test]
fn test_extension_registry_x509_call() {
    let registry = ExtensionRegistry::new();
    let mut outputs = [0u64; 4];

    // Generate EC keypair via extension
    let curve = b"P-256";
    let mut keypair_buf = vec![0u8; 2048];
    let keypair_id = registry.get_id("x509_keypair_ec").unwrap();
    let args = [
        curve.as_ptr() as u64,
        curve.len() as u64,
        keypair_buf.as_mut_ptr() as u64,
        keypair_buf.len() as u64,
    ];
    let keypair_len = registry.call(keypair_id, &args, &mut outputs).unwrap() as usize;
    assert!(keypair_len > 0);

    // Verify it's PEM
    let keypair_str = std::str::from_utf8(&keypair_buf[..keypair_len]).unwrap();
    assert!(keypair_str.contains("BEGIN PRIVATE KEY"));

    // Create self-signed certificate
    let subject = b"CN=registry.test.com,O=Test";
    let days: u64 = 365;
    let mut cert_buf = vec![0u8; 4096];
    let cert_id = registry.get_id("x509_create_self_signed").unwrap();
    let args = [
        subject.as_ptr() as u64,
        subject.len() as u64,
        keypair_buf.as_ptr() as u64,
        keypair_len as u64,
        days,
        cert_buf.as_mut_ptr() as u64,
        cert_buf.len() as u64,
    ];
    let cert_len = registry.call(cert_id, &args, &mut outputs).unwrap() as usize;
    assert!(cert_len > 0);

    // Verify it's PEM certificate
    let cert_str = std::str::from_utf8(&cert_buf[..cert_len]).unwrap();
    assert!(cert_str.contains("BEGIN CERTIFICATE"));

    // Get subject from certificate
    let mut subject_buf = vec![0u8; 512];
    let get_subject_id = registry.get_id("x509_get_subject").unwrap();
    let args = [
        cert_buf.as_ptr() as u64,
        cert_len as u64,
        subject_buf.as_mut_ptr() as u64,
        subject_buf.len() as u64,
    ];
    let subject_len = registry.call(get_subject_id, &args, &mut outputs).unwrap() as usize;
    let subject_str = std::str::from_utf8(&subject_buf[..subject_len]).unwrap();
    assert!(subject_str.contains("registry.test.com"));

    // Check not expired
    let is_expired_id = registry.get_id("x509_is_expired").unwrap();
    let args = [
        cert_buf.as_ptr() as u64,
        cert_len as u64,
    ];
    let expired = registry.call(is_expired_id, &args, &mut outputs).unwrap();
    assert_eq!(expired, 0); // Not expired
}

#[test]
fn test_extension_registry_tls_config() {
    let registry = ExtensionRegistry::new();
    let mut outputs = [0u64; 4];

    // Create TLS client config
    let config_id = registry.get_id("tls_client_config").unwrap();
    let result = registry.call(config_id, &[], &mut outputs).unwrap();
    assert_eq!(result, 1); // Success
}

// =============================================================================
// Synonym Dictionary Tests
// =============================================================================

#[test]
fn test_synonym_expansion() {
    use neurlang::wrappers::synonyms;

    // Compression synonyms
    let expanded = synonyms::expand_synonyms(&["compress"]);
    assert!(expanded.contains(&"shrink".to_string()));
    assert!(expanded.contains(&"deflate".to_string()));
    assert!(expanded.contains(&"zip".to_string()));

    // Crypto synonyms
    let expanded = synonyms::expand_synonyms(&["encrypt"]);
    assert!(expanded.contains(&"cipher".to_string()));
    assert!(expanded.contains(&"secure".to_string()));

    // Are synonyms check
    assert!(synonyms::are_synonyms("compress", "shrink"));
    assert!(synonyms::are_synonyms("encrypt", "cipher"));
    assert!(!synonyms::are_synonyms("compress", "encrypt"));
}

// =============================================================================
// X509 Certificate Tests
// =============================================================================

#[test]
fn test_x509_self_signed_certificate() {
    use neurlang::wrappers::x509;

    // Generate key pair
    let keypair = x509::keypair_generate_ec("P-256").unwrap();
    assert!(keypair.as_str().unwrap().contains("BEGIN PRIVATE KEY"));

    // Create self-signed certificate
    let cert = x509::create_self_signed("CN=test.example.com,O=Test Org", &keypair, 365).unwrap();
    assert!(cert.as_str().unwrap().contains("BEGIN CERTIFICATE"));

    // Parse certificate
    let info = x509::parse_pem(&cert).unwrap();
    assert!(info.subject.contains("test.example.com"));
    assert!(!info.is_ca);

    // Check not expired
    assert!(!x509::is_expired(&cert).unwrap());
}

#[test]
fn test_x509_ca_certificate() {
    use neurlang::wrappers::x509;

    let keypair = x509::keypair_generate_ec("P-256").unwrap();
    let ca_cert = x509::create_ca("CN=My Test CA,O=Test", &keypair, 3650).unwrap();

    let info = x509::parse_pem(&ca_cert).unwrap();
    assert!(info.is_ca);
}

#[test]
fn test_x509_csr_generation() {
    use neurlang::wrappers::x509;

    let keypair = x509::keypair_generate_ec("P-256").unwrap();
    let csr = x509::create_csr("CN=server.example.com", &keypair).unwrap();

    assert!(csr.as_str().unwrap().contains("BEGIN CERTIFICATE REQUEST"));
}

// =============================================================================
// TLS Configuration Tests
// =============================================================================

#[test]
fn test_tls_client_config() {
    use neurlang::wrappers::tls;

    // Create client config with Mozilla CA roots
    let config = tls::client_config();
    assert!(config.is_ok());
}

// Note: TLS connection tests require network access and are in ignored tests
