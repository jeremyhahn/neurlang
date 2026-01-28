//! Synonym Dictionary for RAG-based Wrapper Discovery
//!
//! Maps common terms to their synonyms, enabling the RAG system to match
//! user intents like `@"shrink data"` to wrapper functions like `compress`.

/// The complete synonym dictionary
///
/// Format: (primary_word, [synonyms...])
/// The primary word maps to all synonyms, enabling discovery via any term.
pub const SYNONYMS: &[(&str, &[&str])] = &[
    // ==========================================================================
    // Compression
    // ==========================================================================
    (
        "compress",
        &["shrink", "deflate", "zip", "pack", "reduce", "squeeze"],
    ),
    (
        "decompress",
        &[
            "expand",
            "inflate",
            "unzip",
            "unpack",
            "extract",
            "uncompress",
        ],
    ),
    ("gzip", &["gz", "gunzip"]),
    ("zstd", &["zstandard"]),
    ("lz4", &["fast compress"]),
    // ==========================================================================
    // Cryptography
    // ==========================================================================
    (
        "encrypt",
        &["cipher", "encode", "secure", "protect", "encipher"],
    ),
    ("decrypt", &["decipher", "decode", "unsecure", "unprotect"]),
    ("hash", &["digest", "checksum", "fingerprint"]),
    ("sign", &["signature", "authenticate", "signing"]),
    ("verify", &["validate", "check", "confirm", "verification"]),
    (
        "random",
        &["rand", "rng", "generate random", "secure random"],
    ),
    ("hmac", &["message authentication", "mac"]),
    ("aes", &["aes256", "aes-gcm", "symmetric"]),
    ("chacha", &["chacha20", "poly1305"]),
    ("pbkdf2", &["key derivation", "password hash", "derive key"]),
    ("argon2", &["password hash", "argon"]),
    ("ed25519", &["eddsa", "signature"]),
    ("x25519", &["ecdh", "key exchange", "diffie-hellman"]),
    // ==========================================================================
    // I/O Operations
    // ==========================================================================
    ("read", &["load", "get", "fetch", "open", "input"]),
    ("write", &["save", "store", "put", "output", "persist"]),
    ("file", &["document", "path"]),
    ("append", &["add to", "extend file"]),
    ("delete", &["remove", "unlink", "rm", "erase"]),
    ("copy", &["cp", "duplicate"]),
    ("move", &["mv", "rename"]),
    ("mkdir", &["create directory", "make dir"]),
    ("rmdir", &["remove directory", "delete dir"]),
    ("list", &["ls", "dir", "readdir", "directory contents"]),
    ("exists", &["exist", "is present", "check"]),
    // ==========================================================================
    // Data Operations
    // ==========================================================================
    (
        "parse",
        &["decode", "interpret", "process", "deserialize", "from"],
    ),
    (
        "stringify",
        &["serialize", "encode", "format", "to string", "to"],
    ),
    ("convert", &["transform", "cast"]),
    // ==========================================================================
    // JSON
    // ==========================================================================
    ("json", &["object", "array"]),
    // ==========================================================================
    // Network
    // ==========================================================================
    ("http", &["web", "request", "fetch", "api"]),
    ("get", &["fetch", "retrieve", "download"]),
    ("post", &["send", "submit", "upload"]),
    ("put", &["update"]),
    ("connect", &["open", "establish"]),
    ("close", &["disconnect", "terminate", "end", "shutdown"]),
    ("send", &["write", "transmit"]),
    ("recv", &["receive", "read"]),
    // ==========================================================================
    // Encoding
    // ==========================================================================
    ("base64", &["b64"]),
    ("hex", &["hexadecimal"]),
    ("url", &["uri", "percent"]),
    ("encode", &["to"]),
    ("decode", &["from"]),
    // ==========================================================================
    // X509 / Certificates
    // ==========================================================================
    ("x509", &["certificate", "cert", "ssl", "pki"]),
    ("csr", &["certificate request", "signing request"]),
    ("ca", &["certificate authority", "root", "issuer"]),
    ("keypair", &["key", "private key", "public key"]),
    ("rsa", &["rsa2048", "rsa4096"]),
    ("ec", &["ecdsa", "ecdh", "p256", "p384", "elliptic curve"]),
    ("self-signed", &["selfsigned"]),
    ("expiry", &["expiration", "valid until", "not after"]),
    // ==========================================================================
    // TLS
    // ==========================================================================
    ("tls", &["ssl", "secure", "https", "encrypted"]),
    ("handshake", &["negotiate", "establish"]),
    ("alpn", &["protocol negotiation"]),
    ("sni", &["server name"]),
    ("mtls", &["mutual tls", "client cert", "two-way tls"]),
    // ==========================================================================
    // Date/Time
    // ==========================================================================
    ("datetime", &["date", "time", "timestamp"]),
    ("now", &["current", "today"]),
    ("utc", &["gmt", "zulu"]),
    ("local", &["localtime", "local time"]),
    ("format", &["strftime", "to string"]),
    ("add", &["plus"]),
    ("subtract", &["minus"]),
    ("diff", &["difference", "delta", "between"]),
    ("year", &["years"]),
    ("month", &["months"]),
    ("day", &["days"]),
    ("hour", &["hours"]),
    ("minute", &["minutes"]),
    ("second", &["seconds"]),
    ("weekday", &["day of week"]),
    // ==========================================================================
    // Regex
    // ==========================================================================
    ("regex", &["regexp", "regular expression", "pattern"]),
    ("match", &["test", "check", "matches"]),
    ("find", &["search", "locate"]),
    ("replace", &["substitute", "sub", "gsub"]),
    ("split", &["tokenize"]),
    ("captures", &["groups", "capture groups"]),
    // ==========================================================================
    // Strings
    // ==========================================================================
    ("string", &["str", "text"]),
    ("concat", &["concatenate", "join", "append"]),
    ("substr", &["substring", "slice"]),
    ("trim", &["strip"]),
    ("upper", &["uppercase", "toupper"]),
    ("lower", &["lowercase", "tolower"]),
    ("starts", &["startswith", "prefix"]),
    ("ends", &["endswith", "suffix"]),
    ("contains", &["includes", "has"]),
    ("length", &["len", "size"]),
];

/// Expand a list of keywords with their synonyms
///
/// Given `["compress", "file"]`, returns:
/// `["compress", "shrink", "deflate", ..., "file", "document", "path"]`
pub fn expand_synonyms(keywords: &[&str]) -> Vec<String> {
    let mut expanded = Vec::new();

    for kw in keywords {
        let kw_lower = kw.to_lowercase();

        // Check each synonym entry
        for (primary, syns) in SYNONYMS {
            // If keyword matches primary, add all synonyms
            if *primary == kw_lower {
                expanded.extend(syns.iter().map(|s| s.to_string()));
            }
            // If keyword matches a synonym, add the primary and other synonyms
            else if syns.contains(&kw_lower.as_str()) {
                expanded.push(primary.to_string());
                expanded.extend(
                    syns.iter()
                        .filter(|s| **s != kw_lower)
                        .map(|s| s.to_string()),
                );
            }
        }
    }

    expanded
}

/// Get synonyms for a single word
///
/// Returns the synonyms if the word is a primary word in the dictionary,
/// or None if not found.
pub fn get_synonyms(word: &str) -> Option<&'static [&'static str]> {
    let word_lower = word.to_lowercase();

    for (primary, syns) in SYNONYMS {
        if *primary == word_lower {
            return Some(syns);
        }
    }

    None
}

/// Check if two words are synonymous
///
/// Returns true if both words map to the same primary word.
pub fn are_synonyms(word1: &str, word2: &str) -> bool {
    let w1 = word1.to_lowercase();
    let w2 = word2.to_lowercase();

    if w1 == w2 {
        return true;
    }

    for (primary, syns) in SYNONYMS {
        let w1_matches = *primary == w1 || syns.contains(&w1.as_str());
        let w2_matches = *primary == w2 || syns.contains(&w2.as_str());

        if w1_matches && w2_matches {
            return true;
        }
    }

    false
}

/// Find the primary word for a given term
///
/// Returns the primary word if found, or the input word if not in dictionary.
pub fn get_primary(word: &str) -> &str {
    let word_lower = word.to_lowercase();

    for (primary, syns) in SYNONYMS {
        if *primary == word_lower || syns.contains(&word_lower.as_str()) {
            return primary;
        }
    }

    word
}

/// Get all words that relate to a given word (including itself)
///
/// Returns primary + all synonyms if word is in dictionary.
pub fn get_related(word: &str) -> Vec<&'static str> {
    let word_lower = word.to_lowercase();

    for (primary, syns) in SYNONYMS {
        if *primary == word_lower || syns.contains(&word_lower.as_str()) {
            let mut related = vec![*primary];
            related.extend(syns.iter());
            return related;
        }
    }

    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_synonyms() {
        let expanded = expand_synonyms(&["compress"]);
        assert!(expanded.contains(&"shrink".to_string()));
        assert!(expanded.contains(&"deflate".to_string()));
        assert!(expanded.contains(&"zip".to_string()));
    }

    #[test]
    fn test_expand_multiple() {
        let expanded = expand_synonyms(&["compress", "file"]);
        assert!(expanded.contains(&"shrink".to_string()));
        assert!(expanded.contains(&"document".to_string()));
    }

    #[test]
    fn test_get_synonyms() {
        let syns = get_synonyms("compress").unwrap();
        assert!(syns.contains(&"shrink"));
        assert!(syns.contains(&"deflate"));
    }

    #[test]
    fn test_get_synonyms_not_found() {
        let syns = get_synonyms("xyzzy");
        assert!(syns.is_none());
    }

    #[test]
    fn test_are_synonyms() {
        assert!(are_synonyms("compress", "shrink"));
        assert!(are_synonyms("shrink", "compress"));
        assert!(are_synonyms("shrink", "deflate"));
        assert!(!are_synonyms("compress", "encrypt"));
    }

    #[test]
    fn test_get_primary() {
        assert_eq!(get_primary("shrink"), "compress");
        assert_eq!(get_primary("compress"), "compress");
        assert_eq!(get_primary("xyzzy"), "xyzzy");
    }

    #[test]
    fn test_get_related() {
        let related = get_related("shrink");
        assert!(related.contains(&"compress"));
        assert!(related.contains(&"shrink"));
        assert!(related.contains(&"deflate"));
    }

    #[test]
    fn test_crypto_synonyms() {
        assert!(are_synonyms("encrypt", "cipher"));
        assert!(are_synonyms("hash", "digest"));
        assert!(are_synonyms("sign", "signature"));
    }

    #[test]
    fn test_tls_synonyms() {
        assert!(are_synonyms("tls", "ssl"));
        assert!(are_synonyms("tls", "https"));
        assert!(are_synonyms("x509", "certificate"));
    }
}
