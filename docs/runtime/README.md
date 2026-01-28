# Runtime Documentation

Runtime components for memory management and execution.

## Buffer Pool

### Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Buffer Pool Architecture                         │
└─────────────────────────────────────────────────────────────────────┘

  Initialization (once at startup):
  ┌────────────────────────────────────────────────────────────────┐
  │                                                                 │
  │    mmap() with PROT_READ | PROT_WRITE | PROT_EXEC             │
  │                        │                                        │
  │                        ▼                                        │
  │  ┌────────────────────────────────────────────────────────┐    │
  │  │    256KB Contiguous RWX Memory                          │    │
  │  ├──────┬──────┬──────┬──────┬──────┬──────┬────────────┤    │
  │  │ 4KB  │ 4KB  │ 4KB  │ 4KB  │ ...  │ 4KB  │    64 buffers │    │
  │  └──────┴──────┴──────┴──────┴──────┴──────┴────────────┘    │
  │                                                                 │
  └────────────────────────────────────────────────────────────────┘

  Free List (lock-free ArrayQueue):
  ┌─────────────────────────────────────────────────────────────────┐
  │ [0] ──▶ [1] ──▶ [2] ──▶ [3] ──▶ ... ──▶ [63]                  │
  └─────────────────────────────────────────────────────────────────┘
```

### Acquire/Release Flow

```
  Acquire (O(1), lock-free):
  ┌─────────────────────────────────────────────────────────────────┐
  │                                                                  │
  │   1. Pop index from free list                                   │
  │   2. Calculate buffer pointer = base + (index * 4096)           │
  │   3. Return ExecutableBuffer handle                              │
  │                                                                  │
  │   Time: ~200ns (no syscall!)                                     │
  │                                                                  │
  └─────────────────────────────────────────────────────────────────┘

  Release (O(1), lock-free):
  ┌─────────────────────────────────────────────────────────────────┐
  │                                                                  │
  │   1. Fill buffer with INT3 (0xCC) for safety                    │
  │   2. Push index back to free list                                │
  │                                                                  │
  │   Time: ~100ns                                                   │
  │                                                                  │
  └─────────────────────────────────────────────────────────────────┘
```

### API

```rust
use neurlang::runtime::BufferPool;

// Create pool with 64 buffers (4KB each)
let pool = BufferPool::new(64);

// Custom buffer size
let pool = BufferPool::with_buffer_size(32, 8192);

// Acquire buffer
let mut buffer = pool.acquire().expect("pool exhausted");

// Write code
buffer.write(&machine_code);

// Execute (buffer is RWX)
let func: fn() -> i32 = unsafe { std::mem::transmute(buffer.as_ptr()) };
let result = func();

// Release automatically on drop
drop(buffer);

// Check pool status
println!("In use: {}/{}", pool.in_use(), pool.capacity());
```

### ExecutableBuffer

```rust
pub struct ExecutableBuffer {
    ptr: NonNull<u8>,       // Pointer to executable memory
    size: usize,            // Buffer capacity
    write_pos: usize,       // Current write position
    pool: Arc<BufferPoolInner>,  // Reference to pool
    index: usize,           // Index in pool
}

impl ExecutableBuffer {
    // Write bytes to buffer
    pub fn write(&mut self, data: &[u8]) -> usize;

    // Get pointer for execution
    pub fn as_ptr(&self) -> *const u8;

    // Reset for reuse
    pub fn reset(&mut self);

    // Current usage
    pub fn len(&self) -> usize;
    pub fn capacity(&self) -> usize;
}
```

## Why Pre-allocated RWX?

```
┌─────────────────────────────────────────────────────────────────────┐
│              Traditional JIT vs Neurlang                                │
└─────────────────────────────────────────────────────────────────────┘

  Traditional JIT (per compilation):
  ┌───────────────────────────────────────────────────────────────┐
  │                                                                │
  │  1. mmap() with PROT_READ | PROT_WRITE       ~30μs            │
  │  2. Write machine code                                         │
  │  3. mprotect() to PROT_READ | PROT_EXEC      ~50μs            │
  │  4. Execute                                                    │
  │  5. munmap()                                  ~20μs            │
  │                                                                │
  │  Syscall overhead: ~100μs per compile!                         │
  │                                                                │
  └───────────────────────────────────────────────────────────────┘

  Neurlang (with buffer pool):
  ┌───────────────────────────────────────────────────────────────┐
  │                                                                │
  │  Startup (once):                                               │
  │    mmap() 256KB with RWX                      ~50μs            │
  │                                                                │
  │  Per compilation:                                              │
  │    1. Pop from free list                      ~50ns            │
  │    2. memcpy stencils                        ~2000ns           │
  │    3. Execute                                                  │
  │    4. Push to free list                       ~50ns            │
  │                                                                │
  │  NO syscalls on hot path!                                      │
  │                                                                │
  └───────────────────────────────────────────────────────────────┘
```

## Memory Safety

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Safety Measures                                   │
└─────────────────────────────────────────────────────────────────────┘

  1. INT3 Padding
     ┌─────────────────────────────────────────────────────────────┐
     │ Unused bytes filled with 0xCC (INT3 / breakpoint)           │
     │ Prevents execution of uninitialized memory                  │
     └─────────────────────────────────────────────────────────────┘

  2. Bounds Checking
     ┌─────────────────────────────────────────────────────────────┐
     │ write() checks buffer capacity                              │
     │ Returns bytes written (may be less than requested)          │
     └─────────────────────────────────────────────────────────────┘

  3. RAII Cleanup
     ┌─────────────────────────────────────────────────────────────┐
     │ Buffer automatically returned to pool on drop               │
     │ Memory cleared before reuse                                 │
     └─────────────────────────────────────────────────────────────┘
```

## Platform Support

| Platform | Memory API | Notes |
|----------|------------|-------|
| Linux | `mmap` + `PROT_*` | Full support |
| macOS | `mmap` + `PROT_*` | Works with restrictions |
| Windows | `VirtualAlloc` | `PAGE_EXECUTE_READWRITE` |
| Other | Fallback allocator | May not be executable |

## Configuration

```rust
// Default: 64 buffers × 4KB = 256KB
const DEFAULT_BUFFER_SIZE: usize = 4096;

// Tune for workload
let pool = BufferPool::new(128);  // More concurrent compilations
let pool = BufferPool::with_buffer_size(32, 8192);  // Larger programs
```

---

## Runtime Library Functions

The runtime is a ~100KB library linked into every Neurlang program, providing services that cannot be efficiently inlined.

### Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Runtime Library (~100KB)                         │
└─────────────────────────────────────────────────────────────────────┘

  The runtime is NOT an interpreter. It's a library of helper functions
  called by generated native code, similar to libc for C programs.

  Generated Code:                    Runtime Library:
  ┌─────────────────┐               ┌─────────────────────────────┐
  │ mov rdi, fd     │               │ pub extern "C" fn           │
  │ mov rsi, buf    │ ──────────▶   │ neurlang_file_read(            │
  │ mov rdx, len    │   call        │   fd: u64,                  │
  │ call neurlang_file │               │   buf: u64,                 │
  │      _read      │               │   len: u64                  │
  └─────────────────┘               │ ) -> i64                    │
                                    └─────────────────────────────┘
```

### Function Categories

| Category | Functions | Purpose |
|----------|-----------|---------|
| **I/O** | `neurlang_file_*`, `neurlang_net_*` | OS abstraction for file/network |
| **Concurrency** | `neurlang_spawn`, `neurlang_join`, `neurlang_chan_*` | Task scheduling, channels |
| **Security** | `neurlang_cap_*` | Capability creation and validation |
| **Time** | `neurlang_time_*` | Time operations |
| **Random** | `neurlang_rand_*` | Cryptographic random numbers |

### I/O Functions

```rust
// File operations
#[no_mangle]
pub extern "C" fn neurlang_file_open(
    path: *const u8,
    path_len: u64,
    flags: u64
) -> i64;  // Returns fd or negative error

#[no_mangle]
pub extern "C" fn neurlang_file_read(
    fd: u64,
    buf: u64,    // Pointer to buffer (capability-checked)
    len: u64
) -> i64;  // Returns bytes read or negative error

#[no_mangle]
pub extern "C" fn neurlang_file_write(
    fd: u64,
    buf: u64,
    len: u64
) -> i64;  // Returns bytes written or negative error

#[no_mangle]
pub extern "C" fn neurlang_file_close(fd: u64) -> i64;

// Network operations
#[no_mangle]
pub extern "C" fn neurlang_net_connect(
    host: *const u8,
    host_len: u64,
    port: u64
) -> i64;  // Returns socket fd or negative error

#[no_mangle]
pub extern "C" fn neurlang_net_send(
    fd: u64,
    buf: u64,
    len: u64
) -> i64;

#[no_mangle]
pub extern "C" fn neurlang_net_recv(
    fd: u64,
    buf: u64,
    len: u64
) -> i64;
```

### Concurrency Functions

```rust
#[no_mangle]
pub extern "C" fn neurlang_spawn(
    entry: u64,      // Function pointer
    arg: u64         // Argument to pass
) -> u64;  // Returns task ID

#[no_mangle]
pub extern "C" fn neurlang_join(task_id: u64) -> u64;  // Returns result

#[no_mangle]
pub extern "C" fn neurlang_chan_create() -> u64;  // Returns channel ID

#[no_mangle]
pub extern "C" fn neurlang_chan_send(
    chan_id: u64,
    value: u64
) -> u64;  // Returns 0 on success

#[no_mangle]
pub extern "C" fn neurlang_chan_recv(chan_id: u64) -> u64;  // Blocks until value

#[no_mangle]
pub extern "C" fn neurlang_chan_close(chan_id: u64) -> u64;
```

### Example: neurlang_file_read Implementation

```rust
#[no_mangle]
pub extern "C" fn neurlang_file_read(fd: u64, buf: u64, len: u64) -> i64 {
    // 1. Validate file descriptor
    let file = match FILE_TABLE.get(fd as usize) {
        Some(f) => f,
        None => return -EBADF,
    };

    // 2. Check I/O permissions (sandboxing)
    if !PERMISSIONS.file_read {
        return -EPERM;
    }

    // 3. Validate buffer capability
    let cap = Capability::from_raw(buf);
    if !cap.has_write_permission() {
        return -EFAULT;
    }
    if cap.length() < len {
        return -EFAULT;
    }

    // 4. Perform the actual syscall
    let ptr = cap.base_address() as *mut u8;
    let slice = unsafe { std::slice::from_raw_parts_mut(ptr, len as usize) };

    match file.read(slice) {
        Ok(n) => n as i64,
        Err(e) => -(e.raw_os_error().unwrap_or(EIO) as i64),
    }
}
```

### Platform Abstraction

Same function signature, different implementations per platform:

```
┌─────────────────────────────────────────────────────────────────────┐
│                  Platform Abstraction Layer                          │
└─────────────────────────────────────────────────────────────────────┘

  neurlang_file_read(fd, buf, len)
          │
          ├──── Linux ────▶ read() syscall (syscall number 0)
          │
          ├──── macOS ────▶ read() syscall (syscall number 0x2000003)
          │
          └──── Windows ──▶ ReadFile() API

  Same Neurlang code works on all platforms!
```

| Function | Linux | macOS | Windows |
|----------|-------|-------|---------|
| file_read | `read()` | `read()` | `ReadFile()` |
| file_write | `write()` | `write()` | `WriteFile()` |
| net_socket | `socket()` | `socket()` | `WSASocket()` |
| net_connect | `connect()` | `connect()` | `WSAConnect()` |
| spawn | `clone()` | `pthread_create()` | `CreateThread()` |
| time_now | `clock_gettime()` | `gettimeofday()` | `GetSystemTimeAsFileTime()` |

### I/O Permissions (Sandboxing)

```rust
/// Runtime I/O permissions - deny by default
pub struct IOPermissions {
    pub file_read: bool,           // Allow FILE.read
    pub file_write: bool,          // Allow FILE.write
    pub file_paths: Vec<PathBuf>,  // Allowed paths (whitelist)
    pub net_connect: bool,         // Allow NET.connect
    pub net_listen: bool,          // Allow NET.listen
    pub net_hosts: Vec<String>,    // Allowed hosts (whitelist)
    pub io_print: bool,            // Allow IO.print
    pub io_read: bool,             // Allow IO.read_line
}

impl Default for IOPermissions {
    fn default() -> Self {
        // Deny-by-default is safe
        Self {
            file_read: false,
            file_write: false,
            file_paths: vec![],
            net_connect: false,
            net_listen: false,
            net_hosts: vec![],
            io_print: true,   // Allow print by default (safe)
            io_read: false,
        }
    }
}
```

### Runtime Size Comparison

| Language | Model | Runtime Size |
|----------|-------|--------------|
| C | Native + libc | ~2MB |
| Rust | Native + std | ~300KB |
| Go | Native + runtime | ~2MB (GC, scheduler) |
| **Neurlang** | **Native + runtime** | **~100KB** |
| Java | Bytecode + JVM | ~200MB |
| Python | Interpreted | ~50MB |

**Neurlang is like Rust/Go: native code + small runtime library.**

---

---

## Extension Registry (Tier 2)

The Extension Registry provides a mechanism for calling Rust functions from Neurlang code.
This enables complex operations (crypto, JSON, regex) to be implemented in safe Rust.

### Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Extension Registry                               │
└─────────────────────────────────────────────────────────────────────┘

  Neurlang Code                     Extension Registry
  ┌───────────────────┐         ┌─────────────────────────────────────┐
  │ ext.call r0,      │         │  by_id: HashMap<u32, ExtEntry>      │
  │    sha256, r1, r2 │ ──────▶ │  by_name: HashMap<String, u32>      │
  └───────────────────┘         │                                     │
                                │  Built-in Crypto:                   │
                                │  ├─ sha256 (ID 1)                   │
                                │  ├─ hmac_sha256 (ID 2)              │
                                │  ├─ aes256_gcm_encrypt (ID 3)       │
                                │  ├─ aes256_gcm_decrypt (ID 4)       │
                                │  ├─ constant_time_eq (ID 5)         │
                                │  ├─ secure_random (ID 6)            │
                                │  ├─ pbkdf2_sha256 (ID 7)            │
                                │  ├─ ed25519_sign (ID 8)             │
                                │  ├─ ed25519_verify (ID 9)           │
                                │  └─ x25519_derive (ID 10)           │
                                └─────────────────────────────────────┘
```

### SafeBuffer and Capability Bridge

Extensions receive restricted capabilities, not raw pointers:

```rust
pub struct SafeBuffer {
    base: *const u8,
    length: usize,
    permissions: CapPermissions,
}

impl SafeBuffer {
    /// Read bytes from buffer (checks CAP_READ)
    pub fn read(&self, offset: usize, len: usize) -> Result<&[u8], ExtError>;

    /// Write bytes to buffer (checks CAP_WRITE)
    pub fn write(&mut self, offset: usize, data: &[u8]) -> Result<(), ExtError>;

    /// Restrict permissions (can only shrink)
    pub fn restrict(&self, new_perms: CapPermissions) -> SafeBuffer;
}
```

### Registering Custom Extensions

```rust
use neurlang::runtime::{ExtensionRegistry, ExtCategory, ExtError};
use std::sync::Arc;

let mut registry = ExtensionRegistry::new();

// Register a custom extension
let id = registry.register(
    "my_hash",
    "Custom hash function for domain-specific data",
    2,  // argument count
    true,  // propagate taint
    ExtCategory::Crypto,
    Arc::new(|args, outputs| {
        // args[0] = input pointer
        // args[1] = input length
        // Compute hash...
        outputs[0] = 0;  // Success
        Ok(0)
    }),
);

// Call the extension
let result = registry.call(id, &[input_ptr, input_len], &mut outputs)?;
```

### Built-in Crypto Extensions

| ID | Name | Description | Args |
|----|------|-------------|------|
| 1 | sha256 | SHA-256 hash | input_ptr, input_len, output_ptr |
| 2 | hmac_sha256 | HMAC-SHA256 | key_ptr, key_len, data_ptr, data_len, output_ptr |
| 3 | aes256_gcm_encrypt | AES-256-GCM encrypt | key, nonce, plain, plain_len, cipher, tag |
| 4 | aes256_gcm_decrypt | AES-256-GCM decrypt | key, nonce, cipher, cipher_len, tag, plain |
| 5 | constant_time_eq | Constant-time compare | ptr1, ptr2, len |
| 6 | secure_random | Cryptographic RNG | output_ptr, len |
| 7 | pbkdf2_sha256 | PBKDF2 key derivation | password, pass_len, salt, salt_len, iters, out, out_len |
| 8 | ed25519_sign | Ed25519 signature | secret_key, message, msg_len, signature |
| 9 | ed25519_verify | Ed25519 verify | public_key, message, msg_len, signature |
| 10 | x25519_derive | X25519 key exchange | private_key, public_key, shared_secret |

### Why Use Extensions?

- **Silent bugs are dangerous**: Crypto must be correct on first try
- **Timing attacks**: Extensions use constant-time implementations
- **Ecosystem access**: Use Rust crates (serde, ring, etc.)
- **Domain-specific**: Users add their own business logic

---

---

## Standard Library Extensions (Stdlib)

The Standard Library provides common data structures and utilities as EXT.CALL extensions. These are implemented in Rust and accessed through extension IDs 100+.

### Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Standard Library Extensions                       │
└─────────────────────────────────────────────────────────────────────┘

  Handle-Based Approach:
  ┌─────────────────────────────────────────────────────────────────┐
  │                                                                  │
  │  Neurlang Code              Rust Runtime                        │
  │  ┌────────────────┐        ┌───────────────────────────────┐   │
  │  │ ext.call r0,   │        │ VEC_STORAGE: RwLock<HashMap>   │   │
  │  │   100, r0, r0  │ ─────▶ │   handle → Vec<u64>            │   │
  │  │ ; vec_new()    │        │                                 │   │
  │  │ ; r0 = handle  │        │ Returns: unique u64 handle     │   │
  │  └────────────────┘        └───────────────────────────────┘   │
  │                                                                  │
  │  IR registers only hold u64, so complex data structures         │
  │  are stored in Rust HashMaps and accessed via handles.          │
  │                                                                  │
  └─────────────────────────────────────────────────────────────────┘
```

### Extension ID Ranges

| Category | ID Range | Description |
|----------|----------|-------------|
| **Crypto** | 1-99 | SHA, AES, Ed25519, etc. |
| **Vec** | 100-119 | Dynamic array operations |
| **HashMap** | 120-139 | Key-value store operations |
| **String** | 140-169 | String manipulation |
| **JSON** | 170-189 | JSON parsing and building |

### Vec Operations (100-112)

```
┌────────────────────────────────────────────────────────────────┐
│  Vec<u64> - Dynamic Array                                       │
├────────────────────────────────────────────────────────────────┤
│  ID  │ Name            │ Args                │ Returns         │
├────────────────────────────────────────────────────────────────┤
│ 100  │ vec_new         │ -                   │ handle          │
│ 101  │ vec_with_cap    │ capacity            │ handle          │
│ 102  │ vec_push        │ handle, value       │ 0               │
│ 103  │ vec_pop         │ handle              │ value or 0      │
│ 104  │ vec_get         │ handle, index       │ value           │
│ 105  │ vec_set         │ handle, index,value │ 0               │
│ 106  │ vec_len         │ handle              │ length          │
│ 107  │ vec_capacity    │ handle              │ capacity        │
│ 108  │ vec_clear       │ handle              │ 0               │
│ 109  │ vec_free        │ handle              │ 0               │
│ 111  │ vec_insert      │ handle, idx, value  │ 0               │
│ 112  │ vec_remove      │ handle, index       │ removed value   │
└────────────────────────────────────────────────────────────────┘
```

**Example: Vec usage**
```nasm
; Create vector and add elements
ext.call r1, 100, r0, r0      ; r1 = vec_new()
mov r2, 42
ext.call r0, 102, r1, r2      ; vec_push(r1, 42)
mov r2, 100
ext.call r0, 102, r1, r2      ; vec_push(r1, 100)
ext.call r0, 106, r1, r0      ; r0 = vec_len(r1) → 2
ext.call r0, 103, r1, r0      ; r0 = vec_pop(r1) → 100
ext.call r0, 109, r1, r0      ; vec_free(r1)
halt
```

### HashMap Operations (120-129)

```
┌────────────────────────────────────────────────────────────────┐
│  HashMap<u64, u64> - Key-Value Store                            │
├────────────────────────────────────────────────────────────────┤
│  ID  │ Name             │ Args              │ Returns          │
├────────────────────────────────────────────────────────────────┤
│ 120  │ hashmap_new      │ -                 │ handle           │
│ 121  │ hashmap_insert   │ handle, key,value │ old_value or 0   │
│ 122  │ hashmap_get      │ handle, key       │ value or 0       │
│ 123  │ hashmap_remove   │ handle, key       │ removed or 0     │
│ 124  │ hashmap_contains │ handle, key       │ 1 or 0           │
│ 125  │ hashmap_len      │ handle            │ count            │
│ 126  │ hashmap_clear    │ handle            │ 0                │
│ 127  │ hashmap_free     │ handle            │ 0                │
│ 128  │ hashmap_keys     │ handle            │ vec_handle       │
│ 129  │ hashmap_values   │ handle            │ vec_handle       │
└────────────────────────────────────────────────────────────────┘
```

**Example: HashMap usage**
```nasm
; Create hashmap and store values
ext.call r1, 120, r0, r0      ; r1 = hashmap_new()
mov r2, 1                      ; key = 1
mov r3, 100                    ; value = 100
ext.call r0, 121, r1, r2      ; hashmap_insert(r1, 1, 100)
mov r2, 2
mov r3, 200
ext.call r0, 121, r1, r2      ; hashmap_insert(r1, 2, 200)
mov r2, 1
ext.call r0, 122, r1, r2      ; r0 = hashmap_get(r1, 1) → 100
ext.call r0, 127, r1, r0      ; hashmap_free(r1)
halt
```

### String Operations (140-158)

```
┌────────────────────────────────────────────────────────────────┐
│  String - UTF-8 String                                          │
├────────────────────────────────────────────────────────────────┤
│  ID  │ Name              │ Args               │ Returns        │
├────────────────────────────────────────────────────────────────┤
│ 140  │ string_new        │ -                  │ handle         │
│ 141  │ string_from_bytes │ ptr, len           │ handle         │
│ 142  │ string_len        │ handle             │ byte_length    │
│ 143  │ string_concat     │ handle1, handle2   │ new_handle     │
│ 144  │ string_substr     │ handle, start, len │ new_handle     │
│ 145  │ string_find       │ haystack, needle   │ index or -1    │
│ 146  │ string_replace    │ handle, old, new   │ new_handle     │
│ 147  │ string_split      │ handle, delim      │ vec_handle     │
│ 148  │ string_trim       │ handle             │ new_handle     │
│ 149  │ string_to_upper   │ handle             │ new_handle     │
│ 150  │ string_to_lower   │ handle             │ new_handle     │
│ 151  │ string_starts_with│ handle, prefix     │ 1 or 0         │
│ 152  │ string_ends_with  │ handle, suffix     │ 1 or 0         │
│ 153  │ string_to_bytes   │ handle             │ vec_handle     │
│ 154  │ string_free       │ handle             │ 0              │
│ 155  │ string_parse_int  │ handle             │ integer        │
│ 156  │ string_parse_float│ handle             │ f64_bits       │
│ 157  │ string_from_int   │ value              │ handle         │
│ 158  │ string_from_float │ f64_bits           │ handle         │
└────────────────────────────────────────────────────────────────┘
```

### JSON Operations (170-181)

```
┌────────────────────────────────────────────────────────────────┐
│  JSON - JSON Value                                              │
├────────────────────────────────────────────────────────────────┤
│  ID  │ Name             │ Args              │ Returns          │
├────────────────────────────────────────────────────────────────┤
│ 170  │ json_parse       │ string_handle     │ json_handle      │
│ 171  │ json_stringify   │ json_handle       │ string_handle    │
│ 172  │ json_get         │ json, key_handle  │ json_handle      │
│ 173  │ json_set         │ json, key, value  │ 0                │
│ 174  │ json_get_type    │ json_handle       │ type (0-5)       │
│ 175  │ json_array_len   │ json_handle       │ length           │
│ 176  │ json_array_get   │ json, index       │ json_handle      │
│ 177  │ json_array_push  │ json, value       │ 0                │
│ 178  │ json_object_keys │ json_handle       │ vec<string>      │
│ 179  │ json_free        │ json_handle       │ 0                │
│ 180  │ json_new_object  │ -                 │ json_handle      │
│ 181  │ json_new_array   │ -                 │ json_handle      │
└────────────────────────────────────────────────────────────────┘

JSON Types: NULL=0, BOOL=1, NUMBER=2, STRING=3, ARRAY=4, OBJECT=5
```

### Memory Management

```
┌─────────────────────────────────────────────────────────────────┐
│                    Stdlib Memory Management                       │
└─────────────────────────────────────────────────────────────────┘

  Handle Lifecycle:
  ┌─────────────────────────────────────────────────────────────┐
  │                                                              │
  │  1. CREATE:  ext.call r0, 100, r0, r0   ; vec_new()         │
  │              └─ Allocates Vec in VEC_STORAGE                │
  │              └─ Returns unique handle in r0                 │
  │                                                              │
  │  2. USE:     ext.call r0, 102, r1, r2   ; vec_push()        │
  │              └─ Looks up handle in VEC_STORAGE              │
  │              └─ Performs operation on Vec                   │
  │                                                              │
  │  3. FREE:    ext.call r0, 109, r1, r0   ; vec_free()        │
  │              └─ Removes handle from VEC_STORAGE             │
  │              └─ Rust drops the Vec                          │
  │                                                              │
  │  Thread Safety: Global storage uses RwLock                  │
  │  Handle Uniqueness: AtomicU64 counter                       │
  │                                                              │
  └─────────────────────────────────────────────────────────────┘
```

### Performance Considerations

| Aspect | Impact |
|--------|--------|
| **EXT.CALL overhead** | ~15ns per call |
| **Handle lookup** | O(1) HashMap lookup |
| **Global lock** | RwLock (reads parallel, writes exclusive) |
| **Memory** | Rust manages allocation/deallocation |

**Best practices:**
- Reuse handles instead of creating new ones
- Free handles when done to prevent memory leaks
- Batch operations when possible to reduce call overhead

---

---

## Multi-Worker Strategies

Neurlang supports multiple worker strategies for high-performance server workloads. The runtime automatically detects the best available strategy.

### Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Multi-Worker Strategies                          │
└─────────────────────────────────────────────────────────────────────┘

  Strategy 1: SO_REUSEPORT (Linux, macOS, FreeBSD)
  ┌─────────────────────────────────────────────────────────────────┐
  │                                                                  │
  │  Worker 0 ─────▶ Socket (port 8080) ◀───┐                       │
  │  Worker 1 ─────▶ Socket (port 8080) ◀───┼── Kernel load-balances│
  │  Worker 2 ─────▶ Socket (port 8080) ◀───┘   incoming connections│
  │  Worker 3 ─────▶ Socket (port 8080) ◀───                        │
  │                                                                  │
  │  Each worker has its own socket bound to the same port.         │
  │  Kernel distributes connections across workers.                  │
  │                                                                  │
  └─────────────────────────────────────────────────────────────────┘

  Strategy 2: Shared Listener (Windows, fallback)
  ┌─────────────────────────────────────────────────────────────────┐
  │                                                                  │
  │  Shared Socket (port 8080)                                       │
  │         │                                                        │
  │         ▼                                                        │
  │  ┌─────────────────────────────────────────────────────────┐    │
  │  │              Arc<TcpListener>                            │    │
  │  │    Workers compete for accept() on shared listener       │    │
  │  └─────────────────────────────────────────────────────────┘    │
  │         │                                                        │
  │         ├──── Worker 0 (accepts connection)                     │
  │         ├──── Worker 1 (accepts connection)                     │
  │         ├──── Worker 2 (accepts connection)                     │
  │         └──── Worker 3 (accepts connection)                     │
  │                                                                  │
  └─────────────────────────────────────────────────────────────────┘
```

### Strategy Comparison

| Strategy | Platform | Performance | Notes |
|----------|----------|-------------|-------|
| **SO_REUSEPORT** | Linux, macOS, FreeBSD | Highest | Kernel-level load balancing, no contention |
| **Shared Listener** | Windows, all platforms | Good | Workers compete for accept(), some contention |
| **Single-Threaded** | All | Baseline | No parallelism, simplest |

### Auto-Detection

The runtime automatically selects the best strategy:

```rust
impl WorkerStrategy {
    pub fn detect() -> Self {
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))]
        { WorkerStrategy::ReusePort }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "freebsd")))]
        { WorkerStrategy::SharedListener }
    }
}
```

### CLI Usage

```bash
# Single-threaded (default)
nl run -i server.nl

# 4 workers with auto-detected strategy
nl run -i server.nl --workers 4

# Explicit strategy selection
nl run -i server.nl --workers 4 --strategy reuseport
nl run -i server.nl --workers 4 --strategy shared
```

### Performance Results (Neurlang vs nginx)

Benchmark: 10,000 requests, 50 concurrency, Docker bridge network

| Configuration | Requests/sec | vs nginx |
|---------------|--------------|----------|
| nginx (multi-worker) | 54,218 | baseline |
| Neurlang (workers=0) | 51,001 | 0.94x |
| Neurlang (workers=1, SO_REUSEPORT) | 61,189 | **1.12x** |
| Neurlang (workers=2, SO_REUSEPORT) | 59,982 | **1.10x** |
| Neurlang (workers=4, SO_REUSEPORT) | 61,716 | **1.13x** |

**Key findings:**
- With multi-worker support, Neurlang outperforms nginx by 10-13%
- Even a single worker using SO_REUSEPORT gives 12% better performance
- Diminishing returns beyond 2-4 workers for this workload

### Implementation Details

**SO_REUSEPORT:**
```rust
// Each worker creates its own socket
let socket = Socket::new(Domain::IPV4, Type::STREAM, None)?;
socket.set_reuse_address(true)?;
socket.set_reuse_port(true)?;  // Key option
socket.bind(&addr)?;
socket.listen(128)?;
```

**Shared Listener:**
```rust
// First worker creates the listener
let listener = Arc::new(TcpListener::bind(addr)?);

// Other workers share it
thread::spawn(move || {
    loop {
        let (stream, _) = shared_listener.accept()?;
        handle_connection(stream);
    }
});
```

---

---

## Async I/O Runtime

The Async I/O Runtime provides non-blocking I/O operations with platform-specific event loops for high-performance network and file operations.

### Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        ASYNC I/O RUNTIME                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  EventLoop (platform-specific):                                      │
│  ├── Linux:  epoll (edge-triggered)                                  │
│  ├── macOS:  kqueue                                                  │
│  └── Windows: IOCP                                                   │
│                                                                      │
│  Async Primitives:                                                   │
│  ├── AsyncSocket - TCP/UDP with connect/accept/send/recv             │
│  ├── AsyncFile   - Non-blocking read/write/seek                      │
│  ├── TimerWheel  - Efficient timeout management                      │
│  └── Executor    - Task scheduling with wakers                       │
│                                                                      │
│  Core Components:                                                    │
│  ├── Reactor     - Manages I/O resource readiness state              │
│  ├── Token       - Unique identifier for registered resources        │
│  └── Interest    - Read/write/error interest flags                   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### Components

| Component | File | Purpose |
|-----------|------|---------|
| `AsyncRuntime` | `async_io/mod.rs` | Main runtime combining all components |
| `EventLoop` | `async_io/event_loop.rs` | Platform-specific event notification |
| `Reactor` | `async_io/reactor.rs` | I/O resource readiness tracking |
| `Executor` | `async_io/executor.rs` | Task scheduling with ready/blocked queues |
| `AsyncSocket` | `async_io/socket.rs` | Non-blocking TCP/UDP socket wrapper |
| `AsyncFile` | `async_io/file.rs` | Non-blocking file I/O operations |
| `TimerWheel` | `async_io/timer.rs` | Hierarchical timer wheel for timeouts |

### Usage

```rust
use neurlang::runtime::{AsyncRuntime, Interest};

// Create the async runtime
let mut runtime = AsyncRuntime::new()?;

// Register a file descriptor
let token = runtime.register_fd(socket_fd, Interest::READABLE)?;

// Poll for events
runtime.poll(Some(Duration::from_millis(100)))?;

// Check if ready
if runtime.is_ready(token, Interest::READABLE) {
    // Perform non-blocking read
    runtime.clear_ready(token, Interest::READABLE);
}

// Schedule a timer
let timer_token = runtime.schedule_timer(Duration::from_secs(5));

// Run the event loop
runtime.run(Some(Duration::from_secs(30)))?;
```

### AsyncSocket

```rust
use neurlang::runtime::AsyncSocket;

// Create a TCP socket
let mut socket = AsyncSocket::tcp()?;
socket.set_reuse_addr(true)?;
socket.set_nodelay(true)?;

// Non-blocking connect
match socket.connect(addr) {
    Ok(()) => println!("Connected immediately"),
    Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
        // Connection in progress, wait for writable
    }
    Err(e) => return Err(e),
}

// Non-blocking I/O
let bytes_read = socket.read(&mut buffer)?;
let bytes_written = socket.write(&data)?;
```

### AsyncFile

```rust
use neurlang::runtime::{AsyncFile, OpenOptions};

// Open for reading
let mut file = AsyncFile::open_read("data.txt")?;

// Create/truncate
let mut file = AsyncFile::create("output.txt")?;

// With options
let mut file = AsyncFile::open(
    "log.txt",
    OpenOptions::new().write(true).append(true).create(true)
)?;

// Non-blocking operations
let bytes = file.read(&mut buffer)?;
file.write(b"Hello, world!")?;
file.sync_all()?;
```

### TimerWheel

```rust
use neurlang::runtime::TimerWheel;

let mut timers = TimerWheel::new();

// Insert timers
timers.insert_after(token1, Duration::from_secs(5));
timers.insert_after(token2, Duration::from_secs(10));

// Advance time
timers.advance(Instant::now());

// Check for expired timers
for token in timers.expired_timers() {
    handle_timeout(token);
}

// Get next expiry for poll timeout
let timeout = timers.timeout_until_next();
```

### Platform Event Loop Details

**Linux (epoll):**
- Uses edge-triggered mode for efficiency
- Supports EPOLLONESHOT for thread-safe operation
- Handles EPOLLERR and EPOLLHUP automatically

**macOS/BSD (kqueue):**
- Uses EV_CLEAR for edge-triggered behavior
- Separate filters for read and write events
- Supports EV_EOF for connection close detection

**Windows (IOCP):**
- Completion-based model (vs readiness-based)
- Uses overlapped I/O for async operations
- Integrates with Windows thread pool

### Performance Characteristics

| Operation | Time |
|-----------|------|
| Event loop poll (no events) | ~1μs |
| Register/deregister fd | ~100ns |
| Timer insert/remove | O(1) |
| Timer wheel advance | O(expired) |
| Socket read/write | ~100ns + kernel |

---

## See Also

- [How It Works](../architecture/how-it-works.md)
- [Security Model](../security/README.md)
- [Concurrency Runtime](../concurrency/README.md)
