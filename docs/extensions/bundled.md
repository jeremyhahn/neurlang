# Bundled Extensions

Neurlang ships with a set of bundled extensions that provide common functionality. These extensions are resolved via RAG at assembly time and require no model training.

## Overview

Bundled extensions occupy ID range 200-499 and are automatically available in every Neurlang installation.

| Extension | ID Range | Description |
|-----------|----------|-------------|
| json | 200-219 | JSON parsing and building |
| http | 220-239 | HTTP client operations |
| crypto | 1-99 | Cryptographic operations (hardcoded) |
| fs | 240-259 | File system operations |
| sqlite | 260-279 | SQLite database |
| regex | 280-299 | Regular expressions |
| datetime | 300-319 | Date and time operations |
| env | 320-329 | Environment variables |
| uuid | 330-339 | UUID generation |
| base64 | 340-349 | Base64 encoding/decoding |
| url | 350-359 | URL parsing and encoding |
| log | 360-369 | Logging operations |

---

## JSON Extension (200-219)

Parse, build, and manipulate JSON data.

### Operations

| ID | Name | Intent | Description |
|----|------|--------|-------------|
| 200 | json_parse | "parse JSON string" | Parse string to JSON handle |
| 201 | json_stringify | "convert to JSON string" | Convert JSON handle to string |
| 202 | json_get | "get JSON field" | Get value at key/index |
| 203 | json_set | "set JSON field" | Set value at key/index |
| 204 | json_get_type | "get JSON type" | Get type (null/bool/number/string/array/object) |
| 205 | json_array_len | "get JSON array length" | Get array length |
| 206 | json_array_get | "get JSON array element" | Get element at index |
| 207 | json_array_push | "add to JSON array" | Append element to array |
| 208 | json_object_keys | "get JSON object keys" | Get all keys as Vec<String> |
| 209 | json_free | "free JSON handle" | Deallocate JSON handle |
| 210 | json_new_object | "create JSON object" | Create empty object |
| 211 | json_new_array | "create JSON array" | Create empty array |

### JSON Types

| Type | Value |
|------|-------|
| NULL | 0 |
| BOOL | 1 |
| NUMBER | 2 |
| STRING | 3 |
| ARRAY | 4 |
| OBJECT | 5 |

### Example

```asm
; Parse JSON and extract a field
mov r1, input_string        ; r1 = '{"name": "Alice", "age": 30}'
ext.call r0, @"parse JSON string", r1, r0  ; r0 = json handle

mov r2, key_name            ; r2 = "name"
ext.call r3, @"get JSON field", r0, r2     ; r3 = "Alice"

ext.call r0, @"free JSON handle", r0, r0   ; cleanup
```

---

## HTTP Extension (220-239)

Make HTTP requests to web servers.

### Operations

| ID | Name | Intent | Description |
|----|------|--------|-------------|
| 220 | http_get | "make HTTP GET request" | GET request, returns response |
| 221 | http_post | "make HTTP POST request" | POST with body |
| 222 | http_put | "make HTTP PUT request" | PUT with body |
| 223 | http_delete | "make HTTP DELETE request" | DELETE request |
| 224 | http_head | "make HTTP HEAD request" | HEAD request (headers only) |
| 225 | http_patch | "make HTTP PATCH request" | PATCH with body |
| 226 | http_response_status | "get HTTP status code" | Get status code from response |
| 227 | http_response_body | "get HTTP response body" | Get body as string |
| 228 | http_response_header | "get HTTP header" | Get specific header value |
| 229 | http_set_header | "set HTTP header" | Set header for request |
| 230 | http_set_timeout | "set HTTP timeout" | Set request timeout |
| 231 | http_free | "free HTTP response" | Deallocate response handle |

### Example

```asm
; Make a GET request and get the response body
mov r1, url_string          ; r1 = "https://api.example.com/users"
ext.call r0, @"make HTTP GET request", r1, r0  ; r0 = response handle

ext.call r2, @"get HTTP status code", r0, r0   ; r2 = 200
ext.call r3, @"get HTTP response body", r0, r0 ; r3 = response body

ext.call r0, @"free HTTP response", r0, r0     ; cleanup
```

---

## Crypto Extension (1-99)

Cryptographic operations implemented with audited libraries (ring, sha1, sha2, sha3, blake2, ed25519-dalek).

**Note**: Crypto extensions use hardcoded IDs for security-critical code.

### Hash Functions

| ID | Name | Output Size | Description |
|----|------|-------------|-------------|
| 38 | sha1 | 20 bytes | SHA-1 (legacy, WebSocket handshake only) |
| 1 | sha256 | 32 bytes | SHA-256 (general purpose) |
| 15 | sha384 | 48 bytes | SHA-384 (TLS, high security) |
| 16 | sha512 | 64 bytes | SHA-512 (maximum security) |
| 17 | sha3_256 | 32 bytes | SHA3-256 (post-quantum resistant) |
| 18 | sha3_512 | 64 bytes | SHA3-512 (post-quantum resistant) |
| 19 | blake2b_512 | 64 bytes | BLAKE2b (fast, secure) |
| 20 | blake2s_256 | 32 bytes | BLAKE2s (optimized for 32-bit) |

**WARNING**: SHA-1 (ID 38) is cryptographically broken. Use ONLY for legacy protocol compatibility (e.g., WebSocket Sec-WebSocket-Accept). Never use for security.

### HMAC Functions

| ID | Name | Output Size | Description |
|----|------|-------------|-------------|
| 2 | hmac_sha256 | 32 bytes | HMAC-SHA256 |
| 21 | hmac_sha384 | 48 bytes | HMAC-SHA384 |
| 22 | hmac_sha512 | 64 bytes | HMAC-SHA512 |

### Encryption

| ID | Name | Description |
|----|------|-------------|
| 3 | aes256_gcm_encrypt | AES-256-GCM encryption |
| 4 | aes256_gcm_decrypt | AES-256-GCM decryption |
| 11 | chacha20_poly1305_encrypt | ChaCha20-Poly1305 encryption |
| 12 | chacha20_poly1305_decrypt | ChaCha20-Poly1305 decryption |
| 13 | xchacha20_poly1305_encrypt | XChaCha20-Poly1305 (extended nonce) |
| 14 | xchacha20_poly1305_decrypt | XChaCha20-Poly1305 decryption |

### Key Derivation

| ID | Name | Description |
|----|------|-------------|
| 7 | pbkdf2_sha256 | PBKDF2-HMAC-SHA256 |
| 23 | hkdf_sha256_extract | HKDF extract (TLS 1.3) |
| 24 | hkdf_sha256_expand | HKDF expand (TLS 1.3) |
| 35 | argon2id_hash | Argon2id password hashing |

### Digital Signatures

| ID | Name | Description |
|----|------|-------------|
| 8 | ed25519_sign | Ed25519 signature |
| 9 | ed25519_verify | Ed25519 verification |
| 25 | p256_ecdsa_sign | NIST P-256 ECDSA sign |
| 26 | p256_ecdsa_verify | NIST P-256 ECDSA verify |
| 28 | p384_ecdsa_sign | NIST P-384 ECDSA sign |
| 29 | p384_ecdsa_verify | NIST P-384 ECDSA verify |
| 31 | rsa_pkcs1_sign_sha256 | RSA PKCS#1 v1.5 sign |
| 32 | rsa_pkcs1_verify_sha256 | RSA PKCS#1 v1.5 verify |

### Key Exchange

| ID | Name | Description |
|----|------|-------------|
| 10 | x25519_derive | X25519 (Curve25519) |
| 27 | p256_ecdh | NIST P-256 ECDH |
| 30 | p384_ecdh | NIST P-384 ECDH |

### Utilities

| ID | Name | Description |
|----|------|-------------|
| 5 | constant_time_eq | Timing-safe comparison |
| 6 | secure_random | Cryptographic RNG |

### Examples

```asm
; SHA-256 hash (general purpose)
lea r0, input_buf           ; input pointer
mov r1, input_len           ; input length
lea r2, output_buf          ; 32-byte output buffer
ext.call 1, r0, r1, r2      ; sha256

; SHA-1 for WebSocket handshake (legacy only!)
lea r0, ws_key_magic        ; key + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11"
mov r1, 60                  ; length (24 + 36)
lea r2, sha1_out            ; 20-byte output
ext.call 38, r0, r1, r2     ; sha1

; HMAC-SHA256 for authentication
lea r0, key_ptr             ; key pointer
mov r1, key_len             ; key length
lea r2, data_ptr            ; data pointer
mov r3, data_len            ; data length
lea r4, mac_out             ; 32-byte output
ext.call 2, r0, r1, r2, r3, r4  ; hmac_sha256

; Argon2id for password hashing
lea r0, password_ptr
mov r1, password_len
lea r2, salt_ptr
mov r3, salt_len
mov r4, 3                   ; time cost
mov r5, 65536               ; memory cost (64 MB)
mov r6, 4                   ; parallelism
lea r7, hash_out            ; 32-byte output
ext.call 35, r0, r1, r2, r3, r4, r5, r6, r7  ; argon2id
```

---

## File System Extension (240-259)

Read and write files on the local filesystem.

### Operations

| ID | Name | Intent | Description |
|----|------|--------|-------------|
| 240 | fs_read | "read file contents" | Read entire file to string |
| 241 | fs_read_bytes | "read file as bytes" | Read file to byte Vec |
| 242 | fs_write | "write file" | Write string to file |
| 243 | fs_write_bytes | "write bytes to file" | Write byte Vec to file |
| 244 | fs_append | "append to file" | Append string to file |
| 245 | fs_exists | "check file exists" | Check if path exists |
| 246 | fs_is_file | "check is file" | Check if path is a file |
| 247 | fs_is_dir | "check is directory" | Check if path is a directory |
| 248 | fs_mkdir | "create directory" | Create directory |
| 249 | fs_remove | "remove file" | Delete file |
| 250 | fs_remove_dir | "remove directory" | Delete directory |
| 251 | fs_list_dir | "list directory" | List directory contents |
| 252 | fs_copy | "copy file" | Copy file |
| 253 | fs_rename | "rename file" | Rename/move file |

### Example

```asm
; Read a configuration file
mov r1, path_string          ; r1 = "/etc/app/config.json"
ext.call r0, @"read file contents", r1, r0  ; r0 = file contents as string

; Parse as JSON
ext.call r2, @"parse JSON string", r0, r0   ; r2 = json handle
```

---

## SQLite Extension (260-279)

Local SQLite database operations.

### Operations

| ID | Name | Intent | Description |
|----|------|--------|-------------|
| 260 | sqlite_open | "open SQLite database" | Open/create database file |
| 261 | sqlite_close | "close SQLite database" | Close database connection |
| 262 | sqlite_execute | "execute SQL" | Execute statement (no results) |
| 263 | sqlite_query | "query SQLite" | Query with result set |
| 264 | sqlite_prepare | "prepare SQL statement" | Prepare parameterized statement |
| 265 | sqlite_bind_int | "bind integer parameter" | Bind integer to statement |
| 266 | sqlite_bind_text | "bind text parameter" | Bind string to statement |
| 267 | sqlite_bind_blob | "bind blob parameter" | Bind binary data |
| 268 | sqlite_step | "step SQL statement" | Execute prepared statement |
| 269 | sqlite_reset | "reset SQL statement" | Reset for re-execution |
| 270 | sqlite_finalize | "finalize SQL statement" | Deallocate statement |
| 271 | sqlite_column_int | "get integer column" | Get integer from result |
| 272 | sqlite_column_text | "get text column" | Get string from result |
| 273 | sqlite_last_insert_id | "get last insert ID" | Get last auto-increment ID |

### Example

```asm
; Open database and insert a record
mov r1, db_path              ; r1 = "app.db"
ext.call r0, @"open SQLite database", r1, r0  ; r0 = db handle

mov r2, sql_insert           ; r2 = "INSERT INTO users (name) VALUES (?)"
ext.call r3, @"prepare SQL statement", r0, r2 ; r3 = statement handle

mov r4, user_name            ; r4 = "Alice"
ext.call r0, @"bind text parameter", r3, r4

ext.call r0, @"step SQL statement", r3, r0
ext.call r0, @"finalize SQL statement", r3, r0
ext.call r0, @"close SQLite database", r0, r0
```

---

## Regex Extension (280-299)

Regular expression matching and replacement.

### Operations

| ID | Name | Intent | Description |
|----|------|--------|-------------|
| 280 | regex_compile | "compile regex pattern" | Compile pattern to handle |
| 281 | regex_is_match | "check regex match" | Test if string matches |
| 282 | regex_find | "find regex match" | Find first match |
| 283 | regex_find_all | "find all regex matches" | Find all matches |
| 284 | regex_replace | "replace with regex" | Replace first match |
| 285 | regex_replace_all | "replace all regex matches" | Replace all matches |
| 286 | regex_split | "split by regex" | Split string by pattern |
| 287 | regex_captures | "get regex captures" | Get capture groups |
| 288 | regex_free | "free regex handle" | Deallocate regex handle |

### Example

```asm
; Extract email addresses from text
mov r1, pattern              ; r1 = "[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}"
ext.call r0, @"compile regex pattern", r1, r0  ; r0 = regex handle

mov r2, input_text           ; r2 = "Contact us at info@example.com"
ext.call r3, @"find all regex matches", r0, r2 ; r3 = vec of matches

ext.call r0, @"free regex handle", r0, r0
```

---

## DateTime Extension (300-319)

Date and time operations.

### Operations

| ID | Name | Intent | Description |
|----|------|--------|-------------|
| 300 | datetime_now | "get current time" | Current UTC timestamp |
| 301 | datetime_now_local | "get local time" | Current local timestamp |
| 302 | datetime_parse | "parse date string" | Parse date from string |
| 303 | datetime_format | "format date" | Format timestamp to string |
| 304 | datetime_add_days | "add days to date" | Add days to timestamp |
| 305 | datetime_add_hours | "add hours to date" | Add hours to timestamp |
| 306 | datetime_add_minutes | "add minutes to date" | Add minutes to timestamp |
| 307 | datetime_diff_seconds | "get time difference" | Difference in seconds |
| 308 | datetime_year | "get year" | Extract year |
| 309 | datetime_month | "get month" | Extract month |
| 310 | datetime_day | "get day" | Extract day |
| 311 | datetime_hour | "get hour" | Extract hour |
| 312 | datetime_minute | "get minute" | Extract minute |
| 313 | datetime_second | "get second" | Extract second |
| 314 | datetime_weekday | "get weekday" | Get day of week (0=Sunday) |

### Example

```asm
; Get current time and format it
ext.call r0, @"get current time", r0, r0    ; r0 = timestamp

mov r1, format_string        ; r1 = "%Y-%m-%d %H:%M:%S"
ext.call r2, @"format date", r0, r1         ; r2 = "2026-01-24 14:30:00"
```

---

## Environment Extension (320-329)

Access environment variables.

### Operations

| ID | Name | Intent | Description |
|----|------|--------|-------------|
| 320 | env_get | "get environment variable" | Get env var value |
| 321 | env_set | "set environment variable" | Set env var |
| 322 | env_remove | "remove environment variable" | Unset env var |
| 323 | env_exists | "check environment variable exists" | Check if var exists |
| 324 | env_all | "get all environment variables" | Get all vars as HashMap |

### Example

```asm
; Get API key from environment
mov r1, var_name             ; r1 = "API_KEY"
ext.call r0, @"get environment variable", r1, r0  ; r0 = value or null
```

---

## UUID Extension (330-339)

Generate and parse UUIDs.

### Operations

| ID | Name | Intent | Description |
|----|------|--------|-------------|
| 330 | uuid_v4 | "generate UUID v4" | Random UUID |
| 331 | uuid_v7 | "generate UUID v7" | Timestamp-ordered UUID |
| 332 | uuid_parse | "parse UUID string" | Parse UUID from string |
| 333 | uuid_to_string | "UUID to string" | Convert UUID to string |
| 334 | uuid_is_valid | "validate UUID" | Check if string is valid UUID |

### Example

```asm
; Generate a new UUID
ext.call r0, @"generate UUID v4", r0, r0   ; r0 = UUID handle
ext.call r1, @"UUID to string", r0, r0      ; r1 = "550e8400-e29b-41d4-..."
```

---

## Base64 Extension (340-349)

Base64 encoding and decoding.

### Operations

| ID | Name | Intent | Description |
|----|------|--------|-------------|
| 340 | base64_encode | "encode as base64" | Encode bytes to base64 |
| 341 | base64_decode | "decode base64" | Decode base64 to bytes |
| 342 | base64_encode_url | "encode as URL-safe base64" | URL-safe encoding |
| 343 | base64_decode_url | "decode URL-safe base64" | URL-safe decoding |

### Example

```asm
; Encode binary data
mov r1, data_ptr             ; r1 = binary data
mov r2, data_len             ; r2 = length
ext.call r0, @"encode as base64", r1, r2   ; r0 = base64 string
```

---

## URL Extension (350-359)

Parse and manipulate URLs.

### Operations

| ID | Name | Intent | Description |
|----|------|--------|-------------|
| 350 | url_parse | "parse URL" | Parse URL to components |
| 351 | url_scheme | "get URL scheme" | Get scheme (http, https) |
| 352 | url_host | "get URL host" | Get hostname |
| 353 | url_port | "get URL port" | Get port number |
| 354 | url_path | "get URL path" | Get path component |
| 355 | url_query | "get URL query" | Get query string |
| 356 | url_encode | "URL encode" | Percent-encode string |
| 357 | url_decode | "URL decode" | Percent-decode string |
| 358 | url_join | "join URL paths" | Join URL with path |

### Example

```asm
; Parse a URL and get the host
mov r1, url_string           ; r1 = "https://api.example.com:8080/users?id=1"
ext.call r0, @"parse URL", r1, r0           ; r0 = URL handle
ext.call r2, @"get URL host", r0, r0        ; r2 = "api.example.com"
ext.call r3, @"get URL port", r0, r0        ; r3 = 8080
```

---

## Log Extension (360-369)

Structured logging.

### Operations

| ID | Name | Intent | Description |
|----|------|--------|-------------|
| 360 | log_debug | "log debug message" | Debug level log |
| 361 | log_info | "log info message" | Info level log |
| 362 | log_warn | "log warning message" | Warning level log |
| 363 | log_error | "log error message" | Error level log |
| 364 | log_set_level | "set log level" | Set minimum log level |

### Log Levels

| Level | Value |
|-------|-------|
| DEBUG | 0 |
| INFO | 1 |
| WARN | 2 |
| ERROR | 3 |

### Example

```asm
; Log an info message
mov r1, message              ; r1 = "Server started on port 8080"
ext.call r0, @"log info message", r1, r0
```

---

## See Also

- [Three-Layer Architecture](../architecture/three-layers.md) - Why bundled extensions exist
- [RAG-Based Extension Resolution](../architecture/rag-extensions.md) - How resolution works
- [Creating Extensions](./creating.md) - How to build your own extensions
