//! Cryptographic Extensions
//!
//! All crypto operations: hash, HMAC, encrypt, sign, key derivation.
//! Uses audited libraries: ring, ed25519-dalek, x25519-dalek, sha2, blake2, etc.
//!
//! # Memory Safety
//!
//! All operations use BufferHandle for safe memory access. No raw pointers.
//! - Input: BufferHandle pointing to data in HandleManager
//! - Output: New BufferHandle returned, caller must free with HandleManager::remove()

use super::{ExtCategory, ExtError, ExtensionRegistry, HandleManager, OwnedBuffer};
use std::sync::Arc;

// Crypto libraries - using audited, production-ready crates
use blake2::{Blake2b512, Blake2s256};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305,
};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use hkdf::Hkdf;
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::rand::SecureRandom;
use ring::{digest, hmac, pbkdf2, rand as ring_rand};
use sha1::Sha1;
use sha2::Sha256;
use sha3::{Sha3_256, Sha3_512};
use subtle::ConstantTimeEq;
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret};

/// Helper to get buffer from handle with proper error
fn get_buffer(handle: u64) -> Result<OwnedBuffer, ExtError> {
    HandleManager::get(handle)
        .ok_or_else(|| ExtError::ExtensionError(format!("Invalid buffer handle: {}", handle)))
}

/// Register all crypto extensions
pub fn register_crypto(registry: &mut ExtensionRegistry) {
    register_hash_functions(registry);
    register_hmac_functions(registry);
    register_encryption(registry);
    register_signatures(registry);
    register_key_derivation(registry);
    register_key_exchange(registry);
    register_random(registry);
}

fn register_hash_functions(registry: &mut ExtensionRegistry) {
    // SHA-256: input_handle -> output_handle (32 bytes)
    registry.register(
        "sha256",
        "Compute SHA-256 hash. Args: input_handle. Returns output_handle (32 bytes).",
        1,
        true,
        ExtCategory::Crypto,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let input = get_buffer(args[0])?;
            let hash = digest::digest(&digest::SHA256, input.as_slice());
            let output = OwnedBuffer::from_slice(hash.as_ref());
            outputs[0] = HandleManager::store(output);
            Ok(0)
        }),
    );

    // SHA-384
    registry.register(
        "sha384",
        "Compute SHA-384 hash. Args: input_handle. Returns output_handle (48 bytes).",
        1,
        true,
        ExtCategory::Crypto,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let input = get_buffer(args[0])?;
            let hash = digest::digest(&digest::SHA384, input.as_slice());
            let output = OwnedBuffer::from_slice(hash.as_ref());
            outputs[0] = HandleManager::store(output);
            Ok(0)
        }),
    );

    // SHA-512
    registry.register(
        "sha512",
        "Compute SHA-512 hash. Args: input_handle. Returns output_handle (64 bytes).",
        1,
        true,
        ExtCategory::Crypto,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let input = get_buffer(args[0])?;
            let hash = digest::digest(&digest::SHA512, input.as_slice());
            let output = OwnedBuffer::from_slice(hash.as_ref());
            outputs[0] = HandleManager::store(output);
            Ok(0)
        }),
    );

    // SHA-1 (legacy, for WebSocket handshake compatibility)
    registry.register(
        "sha1",
        "Compute SHA-1 hash (LEGACY). Args: input_handle. Returns output_handle (20 bytes).",
        1,
        true,
        ExtCategory::Crypto,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            use sha1::Digest;
            let input = get_buffer(args[0])?;
            let hash = Sha1::digest(input.as_slice());
            let output = OwnedBuffer::from_slice(hash.as_slice());
            outputs[0] = HandleManager::store(output);
            Ok(0)
        }),
    );

    // SHA3-256
    registry.register(
        "sha3_256",
        "Compute SHA3-256 hash. Args: input_handle. Returns output_handle (32 bytes).",
        1,
        true,
        ExtCategory::Crypto,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            use sha3::Digest;
            let input = get_buffer(args[0])?;
            let hash = Sha3_256::digest(input.as_slice());
            let output = OwnedBuffer::from_slice(hash.as_slice());
            outputs[0] = HandleManager::store(output);
            Ok(0)
        }),
    );

    // SHA3-512
    registry.register(
        "sha3_512",
        "Compute SHA3-512 hash. Args: input_handle. Returns output_handle (64 bytes).",
        1,
        true,
        ExtCategory::Crypto,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            use sha3::Digest;
            let input = get_buffer(args[0])?;
            let hash = Sha3_512::digest(input.as_slice());
            let output = OwnedBuffer::from_slice(hash.as_slice());
            outputs[0] = HandleManager::store(output);
            Ok(0)
        }),
    );

    // BLAKE2b-512
    registry.register(
        "blake2b_512",
        "Compute BLAKE2b-512 hash. Args: input_handle. Returns output_handle (64 bytes).",
        1,
        true,
        ExtCategory::Crypto,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            use blake2::Digest;
            let input = get_buffer(args[0])?;
            let hash = Blake2b512::digest(input.as_slice());
            let output = OwnedBuffer::from_slice(hash.as_slice());
            outputs[0] = HandleManager::store(output);
            Ok(0)
        }),
    );

    // BLAKE2s-256
    registry.register(
        "blake2s_256",
        "Compute BLAKE2s-256 hash. Args: input_handle. Returns output_handle (32 bytes).",
        1,
        true,
        ExtCategory::Crypto,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            use blake2::Digest;
            let input = get_buffer(args[0])?;
            let hash = Blake2s256::digest(input.as_slice());
            let output = OwnedBuffer::from_slice(hash.as_slice());
            outputs[0] = HandleManager::store(output);
            Ok(0)
        }),
    );
}

fn register_hmac_functions(registry: &mut ExtensionRegistry) {
    // HMAC-SHA256: key_handle, data_handle -> output_handle (32 bytes)
    registry.register(
        "hmac_sha256",
        "Compute HMAC-SHA256. Args: key_handle, data_handle. Returns output_handle (32 bytes).",
        2,
        true,
        ExtCategory::Crypto,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let key = get_buffer(args[0])?;
            let data = get_buffer(args[1])?;
            let signing_key = hmac::Key::new(hmac::HMAC_SHA256, key.as_slice());
            let tag = hmac::sign(&signing_key, data.as_slice());
            let output = OwnedBuffer::from_slice(tag.as_ref());
            outputs[0] = HandleManager::store(output);
            Ok(0)
        }),
    );

    // HMAC-SHA384
    registry.register(
        "hmac_sha384",
        "Compute HMAC-SHA384. Args: key_handle, data_handle. Returns output_handle (48 bytes).",
        2,
        true,
        ExtCategory::Crypto,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let key = get_buffer(args[0])?;
            let data = get_buffer(args[1])?;
            let signing_key = hmac::Key::new(hmac::HMAC_SHA384, key.as_slice());
            let tag = hmac::sign(&signing_key, data.as_slice());
            let output = OwnedBuffer::from_slice(tag.as_ref());
            outputs[0] = HandleManager::store(output);
            Ok(0)
        }),
    );

    // HMAC-SHA512
    registry.register(
        "hmac_sha512",
        "Compute HMAC-SHA512. Args: key_handle, data_handle. Returns output_handle (64 bytes).",
        2,
        true,
        ExtCategory::Crypto,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let key = get_buffer(args[0])?;
            let data = get_buffer(args[1])?;
            let signing_key = hmac::Key::new(hmac::HMAC_SHA512, key.as_slice());
            let tag = hmac::sign(&signing_key, data.as_slice());
            let output = OwnedBuffer::from_slice(tag.as_ref());
            outputs[0] = HandleManager::store(output);
            Ok(0)
        }),
    );
}

fn register_encryption(registry: &mut ExtensionRegistry) {
    // AES-256-GCM encrypt: key_handle(32), nonce_handle(12), plaintext_handle
    // Returns: ciphertext_handle in outputs[0], tag_handle in outputs[1]
    registry.register(
        "aes256_gcm_encrypt",
        "Encrypt with AES-256-GCM. Args: key(32), nonce(12), plaintext. outputs[0]=ciphertext, outputs[1]=tag(16). Returns 0 or error.",
        3, true, ExtCategory::Crypto,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let key_buf = get_buffer(args[0])?;
            let nonce_buf = get_buffer(args[1])?;
            let plaintext = get_buffer(args[2])?;

            if key_buf.len() != 32 {
                return Err(ExtError::ExtensionError("AES-256 key must be 32 bytes".into()));
            }
            if nonce_buf.len() != 12 {
                return Err(ExtError::ExtensionError("AES-GCM nonce must be 12 bytes".into()));
            }

            let unbound_key = UnboundKey::new(&AES_256_GCM, key_buf.as_slice())
                .map_err(|_| ExtError::ExtensionError("Invalid AES key".into()))?;
            let key = LessSafeKey::new(unbound_key);

            let nonce_arr: [u8; 12] = nonce_buf.as_slice().try_into()
                .map_err(|_| ExtError::ExtensionError("Invalid nonce".into()))?;
            let nonce = Nonce::assume_unique_for_key(nonce_arr);

            let mut in_out = plaintext.as_slice().to_vec();
            let tag = key.seal_in_place_separate_tag(nonce, Aad::empty(), &mut in_out)
                .map_err(|_| ExtError::ExtensionError("Encryption failed".into()))?;

            outputs[0] = HandleManager::store(OwnedBuffer::from_vec(in_out));
            outputs[1] = HandleManager::store(OwnedBuffer::from_slice(tag.as_ref()));
            Ok(0)
        }),
    );

    // AES-256-GCM decrypt: key_handle(32), nonce_handle(12), ciphertext_handle, tag_handle(16)
    // Returns: plaintext_handle in outputs[0], or error code -1 on auth failure
    registry.register(
        "aes256_gcm_decrypt",
        "Decrypt with AES-256-GCM. Args: key(32), nonce(12), ciphertext, tag(16). outputs[0]=plaintext. Returns 0 or -1 on auth failure.",
        4, true, ExtCategory::Crypto,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let key_buf = get_buffer(args[0])?;
            let nonce_buf = get_buffer(args[1])?;
            let ciphertext = get_buffer(args[2])?;
            let tag = get_buffer(args[3])?;

            if key_buf.len() != 32 {
                return Err(ExtError::ExtensionError("AES-256 key must be 32 bytes".into()));
            }
            if nonce_buf.len() != 12 {
                return Err(ExtError::ExtensionError("AES-GCM nonce must be 12 bytes".into()));
            }
            if tag.len() != 16 {
                return Err(ExtError::ExtensionError("AES-GCM tag must be 16 bytes".into()));
            }

            let unbound_key = UnboundKey::new(&AES_256_GCM, key_buf.as_slice())
                .map_err(|_| ExtError::ExtensionError("Invalid AES key".into()))?;
            let key = LessSafeKey::new(unbound_key);

            let nonce_arr: [u8; 12] = nonce_buf.as_slice().try_into()
                .map_err(|_| ExtError::ExtensionError("Invalid nonce".into()))?;
            let nonce = Nonce::assume_unique_for_key(nonce_arr);

            // Append tag to ciphertext for ring's API
            let mut in_out = ciphertext.as_slice().to_vec();
            in_out.extend_from_slice(tag.as_slice());

            match key.open_in_place(nonce, Aad::empty(), &mut in_out) {
                Ok(plaintext) => {
                    outputs[0] = HandleManager::store(OwnedBuffer::from_slice(plaintext));
                    Ok(0)
                }
                Err(_) => {
                    outputs[0] = 0; // No valid handle on failure
                    Ok(-1) // Authentication failure
                }
            }
        }),
    );

    // ChaCha20-Poly1305 encrypt
    registry.register(
        "chacha20_poly1305_encrypt",
        "Encrypt with ChaCha20-Poly1305. Args: key(32), nonce(12), plaintext. outputs[0]=ciphertext, outputs[1]=tag(16). Returns 0.",
        3, true, ExtCategory::Crypto,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let key_buf = get_buffer(args[0])?;
            let nonce_buf = get_buffer(args[1])?;
            let plaintext = get_buffer(args[2])?;

            if key_buf.len() != 32 {
                return Err(ExtError::ExtensionError("ChaCha20 key must be 32 bytes".into()));
            }
            if nonce_buf.len() != 12 {
                return Err(ExtError::ExtensionError("ChaCha20 nonce must be 12 bytes".into()));
            }

            let key: [u8; 32] = key_buf.as_slice().try_into()
                .map_err(|_| ExtError::ExtensionError("Invalid key".into()))?;
            let nonce: [u8; 12] = nonce_buf.as_slice().try_into()
                .map_err(|_| ExtError::ExtensionError("Invalid nonce".into()))?;

            let cipher = ChaCha20Poly1305::new(&key.into());
            let ct_tag = cipher.encrypt(&nonce.into(), plaintext.as_slice())
                .map_err(|_| ExtError::ExtensionError("Encryption failed".into()))?;

            // ct_tag contains ciphertext + 16-byte tag
            let (ct, tag) = ct_tag.split_at(ct_tag.len().saturating_sub(16));
            outputs[0] = HandleManager::store(OwnedBuffer::from_slice(ct));
            outputs[1] = HandleManager::store(OwnedBuffer::from_slice(tag));
            Ok(0)
        }),
    );

    // ChaCha20-Poly1305 decrypt
    registry.register(
        "chacha20_poly1305_decrypt",
        "Decrypt with ChaCha20-Poly1305. Args: key(32), nonce(12), ciphertext, tag(16). outputs[0]=plaintext. Returns 0 or -1.",
        4, true, ExtCategory::Crypto,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let key_buf = get_buffer(args[0])?;
            let nonce_buf = get_buffer(args[1])?;
            let ciphertext = get_buffer(args[2])?;
            let tag = get_buffer(args[3])?;

            if key_buf.len() != 32 {
                return Err(ExtError::ExtensionError("ChaCha20 key must be 32 bytes".into()));
            }
            if nonce_buf.len() != 12 {
                return Err(ExtError::ExtensionError("ChaCha20 nonce must be 12 bytes".into()));
            }
            if tag.len() != 16 {
                return Err(ExtError::ExtensionError("Poly1305 tag must be 16 bytes".into()));
            }

            let key: [u8; 32] = key_buf.as_slice().try_into()
                .map_err(|_| ExtError::ExtensionError("Invalid key".into()))?;
            let nonce: [u8; 12] = nonce_buf.as_slice().try_into()
                .map_err(|_| ExtError::ExtensionError("Invalid nonce".into()))?;

            // Combine ciphertext and tag
            let mut ct_tag = ciphertext.as_slice().to_vec();
            ct_tag.extend_from_slice(tag.as_slice());

            let cipher = ChaCha20Poly1305::new(&key.into());
            match cipher.decrypt(&nonce.into(), ct_tag.as_slice()) {
                Ok(pt) => {
                    outputs[0] = HandleManager::store(OwnedBuffer::from_vec(pt));
                    Ok(0)
                }
                Err(_) => {
                    outputs[0] = 0;
                    Ok(-1)
                }
            }
        }),
    );

    // Constant-time comparison
    registry.register(
        "constant_time_eq",
        "Constant-time byte comparison. Args: buf1_handle, buf2_handle. Returns 1 if equal, 0 if not.",
        2, true, ExtCategory::Crypto,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let buf1 = get_buffer(args[0])?;
            let buf2 = get_buffer(args[1])?;

            // Use subtle crate's constant-time comparison
            let result = if buf1.len() == buf2.len() && buf1.as_slice().ct_eq(buf2.as_slice()).into() {
                1u64
            } else {
                0u64
            };
            outputs[0] = result;
            Ok(result as i64)
        }),
    );
}

fn register_random(registry: &mut ExtensionRegistry) {
    // Secure random: length -> output_handle
    registry.register(
        "secure_random",
        "Generate secure random bytes. Args: length. Returns output_handle.",
        1,
        true,
        ExtCategory::Crypto,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let len = args[0] as usize;
            if len > 1024 * 1024 {
                return Err(ExtError::ExtensionError(
                    "Random length too large (max 1MB)".into(),
                ));
            }

            let mut buf = vec![0u8; len];
            ring_rand::SystemRandom::new()
                .fill(&mut buf)
                .map_err(|_| ExtError::ExtensionError("Random generation failed".into()))?;

            outputs[0] = HandleManager::store(OwnedBuffer::from_vec(buf));
            Ok(0)
        }),
    );
}

fn register_signatures(registry: &mut ExtensionRegistry) {
    // Ed25519 sign: secret_key_handle(32), message_handle -> signature_handle(64)
    registry.register(
        "ed25519_sign",
        "Sign with Ed25519. Args: secret_key(32), message. Returns signature_handle(64).",
        2,
        true,
        ExtCategory::Crypto,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let sk_buf = get_buffer(args[0])?;
            let msg = get_buffer(args[1])?;

            if sk_buf.len() != 32 {
                return Err(ExtError::ExtensionError(
                    "Ed25519 secret key must be 32 bytes".into(),
                ));
            }

            let sk: [u8; 32] = sk_buf
                .as_slice()
                .try_into()
                .map_err(|_| ExtError::ExtensionError("Invalid key".into()))?;
            let signing_key = SigningKey::from_bytes(&sk);
            let sig = signing_key.sign(msg.as_slice());

            outputs[0] = HandleManager::store(OwnedBuffer::from_slice(&sig.to_bytes()));
            Ok(0)
        }),
    );

    // Ed25519 verify: public_key_handle(32), message_handle, signature_handle(64)
    // Returns 1 if valid, 0 if not
    registry.register(
        "ed25519_verify",
        "Verify Ed25519 signature. Args: public_key(32), message, signature(64). Returns 1 if valid, 0 if not.",
        3, true, ExtCategory::Crypto,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let pk_buf = get_buffer(args[0])?;
            let msg = get_buffer(args[1])?;
            let sig_buf = get_buffer(args[2])?;

            if pk_buf.len() != 32 {
                return Err(ExtError::ExtensionError("Ed25519 public key must be 32 bytes".into()));
            }
            if sig_buf.len() != 64 {
                return Err(ExtError::ExtensionError("Ed25519 signature must be 64 bytes".into()));
            }

            let pk: [u8; 32] = pk_buf.as_slice().try_into()
                .map_err(|_| ExtError::ExtensionError("Invalid key".into()))?;
            let sig: [u8; 64] = sig_buf.as_slice().try_into()
                .map_err(|_| ExtError::ExtensionError("Invalid signature".into()))?;

            let vk = match VerifyingKey::from_bytes(&pk) {
                Ok(k) => k,
                Err(_) => {
                    outputs[0] = 0;
                    return Ok(0);
                }
            };
            let signature = Signature::from_bytes(&sig);

            let result = if vk.verify(msg.as_slice(), &signature).is_ok() { 1u64 } else { 0u64 };
            outputs[0] = result;
            Ok(result as i64)
        }),
    );
}

fn register_key_derivation(registry: &mut ExtensionRegistry) {
    // PBKDF2-SHA256: password_handle, salt_handle, iterations, output_length -> output_handle
    registry.register(
        "pbkdf2_sha256",
        "Derive key with PBKDF2-SHA256. Args: password, salt, iterations, output_len. Returns derived_key_handle.",
        4, true, ExtCategory::Crypto,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let password = get_buffer(args[0])?;
            let salt = get_buffer(args[1])?;
            let iterations = args[2] as u32;
            let out_len = args[3] as usize;

            if iterations == 0 {
                return Err(ExtError::ExtensionError("Iterations must be > 0".into()));
            }
            if out_len > 1024 {
                return Err(ExtError::ExtensionError("Output length too large (max 1024)".into()));
            }

            let iterations = std::num::NonZeroU32::new(iterations)
                .ok_or_else(|| ExtError::ExtensionError("Iterations must be > 0".into()))?;

            let mut output = vec![0u8; out_len];
            pbkdf2::derive(
                pbkdf2::PBKDF2_HMAC_SHA256,
                iterations,
                salt.as_slice(),
                password.as_slice(),
                &mut output
            );

            outputs[0] = HandleManager::store(OwnedBuffer::from_vec(output));
            Ok(0)
        }),
    );

    // HKDF-SHA256 extract: salt_handle, ikm_handle -> prk_handle(32)
    registry.register(
        "hkdf_sha256_extract",
        "HKDF-SHA256 extract. Args: salt, ikm. Returns prk_handle(32).",
        2,
        true,
        ExtCategory::Crypto,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let salt = get_buffer(args[0])?;
            let ikm = get_buffer(args[1])?;

            let (prk, _) = Hkdf::<Sha256>::extract(Some(salt.as_slice()), ikm.as_slice());
            outputs[0] = HandleManager::store(OwnedBuffer::from_slice(prk.as_slice()));
            Ok(0)
        }),
    );

    // HKDF-SHA256 expand: prk_handle(32), info_handle, output_length -> okm_handle
    registry.register(
        "hkdf_sha256_expand",
        "HKDF-SHA256 expand. Args: prk(32), info, output_len. Returns okm_handle.",
        3,
        true,
        ExtCategory::Crypto,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let prk = get_buffer(args[0])?;
            let info = get_buffer(args[1])?;
            let out_len = args[2] as usize;

            if out_len > 255 * 32 {
                return Err(ExtError::ExtensionError("Output length too large".into()));
            }

            let hk = Hkdf::<Sha256>::from_prk(prk.as_slice())
                .map_err(|_| ExtError::ExtensionError("Invalid PRK".into()))?;

            let mut output = vec![0u8; out_len];
            hk.expand(info.as_slice(), &mut output)
                .map_err(|_| ExtError::ExtensionError("HKDF expand failed".into()))?;

            outputs[0] = HandleManager::store(OwnedBuffer::from_vec(output));
            Ok(0)
        }),
    );
}

fn register_key_exchange(registry: &mut ExtensionRegistry) {
    // X25519 key exchange: secret_key_handle(32), public_key_handle(32) -> shared_secret_handle(32)
    registry.register(
        "x25519_derive",
        "X25519 key exchange. Args: secret_key(32), public_key(32). Returns shared_secret_handle(32).",
        2, true, ExtCategory::Crypto,
        Arc::new(|args: &[u64], outputs: &mut [u64]| {
            let sk_buf = get_buffer(args[0])?;
            let pk_buf = get_buffer(args[1])?;

            if sk_buf.len() != 32 {
                return Err(ExtError::ExtensionError("X25519 secret key must be 32 bytes".into()));
            }
            if pk_buf.len() != 32 {
                return Err(ExtError::ExtensionError("X25519 public key must be 32 bytes".into()));
            }

            let sk: [u8; 32] = sk_buf.as_slice().try_into()
                .map_err(|_| ExtError::ExtensionError("Invalid key".into()))?;
            let pk: [u8; 32] = pk_buf.as_slice().try_into()
                .map_err(|_| ExtError::ExtensionError("Invalid key".into()))?;

            let secret = StaticSecret::from(sk);
            let public = X25519PublicKey::from(pk);
            let shared = secret.diffie_hellman(&public);

            outputs[0] = HandleManager::store(OwnedBuffer::from_slice(shared.as_bytes()));
            Ok(0)
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_safe() {
        let mut registry = ExtensionRegistry::new();
        register_crypto(&mut registry);

        // Look up extension ID by name
        let sha256_id = registry.get_id("sha256").expect("sha256 not found");

        // Store input in HandleManager
        let input = OwnedBuffer::from_slice(b"hello");
        let input_handle = HandleManager::store(input);

        // Call extension
        let mut outputs = [0u64; 8];
        let result = registry.call(sha256_id, &[input_handle], &mut outputs);
        assert!(result.is_ok());

        // Get output from HandleManager
        let output = HandleManager::get(outputs[0]).unwrap();
        assert_eq!(output.len(), 32);

        // Verify known hash
        let expected =
            hex::decode("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824")
                .unwrap();
        assert_eq!(output.as_slice(), expected.as_slice());

        // Cleanup
        HandleManager::remove(input_handle);
        HandleManager::remove(outputs[0]);
    }

    #[test]
    fn test_aes_gcm_roundtrip_safe() {
        let mut registry = ExtensionRegistry::new();
        register_crypto(&mut registry);

        // Look up extension IDs by name
        let encrypt_id = registry
            .get_id("aes256_gcm_encrypt")
            .expect("encrypt not found");
        let decrypt_id = registry
            .get_id("aes256_gcm_decrypt")
            .expect("decrypt not found");

        // Setup
        let key = HandleManager::store(OwnedBuffer::from_slice(&[0x42u8; 32]));
        let nonce = HandleManager::store(OwnedBuffer::from_slice(&[0x11u8; 12]));
        let plaintext = HandleManager::store(OwnedBuffer::from_slice(b"secret message"));

        // Encrypt
        let mut outputs = [0u64; 8];
        let result = registry.call(encrypt_id, &[key, nonce, plaintext], &mut outputs);
        assert!(result.is_ok());
        let ciphertext = outputs[0];
        let tag = outputs[1];

        // Need fresh nonce handle for decrypt
        let nonce2 = HandleManager::store(OwnedBuffer::from_slice(&[0x11u8; 12]));

        // Decrypt
        let result = registry.call(decrypt_id, &[key, nonce2, ciphertext, tag], &mut outputs);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0); // Success

        let decrypted = HandleManager::get(outputs[0]).unwrap();
        assert_eq!(decrypted.as_slice(), b"secret message");

        // Cleanup
        HandleManager::remove(key);
        HandleManager::remove(nonce);
        HandleManager::remove(nonce2);
        HandleManager::remove(plaintext);
        HandleManager::remove(ciphertext);
        HandleManager::remove(tag);
        HandleManager::remove(outputs[0]);
    }

    #[test]
    fn test_invalid_handle_returns_error() {
        let mut registry = ExtensionRegistry::new();
        register_crypto(&mut registry);

        let mut outputs = [0u64; 8];
        let result = registry.call(1, &[99999], &mut outputs); // Invalid handle
        assert!(result.is_err());
    }
}
