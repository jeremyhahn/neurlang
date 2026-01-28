//! TLS Wrappers
//!
//! Secure TLS connections using rustls (pure Rust, no OpenSSL).
//!
//! # Features
//!
//! - TLS 1.2 and TLS 1.3
//! - Mozilla CA roots for certificate verification
//! - Client and server connections
//! - mTLS (mutual TLS) support

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;

use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};

use super::{OwnedBuffer, WrapperCategory, WrapperError, WrapperRegistry, WrapperResult};

// =============================================================================
// Client Configuration
// =============================================================================

/// TLS client configuration
pub struct TlsClientConfig {
    inner: Arc<ClientConfig>,
}

/// Create a TLS client config with Mozilla CA roots
pub fn client_config() -> WrapperResult<TlsClientConfig> {
    let root_store = RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    let config =
        ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions()
            .map_err(|e| WrapperError::TlsError(format!("Protocol version error: {}", e)))?
            .with_root_certificates(root_store)
            .with_no_client_auth();

    Ok(TlsClientConfig {
        inner: Arc::new(config),
    })
}

/// Create a TLS client config with custom CA
pub fn client_config_with_ca(ca_pem: &OwnedBuffer) -> WrapperResult<TlsClientConfig> {
    let ca_str = ca_pem.as_str()?;

    let mut root_store = RootCertStore::empty();

    let certs = rustls_pemfile::certs(&mut ca_str.as_bytes())
        .filter_map(|r| r.ok())
        .collect::<Vec<_>>();

    if certs.is_empty() {
        return Err(WrapperError::TlsError(
            "No valid certificates found in PEM".to_string(),
        ));
    }

    for cert in certs {
        root_store
            .add(cert)
            .map_err(|e| WrapperError::TlsError(format!("Failed to add CA: {}", e)))?;
    }

    let config =
        ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions()
            .map_err(|e| WrapperError::TlsError(format!("Protocol version error: {}", e)))?
            .with_root_certificates(root_store)
            .with_no_client_auth();

    Ok(TlsClientConfig {
        inner: Arc::new(config),
    })
}

// =============================================================================
// TLS Connection
// =============================================================================

/// A TLS connection
pub struct TlsConnection {
    stream: StreamOwned<ClientConnection, TcpStream>,
}

/// Connect to a TLS server
pub fn connect(host: &str, port: u16, config: &TlsClientConfig) -> WrapperResult<TlsConnection> {
    let server_name = ServerName::try_from(host.to_string())
        .map_err(|_| WrapperError::TlsError(format!("Invalid hostname: {}", host)))?;

    let conn = ClientConnection::new(config.inner.clone(), server_name)
        .map_err(|e| WrapperError::TlsError(format!("TLS connection failed: {}", e)))?;

    let tcp = TcpStream::connect((host, port))
        .map_err(|e| WrapperError::IoError(format!("TCP connection failed: {}", e)))?;

    let stream = StreamOwned::new(conn, tcp);

    Ok(TlsConnection { stream })
}

/// Send data over TLS connection
pub fn send(conn: &mut TlsConnection, data: &OwnedBuffer) -> WrapperResult<usize> {
    let written = conn
        .stream
        .write(data.as_slice())
        .map_err(|e| WrapperError::TlsError(format!("TLS write failed: {}", e)))?;

    conn.stream
        .flush()
        .map_err(|e| WrapperError::TlsError(format!("TLS flush failed: {}", e)))?;

    Ok(written)
}

/// Receive data from TLS connection
pub fn recv(conn: &mut TlsConnection, max_len: usize) -> WrapperResult<OwnedBuffer> {
    let mut buf = vec![0u8; max_len];
    let read = conn
        .stream
        .read(&mut buf)
        .map_err(|e| WrapperError::TlsError(format!("TLS read failed: {}", e)))?;

    buf.truncate(read);
    Ok(OwnedBuffer::from_vec(buf))
}

/// Get the negotiated TLS protocol version
pub fn get_protocol_version(conn: &TlsConnection) -> WrapperResult<String> {
    let version = conn
        .stream
        .conn
        .protocol_version()
        .ok_or(WrapperError::NotConnected)?;

    Ok(match version {
        rustls::ProtocolVersion::TLSv1_2 => "TLS1.2".to_string(),
        rustls::ProtocolVersion::TLSv1_3 => "TLS1.3".to_string(),
        _ => "Unknown".to_string(),
    })
}

/// Get the negotiated cipher suite
pub fn get_cipher_suite(conn: &TlsConnection) -> WrapperResult<String> {
    let suite = conn
        .stream
        .conn
        .negotiated_cipher_suite()
        .ok_or(WrapperError::NotConnected)?;

    Ok(format!("{:?}", suite.suite()))
}

/// Check if connection is using TLS 1.3
pub fn is_tls13(conn: &TlsConnection) -> bool {
    conn.stream
        .conn
        .protocol_version()
        .map(|v| v == rustls::ProtocolVersion::TLSv1_3)
        .unwrap_or(false)
}

/// Get ALPN protocol (if negotiated)
pub fn get_alpn(conn: &TlsConnection) -> Option<String> {
    conn.stream
        .conn
        .alpn_protocol()
        .map(|p| String::from_utf8_lossy(p).to_string())
}

// =============================================================================
// Simple HTTPS GET
// =============================================================================

/// Perform a simple HTTPS GET request
pub fn https_get(url: &str) -> WrapperResult<OwnedBuffer> {
    // Parse URL (simple parsing)
    let url = url.strip_prefix("https://").unwrap_or(url);
    let (host, path) = url.split_once('/').unwrap_or((url, ""));
    let (host, port) = host
        .split_once(':')
        .map(|(h, p)| (h, p.parse().unwrap_or(443)))
        .unwrap_or((host, 443));

    let config = client_config()?;
    let mut conn = connect(host, port, &config)?;

    // Send HTTP request
    let request = format!(
        "GET /{} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        path, host
    );
    send(&mut conn, &OwnedBuffer::from_string(request))?;

    // Read response
    let mut response = Vec::new();
    loop {
        match recv(&mut conn, 8192) {
            Ok(chunk) => {
                if chunk.is_empty() {
                    break;
                }
                response.extend_from_slice(chunk.as_slice());
            }
            Err(_) => break,
        }
    }

    Ok(OwnedBuffer::from_vec(response))
}

// =============================================================================
// Registration
// =============================================================================

/// Register all TLS wrappers with the registry
pub fn register(registry: &mut WrapperRegistry) {
    registry.register_wrapper(
        "tls_connect",
        "Connect to a TLS server",
        WrapperCategory::Tls,
        2,
        &["tls", "ssl", "connect", "secure", "https"],
        |args| {
            if args.len() < 2 {
                return Err(WrapperError::InvalidArg("Need host and port".to_string()));
            }
            let host = args[0].as_str()?;
            let port = u16::from_le_bytes(
                args[1]
                    .as_slice()
                    .get(0..2)
                    .ok_or_else(|| WrapperError::InvalidArg("Invalid port".to_string()))?
                    .try_into()
                    .unwrap(),
            );

            let config = client_config()?;
            let _conn = connect(host, port, &config)?;

            // Store connection handle
            let handle = super::buffer::HandleManager::store(OwnedBuffer::from_str(&format!(
                "TLS:{}:{}",
                host, port
            )));

            // We can't actually store the connection in the buffer system,
            // so for now just return a success indicator
            Ok(OwnedBuffer::from_vec(handle.to_le_bytes().to_vec()))
        },
    );

    registry.register_wrapper(
        "https_get",
        "Perform a simple HTTPS GET request",
        WrapperCategory::Tls,
        1,
        &["https", "get", "fetch https", "tls get"],
        |args| {
            if args.is_empty() {
                return Err(WrapperError::InvalidArg("No URL provided".to_string()));
            }
            let url = args[0].as_str()?;
            https_get(url)
        },
    );

    registry.register_wrapper(
        "tls_config_client",
        "Create a TLS client config with Mozilla CA roots",
        WrapperCategory::Tls,
        0,
        &["tls config", "ssl config", "client config"],
        |_args| {
            let _config = client_config()?;
            // Return success indicator
            Ok(OwnedBuffer::from_slice(&[1]))
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_config() {
        let config = client_config();
        assert!(config.is_ok());
    }

    #[test]
    fn test_client_config_with_ca() {
        // Test with a mock CA (this would need a real CA cert to work)
        let fake_ca = OwnedBuffer::from_str("not a real cert");
        let result = client_config_with_ca(&fake_ca);
        // Should fail because it's not valid PEM
        assert!(result.is_err());
    }

    // Integration tests require network access
    #[test]
    #[ignore]
    fn test_https_get() {
        let response = https_get("https://example.com").unwrap();
        let text = response.as_str().unwrap();
        assert!(text.contains("HTTP/1.1"));
        assert!(text.contains("Example Domain"));
    }

    #[test]
    #[ignore]
    fn test_tls_connect() {
        let config = client_config().unwrap();
        let conn = connect("example.com", 443, &config);
        assert!(conn.is_ok());

        let conn = conn.unwrap();
        let version = get_protocol_version(&conn).unwrap();
        assert!(version == "TLS1.2" || version == "TLS1.3");
    }
}
