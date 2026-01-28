//! Owned Buffer Type
//!
//! Memory-safe buffer for passing data between extension operations.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;

/// Memory-safe buffer that owns its data
#[derive(Clone, Debug)]
pub struct OwnedBuffer {
    data: Vec<u8>,
}

impl OwnedBuffer {
    /// Create a new empty buffer
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Create a buffer with a specific capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }

    /// Create a buffer with the given size, filled with zeros
    pub fn zeroed(size: usize) -> Self {
        Self {
            data: vec![0u8; size],
        }
    }

    /// Create a buffer from a Vec
    pub fn from_vec(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Create a buffer from a slice (copies data)
    pub fn from_slice(data: &[u8]) -> Self {
        Self {
            data: data.to_vec(),
        }
    }

    /// Create a buffer from a string (copies data)
    pub fn from_str(s: &str) -> Self {
        Self {
            data: s.as_bytes().to_vec(),
        }
    }

    /// Create a buffer from a String
    pub fn from_string(s: String) -> Self {
        Self {
            data: s.into_bytes(),
        }
    }

    /// Get the length of the buffer
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get the capacity of the buffer
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Get the buffer as a slice
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// Get the buffer as a mutable slice
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Consume the buffer and return the underlying Vec
    pub fn into_vec(self) -> Vec<u8> {
        self.data
    }

    /// Try to convert the buffer to a UTF-8 string
    pub fn to_string(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.data.clone())
    }

    /// Try to convert the buffer to a UTF-8 string slice
    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.data)
    }

    /// Append data to the buffer
    pub fn extend(&mut self, data: &[u8]) {
        self.data.extend_from_slice(data);
    }

    /// Append another buffer to this one
    pub fn extend_buffer(&mut self, other: &OwnedBuffer) {
        self.data.extend_from_slice(&other.data);
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Resize the buffer
    pub fn resize(&mut self, new_len: usize, value: u8) {
        self.data.resize(new_len, value);
    }

    /// Truncate the buffer to a specific length
    pub fn truncate(&mut self, len: usize) {
        self.data.truncate(len);
    }

    /// Get a byte at an index
    pub fn get(&self, index: usize) -> Option<u8> {
        self.data.get(index).copied()
    }

    /// Set a byte at an index
    pub fn set(&mut self, index: usize, value: u8) -> bool {
        if index < self.data.len() {
            self.data[index] = value;
            true
        } else {
            false
        }
    }

    /// Get a range of bytes
    pub fn get_range(&self, start: usize, end: usize) -> Option<&[u8]> {
        self.data.get(start..end)
    }

    /// Split the buffer at an index
    pub fn split_at(&self, mid: usize) -> (OwnedBuffer, OwnedBuffer) {
        let (left, right) = self.data.split_at(mid.min(self.data.len()));
        (
            OwnedBuffer::from_slice(left),
            OwnedBuffer::from_slice(right),
        )
    }

    /// Create a sub-buffer (copies data)
    pub fn slice(&self, start: usize, len: usize) -> OwnedBuffer {
        let end = (start + len).min(self.data.len());
        let start = start.min(self.data.len());
        OwnedBuffer::from_slice(&self.data[start..end])
    }
}

impl Default for OwnedBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Vec<u8>> for OwnedBuffer {
    fn from(data: Vec<u8>) -> Self {
        Self::from_vec(data)
    }
}

impl From<&[u8]> for OwnedBuffer {
    fn from(data: &[u8]) -> Self {
        Self::from_slice(data)
    }
}

impl From<&str> for OwnedBuffer {
    fn from(s: &str) -> Self {
        Self::from_str(s)
    }
}

impl From<String> for OwnedBuffer {
    fn from(s: String) -> Self {
        Self::from_string(s)
    }
}

impl AsRef<[u8]> for OwnedBuffer {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

impl AsMut<[u8]> for OwnedBuffer {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
}

impl std::ops::Deref for OwnedBuffer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl std::ops::DerefMut for OwnedBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl PartialEq for OwnedBuffer {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl Eq for OwnedBuffer {}

impl std::hash::Hash for OwnedBuffer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.data.hash(state);
    }
}

// =============================================================================
// Handle Management
// =============================================================================

/// A handle to a buffer stored in the global handle manager
pub type BufferHandle = u64;

static NEXT_HANDLE: AtomicU64 = AtomicU64::new(1);

fn next_handle() -> BufferHandle {
    NEXT_HANDLE.fetch_add(1, Ordering::Relaxed)
}

lazy_static::lazy_static! {
    /// Global storage for buffers that need to be passed across FFI boundaries
    static ref BUFFER_STORAGE: RwLock<HashMap<BufferHandle, OwnedBuffer>> = RwLock::new(HashMap::new());
}

/// Manager for buffer handles
pub struct HandleManager;

impl HandleManager {
    /// Store a buffer and return its handle
    pub fn store(buffer: OwnedBuffer) -> BufferHandle {
        let handle = next_handle();
        let mut storage = BUFFER_STORAGE.write().unwrap();
        storage.insert(handle, buffer);
        handle
    }

    /// Get a buffer by handle (clones the data)
    pub fn get(handle: BufferHandle) -> Option<OwnedBuffer> {
        let storage = BUFFER_STORAGE.read().unwrap();
        storage.get(&handle).cloned()
    }

    /// Get a reference to a buffer's data by handle
    pub fn with<F, R>(handle: BufferHandle, f: F) -> Option<R>
    where
        F: FnOnce(&OwnedBuffer) -> R,
    {
        let storage = BUFFER_STORAGE.read().unwrap();
        storage.get(&handle).map(f)
    }

    /// Mutate a buffer by handle
    pub fn with_mut<F, R>(handle: BufferHandle, f: F) -> Option<R>
    where
        F: FnOnce(&mut OwnedBuffer) -> R,
    {
        let mut storage = BUFFER_STORAGE.write().unwrap();
        storage.get_mut(&handle).map(f)
    }

    /// Remove a buffer by handle and return it
    pub fn remove(handle: BufferHandle) -> Option<OwnedBuffer> {
        let mut storage = BUFFER_STORAGE.write().unwrap();
        storage.remove(&handle)
    }

    /// Check if a handle exists
    pub fn exists(handle: BufferHandle) -> bool {
        let storage = BUFFER_STORAGE.read().unwrap();
        storage.contains_key(&handle)
    }

    /// Get the number of stored buffers
    pub fn count() -> usize {
        let storage = BUFFER_STORAGE.read().unwrap();
        storage.len()
    }

    /// Clear all stored buffers
    pub fn clear() {
        let mut storage = BUFFER_STORAGE.write().unwrap();
        storage.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_owned_buffer_creation() {
        let buf = OwnedBuffer::new();
        assert!(buf.is_empty());

        let buf = OwnedBuffer::from_slice(b"hello");
        assert_eq!(buf.len(), 5);
        assert_eq!(buf.as_slice(), b"hello");

        let buf = OwnedBuffer::from_str("world");
        assert_eq!(buf.as_str().unwrap(), "world");
    }

    #[test]
    fn test_handle_manager() {
        let buf = OwnedBuffer::from_slice(b"test data");
        let handle = HandleManager::store(buf);

        let retrieved = HandleManager::get(handle).unwrap();
        assert_eq!(retrieved.as_slice(), b"test data");

        assert!(HandleManager::exists(handle));

        let removed = HandleManager::remove(handle).unwrap();
        assert_eq!(removed.as_slice(), b"test data");

        assert!(!HandleManager::exists(handle));
    }
}
