# Safe Wrappers System

Production-grade safe wrappers providing memory-safe access to native capabilities.

## Vision

A production-grade AI programming system where:
- **Small local model** iterates 1000x faster than LLM APIs, at zero marginal cost
- **Compiled IR** executes at native speed
- **Safe wrappers** provide memory-safe access to native capabilities
- **RAG system** discovers any capability by intent
- **Guaranteed correct output** - no crashes, no memory corruption

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         NEURLANG PRODUCTION STACK                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌──────────────────┐    ┌──────────────────┐    ┌──────────────────┐       │
│  │  SMALL MODEL     │    │  COMPILED IR     │    │  SAFE WRAPPERS   │       │
│  │  (5ms inference) │ -> │  (native speed)  │ -> │  (memory safe)   │       │
│  │  1000x iteration │    │  copy-and-patch  │    │  Rust crates     │       │
│  └──────────────────┘    └──────────────────┘    └──────────────────┘       │
│           │                       │                       │                  │
│           └───────────────────────┴───────────────────────┘                  │
│                                   │                                          │
│                          ┌────────▼────────┐                                 │
│                          │   RAG SYSTEM    │                                 │
│                          │ Intent → ID     │                                 │
│                          └─────────────────┘                                 │
│                                                                              │
│  NO raw .so access. NO libffi. NO unsafe C calls.                           │
│  All capabilities via verified, memory-safe Rust wrappers.                  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Design Principles

### 1. Correct Output
Every operation produces correct results or returns error. No crashes. No undefined behavior. No memory corruption.

### 2. Verified Implementation
All wrappers are Rust code we control and test. No arbitrary C code running in our process.

### 3. Guaranteed to Work
Simple APIs that are hard to misuse. `@"compress buffer"` just works.

### 4. Simple Mental Model
Model learns one way to do things (the safe way). No "raw vs wrapper" decisions.

## What About Missing Capabilities?

```
Need something not in neurlang?
         ↓
Ask a larger LLM (Claude/GPT) to write a Rust wrapper
         ↓
Add wrapper to neurlang (or contribute upstream)
         ↓
Now the small model can use it safely
```

This keeps the system clean while remaining extensible.

## Module Overview

| Module | Description | Rust Crate |
|--------|-------------|------------|
| [buffer](buffer.md) | Memory-safe buffer type and handle management | std |
| [compression](compression.md) | Compress/decompress data | flate2, lz4, zstd |
| [encoding](encoding.md) | Base64, hex, URL encoding | base64, hex |
| [fs](fs.md) | File system operations | std::fs |
| [datetime](datetime.md) | Date and time operations | chrono |
| [regex](regex.md) | Regular expressions | regex |
| [x509](x509.md) | X509 certificate handling | rcgen, x509-parser |
| [tls](tls.md) | TLS connections | rustls |
| [synonyms](synonyms.md) | RAG synonym dictionary | - |

## What The Model Sees

```asm
; Simple, consistent API - all operations return results or errors

; Compression
ext.call r0, @"compress", input_buffer
ext.call r1, @"decompress", compressed_buffer

; Hashing
ext.call r0, @"hash sha256", data
ext.call r1, @"hash sha512", data

; Encryption
ext.call r0, @"encrypt aes256", plaintext, key
ext.call r1, @"decrypt aes256", ciphertext, key

; HTTP
ext.call r0, @"http get", url
ext.call r1, @"http post", url, body

; JSON
ext.call r0, @"json parse", json_string
ext.call r1, @"json get", json_obj, "field.path"
ext.call r2, @"json stringify", data

; File System
ext.call r0, @"read file", path
ext.call r1, @"write file", path, contents
ext.call r2, @"list dir", path

; X509 Certificates
ext.call r0, @"x509 create self-signed", subject, key
ext.call r1, @"x509 parse", cert_pem
ext.call r2, @"x509 verify", cert, ca_cert

; TLS
ext.call r0, @"tls connect", host, port
ext.call r1, @"tls send", conn, data
ext.call r2, @"tls recv", conn

; All operations:
; - Return OwnedBuffer or error code
; - Handle allocation internally
; - Are memory safe
; - Are tested and verified
```

## Wrapper Categories Summary

| Category | Operations | Coverage |
|----------|------------|----------|
| Compression | ~10 | zlib, gzip, lz4, zstd |
| Cryptography | ~20 | SHA, HMAC, AES, ChaCha20, Ed25519 |
| HTTP | ~10 | GET, POST, PUT, DELETE, headers |
| JSON | ~15 | parse, stringify, get, set, arrays |
| File System | ~15 | read, write, list, exists, copy |
| Strings | ~10 | split, join, replace, trim |
| Regex | ~5 | match, find, replace, split |
| Date/Time | ~10 | parse, format, arithmetic |
| Encoding | ~10 | base64, hex, URL |
| X509 | ~12 | create, parse, verify certs |
| TLS | ~10 | connect, send, recv |

**TOTAL: ~120 wrapper operations covering ~95% of common use cases**

## File Structure

```
src/wrappers/
├── mod.rs              # WrapperRegistry and traits
├── buffer.rs           # OwnedBuffer type and HandleManager
├── synonyms.rs         # Synonym dictionary for RAG
│
├── compression.rs      # flate2, lz4, zstd
├── encoding.rs         # base64, hex, url
├── fs.rs               # std::fs operations
├── datetime.rs         # chrono
├── regex.rs            # regex crate
├── x509.rs             # rcgen, x509-parser
└── tls.rs              # rustls
```

## What We Removed

- ❌ libffi
- ❌ Raw .so loading
- ❌ Metadata files (.mod, .sig)
- ❌ ELF symbol scanning
- ❌ Unsafe C calls

## What We Keep

- ✓ Safe Rust wrappers
- ✓ Simple RAG for discovery
- ✓ Memory safety
- ✓ Guaranteed correctness
- ✓ Clean, simple design

## Next Steps

See individual module documentation for detailed API references:

1. [Buffer](buffer.md) - Start here to understand the core data type
2. [Compression](compression.md) - Most commonly used wrapper
3. [Encoding](encoding.md) - Base64, hex for data interchange
4. [TLS](tls.md) - Secure network connections
5. [X509](x509.md) - Certificate management
