# TLS Module

Secure TLS connections using rustls (pure Rust).

## Overview

The TLS module provides safe, memory-secure TLS 1.2 and TLS 1.3 connections using `rustls`. This is a pure Rust implementation with no OpenSSL dependency, providing verified memory safety.

## Features

- **TLS 1.2 and TLS 1.3** - Modern protocol support
- **Pure Rust** - No OpenSSL, no unsafe C code
- **Mozilla CA roots** - Built-in trusted CA certificates
- **Client and server** - Both connection types supported
- **mTLS** - Mutual TLS authentication

## API Reference

### Client Configuration

```rust
/// Create a TLS client config with Mozilla CA roots
pub fn client_config() -> WrapperResult<TlsClientConfig>;

/// Create a TLS client config with custom CA
pub fn client_config_with_ca(ca_pem: &OwnedBuffer) -> WrapperResult<TlsClientConfig>;

/// Create a TLS client config for mTLS (with client cert)
pub fn client_config_mtls(
    ca_pem: &OwnedBuffer,
    client_cert_pem: &OwnedBuffer,
    client_key_pem: &OwnedBuffer,
) -> WrapperResult<TlsClientConfig>;

/// Disable certificate verification (DANGEROUS - testing only!)
pub fn client_config_insecure() -> TlsClientConfig;
```

### Server Configuration

```rust
/// Create a TLS server config with certificate and key
pub fn server_config(
    cert_chain_pem: &OwnedBuffer,
    key_pem: &OwnedBuffer,
) -> WrapperResult<TlsServerConfig>;

/// Create a TLS server config requiring client certs (mTLS)
pub fn server_config_mtls(
    cert_chain_pem: &OwnedBuffer,
    key_pem: &OwnedBuffer,
    client_ca_pem: &OwnedBuffer,
) -> WrapperResult<TlsServerConfig>;
```

### Client Connections

```rust
/// Connect to a TLS server
pub fn connect(
    host: &str,
    port: u16,
    config: &TlsClientConfig,
) -> WrapperResult<TlsConnection>;

/// Connect with custom SNI (Server Name Indication)
pub fn connect_sni(
    host: &str,
    port: u16,
    sni: &str,
    config: &TlsClientConfig,
) -> WrapperResult<TlsConnection>;
```

### Server Connections

```rust
/// Accept a TLS connection on a TCP stream
pub fn accept(
    tcp_stream: TcpStream,
    config: &TlsServerConfig,
) -> WrapperResult<TlsConnection>;
```

### Data Transfer

```rust
/// Send data over TLS connection
pub fn send(conn: &mut TlsConnection, data: &OwnedBuffer) -> WrapperResult<usize>;

/// Receive data from TLS connection
pub fn recv(conn: &mut TlsConnection, max_len: usize) -> WrapperResult<OwnedBuffer>;

/// Receive with timeout (milliseconds)
pub fn recv_timeout(
    conn: &mut TlsConnection,
    max_len: usize,
    timeout_ms: u64,
) -> WrapperResult<OwnedBuffer>;

/// Close TLS connection gracefully
pub fn close(conn: TlsConnection) -> WrapperResult<()>;
```

### Connection Information

```rust
/// Get negotiated TLS protocol version
pub fn get_protocol_version(conn: &TlsConnection) -> WrapperResult<String>;

/// Get negotiated cipher suite
pub fn get_cipher_suite(conn: &TlsConnection) -> WrapperResult<String>;

/// Get peer certificate (if presented)
pub fn get_peer_cert(conn: &TlsConnection) -> WrapperResult<Option<OwnedBuffer>>;

/// Check if connection is using TLS 1.3
pub fn is_tls13(conn: &TlsConnection) -> bool;

/// Get ALPN protocol (if negotiated)
pub fn get_alpn(conn: &TlsConnection) -> Option<String>;
```

### ALPN (Application-Layer Protocol Negotiation)

```rust
/// Set ALPN protocols for client config
pub fn client_config_alpn(
    config: &mut TlsClientConfig,
    protocols: &[&str],
);

/// Set ALPN protocols for server config
pub fn server_config_alpn(
    config: &mut TlsServerConfig,
    protocols: &[&str],
);
```

## Usage Examples

### Simple HTTPS Client

```rust
use neurlang::wrappers::{OwnedBuffer, tls};

// Create config with Mozilla CA roots
let config = tls::client_config()?;

// Connect to server
let mut conn = tls::connect("example.com", 443, &config)?;

// Send HTTP request
let request = OwnedBuffer::from_str(
    "GET / HTTP/1.1\r\nHost: example.com\r\nConnection: close\r\n\r\n"
);
tls::send(&mut conn, &request)?;

// Receive response
let response = tls::recv(&mut conn, 8192)?;
println!("Response:\n{}", response.as_str().unwrap());

// Check TLS version
let version = tls::get_protocol_version(&conn)?;
println!("TLS version: {}", version);  // "TLS1.3"

// Close connection
tls::close(conn)?;
```

### TLS Server

```rust
use neurlang::wrappers::{OwnedBuffer, tls, x509};
use std::net::TcpListener;

// Generate certificate (or load from file)
let key = x509::keypair_generate_ec("P-256")?;
let cert = x509::create_self_signed("CN=localhost", &key, 365)?;

// Create server config
let config = tls::server_config(&cert, &key)?;

// Accept connections
let listener = TcpListener::bind("0.0.0.0:8443")?;
for stream in listener.incoming() {
    let stream = stream?;
    let mut conn = tls::accept(stream, &config)?;

    // Handle connection
    let data = tls::recv(&mut conn, 4096)?;
    let response = OwnedBuffer::from_str("HTTP/1.1 200 OK\r\n\r\nHello!");
    tls::send(&mut conn, &response)?;
    tls::close(conn)?;
}
```

### Mutual TLS (mTLS)

```rust
// Client with certificate
let client_key = x509::keypair_generate_ec("P-256")?;
let client_cert = x509::create_self_signed("CN=client", &client_key, 365)?;

let config = tls::client_config_mtls(
    &ca_cert,
    &client_cert,
    &client_key,
)?;

let conn = tls::connect("secure.example.com", 443, &config)?;

// Server side
let server_config = tls::server_config_mtls(
    &server_cert,
    &server_key,
    &client_ca,  // CA that signed client certs
)?;
```

### Custom CA

```rust
// For internal services with private CA
let ca_pem = OwnedBuffer::from_str(include_str!("internal-ca.pem"));
let config = tls::client_config_with_ca(&ca_pem)?;

let conn = tls::connect("internal.service.local", 8443, &config)?;
```

### ALPN for HTTP/2

```rust
let mut config = tls::client_config()?;
tls::client_config_alpn(&mut config, &["h2", "http/1.1"]);

let conn = tls::connect("example.com", 443, &config)?;

match tls::get_alpn(&conn) {
    Some(proto) if proto == "h2" => println!("Using HTTP/2"),
    Some(proto) if proto == "http/1.1" => println!("Using HTTP/1.1"),
    _ => println!("No ALPN"),
}
```

## IR Assembly Usage

```asm
; Create client config
ext.call r0, @"tls config client"

; Connect
mov r1, host_ptr
mov r2, 443        ; port
mov r3, config
ext.call r0, @"tls connect", r1, r2, r3

; Send data
mov r1, conn
mov r2, data_ptr
ext.call r0, @"tls send", r1, r2

; Receive data
mov r1, conn
mov r2, 4096       ; max length
ext.call r0, @"tls recv", r1, r2

; Get protocol version
mov r1, conn
ext.call r0, @"tls get protocol version", r1

; Close
mov r1, conn
ext.call r0, @"tls close", r1
```

## RAG Keywords

| Intent | Resolves To |
|--------|-------------|
| "tls", "ssl", "secure connection" | `connect` |
| "tls config", "ssl config", "client config" | `client_config` |
| "tls connect", "secure connect", "https" | `connect` |
| "tls send", "ssl write" | `send` |
| "tls recv", "ssl read" | `recv` |
| "tls server", "ssl accept" | `accept` |
| "mtls", "mutual tls", "client cert" | `client_config_mtls` |
| "tls version", "protocol version" | `get_protocol_version` |

## Error Handling

```rust
use neurlang::wrappers::{WrapperError, tls};

match tls::connect("example.com", 443, &config) {
    Ok(conn) => {
        println!("Connected!");
    }
    Err(WrapperError::TlsError(msg)) => {
        // TLS-specific error
        eprintln!("TLS error: {}", msg);
    }
    Err(WrapperError::IoError(msg)) => {
        // Network error
        eprintln!("IO error: {}", msg);
    }
    Err(e) => {
        eprintln!("Unexpected error: {}", e);
    }
}
```

Common errors:
- `TlsError("certificate verify failed")` - Server cert not trusted
- `TlsError("handshake failed")` - TLS negotiation failed
- `TlsError("alert received: ...")` - Server sent TLS alert
- `IoError("connection refused")` - Server not listening
- `IoError("connection reset")` - Connection dropped

## Supported Cipher Suites

### TLS 1.3 (Preferred)

- TLS_AES_256_GCM_SHA384
- TLS_AES_128_GCM_SHA256
- TLS_CHACHA20_POLY1305_SHA256

### TLS 1.2

- TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384
- TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256
- TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256
- TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384
- TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256

## Security Considerations

### Certificate Verification

**ALWAYS verify certificates in production:**

```rust
// GOOD: Uses Mozilla CA roots
let config = tls::client_config()?;

// GOOD: Uses custom CA for internal services
let config = tls::client_config_with_ca(&internal_ca)?;

// DANGEROUS: Disables verification - TESTING ONLY!
let config = tls::client_config_insecure();
```

### Protocol Version

rustls only supports TLS 1.2 and TLS 1.3. Legacy protocols (SSL 3.0, TLS 1.0, TLS 1.1) are not supported.

### Cipher Suites

All supported cipher suites provide:
- Forward secrecy (ECDHE)
- Authenticated encryption (GCM or Poly1305)
- No known vulnerabilities

## Performance Characteristics

| Operation | Approximate Time |
|-----------|------------------|
| Config creation | ~1ms |
| TLS 1.3 handshake | ~1 RTT |
| TLS 1.2 handshake | ~2 RTT |
| Encrypt/decrypt | ~1 GB/s |

## Dependencies

```toml
[dependencies]
rustls = "0.23"            # TLS implementation
webpki-roots = "0.26"      # Mozilla CA roots
rustls-pemfile = "2.1"     # PEM parsing
```

## See Also

- [Buffer Module](buffer.md) - OwnedBuffer type
- [X509 Module](x509.md) - Certificate generation and parsing
- [HTTP Extensions](../extensions/bundled.md) - Higher-level HTTP client
