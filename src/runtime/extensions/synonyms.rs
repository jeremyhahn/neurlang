//! Extension name synonyms for RAG resolution
//!
//! Maps common intent phrases to canonical extension names.

use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    /// Synonyms for extension names
    pub static ref SYNONYMS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();

        // Crypto
        m.insert("hash sha256", "sha256");
        m.insert("sha256 hash", "sha256");
        m.insert("compute sha256", "sha256");
        m.insert("hash sha1", "sha1");
        m.insert("sha1 hash", "sha1");
        m.insert("hash sha384", "sha384");
        m.insert("hash sha512", "sha512");
        m.insert("hash blake2b", "blake2b_512");
        m.insert("hash blake2s", "blake2s_256");
        m.insert("hmac sha256", "hmac_sha256");
        m.insert("hmac sha384", "hmac_sha384");
        m.insert("hmac sha512", "hmac_sha512");
        m.insert("encrypt aes", "aes256_gcm_encrypt");
        m.insert("decrypt aes", "aes256_gcm_decrypt");
        m.insert("encrypt chacha", "chacha20_poly1305_encrypt");
        m.insert("decrypt chacha", "chacha20_poly1305_decrypt");
        m.insert("sign ed25519", "ed25519_sign");
        m.insert("verify ed25519", "ed25519_verify");
        m.insert("derive key", "x25519_derive");
        m.insert("random bytes", "secure_random");
        m.insert("generate random", "secure_random");
        m.insert("pbkdf2", "pbkdf2_sha256");
        m.insert("constant time compare", "constant_time_eq");

        // Collections
        m.insert("new vec", "vec_new");
        m.insert("create vec", "vec_new");
        m.insert("new vector", "vec_new");
        m.insert("push to vec", "vec_push");
        m.insert("pop from vec", "vec_pop");
        m.insert("new hashmap", "hashmap_new");
        m.insert("create hashmap", "hashmap_new");
        m.insert("new map", "hashmap_new");

        // Strings
        m.insert("new string", "string_new");
        m.insert("create string", "string_new");
        m.insert("string length", "string_len");
        m.insert("concatenate", "string_concat");
        m.insert("concat strings", "string_concat");
        m.insert("substring", "string_substr");
        m.insert("find in string", "string_find");
        m.insert("replace in string", "string_replace");
        m.insert("split string", "string_split");
        m.insert("trim string", "string_trim");
        m.insert("uppercase", "string_to_upper");
        m.insert("to uppercase", "string_to_upper");
        m.insert("lowercase", "string_to_lower");
        m.insert("to lowercase", "string_to_lower");
        m.insert("parse integer", "string_parse_int");
        m.insert("string to int", "string_parse_int");
        m.insert("int to string", "string_from_int");
        m.insert("integer to string", "string_from_int");

        // JSON
        m.insert("parse json", "json_parse");
        m.insert("json to string", "json_stringify");
        m.insert("stringify json", "json_stringify");
        m.insert("json get", "json_get");
        m.insert("json set", "json_set");
        m.insert("new json object", "json_new_object");
        m.insert("create json object", "json_new_object");
        m.insert("new json array", "json_new_array");

        // HTTP
        m.insert("http get request", "http_get");
        m.insert("get request", "http_get");
        m.insert("http post request", "http_post");
        m.insert("post request", "http_post");
        m.insert("http put request", "http_put");
        m.insert("put request", "http_put");
        m.insert("http delete request", "http_delete");
        m.insert("delete request", "http_delete");

        // Compression
        m.insert("compress zlib", "zlib_compress");
        m.insert("decompress zlib", "zlib_decompress");
        m.insert("compress gzip", "gzip_compress");
        m.insert("decompress gzip", "gzip_decompress");
        m.insert("compress lz4", "lz4_compress");
        m.insert("decompress lz4", "lz4_decompress");
        m.insert("compress zstd", "zstd_compress");
        m.insert("decompress zstd", "zstd_decompress");

        // Encoding
        m.insert("encode base64", "base64_encode");
        m.insert("decode base64", "base64_decode");
        m.insert("to base64", "base64_encode");
        m.insert("from base64", "base64_decode");
        m.insert("encode hex", "hex_encode");
        m.insert("decode hex", "hex_decode");
        m.insert("to hex", "hex_encode");
        m.insert("from hex", "hex_decode");
        m.insert("url encode", "url_encode");
        m.insert("url decode", "url_decode");
        m.insert("percent encode", "url_encode");

        // DateTime
        m.insert("current time", "datetime_now");
        m.insert("now", "datetime_now");
        m.insert("timestamp", "datetime_now");
        m.insert("parse date", "datetime_parse");
        m.insert("format date", "datetime_format");

        // Regex
        m.insert("regex match", "regex_match");
        m.insert("match pattern", "regex_match");
        m.insert("regex find", "regex_find");
        m.insert("regex replace", "regex_replace");
        m.insert("regex split", "regex_split");

        // FileSystem
        m.insert("read file", "fs_read");
        m.insert("write file", "fs_write");
        m.insert("file exists", "fs_exists");
        m.insert("delete file", "fs_delete");
        m.insert("make directory", "fs_mkdir");
        m.insert("list directory", "fs_list_dir");

        // TLS
        m.insert("tls connect", "tls_connect");
        m.insert("ssl connect", "tls_connect");
        m.insert("secure connect", "tls_connect");

        m
    };
}

/// Expand a synonym to its canonical name
pub fn expand_synonyms(name: &str) -> &str {
    let lower = name.to_lowercase();
    SYNONYMS.get(lower.as_str()).copied().unwrap_or(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_synonyms() {
        assert_eq!(expand_synonyms("hash sha256"), "sha256");
        assert_eq!(expand_synonyms("new vec"), "vec_new");
        assert_eq!(expand_synonyms("parse json"), "json_parse");
        assert_eq!(expand_synonyms("unknown"), "unknown");
    }
}
