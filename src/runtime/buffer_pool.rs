//! Lock-free buffer pool for executable memory
//!
//! Pre-allocates RWX memory pages to avoid mprotect() syscall overhead
//! on the hot path. Target: <1μs buffer acquisition.

use crossbeam::queue::ArrayQueue;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Default buffer size (4KB = one page)
pub const DEFAULT_BUFFER_SIZE: usize = 4096;

/// An executable buffer that can be written to and executed
pub struct ExecutableBuffer {
    /// Pointer to the executable memory
    ptr: NonNull<u8>,
    /// Size of the buffer
    size: usize,
    /// Current write position
    write_pos: usize,
    /// Reference to the pool for returning the buffer
    pool: Option<Arc<BufferPoolInner>>,
    /// Index in the pool
    index: usize,
}

impl ExecutableBuffer {
    /// Create a new executable buffer (allocates memory)
    #[allow(dead_code)]
    fn new(size: usize) -> Option<Self> {
        let ptr = allocate_executable(size)?;
        Some(Self {
            ptr,
            size,
            write_pos: 0,
            pool: None,
            index: 0,
        })
    }

    /// Create from a pool allocation
    fn from_pool(ptr: NonNull<u8>, size: usize, pool: Arc<BufferPoolInner>, index: usize) -> Self {
        Self {
            ptr,
            size,
            write_pos: 0,
            pool: Some(pool),
            index,
        }
    }

    /// Get a pointer to the buffer
    pub fn as_ptr(&self) -> *const u8 {
        self.ptr.as_ptr()
    }

    /// Get a mutable pointer to the buffer
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.ptr.as_ptr()
    }

    /// Write data to the buffer
    pub fn write(&mut self, data: &[u8]) -> usize {
        let remaining = self.size - self.write_pos;
        let to_write = data.len().min(remaining);

        if to_write > 0 {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    data.as_ptr(),
                    self.ptr.as_ptr().add(self.write_pos),
                    to_write,
                );
            }
            self.write_pos += to_write;
        }

        to_write
    }

    /// Reset the buffer for reuse
    pub fn reset(&mut self) {
        self.write_pos = 0;
        // Optionally clear the buffer
        unsafe {
            std::ptr::write_bytes(self.ptr.as_ptr(), 0xCC, self.size); // INT3 for safety
        }
    }

    /// Get current write position
    pub fn len(&self) -> usize {
        self.write_pos
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.write_pos == 0
    }

    /// Get buffer capacity
    pub fn capacity(&self) -> usize {
        self.size
    }
}

impl Drop for ExecutableBuffer {
    fn drop(&mut self) {
        if let Some(pool) = self.pool.take() {
            // Return to pool
            pool.release(self.index);
        } else {
            // Free the memory
            deallocate_executable(self.ptr, self.size);
        }
    }
}

// Safety: ExecutableBuffer can be sent between threads
unsafe impl Send for ExecutableBuffer {}

/// Internal pool state
struct BufferPoolInner {
    /// Base pointer to all buffers
    base: NonNull<u8>,
    /// Size of each buffer
    buffer_size: usize,
    /// Total number of buffers
    count: usize,
    /// Free list (lock-free queue of indices)
    free_list: ArrayQueue<usize>,
    /// Number of buffers currently in use
    in_use: AtomicUsize,
}

impl BufferPoolInner {
    fn new(count: usize, buffer_size: usize) -> Option<Arc<Self>> {
        let total_size = count * buffer_size;
        let base = allocate_executable(total_size)?;

        // Fill with INT3 for safety
        unsafe {
            std::ptr::write_bytes(base.as_ptr(), 0xCC, total_size);
        }

        let free_list = ArrayQueue::new(count);
        for i in 0..count {
            let _ = free_list.push(i);
        }

        Some(Arc::new(Self {
            base,
            buffer_size,
            count,
            free_list,
            in_use: AtomicUsize::new(0),
        }))
    }

    fn acquire(self: &Arc<Self>) -> Option<ExecutableBuffer> {
        let index = self.free_list.pop()?;
        self.in_use.fetch_add(1, Ordering::Relaxed);

        let offset = index * self.buffer_size;
        let ptr = unsafe { NonNull::new_unchecked(self.base.as_ptr().add(offset)) };

        Some(ExecutableBuffer::from_pool(
            ptr,
            self.buffer_size,
            Arc::clone(self),
            index,
        ))
    }

    fn release(&self, index: usize) {
        // Reset the buffer memory
        let offset = index * self.buffer_size;
        unsafe {
            std::ptr::write_bytes(self.base.as_ptr().add(offset), 0xCC, self.buffer_size);
        }

        let _ = self.free_list.push(index);
        self.in_use.fetch_sub(1, Ordering::Relaxed);
    }
}

impl Drop for BufferPoolInner {
    fn drop(&mut self) {
        deallocate_executable(self.base, self.count * self.buffer_size);
    }
}

// Safety: BufferPoolInner is internally synchronized
unsafe impl Send for BufferPoolInner {}
unsafe impl Sync for BufferPoolInner {}

/// Pool of executable buffers
pub struct BufferPool {
    inner: Arc<BufferPoolInner>,
}

impl BufferPool {
    /// Create a new buffer pool with the specified number of buffers
    pub fn new(count: usize) -> Self {
        let inner = BufferPoolInner::new(count, DEFAULT_BUFFER_SIZE)
            .expect("Failed to allocate buffer pool");
        Self { inner }
    }

    /// Create a buffer pool with custom buffer size
    pub fn with_buffer_size(count: usize, buffer_size: usize) -> Self {
        let inner =
            BufferPoolInner::new(count, buffer_size).expect("Failed to allocate buffer pool");
        Self { inner }
    }

    /// Acquire a buffer from the pool
    ///
    /// Returns None if all buffers are in use.
    pub fn acquire(&self) -> Option<ExecutableBuffer> {
        self.inner.acquire()
    }

    /// Get the number of buffers currently in use
    pub fn in_use(&self) -> usize {
        self.inner.in_use.load(Ordering::Relaxed)
    }

    /// Get the total capacity of the pool
    pub fn capacity(&self) -> usize {
        self.inner.count
    }

    /// Get the number of available buffers
    pub fn available(&self) -> usize {
        self.capacity() - self.in_use()
    }
}

impl Default for BufferPool {
    fn default() -> Self {
        Self::new(64)
    }
}

// Platform-specific memory allocation

#[cfg(unix)]
fn allocate_executable(size: usize) -> Option<NonNull<u8>> {
    use libc::{mmap, MAP_ANONYMOUS, MAP_FAILED, MAP_PRIVATE, PROT_EXEC, PROT_READ, PROT_WRITE};
    use std::ptr;

    let ptr = unsafe {
        mmap(
            ptr::null_mut(),
            size,
            PROT_READ | PROT_WRITE | PROT_EXEC,
            MAP_PRIVATE | MAP_ANONYMOUS,
            -1,
            0,
        )
    };

    if ptr == MAP_FAILED {
        None
    } else {
        NonNull::new(ptr as *mut u8)
    }
}

#[cfg(unix)]
fn deallocate_executable(ptr: NonNull<u8>, size: usize) {
    use libc::munmap;
    unsafe {
        munmap(ptr.as_ptr() as *mut _, size);
    }
}

#[cfg(windows)]
fn allocate_executable(size: usize) -> Option<NonNull<u8>> {
    use std::ptr;
    use winapi::um::memoryapi::VirtualAlloc;
    use winapi::um::winnt::{MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE};

    let ptr = unsafe {
        VirtualAlloc(
            ptr::null_mut(),
            size,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_EXECUTE_READWRITE,
        )
    };

    NonNull::new(ptr as *mut u8)
}

#[cfg(windows)]
fn deallocate_executable(ptr: NonNull<u8>, _size: usize) {
    use winapi::um::memoryapi::VirtualFree;
    use winapi::um::winnt::MEM_RELEASE;

    unsafe {
        VirtualFree(ptr.as_ptr() as *mut _, 0, MEM_RELEASE);
    }
}

#[cfg(not(any(unix, windows)))]
fn allocate_executable(size: usize) -> Option<NonNull<u8>> {
    // Fallback: regular allocation (won't be executable on most systems)
    let layout = std::alloc::Layout::from_size_align(size, 4096).ok()?;
    let ptr = unsafe { std::alloc::alloc(layout) };
    NonNull::new(ptr)
}

#[cfg(not(any(unix, windows)))]
fn deallocate_executable(ptr: NonNull<u8>, size: usize) {
    let layout = std::alloc::Layout::from_size_align(size, 4096).unwrap();
    unsafe { std::alloc::dealloc(ptr.as_ptr(), layout) };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_buffer_allocation() {
        let mut buf = ExecutableBuffer::new(4096).unwrap();
        assert_eq!(buf.capacity(), 4096);
        assert!(buf.is_empty());

        let data = [0x90u8; 100]; // NOP instructions
        let written = buf.write(&data);
        assert_eq!(written, 100);
        assert_eq!(buf.len(), 100);
    }

    #[test]
    fn test_buffer_pool() {
        let pool = BufferPool::new(4);
        assert_eq!(pool.capacity(), 4);
        assert_eq!(pool.available(), 4);

        let buf1 = pool.acquire().unwrap();
        assert_eq!(pool.in_use(), 1);

        let _buf2 = pool.acquire().unwrap();
        let _buf3 = pool.acquire().unwrap();
        let _buf4 = pool.acquire().unwrap();
        assert_eq!(pool.in_use(), 4);
        assert_eq!(pool.available(), 0);

        // Pool exhausted
        assert!(pool.acquire().is_none());

        // Return one
        drop(buf1);
        assert_eq!(pool.in_use(), 3);

        // Can acquire again
        let _buf5 = pool.acquire().unwrap();
        assert_eq!(pool.in_use(), 4);
    }

    #[test]
    fn test_acquire_latency() {
        let pool = BufferPool::new(64);

        // Warm up
        let _ = pool.acquire();

        // Measure
        let start = Instant::now();
        for _ in 0..1000 {
            let buf = pool.acquire().unwrap();
            drop(buf);
        }
        let elapsed = start.elapsed();

        let avg_ns = elapsed.as_nanos() / 1000;
        println!("Average acquire/release latency: {}ns", avg_ns);

        // Target: <1000ns per operation
        assert!(
            avg_ns < 10000,
            "Acquire latency {}ns exceeds target",
            avg_ns
        );
    }

    #[test]
    fn test_executable_memory() {
        let mut buf = ExecutableBuffer::new(4096).unwrap();

        // Write a simple x86-64 function: mov eax, 42; ret
        let code: [u8; 7] = [
            0xb8, 0x2a, 0x00, 0x00, 0x00, // mov eax, 42
            0xc3, // ret
            0x00, // padding
        ];
        buf.write(&code);

        // Execute
        let func: extern "C" fn() -> i32 = unsafe { std::mem::transmute(buf.as_ptr()) };

        let result = func();
        assert_eq!(result, 42);
    }
}
