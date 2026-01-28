//! TLS Extensions
//!
//! Secure TLS connections using rustls (pure Rust, no OpenSSL).

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};

use super::buffer::{HandleManager, OwnedBuffer};
use super::{ext_ids, ExtCategory, ExtError, ExtensionRegistry};

// =============================================================================
// TLS Connection Storage
// =============================================================================

lazy_static::lazy_static! {
    /// Storage for TLS connections
    static ref TLS_CONNECTIONS: RwLock<HashMap<u64, TlsConnection>> = RwLock::new(HashMap::new());
}

static NEXT_TLS_HANDLE: AtomicU64 = AtomicU64::new(1);

fn next_tls_handle() -> u64 {
    NEXT_TLS_HANDLE.fetch_add(1, Ordering::Relaxed)
}

/// A TLS connection wrapper
pub struct TlsConnection {
    stream: StreamOwned<ClientConnection, TcpStream>,
}

// =============================================================================
// TLS Operations
// =============================================================================

/// Create a TLS client config with Mozilla CA roots
fn create_client_config() -> Result<Arc<ClientConfig>, ExtError> {
    let root_store = RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    let config =
        ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions()
            .map_err(|e| ExtError::ExtensionError(format!("Protocol version error: {}", e)))?
            .with_root_certificates(root_store)
            .with_no_client_auth();

    Ok(Arc::new(config))
}

/// Connect to a TLS server
fn tls_connect_impl(host: &str, port: u16) -> Result<u64, ExtError> {
    let config = create_client_config()?;

    let server_name = ServerName::try_from(host.to_string())
        .map_err(|_| ExtError::ExtensionError(format!("Invalid hostname: {}", host)))?;

    let conn = ClientConnection::new(config, server_name)
        .map_err(|e| ExtError::ExtensionError(format!("TLS connection failed: {}", e)))?;

    let tcp = TcpStream::connect((host, port))
        .map_err(|e| ExtError::ExtensionError(format!("TCP connection failed: {}", e)))?;

    let stream = StreamOwned::new(conn, tcp);
    let handle = next_tls_handle();

    TLS_CONNECTIONS
        .write()
        .unwrap()
        .insert(handle, TlsConnection { stream });

    Ok(handle)
}

/// Send data over TLS connection
fn tls_send_impl(handle: u64, data: &[u8]) -> Result<usize, ExtError> {
    let mut connections = TLS_CONNECTIONS.write().unwrap();
    let conn = connections
        .get_mut(&handle)
        .ok_or_else(|| ExtError::ExtensionError("Invalid TLS connection handle".to_string()))?;

    let written = conn
        .stream
        .write(data)
        .map_err(|e| ExtError::ExtensionError(format!("TLS write failed: {}", e)))?;

    conn.stream
        .flush()
        .map_err(|e| ExtError::ExtensionError(format!("TLS flush failed: {}", e)))?;

    Ok(written)
}

/// Receive data from TLS connection
fn tls_recv_impl(handle: u64, max_len: usize) -> Result<Vec<u8>, ExtError> {
    let mut connections = TLS_CONNECTIONS.write().unwrap();
    let conn = connections
        .get_mut(&handle)
        .ok_or_else(|| ExtError::ExtensionError("Invalid TLS connection handle".to_string()))?;

    let mut buf = vec![0u8; max_len];
    let read = conn
        .stream
        .read(&mut buf)
        .map_err(|e| ExtError::ExtensionError(format!("TLS read failed: {}", e)))?;

    buf.truncate(read);
    Ok(buf)
}

/// Close TLS connection
fn tls_close_impl(handle: u64) -> Result<(), ExtError> {
    TLS_CONNECTIONS.write().unwrap().remove(&handle);
    Ok(())
}

// =============================================================================
// Extension Registration
// =============================================================================

pub fn register_tls(registry: &mut ExtensionRegistry) {
    // tls_connect: Connect to TLS server
    registry.register_with_id(
        ext_ids::TLS_CONNECT,
        "tls_connect",
        "Connect to TLS server. Args: host_handle, port. Returns tls_handle.",
        2,
        true,
        ExtCategory::Tls,
        Arc::new(|args, outputs| {
            let host_handle = args[0];
            let port = args[1] as u16;

            let host_buf = HandleManager::get(host_handle)
                .ok_or_else(|| ExtError::ExtensionError("Invalid host handle".to_string()))?;
            let host = host_buf
                .as_str()
                .map_err(|e| ExtError::ExtensionError(format!("Invalid UTF-8 in host: {}", e)))?;

            let handle = tls_connect_impl(host, port)?;
            outputs[0] = handle;
            Ok(handle as i64)
        }),
    );

    // tls_send: Send data
    registry.register_with_id(
        ext_ids::TLS_SEND,
        "tls_send",
        "Send data over TLS connection. Args: tls_handle, data_handle. Returns bytes written.",
        2,
        true,
        ExtCategory::Tls,
        Arc::new(|args, outputs| {
            let tls_handle = args[0];
            let data_handle = args[1];

            let data_buf = HandleManager::get(data_handle)
                .ok_or_else(|| ExtError::ExtensionError("Invalid data handle".to_string()))?;

            let written = tls_send_impl(tls_handle, data_buf.as_slice())?;
            outputs[0] = written as u64;
            Ok(written as i64)
        }),
    );

    // tls_recv: Receive data
    registry.register_with_id(
        ext_ids::TLS_RECV,
        "tls_recv",
        "Receive data from TLS connection. Args: tls_handle, max_len. Returns buffer_handle.",
        2,
        true,
        ExtCategory::Tls,
        Arc::new(|args, outputs| {
            let tls_handle = args[0];
            let max_len = args[1] as usize;

            let data = tls_recv_impl(tls_handle, max_len)?;
            let result_handle = HandleManager::store(OwnedBuffer::from_vec(data));
            outputs[0] = result_handle;
            Ok(result_handle as i64)
        }),
    );

    // tls_close: Close connection
    registry.register_with_id(
        ext_ids::TLS_CLOSE,
        "tls_close",
        "Close TLS connection. Args: tls_handle. Returns 0.",
        1,
        true,
        ExtCategory::Tls,
        Arc::new(|args, outputs| {
            let tls_handle = args[0];
            tls_close_impl(tls_handle)?;
            outputs[0] = 0;
            Ok(0)
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_client_config() {
        let config = create_client_config();
        assert!(config.is_ok());
    }

    // Integration tests require network access
    #[test]
    #[ignore]
    fn test_tls_connect() {
        let handle = tls_connect_impl("example.com", 443).unwrap();
        assert!(handle > 0);
        tls_close_impl(handle).unwrap();
    }
}
