//! Integration tests for cryptographic extensions
//!
//! These tests verify that the crypto extensions work correctly end-to-end,
//! using the safe HandleManager-based API (no raw pointers).

use ed25519_dalek::SigningKey;
use neurlang::runtime::{ExtensionRegistry, HandleManager, OwnedBuffer};
use x25519_dalek::StaticSecret;

/// Helper to store data and get a handle
fn store_buffer(data: &[u8]) -> u64 {
    HandleManager::store(OwnedBuffer::from_slice(data))
}

/// Helper to get data from a handle
fn get_buffer(handle: u64) -> Vec<u8> {
    HandleManager::get(handle)
        .map(|b| b.as_slice().to_vec())
        .unwrap_or_default()
}

/// Helper to call an extension by name and check for success
fn call_ext(registry: &ExtensionRegistry, name: &str, args: &[u64]) -> (i64, Vec<u64>) {
    let id = registry
        .get_id(name)
        .unwrap_or_else(|| panic!("Extension '{}' not found", name));
    let mut outputs = vec![0u64; 8];
    let result = registry
        .call(id, args, &mut outputs)
        .expect("Extension call failed");
    (result, outputs)
}

// === SHA-256 Integration Tests ===

#[test]
fn test_sha256_integration_known_vectors() {
    let registry = ExtensionRegistry::new();

    // Test vector: empty string
    let input1 = store_buffer(&[]);
    let (result, outputs) = call_ext(&registry, "sha256", &[input1]);
    assert_eq!(result, 0);
    let hash1 = get_buffer(outputs[0]);
    let expected1 =
        hex::decode("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855").unwrap();
    assert_eq!(hash1, expected1);
    HandleManager::remove(input1);
    HandleManager::remove(outputs[0]);

    // Test vector: "abc"
    let input2 = store_buffer(b"abc");
    let (result, outputs) = call_ext(&registry, "sha256", &[input2]);
    assert_eq!(result, 0);
    let hash2 = get_buffer(outputs[0]);
    let expected2 =
        hex::decode("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad").unwrap();
    assert_eq!(hash2, expected2);
    HandleManager::remove(input2);
    HandleManager::remove(outputs[0]);

    // Test vector: "hello"
    let input3 = store_buffer(b"hello");
    let (result, outputs) = call_ext(&registry, "sha256", &[input3]);
    assert_eq!(result, 0);
    let hash3 = get_buffer(outputs[0]);
    let expected3 =
        hex::decode("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824").unwrap();
    assert_eq!(hash3, expected3);
    HandleManager::remove(input3);
    HandleManager::remove(outputs[0]);
}

#[test]
fn test_sha256_large_input() {
    let registry = ExtensionRegistry::new();

    // Test with a large input (1MB)
    let large_input: Vec<u8> = (0..1_000_000).map(|i| (i % 256) as u8).collect();
    let handle = store_buffer(&large_input);

    let (result, outputs) = call_ext(&registry, "sha256", &[handle]);
    assert_eq!(result, 0);

    let hash = get_buffer(outputs[0]);
    assert_eq!(hash.len(), 32);
    assert!(hash.iter().any(|&b| b != 0));

    HandleManager::remove(handle);
    HandleManager::remove(outputs[0]);
}

// === HMAC-SHA256 Integration Tests ===

#[test]
fn test_hmac_sha256_integration() {
    let registry = ExtensionRegistry::new();

    let key = store_buffer(b"key");
    let data = store_buffer(b"The quick brown fox jumps over the lazy dog");

    let (result, outputs) = call_ext(&registry, "hmac_sha256", &[key, data]);
    assert_eq!(result, 0);

    let mac = get_buffer(outputs[0]);
    let expected =
        hex::decode("f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8").unwrap();
    assert_eq!(mac, expected);

    HandleManager::remove(key);
    HandleManager::remove(data);
    HandleManager::remove(outputs[0]);
}

#[test]
fn test_hmac_sha256_different_keys_produce_different_macs() {
    let registry = ExtensionRegistry::new();

    let key1 = store_buffer(b"secret1");
    let key2 = store_buffer(b"secret2");
    let data = store_buffer(b"message");

    let (_, outputs1) = call_ext(&registry, "hmac_sha256", &[key1, data]);
    let mac1 = get_buffer(outputs1[0]);

    // Need fresh data handle since it may have been consumed
    let data2 = store_buffer(b"message");
    let (_, outputs2) = call_ext(&registry, "hmac_sha256", &[key2, data2]);
    let mac2 = get_buffer(outputs2[0]);

    assert_ne!(mac1, mac2);

    HandleManager::remove(key1);
    HandleManager::remove(key2);
    HandleManager::remove(data);
    HandleManager::remove(data2);
    HandleManager::remove(outputs1[0]);
    HandleManager::remove(outputs2[0]);
}

// === AES-256-GCM Integration Tests ===

#[test]
fn test_aes256_gcm_roundtrip() {
    let registry = ExtensionRegistry::new();

    // Test with various message sizes
    for size in [0, 1, 15, 16, 17, 100, 1000] {
        let key_data = [0x42u8; 32];
        let nonce_data = [0x11u8; 12];
        let plaintext_data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();

        let key = store_buffer(&key_data);
        let nonce = store_buffer(&nonce_data);
        let plaintext = store_buffer(&plaintext_data);

        // Encrypt
        let (enc_result, enc_outputs) =
            call_ext(&registry, "aes256_gcm_encrypt", &[key, nonce, plaintext]);
        assert_eq!(enc_result, 0, "Encrypt failed for size {}", size);
        let ciphertext = enc_outputs[0];
        let tag = enc_outputs[1];

        // Need fresh handles for decrypt
        let key2 = store_buffer(&key_data);
        let nonce2 = store_buffer(&nonce_data);

        // Decrypt
        let (dec_result, dec_outputs) = call_ext(
            &registry,
            "aes256_gcm_decrypt",
            &[key2, nonce2, ciphertext, tag],
        );
        assert_eq!(dec_result, 0, "Decrypt failed for size {}", size);

        let decrypted = get_buffer(dec_outputs[0]);
        assert_eq!(
            plaintext_data, decrypted,
            "Roundtrip failed for size {}",
            size
        );

        // Cleanup
        HandleManager::remove(key);
        HandleManager::remove(nonce);
        HandleManager::remove(plaintext);
        HandleManager::remove(key2);
        HandleManager::remove(nonce2);
        HandleManager::remove(ciphertext);
        HandleManager::remove(tag);
        HandleManager::remove(dec_outputs[0]);
    }
}

#[test]
fn test_aes256_gcm_authentication_failure_on_modified_ciphertext() {
    let registry = ExtensionRegistry::new();

    let key_data = [0x42u8; 32];
    let nonce_data = [0x11u8; 12];
    let plaintext_data = b"Secret message";

    let key = store_buffer(&key_data);
    let nonce = store_buffer(&nonce_data);
    let plaintext = store_buffer(plaintext_data);

    // Encrypt
    let (_, enc_outputs) = call_ext(&registry, "aes256_gcm_encrypt", &[key, nonce, plaintext]);
    let ciphertext_handle = enc_outputs[0];
    let tag = enc_outputs[1];

    // Modify ciphertext
    let mut ciphertext_data = get_buffer(ciphertext_handle);
    ciphertext_data[0] ^= 0xFF;
    HandleManager::remove(ciphertext_handle);
    let modified_ct = store_buffer(&ciphertext_data);

    // Fresh handles for decrypt
    let key2 = store_buffer(&key_data);
    let nonce2 = store_buffer(&nonce_data);

    // Attempt decryption (should fail)
    let (result, _) = call_ext(
        &registry,
        "aes256_gcm_decrypt",
        &[key2, nonce2, modified_ct, tag],
    );
    assert_eq!(result, -1, "Should return -1 on authentication failure");

    // Cleanup
    HandleManager::remove(key);
    HandleManager::remove(nonce);
    HandleManager::remove(plaintext);
    HandleManager::remove(key2);
    HandleManager::remove(nonce2);
    HandleManager::remove(modified_ct);
    HandleManager::remove(tag);
}

#[test]
fn test_aes256_gcm_authentication_failure_on_modified_tag() {
    let registry = ExtensionRegistry::new();

    let key_data = [0x42u8; 32];
    let nonce_data = [0x11u8; 12];
    let plaintext_data = b"Secret message";

    let key = store_buffer(&key_data);
    let nonce = store_buffer(&nonce_data);
    let plaintext = store_buffer(plaintext_data);

    // Encrypt
    let (_, enc_outputs) = call_ext(&registry, "aes256_gcm_encrypt", &[key, nonce, plaintext]);
    let ciphertext = enc_outputs[0];
    let tag_handle = enc_outputs[1];

    // Modify tag
    let mut tag_data = get_buffer(tag_handle);
    tag_data[0] ^= 0xFF;
    HandleManager::remove(tag_handle);
    let modified_tag = store_buffer(&tag_data);

    // Fresh handles for decrypt
    let key2 = store_buffer(&key_data);
    let nonce2 = store_buffer(&nonce_data);

    // Attempt decryption (should fail)
    let (result, _) = call_ext(
        &registry,
        "aes256_gcm_decrypt",
        &[key2, nonce2, ciphertext, modified_tag],
    );
    assert_eq!(result, -1, "Should return -1 on authentication failure");

    // Cleanup
    HandleManager::remove(key);
    HandleManager::remove(nonce);
    HandleManager::remove(plaintext);
    HandleManager::remove(key2);
    HandleManager::remove(nonce2);
    HandleManager::remove(ciphertext);
    HandleManager::remove(modified_tag);
}

#[test]
fn test_aes256_gcm_different_nonces() {
    let registry = ExtensionRegistry::new();

    let key_data = [0x42u8; 32];
    let nonce1_data = [0x11u8; 12];
    let nonce2_data = [0x22u8; 12];
    let plaintext_data = b"Same message";

    // First encryption
    let key1 = store_buffer(&key_data);
    let nonce1 = store_buffer(&nonce1_data);
    let pt1 = store_buffer(plaintext_data);
    let (_, out1) = call_ext(&registry, "aes256_gcm_encrypt", &[key1, nonce1, pt1]);
    let ct1 = get_buffer(out1[0]);
    let tag1 = get_buffer(out1[1]);

    // Second encryption with different nonce
    let key2 = store_buffer(&key_data);
    let nonce2 = store_buffer(&nonce2_data);
    let pt2 = store_buffer(plaintext_data);
    let (_, out2) = call_ext(&registry, "aes256_gcm_encrypt", &[key2, nonce2, pt2]);
    let ct2 = get_buffer(out2[0]);
    let tag2 = get_buffer(out2[1]);

    // Ciphertexts and tags should be different
    assert_ne!(ct1, ct2);
    assert_ne!(tag1, tag2);

    // Cleanup
    HandleManager::remove(key1);
    HandleManager::remove(nonce1);
    HandleManager::remove(pt1);
    HandleManager::remove(key2);
    HandleManager::remove(nonce2);
    HandleManager::remove(pt2);
    HandleManager::remove(out1[0]);
    HandleManager::remove(out1[1]);
    HandleManager::remove(out2[0]);
    HandleManager::remove(out2[1]);
}

// === Ed25519 Integration Tests ===

#[test]
fn test_ed25519_sign_verify_roundtrip() {
    let registry = ExtensionRegistry::new();

    // Test with various message sizes
    for size in [0, 1, 100, 1000] {
        let secret_key_data = [42u8; 32];
        let signing_key = SigningKey::from_bytes(&secret_key_data);
        let public_key_data = signing_key.verifying_key().to_bytes();

        let message_data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();

        // Sign
        let sk = store_buffer(&secret_key_data);
        let msg = store_buffer(&message_data);
        let (sign_result, sign_outputs) = call_ext(&registry, "ed25519_sign", &[sk, msg]);
        assert_eq!(sign_result, 0, "Sign failed for size {}", size);
        let sig = sign_outputs[0];

        // Verify
        let pk = store_buffer(&public_key_data);
        let msg2 = store_buffer(&message_data);
        let (verify_result, verify_outputs) =
            call_ext(&registry, "ed25519_verify", &[pk, msg2, sig]);
        assert_eq!(verify_result, 1, "Verify failed for size {}", size);
        assert_eq!(verify_outputs[0], 1);

        // Cleanup
        HandleManager::remove(sk);
        HandleManager::remove(msg);
        HandleManager::remove(sig);
        HandleManager::remove(pk);
        HandleManager::remove(msg2);
    }
}

#[test]
fn test_ed25519_different_messages_different_signatures() {
    let registry = ExtensionRegistry::new();

    let secret_key_data = [42u8; 32];

    let sk1 = store_buffer(&secret_key_data);
    let msg1 = store_buffer(b"Hello");
    let (_, out1) = call_ext(&registry, "ed25519_sign", &[sk1, msg1]);
    let sig1 = get_buffer(out1[0]);

    let sk2 = store_buffer(&secret_key_data);
    let msg2 = store_buffer(b"World");
    let (_, out2) = call_ext(&registry, "ed25519_sign", &[sk2, msg2]);
    let sig2 = get_buffer(out2[0]);

    assert_ne!(sig1, sig2);

    HandleManager::remove(sk1);
    HandleManager::remove(msg1);
    HandleManager::remove(out1[0]);
    HandleManager::remove(sk2);
    HandleManager::remove(msg2);
    HandleManager::remove(out2[0]);
}

#[test]
fn test_ed25519_signature_verification_fails_for_wrong_message() {
    let registry = ExtensionRegistry::new();

    let secret_key_data = [42u8; 32];
    let signing_key = SigningKey::from_bytes(&secret_key_data);
    let public_key_data = signing_key.verifying_key().to_bytes();

    // Sign message1
    let sk = store_buffer(&secret_key_data);
    let msg1 = store_buffer(b"Original message");
    let (_, sign_outputs) = call_ext(&registry, "ed25519_sign", &[sk, msg1]);
    let sig = sign_outputs[0];

    // Try to verify with message2 (should fail)
    let pk = store_buffer(&public_key_data);
    let msg2 = store_buffer(b"Different message");
    let (result, outputs) = call_ext(&registry, "ed25519_verify", &[pk, msg2, sig]);
    assert_eq!(result, 0, "Signature should not verify for wrong message");
    assert_eq!(outputs[0], 0);

    HandleManager::remove(sk);
    HandleManager::remove(msg1);
    HandleManager::remove(sig);
    HandleManager::remove(pk);
    HandleManager::remove(msg2);
}

#[test]
fn test_ed25519_signature_verification_fails_for_wrong_key() {
    let registry = ExtensionRegistry::new();

    let secret_key1_data = [42u8; 32];
    let secret_key2_data = [43u8; 32]; // Different key
    let signing_key2 = SigningKey::from_bytes(&secret_key2_data);
    let public_key2_data = signing_key2.verifying_key().to_bytes();

    // Sign with key1
    let sk1 = store_buffer(&secret_key1_data);
    let msg = store_buffer(b"Test message");
    let (_, sign_outputs) = call_ext(&registry, "ed25519_sign", &[sk1, msg]);
    let sig = sign_outputs[0];

    // Try to verify with key2's public key (should fail)
    let pk2 = store_buffer(&public_key2_data);
    let msg2 = store_buffer(b"Test message");
    let (result, outputs) = call_ext(&registry, "ed25519_verify", &[pk2, msg2, sig]);
    assert_eq!(
        result, 0,
        "Signature should not verify with wrong public key"
    );
    assert_eq!(outputs[0], 0);

    HandleManager::remove(sk1);
    HandleManager::remove(msg);
    HandleManager::remove(sig);
    HandleManager::remove(pk2);
    HandleManager::remove(msg2);
}

// === X25519 Integration Tests ===

#[test]
fn test_x25519_key_agreement() {
    let registry = ExtensionRegistry::new();

    // Alice's key pair
    let alice_secret_data = [1u8; 32];
    let alice_public_data =
        x25519_dalek::PublicKey::from(&StaticSecret::from(alice_secret_data)).to_bytes();

    // Bob's key pair
    let bob_secret_data = [2u8; 32];
    let bob_public_data =
        x25519_dalek::PublicKey::from(&StaticSecret::from(bob_secret_data)).to_bytes();

    // Alice computes shared secret
    let alice_sk = store_buffer(&alice_secret_data);
    let bob_pk = store_buffer(&bob_public_data);
    let (result1, out1) = call_ext(&registry, "x25519_derive", &[alice_sk, bob_pk]);
    assert_eq!(result1, 0);
    let alice_shared = get_buffer(out1[0]);

    // Bob computes shared secret
    let bob_sk = store_buffer(&bob_secret_data);
    let alice_pk = store_buffer(&alice_public_data);
    let (result2, out2) = call_ext(&registry, "x25519_derive", &[bob_sk, alice_pk]);
    assert_eq!(result2, 0);
    let bob_shared = get_buffer(out2[0]);

    // Shared secrets must match
    assert_eq!(alice_shared, bob_shared);
    // Shared secret should not be all zeros
    assert!(alice_shared.iter().any(|&b| b != 0));

    HandleManager::remove(alice_sk);
    HandleManager::remove(bob_pk);
    HandleManager::remove(out1[0]);
    HandleManager::remove(bob_sk);
    HandleManager::remove(alice_pk);
    HandleManager::remove(out2[0]);
}

#[test]
fn test_x25519_different_parties_produce_different_shared_secrets() {
    let registry = ExtensionRegistry::new();

    // Alice
    let alice_secret_data = [1u8; 32];

    // Bob
    let bob_secret_data = [2u8; 32];
    let bob_public_data =
        x25519_dalek::PublicKey::from(&StaticSecret::from(bob_secret_data)).to_bytes();

    // Carol
    let carol_secret_data = [3u8; 32];
    let carol_public_data =
        x25519_dalek::PublicKey::from(&StaticSecret::from(carol_secret_data)).to_bytes();

    // Alice-Bob shared secret
    let alice_sk1 = store_buffer(&alice_secret_data);
    let bob_pk = store_buffer(&bob_public_data);
    let (_, out1) = call_ext(&registry, "x25519_derive", &[alice_sk1, bob_pk]);
    let alice_bob_shared = get_buffer(out1[0]);

    // Alice-Carol shared secret
    let alice_sk2 = store_buffer(&alice_secret_data);
    let carol_pk = store_buffer(&carol_public_data);
    let (_, out2) = call_ext(&registry, "x25519_derive", &[alice_sk2, carol_pk]);
    let alice_carol_shared = get_buffer(out2[0]);

    // Different pairs should produce different shared secrets
    assert_ne!(alice_bob_shared, alice_carol_shared);

    HandleManager::remove(alice_sk1);
    HandleManager::remove(bob_pk);
    HandleManager::remove(out1[0]);
    HandleManager::remove(alice_sk2);
    HandleManager::remove(carol_pk);
    HandleManager::remove(out2[0]);
}

// === PBKDF2 Integration Tests ===

#[test]
fn test_pbkdf2_deterministic() {
    let registry = ExtensionRegistry::new();

    let password_data = b"password123";
    let salt_data = b"somesalt";
    let iterations: u64 = 1000;
    let out_len: u64 = 32;

    // First derivation
    let pwd1 = store_buffer(password_data);
    let salt1 = store_buffer(salt_data);
    let (result1, out1) = call_ext(
        &registry,
        "pbkdf2_sha256",
        &[pwd1, salt1, iterations, out_len],
    );
    assert_eq!(result1, 0);
    let key1 = get_buffer(out1[0]);

    // Second derivation with same params
    let pwd2 = store_buffer(password_data);
    let salt2 = store_buffer(salt_data);
    let (result2, out2) = call_ext(
        &registry,
        "pbkdf2_sha256",
        &[pwd2, salt2, iterations, out_len],
    );
    assert_eq!(result2, 0);
    let key2 = get_buffer(out2[0]);

    // Results should match
    assert_eq!(key1, key2);

    HandleManager::remove(pwd1);
    HandleManager::remove(salt1);
    HandleManager::remove(out1[0]);
    HandleManager::remove(pwd2);
    HandleManager::remove(salt2);
    HandleManager::remove(out2[0]);
}

#[test]
fn test_pbkdf2_different_salts() {
    let registry = ExtensionRegistry::new();

    let password_data = b"password";
    let iterations: u64 = 1000;
    let out_len: u64 = 32;

    let pwd1 = store_buffer(password_data);
    let salt1 = store_buffer(b"salt1");
    let (_, out1) = call_ext(
        &registry,
        "pbkdf2_sha256",
        &[pwd1, salt1, iterations, out_len],
    );
    let key1 = get_buffer(out1[0]);

    let pwd2 = store_buffer(password_data);
    let salt2 = store_buffer(b"salt2");
    let (_, out2) = call_ext(
        &registry,
        "pbkdf2_sha256",
        &[pwd2, salt2, iterations, out_len],
    );
    let key2 = get_buffer(out2[0]);

    assert_ne!(key1, key2);

    HandleManager::remove(pwd1);
    HandleManager::remove(salt1);
    HandleManager::remove(out1[0]);
    HandleManager::remove(pwd2);
    HandleManager::remove(salt2);
    HandleManager::remove(out2[0]);
}

#[test]
fn test_pbkdf2_different_output_lengths() {
    let registry = ExtensionRegistry::new();

    let password_data = b"password";
    let salt_data = b"salt";
    let iterations: u64 = 1000;

    let pwd1 = store_buffer(password_data);
    let salt1 = store_buffer(salt_data);
    let (_, out16) = call_ext(&registry, "pbkdf2_sha256", &[pwd1, salt1, iterations, 16]);
    let key16 = get_buffer(out16[0]);

    let pwd2 = store_buffer(password_data);
    let salt2 = store_buffer(salt_data);
    let (_, out32) = call_ext(&registry, "pbkdf2_sha256", &[pwd2, salt2, iterations, 32]);
    let key32 = get_buffer(out32[0]);

    let pwd3 = store_buffer(password_data);
    let salt3 = store_buffer(salt_data);
    let (_, out64) = call_ext(&registry, "pbkdf2_sha256", &[pwd3, salt3, iterations, 64]);
    let key64 = get_buffer(out64[0]);

    // The first 16 bytes should match for all output lengths
    assert_eq!(&key16[..16], &key32[..16]);
    assert_eq!(&key16[..16], &key64[..16]);

    HandleManager::remove(pwd1);
    HandleManager::remove(salt1);
    HandleManager::remove(out16[0]);
    HandleManager::remove(pwd2);
    HandleManager::remove(salt2);
    HandleManager::remove(out32[0]);
    HandleManager::remove(pwd3);
    HandleManager::remove(salt3);
    HandleManager::remove(out64[0]);
}

// === Secure Random Integration Tests ===

#[test]
fn test_secure_random_produces_random_bytes() {
    let registry = ExtensionRegistry::new();

    let (_, out1) = call_ext(&registry, "secure_random", &[32]);
    let buf1 = get_buffer(out1[0]);

    let (_, out2) = call_ext(&registry, "secure_random", &[32]);
    let buf2 = get_buffer(out2[0]);

    // Should be different (extremely high probability)
    assert_ne!(buf1, buf2);
    // At least one should not be all zeros
    assert!(buf1.iter().any(|&b| b != 0) || buf2.iter().any(|&b| b != 0));

    HandleManager::remove(out1[0]);
    HandleManager::remove(out2[0]);
}

#[test]
fn test_secure_random_various_sizes() {
    let registry = ExtensionRegistry::new();

    for size in [1u64, 16, 32, 64, 128, 256] {
        let (result, outputs) = call_ext(&registry, "secure_random", &[size]);
        assert_eq!(result, 0, "Random generation failed for size {}", size);
        let buf = get_buffer(outputs[0]);
        assert_eq!(buf.len(), size as usize);
        HandleManager::remove(outputs[0]);
    }
}

// === Constant-Time Comparison Integration Tests ===

#[test]
fn test_constant_time_eq_equal() {
    let registry = ExtensionRegistry::new();

    let a = store_buffer(&[1u8, 2, 3, 4, 5, 6, 7, 8]);
    let b = store_buffer(&[1u8, 2, 3, 4, 5, 6, 7, 8]);

    let (result, outputs) = call_ext(&registry, "constant_time_eq", &[a, b]);
    assert_eq!(result, 1);
    assert_eq!(outputs[0], 1);

    HandleManager::remove(a);
    HandleManager::remove(b);
}

#[test]
fn test_constant_time_eq_not_equal() {
    let registry = ExtensionRegistry::new();

    let a = store_buffer(&[1u8, 2, 3, 4, 5, 6, 7, 8]);
    let b = store_buffer(&[1u8, 2, 3, 4, 5, 6, 7, 9]); // Last byte different

    let (result, outputs) = call_ext(&registry, "constant_time_eq", &[a, b]);
    assert_eq!(result, 0);
    assert_eq!(outputs[0], 0);

    HandleManager::remove(a);
    HandleManager::remove(b);
}

#[test]
fn test_constant_time_eq_empty() {
    let registry = ExtensionRegistry::new();

    let a = store_buffer(&[]);
    let b = store_buffer(&[]);

    let (result, outputs) = call_ext(&registry, "constant_time_eq", &[a, b]);
    assert_eq!(result, 1); // Empty slices are equal
    assert_eq!(outputs[0], 1);

    HandleManager::remove(a);
    HandleManager::remove(b);
}

// === Combined Crypto Workflow Tests ===

#[test]
fn test_complete_encryption_workflow() {
    let registry = ExtensionRegistry::new();

    // 1. Generate random key and nonce
    let (_, key_out) = call_ext(&registry, "secure_random", &[32]);
    let key_data = get_buffer(key_out[0]);
    let (_, nonce_out) = call_ext(&registry, "secure_random", &[12]);
    let nonce_data = get_buffer(nonce_out[0]);

    // 2. Encrypt message
    let plaintext_data = b"This is a secret message that needs protection!";
    let key = store_buffer(&key_data);
    let nonce = store_buffer(&nonce_data);
    let plaintext = store_buffer(plaintext_data);

    let (_, enc_outputs) = call_ext(&registry, "aes256_gcm_encrypt", &[key, nonce, plaintext]);
    let ciphertext = enc_outputs[0];
    let tag = enc_outputs[1];

    // 3. Decrypt message
    let key2 = store_buffer(&key_data);
    let nonce2 = store_buffer(&nonce_data);

    let (result, dec_outputs) = call_ext(
        &registry,
        "aes256_gcm_decrypt",
        &[key2, nonce2, ciphertext, tag],
    );

    assert_eq!(result, 0);
    let decrypted = get_buffer(dec_outputs[0]);
    assert_eq!(decrypted.as_slice(), plaintext_data);

    // Cleanup
    HandleManager::remove(key_out[0]);
    HandleManager::remove(nonce_out[0]);
    HandleManager::remove(key);
    HandleManager::remove(nonce);
    HandleManager::remove(plaintext);
    HandleManager::remove(ciphertext);
    HandleManager::remove(tag);
    HandleManager::remove(key2);
    HandleManager::remove(nonce2);
    HandleManager::remove(dec_outputs[0]);
}

#[test]
fn test_complete_signing_workflow() {
    let registry = ExtensionRegistry::new();

    // 1. Generate random secret key
    let (_, sk_out) = call_ext(&registry, "secure_random", &[32]);
    let secret_key_data = get_buffer(sk_out[0]);

    // 2. Derive public key
    let secret_key_arr: [u8; 32] = secret_key_data.clone().try_into().unwrap();
    let signing_key = SigningKey::from_bytes(&secret_key_arr);
    let public_key_data = signing_key.verifying_key().to_bytes();

    // 3. Sign a message
    let message_data = b"Important document that needs to be signed";
    let sk = store_buffer(&secret_key_data);
    let msg = store_buffer(message_data);

    let (_, sign_outputs) = call_ext(&registry, "ed25519_sign", &[sk, msg]);
    let sig = sign_outputs[0];

    // 4. Verify the signature
    let pk = store_buffer(&public_key_data);
    let msg2 = store_buffer(message_data);

    let (result, verify_outputs) = call_ext(&registry, "ed25519_verify", &[pk, msg2, sig]);

    assert_eq!(result, 1, "Signature verification failed");
    assert_eq!(verify_outputs[0], 1);

    // Cleanup
    HandleManager::remove(sk_out[0]);
    HandleManager::remove(sk);
    HandleManager::remove(msg);
    HandleManager::remove(sig);
    HandleManager::remove(pk);
    HandleManager::remove(msg2);
}

#[test]
fn test_complete_key_exchange_and_encryption_workflow() {
    let registry = ExtensionRegistry::new();

    // 1. Alice generates her key pair
    let (_, alice_sk_out) = call_ext(&registry, "secure_random", &[32]);
    let alice_secret_data = get_buffer(alice_sk_out[0]);
    let alice_secret_arr: [u8; 32] = alice_secret_data.clone().try_into().unwrap();
    let alice_public_data =
        x25519_dalek::PublicKey::from(&StaticSecret::from(alice_secret_arr)).to_bytes();

    // 2. Bob generates his key pair
    let (_, bob_sk_out) = call_ext(&registry, "secure_random", &[32]);
    let bob_secret_data = get_buffer(bob_sk_out[0]);
    let bob_secret_arr: [u8; 32] = bob_secret_data.clone().try_into().unwrap();
    let bob_public_data =
        x25519_dalek::PublicKey::from(&StaticSecret::from(bob_secret_arr)).to_bytes();

    // 3. Alice performs key exchange to get shared secret
    let alice_sk = store_buffer(&alice_secret_data);
    let bob_pk = store_buffer(&bob_public_data);
    let (_, shared_out) = call_ext(&registry, "x25519_derive", &[alice_sk, bob_pk]);
    let shared_secret_data = get_buffer(shared_out[0]);

    // 4. Use shared secret as encryption key
    let nonce_data = [0x11u8; 12];
    let plaintext_data = b"Message encrypted with key derived from key exchange";

    let key = store_buffer(&shared_secret_data);
    let nonce = store_buffer(&nonce_data);
    let plaintext = store_buffer(plaintext_data);

    let (_, enc_outputs) = call_ext(&registry, "aes256_gcm_encrypt", &[key, nonce, plaintext]);
    let ciphertext = enc_outputs[0];
    let tag = enc_outputs[1];

    // 5. Bob computes same shared secret
    let bob_sk = store_buffer(&bob_secret_data);
    let alice_pk = store_buffer(&alice_public_data);
    let (_, bob_shared_out) = call_ext(&registry, "x25519_derive", &[bob_sk, alice_pk]);
    let bob_shared_data = get_buffer(bob_shared_out[0]);

    // 6. Bob decrypts using his derived key
    let bob_key = store_buffer(&bob_shared_data);
    let nonce2 = store_buffer(&nonce_data);

    let (result, dec_outputs) = call_ext(
        &registry,
        "aes256_gcm_decrypt",
        &[bob_key, nonce2, ciphertext, tag],
    );

    assert_eq!(result, 0);
    let decrypted = get_buffer(dec_outputs[0]);
    assert_eq!(decrypted.as_slice(), plaintext_data);

    // Cleanup
    HandleManager::remove(alice_sk_out[0]);
    HandleManager::remove(bob_sk_out[0]);
    HandleManager::remove(alice_sk);
    HandleManager::remove(bob_pk);
    HandleManager::remove(shared_out[0]);
    HandleManager::remove(key);
    HandleManager::remove(nonce);
    HandleManager::remove(plaintext);
    HandleManager::remove(ciphertext);
    HandleManager::remove(tag);
    HandleManager::remove(bob_sk);
    HandleManager::remove(alice_pk);
    HandleManager::remove(bob_shared_out[0]);
    HandleManager::remove(bob_key);
    HandleManager::remove(nonce2);
    HandleManager::remove(dec_outputs[0]);
}

#[test]
fn test_password_based_encryption_workflow() {
    let registry = ExtensionRegistry::new();

    // 1. User's password
    let password_data = b"user_password_123";

    // 2. Generate random salt
    let (_, salt_out) = call_ext(&registry, "secure_random", &[16]);
    let salt_data = get_buffer(salt_out[0]);

    // 3. Derive key from password using PBKDF2
    let pwd = store_buffer(password_data);
    let salt = store_buffer(&salt_data);
    let (_, key_out) = call_ext(&registry, "pbkdf2_sha256", &[pwd, salt, 100000, 32]);
    let key_data = get_buffer(key_out[0]);

    // 4. Generate nonce
    let (_, nonce_out) = call_ext(&registry, "secure_random", &[12]);
    let nonce_data = get_buffer(nonce_out[0]);

    // 5. Encrypt data
    let plaintext_data = b"User's sensitive data";
    let key = store_buffer(&key_data);
    let nonce = store_buffer(&nonce_data);
    let plaintext = store_buffer(plaintext_data);

    let (_, enc_outputs) = call_ext(&registry, "aes256_gcm_encrypt", &[key, nonce, plaintext]);
    let ciphertext = enc_outputs[0];
    let tag = enc_outputs[1];

    // 6. Later: Re-derive key from same password and salt
    let pwd2 = store_buffer(password_data);
    let salt2 = store_buffer(&salt_data);
    let (_, key2_out) = call_ext(&registry, "pbkdf2_sha256", &[pwd2, salt2, 100000, 32]);
    let key2_data = get_buffer(key2_out[0]);

    // 7. Decrypt with re-derived key
    let key2 = store_buffer(&key2_data);
    let nonce2 = store_buffer(&nonce_data);

    let (result, dec_outputs) = call_ext(
        &registry,
        "aes256_gcm_decrypt",
        &[key2, nonce2, ciphertext, tag],
    );

    assert_eq!(result, 0);
    let decrypted = get_buffer(dec_outputs[0]);
    assert_eq!(decrypted.as_slice(), plaintext_data);

    // Cleanup
    HandleManager::remove(salt_out[0]);
    HandleManager::remove(pwd);
    HandleManager::remove(salt);
    HandleManager::remove(key_out[0]);
    HandleManager::remove(nonce_out[0]);
    HandleManager::remove(key);
    HandleManager::remove(nonce);
    HandleManager::remove(plaintext);
    HandleManager::remove(ciphertext);
    HandleManager::remove(tag);
    HandleManager::remove(pwd2);
    HandleManager::remove(salt2);
    HandleManager::remove(key2_out[0]);
    HandleManager::remove(key2);
    HandleManager::remove(nonce2);
    HandleManager::remove(dec_outputs[0]);
}
