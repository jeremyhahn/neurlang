//! HTTP Client Extensions
//!
//! Wraps the `ureq` crate for simple HTTP requests.
//! Extension IDs: 190-209

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;

use super::strings::STRING_STORAGE;
use crate::runtime::{ExtCategory, ExtensionRegistry};

lazy_static::lazy_static! {
    /// Storage for HTTP responses (body as string handle)
    pub static ref RESPONSE_STORAGE: RwLock<HashMap<u64, HttpResponse>> = RwLock::new(HashMap::new());
}

static NEXT_RESPONSE_HANDLE: AtomicU64 = AtomicU64::new(1);

/// HTTP response wrapper
pub struct HttpResponse {
    pub status: u16,
    pub body_handle: u64, // String handle
    pub headers: HashMap<String, String>,
}

fn next_response_handle() -> u64 {
    NEXT_RESPONSE_HANDLE.fetch_add(1, Ordering::Relaxed)
}

/// Store a string and return its handle
fn store_string(s: String) -> u64 {
    let handle = super::strings::next_handle();
    STRING_STORAGE.write().unwrap().insert(handle, s);
    handle
}

/// Get string by handle
fn get_string(handle: u64) -> Option<String> {
    STRING_STORAGE.read().unwrap().get(&handle).cloned()
}

// ============================================================================
// HTTP Operations
// ============================================================================

/// HTTP GET request
/// Args: url_handle
/// Returns: response_handle (0 on error)
fn http_get(args: &[u64], _outputs: &mut [u64]) -> Result<i64, crate::runtime::ExtError> {
    let url_handle = args[0];

    let url = match get_string(url_handle) {
        Some(u) => u,
        None => return Ok(0),
    };

    match ureq::get(&url).call() {
        Ok(response) => {
            let status = response.status();
            let body = response.into_string().unwrap_or_default();
            let body_handle = store_string(body);

            let resp = HttpResponse {
                status,
                body_handle,
                headers: HashMap::new(),
            };

            let handle = next_response_handle();
            RESPONSE_STORAGE.write().unwrap().insert(handle, resp);
            Ok(handle as i64)
        }
        Err(_) => Ok(0),
    }
}

/// HTTP POST request with body
/// Args: url_handle, body_handle, content_type_handle
/// Returns: response_handle (0 on error)
fn http_post(args: &[u64], _outputs: &mut [u64]) -> Result<i64, crate::runtime::ExtError> {
    let url_handle = args[0];
    let body_handle = args[1];
    let content_type_handle = args[2];

    let url = match get_string(url_handle) {
        Some(u) => u,
        None => return Ok(0),
    };

    let body = get_string(body_handle).unwrap_or_default();
    let content_type =
        get_string(content_type_handle).unwrap_or_else(|| "application/json".to_string());

    match ureq::post(&url)
        .set("Content-Type", &content_type)
        .send_string(&body)
    {
        Ok(response) => {
            let status = response.status();
            let resp_body = response.into_string().unwrap_or_default();
            let body_handle = store_string(resp_body);

            let resp = HttpResponse {
                status,
                body_handle,
                headers: HashMap::new(),
            };

            let handle = next_response_handle();
            RESPONSE_STORAGE.write().unwrap().insert(handle, resp);
            Ok(handle as i64)
        }
        Err(_) => Ok(0),
    }
}

/// HTTP PUT request
/// Args: url_handle, body_handle, content_type_handle
/// Returns: response_handle
fn http_put(args: &[u64], _outputs: &mut [u64]) -> Result<i64, crate::runtime::ExtError> {
    let url_handle = args[0];
    let body_handle = args[1];
    let content_type_handle = args[2];

    let url = match get_string(url_handle) {
        Some(u) => u,
        None => return Ok(0),
    };

    let body = get_string(body_handle).unwrap_or_default();
    let content_type =
        get_string(content_type_handle).unwrap_or_else(|| "application/json".to_string());

    match ureq::put(&url)
        .set("Content-Type", &content_type)
        .send_string(&body)
    {
        Ok(response) => {
            let status = response.status();
            let resp_body = response.into_string().unwrap_or_default();
            let body_handle = store_string(resp_body);

            let resp = HttpResponse {
                status,
                body_handle,
                headers: HashMap::new(),
            };

            let handle = next_response_handle();
            RESPONSE_STORAGE.write().unwrap().insert(handle, resp);
            Ok(handle as i64)
        }
        Err(_) => Ok(0),
    }
}

/// HTTP DELETE request
/// Args: url_handle
/// Returns: response_handle
fn http_delete(args: &[u64], _outputs: &mut [u64]) -> Result<i64, crate::runtime::ExtError> {
    let url_handle = args[0];

    let url = match get_string(url_handle) {
        Some(u) => u,
        None => return Ok(0),
    };

    match ureq::delete(&url).call() {
        Ok(response) => {
            let status = response.status();
            let body = response.into_string().unwrap_or_default();
            let body_handle = store_string(body);

            let resp = HttpResponse {
                status,
                body_handle,
                headers: HashMap::new(),
            };

            let handle = next_response_handle();
            RESPONSE_STORAGE.write().unwrap().insert(handle, resp);
            Ok(handle as i64)
        }
        Err(_) => Ok(0),
    }
}

/// Get response status code
/// Args: response_handle
/// Returns: status code (e.g., 200, 404, 500)
fn http_response_status(
    args: &[u64],
    _outputs: &mut [u64],
) -> Result<i64, crate::runtime::ExtError> {
    let handle = args[0];

    let storage = RESPONSE_STORAGE.read().unwrap();
    match storage.get(&handle) {
        Some(resp) => Ok(resp.status as i64),
        None => Ok(0),
    }
}

/// Get response body as string handle
/// Args: response_handle
/// Returns: string_handle
fn http_response_body(args: &[u64], _outputs: &mut [u64]) -> Result<i64, crate::runtime::ExtError> {
    let handle = args[0];

    let storage = RESPONSE_STORAGE.read().unwrap();
    match storage.get(&handle) {
        Some(resp) => Ok(resp.body_handle as i64),
        None => Ok(0),
    }
}

/// Free a response
/// Args: response_handle
/// Returns: 0
fn http_response_free(args: &[u64], _outputs: &mut [u64]) -> Result<i64, crate::runtime::ExtError> {
    let handle = args[0];
    RESPONSE_STORAGE.write().unwrap().remove(&handle);
    Ok(0)
}

/// HTTP GET with headers
/// Args: url_handle, headers_json_handle
/// Returns: response_handle
fn http_get_with_headers(
    args: &[u64],
    _outputs: &mut [u64],
) -> Result<i64, crate::runtime::ExtError> {
    let url_handle = args[0];
    let headers_json_handle = args[1];

    let url = match get_string(url_handle) {
        Some(u) => u,
        None => return Ok(0),
    };

    let mut request = ureq::get(&url);

    // Parse headers from JSON if provided
    if let Some(headers_json) = get_string(headers_json_handle) {
        if let Ok(headers) = serde_json::from_str::<HashMap<String, String>>(&headers_json) {
            for (key, value) in headers {
                request = request.set(&key, &value);
            }
        }
    }

    match request.call() {
        Ok(response) => {
            let status = response.status();
            let body = response.into_string().unwrap_or_default();
            let body_handle = store_string(body);

            let resp = HttpResponse {
                status,
                body_handle,
                headers: HashMap::new(),
            };

            let handle = next_response_handle();
            RESPONSE_STORAGE.write().unwrap().insert(handle, resp);
            Ok(handle as i64)
        }
        Err(_) => Ok(0),
    }
}

/// HTTP POST with headers
/// Args: url_handle, body_handle, headers_json_handle
/// Returns: response_handle
fn http_post_with_headers(
    args: &[u64],
    _outputs: &mut [u64],
) -> Result<i64, crate::runtime::ExtError> {
    let url_handle = args[0];
    let body_handle = args[1];
    let headers_json_handle = args[2];

    let url = match get_string(url_handle) {
        Some(u) => u,
        None => return Ok(0),
    };

    let body = get_string(body_handle).unwrap_or_default();

    let mut request = ureq::post(&url);

    // Parse headers from JSON if provided
    if let Some(headers_json) = get_string(headers_json_handle) {
        if let Ok(headers) = serde_json::from_str::<HashMap<String, String>>(&headers_json) {
            for (key, value) in headers {
                request = request.set(&key, &value);
            }
        }
    }

    match request.send_string(&body) {
        Ok(response) => {
            let status = response.status();
            let resp_body = response.into_string().unwrap_or_default();
            let body_handle = store_string(resp_body);

            let resp = HttpResponse {
                status,
                body_handle,
                headers: HashMap::new(),
            };

            let handle = next_response_handle();
            RESPONSE_STORAGE.write().unwrap().insert(handle, resp);
            Ok(handle as i64)
        }
        Err(_) => Ok(0),
    }
}

// ============================================================================
// Registration
// ============================================================================

pub fn register_http_extensions(registry: &mut ExtensionRegistry) {
    use super::super::ext_ids;
    use std::sync::Arc;

    // HTTP GET (190)
    registry.register_with_id(
        ext_ids::HTTP_GET,
        "http_get",
        "HTTP GET request",
        1,
        false,
        ExtCategory::Other,
        Arc::new(http_get),
    );

    // HTTP POST (191)
    registry.register_with_id(
        ext_ids::HTTP_POST,
        "http_post",
        "HTTP POST request with body",
        3,
        false,
        ExtCategory::Other,
        Arc::new(http_post),
    );

    // HTTP PUT (192)
    registry.register_with_id(
        ext_ids::HTTP_PUT,
        "http_put",
        "HTTP PUT request",
        3,
        false,
        ExtCategory::Other,
        Arc::new(http_put),
    );

    // HTTP DELETE (193)
    registry.register_with_id(
        ext_ids::HTTP_DELETE,
        "http_delete",
        "HTTP DELETE request",
        1,
        false,
        ExtCategory::Other,
        Arc::new(http_delete),
    );

    // Response status (194)
    registry.register_with_id(
        ext_ids::HTTP_RESPONSE_STATUS,
        "http_response_status",
        "Get HTTP response status code",
        1,
        false,
        ExtCategory::Other,
        Arc::new(http_response_status),
    );

    // Response body (195)
    registry.register_with_id(
        ext_ids::HTTP_RESPONSE_BODY,
        "http_response_body",
        "Get HTTP response body as string",
        1,
        false,
        ExtCategory::Other,
        Arc::new(http_response_body),
    );

    // Response free (196)
    registry.register_with_id(
        ext_ids::HTTP_RESPONSE_FREE,
        "http_response_free",
        "Free HTTP response",
        1,
        false,
        ExtCategory::Other,
        Arc::new(http_response_free),
    );

    // GET with headers (197)
    registry.register_with_id(
        ext_ids::HTTP_GET_WITH_HEADERS,
        "http_get_with_headers",
        "HTTP GET with custom headers",
        2,
        false,
        ExtCategory::Other,
        Arc::new(http_get_with_headers),
    );

    // POST with headers (198)
    registry.register_with_id(
        ext_ids::HTTP_POST_WITH_HEADERS,
        "http_post_with_headers",
        "HTTP POST with custom headers",
        3,
        false,
        ExtCategory::Other,
        Arc::new(http_post_with_headers),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_get_invalid_handle() {
        let result = http_get(&[99999], &mut []).unwrap();
        assert_eq!(result, 0); // Invalid handle returns 0
    }

    #[test]
    fn test_response_storage() {
        let body_handle = store_string("test body".to_string());

        let resp = HttpResponse {
            status: 200,
            body_handle,
            headers: HashMap::new(),
        };

        let handle = next_response_handle();
        RESPONSE_STORAGE.write().unwrap().insert(handle, resp);

        let status = http_response_status(&[handle], &mut []).unwrap();
        assert_eq!(status, 200);

        let body = http_response_body(&[handle], &mut []).unwrap();
        assert_eq!(body, body_handle as i64);

        http_response_free(&[handle], &mut []).unwrap();
        assert!(RESPONSE_STORAGE.read().unwrap().get(&handle).is_none());
    }
}
