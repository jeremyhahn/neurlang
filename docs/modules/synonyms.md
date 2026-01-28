# Synonyms Module

RAG synonym dictionary for wrapper discovery.

## Overview

The synonyms module provides a dictionary of synonyms that enables the RAG system to match user intents to wrapper functions. When a user asks for `@"shrink data"`, the synonym dictionary maps "shrink" to "compress", allowing the system to find the correct wrapper.

## How It Works

```
User intent: @"shrink file size"
         ↓
Tokenize: ["shrink", "file", "size"]
         ↓
Expand synonyms:
  - shrink → compress, deflate, zip, pack
  - file → document, path
  - size → length, bytes
         ↓
Match against wrapper keywords
         ↓
Best match: compress (score: 0.85)
```

## Synonym Dictionary

### Compression

| Primary | Synonyms |
|---------|----------|
| compress | shrink, deflate, zip, pack, reduce, squeeze |
| decompress | expand, inflate, unzip, unpack, extract, uncompress |
| gzip | gz, gunzip |
| zstd | zstandard |

### Cryptography

| Primary | Synonyms |
|---------|----------|
| encrypt | cipher, encode, secure, protect, encipher |
| decrypt | decipher, decode, unsecure, deciper |
| hash | digest, checksum, fingerprint |
| sign | signature, authenticate |
| verify | validate, check, confirm |
| random | rand, rng, generate random |

### I/O Operations

| Primary | Synonyms |
|---------|----------|
| read | load, get, fetch, open, input |
| write | save, store, put, output, persist |
| file | document, path |
| append | add to |
| delete | remove, unlink, rm, erase |

### Data Operations

| Primary | Synonyms |
|---------|----------|
| parse | decode, interpret, process, deserialize |
| stringify | serialize, encode, format, to string |
| convert | transform, cast |

### Network

| Primary | Synonyms |
|---------|----------|
| http | web, request, fetch, api |
| get | fetch, retrieve, download |
| post | send, submit, upload |
| connect | open, establish |
| close | disconnect, terminate, end |

### Encoding

| Primary | Synonyms |
|---------|----------|
| base64 | b64 |
| hex | hexadecimal |
| url | uri, percent |
| encode | to |
| decode | from |

### X509/Certificates

| Primary | Synonyms |
|---------|----------|
| x509 | certificate, cert, ssl, pki |
| csr | certificate request, signing request |
| ca | certificate authority, root, issuer |
| keypair | key, private key, public key |
| rsa | |
| ec | ecdsa, ecdh, p256, p384 |

### TLS

| Primary | Synonyms |
|---------|----------|
| tls | ssl, secure, https, encrypted |
| handshake | negotiate, establish |
| verify | validate, authenticate |

### Date/Time

| Primary | Synonyms |
|---------|----------|
| datetime | date, time, timestamp |
| now | current, today |
| parse | strptime, from string |
| format | strftime, to string |
| utc | gmt, zulu |
| local | localtime |

### Regex

| Primary | Synonyms |
|---------|----------|
| regex | regexp, regular expression, pattern |
| match | test, check |
| find | search, locate |
| replace | substitute, sub, gsub |
| split | tokenize |

## API Reference

### Expanding Synonyms

```rust
/// Expand a list of keywords with their synonyms
pub fn expand_synonyms(keywords: &[&str]) -> Vec<String>;

/// Get synonyms for a single word
pub fn get_synonyms(word: &str) -> Option<&'static [&'static str]>;

/// Check if two words are synonymous
pub fn are_synonyms(word1: &str, word2: &str) -> bool;
```

### Synonym Data

```rust
/// The full synonym dictionary
pub const SYNONYMS: &[(&str, &[&str])] = &[
    ("compress", &["shrink", "deflate", "zip", "pack", "reduce"]),
    ("decompress", &["expand", "inflate", "unzip", "unpack", "extract"]),
    // ... etc
];
```

## Usage Examples

### Expanding Keywords

```rust
use neurlang::wrappers::synonyms;

let keywords = &["compress", "file"];
let expanded = synonyms::expand_synonyms(keywords);

// expanded now contains:
// ["compress", "shrink", "deflate", "zip", "pack", "reduce",
//  "file", "document", "path"]
```

### Checking Synonyms

```rust
// These are synonymous
assert!(synonyms::are_synonyms("compress", "shrink"));
assert!(synonyms::are_synonyms("zip", "compress"));

// These are not
assert!(!synonyms::are_synonyms("compress", "encrypt"));
```

### In Wrapper Registration

```rust
impl WrapperRegistry {
    pub fn register_wrapper<F>(
        &mut self,
        name: &str,
        description: &str,
        keywords: &[&str],
        func: F,
    ) {
        // Expand keywords with synonyms
        let mut all_keywords: Vec<String> = keywords.iter()
            .map(|s| s.to_lowercase())
            .collect();
        all_keywords.extend(expand_synonyms(keywords));

        // Index for RAG
        for kw in &all_keywords {
            self.keywords.entry(kw.clone())
                .or_default()
                .push(id);
        }
        // ...
    }
}
```

## Scoring Algorithm

The RAG search uses synonym expansion in its scoring:

```rust
pub fn search(&self, query: &str) -> Option<u64> {
    let words: Vec<&str> = query.to_lowercase()
        .split_whitespace()
        .collect();

    let mut scores: HashMap<u64, f32> = HashMap::new();

    for word in &words {
        // Direct keyword match: +1.0
        if let Some(ids) = self.keywords.get(*word) {
            for &id in ids {
                *scores.entry(id).or_default() += 1.0;
            }
        }

        // Synonym match (already expanded in keywords): included above

        // Partial match: +0.5
        for (kw, ids) in &self.keywords {
            if kw.contains(*word) || word.contains(kw.as_str()) {
                for &id in ids {
                    *scores.entry(id).or_default() += 0.5;
                }
            }
        }
    }

    // Return best match above threshold
    scores.into_iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .filter(|(_, score)| *score >= 0.5)
        .map(|(id, _)| id)
}
```

## Adding New Synonyms

To add new synonyms, update the `SYNONYMS` constant:

```rust
pub const SYNONYMS: &[(&str, &[&str])] = &[
    // Existing entries...

    // New domain-specific synonyms
    ("blockchain", &["chain", "ledger", "distributed"]),
    ("transaction", &["tx", "transfer"]),
    ("wallet", &["account", "address"]),
];
```

## Design Decisions

### Why Static Synonyms?

1. **Predictable** - Same query always maps to same result
2. **Fast** - No runtime computation or ML inference
3. **Auditable** - Can review and test all mappings
4. **Controllable** - Can ensure safe operations are discovered

### Why Not Embeddings?

While semantic embeddings could provide more flexible matching, we chose static synonyms because:

1. **Smaller footprint** - No embedding model needed
2. **Faster** - Simple string matching vs vector similarity
3. **Deterministic** - Results are predictable
4. **Domain-specific** - Can tune for programming terms

### Synonym vs Keyword

- **Keywords** are registered with each wrapper
- **Synonyms** expand those keywords automatically
- A wrapper for "compress" doesn't need to list "shrink" explicitly

## Completeness

The synonym dictionary aims to cover:

| Category | Coverage |
|----------|----------|
| Compression | High |
| Cryptography | High |
| File I/O | High |
| Encoding | High |
| Networking | Medium |
| Date/Time | Medium |
| Regex | Medium |
| Domain-specific | Low (extensible) |

## Testing Synonyms

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_synonyms() {
        let syns = get_synonyms("compress").unwrap();
        assert!(syns.contains(&"shrink"));
        assert!(syns.contains(&"deflate"));
    }

    #[test]
    fn test_bidirectional() {
        // If A is synonym of B, B should map to same wrapper
        assert!(are_synonyms("compress", "shrink"));
        // Note: the reverse isn't automatically true in the data structure,
        // but both should match the same wrapper
    }

    #[test]
    fn test_expansion() {
        let expanded = expand_synonyms(&["compress"]);
        assert!(expanded.len() > 1);
        assert!(expanded.contains(&"shrink".to_string()));
    }
}
```

## See Also

- [Wrapper Registry](README.md) - How wrappers are registered and discovered
- [RAG Extensions](../architecture/rag-extensions.md) - Overall RAG architecture
