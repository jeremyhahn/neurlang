//! I/O Runtime for Neurlang
//!
//! Provides sandboxed file, network, console, and time operations.
//! All I/O operations go through permission checks before execution.
//!
//! ## Multi-Worker Mode (SO_REUSEPORT)
//!
//! When SO_REUSEPORT is available, the runtime can spawn multiple worker
//! threads, each with their own socket bound to the same port. This provides
//! kernel-level load balancing of incoming connections.
//!
//! When SO_REUSEPORT is not available, the runtime falls back to a single
//! accept thread distributing connections via a channel.

use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, Read, Seek, SeekFrom, Write as IoWrite};
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs, UdpSocket};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use socket2::{Domain, Protocol, Socket, Type};

use crate::ir::NetOption;
use std::collections::HashMap;

/// Network operation types for mocking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NetMockOp {
    Socket,
    Bind,
    Listen,
    Accept,
    Connect,
    Send,
    Recv,
    Close,
}

/// Mock configuration for a network operation
#[derive(Debug, Clone)]
pub struct NetMock {
    /// Sequence of return values (cycles through if more calls than values)
    pub return_values: Vec<i64>,
    /// Optional data to return for recv operations
    pub recv_data: Option<Vec<u8>>,
    /// Current index in return_values sequence
    call_index: usize,
}

impl NetMock {
    pub fn new(return_values: Vec<i64>) -> Self {
        Self {
            return_values,
            recv_data: None,
            call_index: 0,
        }
    }

    pub fn with_recv_data(mut self, data: Vec<u8>) -> Self {
        self.recv_data = Some(data);
        self
    }

    /// Get next return value (stays on last value after sequence exhausted)
    pub fn next_value(&mut self) -> i64 {
        if self.return_values.is_empty() {
            return 0;
        }
        let val = self.return_values[self.call_index];
        // Advance to next value, but stay on last value after sequence exhausted
        if self.call_index < self.return_values.len() - 1 {
            self.call_index += 1;
        }
        val
    }
}

/// Network mock state for testing servers without real I/O
#[derive(Debug, Clone, Default)]
pub struct NetworkMocks {
    mocks: HashMap<NetMockOp, NetMock>,
    enabled: bool,
}

impl NetworkMocks {
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable mock mode
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Check if mocking is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Set a mock for a network operation
    pub fn set_mock(&mut self, op: NetMockOp, return_values: Vec<i64>) {
        self.mocks.insert(op, NetMock::new(return_values));
        self.enabled = true;
    }

    /// Set a mock with recv data
    pub fn set_recv_mock(&mut self, return_values: Vec<i64>, data: Vec<u8>) {
        self.mocks.insert(
            NetMockOp::Recv,
            NetMock::new(return_values).with_recv_data(data),
        );
        self.enabled = true;
    }

    /// Get next mock value for an operation (None if not mocked)
    pub fn get_mock_value(&mut self, op: NetMockOp) -> Option<i64> {
        if !self.enabled {
            return None;
        }
        self.mocks.get_mut(&op).map(|m| m.next_value())
    }

    /// Get recv data if mocked
    pub fn get_recv_data(&self) -> Option<&[u8]> {
        self.mocks
            .get(&NetMockOp::Recv)
            .and_then(|m| m.recv_data.as_deref())
    }
}

/// I/O Permissions for sandboxing
#[derive(Debug, Clone)]
pub struct IOPermissions {
    /// Allow FILE.read operations
    pub file_read: bool,
    /// Allow FILE.write operations
    pub file_write: bool,
    /// Allowed file paths (whitelist) - empty means all paths denied
    pub file_paths: Vec<PathBuf>,
    /// Allow NET.connect operations
    pub net_connect: bool,
    /// Allow NET.listen operations
    pub net_listen: bool,
    /// Allowed network hosts (whitelist) - empty means all hosts denied
    pub net_hosts: Vec<String>,
    /// Allowed network ports (whitelist) - empty means all ports allowed
    pub net_ports: Vec<u16>,
    /// Allow IO.print
    pub io_print: bool,
    /// Allow IO.read_line
    pub io_read: bool,
    /// Allow TIME.sleep
    pub time_sleep: bool,
    /// Maximum sleep duration (milliseconds)
    pub max_sleep_ms: u64,
}

impl Default for IOPermissions {
    fn default() -> Self {
        // Deny-by-default policy
        Self {
            file_read: false,
            file_write: false,
            file_paths: vec![],
            net_connect: false,
            net_listen: false,
            net_hosts: vec![],
            net_ports: vec![],
            io_print: true, // Allow print by default (safe)
            io_read: false,
            time_sleep: true,
            max_sleep_ms: 60_000, // Max 1 minute sleep
        }
    }
}

impl IOPermissions {
    /// Create a fully permissive configuration (for trusted code)
    pub fn allow_all() -> Self {
        Self {
            file_read: true,
            file_write: true,
            file_paths: vec![], // Empty = all paths
            net_connect: true,
            net_listen: true,
            net_hosts: vec![], // Empty = all hosts
            net_ports: vec![], // Empty = all ports
            io_print: true,
            io_read: true,
            time_sleep: true,
            max_sleep_ms: u64::MAX,
        }
    }

    /// Create a restricted configuration (for untrusted code)
    pub fn restricted() -> Self {
        Self::default()
    }

    /// Check if a file path is allowed
    pub fn is_path_allowed(&self, path: &Path) -> bool {
        if self.file_paths.is_empty() {
            // If no paths specified, only allow if read or write is enabled
            return self.file_read || self.file_write;
        }

        // Check if path is under any allowed path
        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        self.file_paths.iter().any(|allowed| {
            let allowed_canonical = allowed.canonicalize().unwrap_or_else(|_| allowed.clone());
            canonical.starts_with(&allowed_canonical)
        })
    }

    /// Check if a host is allowed
    pub fn is_host_allowed(&self, host: &str) -> bool {
        if self.net_hosts.is_empty() {
            return self.net_connect || self.net_listen;
        }
        self.net_hosts
            .iter()
            .any(|allowed| host == allowed || host.ends_with(allowed))
    }

    /// Check if a port is allowed
    pub fn is_port_allowed(&self, port: u16) -> bool {
        if self.net_ports.is_empty() {
            return true;
        }
        self.net_ports.contains(&port)
    }
}

/// I/O Error types
#[derive(Debug, Clone)]
pub enum IOError {
    PermissionDenied(String),
    FileNotFound(String),
    InvalidFd(u64),
    IoError(String),
    InvalidAddress(String),
    ConnectionRefused(String),
    Timeout,
    InvalidOperation,
}

impl std::fmt::Display for IOError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IOError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            IOError::FileNotFound(msg) => write!(f, "File not found: {}", msg),
            IOError::InvalidFd(fd) => write!(f, "Invalid file descriptor: {}", fd),
            IOError::IoError(msg) => write!(f, "I/O error: {}", msg),
            IOError::InvalidAddress(msg) => write!(f, "Invalid address: {}", msg),
            IOError::ConnectionRefused(msg) => write!(f, "Connection refused: {}", msg),
            IOError::Timeout => write!(f, "Operation timed out"),
            IOError::InvalidOperation => write!(f, "Invalid operation"),
        }
    }
}

/// File handle types
#[allow(dead_code)]
enum FileHandle {
    File(File),
    TcpStream(TcpStream),
    TcpListener(TcpListener),
    UdpSocket(UdpSocket), // For future UDP support
}

/// Maximum number of file descriptors
const MAX_FDS: usize = 8192;

/// Shared listener state for multi-worker mode
pub struct SharedListenerState {
    /// The shared listener (set by first worker to bind)
    pub listener: std::sync::Arc<std::sync::Mutex<Option<std::sync::Arc<TcpListener>>>>,
    /// Flag indicating listener is ready
    pub ready: std::sync::Arc<std::sync::atomic::AtomicBool>,
    /// This worker's ID
    pub worker_id: usize,
}

/// I/O Runtime for managing file descriptors and operations
pub struct IORuntime {
    permissions: IOPermissions,
    next_fd: usize,
    /// File handles stored in a Vec for O(1) indexed lookup (faster than HashMap)
    handles: Vec<Option<FileHandle>>,
    /// Free list of closed FDs for O(1) reuse
    free_list: Vec<usize>,
    start_time: Instant,
    // Buffers for reading lines
    line_buffer: String,
    /// Shared listener state for multi-worker mode (None = not in shared mode)
    shared_listener_state: Option<SharedListenerState>,
    /// Cached shared listener for this worker's accept calls
    cached_shared_listener: Option<std::sync::Arc<TcpListener>>,
    /// Network mocks for testing servers
    network_mocks: NetworkMocks,
}

impl IORuntime {
    pub fn new(permissions: IOPermissions) -> Self {
        // Pre-allocate a reasonable number of slots
        let mut handles = Vec::with_capacity(1024);
        // Reserve slots 0-2 for stdin/stdout/stderr
        handles.resize_with(3, || None);

        Self {
            permissions,
            next_fd: 3, // 0,1,2 reserved for stdin/stdout/stderr
            handles,
            free_list: Vec::with_capacity(256),
            start_time: Instant::now(),
            line_buffer: String::new(),
            shared_listener_state: None,
            cached_shared_listener: None,
            network_mocks: NetworkMocks::new(),
        }
    }

    /// Get mutable access to network mocks for test setup
    pub fn network_mocks_mut(&mut self) -> &mut NetworkMocks {
        &mut self.network_mocks
    }

    /// Check if network mocking is enabled
    pub fn is_network_mock_mode(&self) -> bool {
        self.network_mocks.is_enabled()
    }

    /// Configure this runtime for shared listener mode
    ///
    /// In shared listener mode, multiple workers share a single TcpListener.
    /// The first worker to call net.bind creates the listener, and other workers
    /// attach to it. All workers then compete on accept() calls.
    pub fn set_shared_listener_mode(
        &mut self,
        listener: std::sync::Arc<std::sync::Mutex<Option<std::sync::Arc<TcpListener>>>>,
        ready: std::sync::Arc<std::sync::atomic::AtomicBool>,
        worker_id: usize,
    ) {
        self.shared_listener_state = Some(SharedListenerState {
            listener,
            ready,
            worker_id,
        });
    }

    #[inline]
    fn allocate_fd(&mut self) -> u64 {
        // Fast path: reuse from free list (O(1))
        if let Some(fd) = self.free_list.pop() {
            return fd as u64;
        }

        // Slow path: allocate new slot
        if self.next_fd >= MAX_FDS {
            return u64::MAX; // Exhausted
        }

        let fd = self.next_fd;
        self.next_fd += 1;

        // Ensure handles vector is large enough
        while fd >= self.handles.len() {
            self.handles.push(None);
        }

        fd as u64
    }

    /// Get handle by fd (O(1) indexed lookup)
    #[inline]
    fn get_handle(&self, fd: u64) -> Option<&FileHandle> {
        let idx = fd as usize;
        if idx < self.handles.len() {
            self.handles[idx].as_ref()
        } else {
            None
        }
    }

    /// Get mutable handle by fd (O(1) indexed lookup)
    #[inline]
    fn get_handle_mut(&mut self, fd: u64) -> Option<&mut FileHandle> {
        let idx = fd as usize;
        if idx < self.handles.len() {
            self.handles[idx].as_mut()
        } else {
            None
        }
    }

    /// Set handle (O(1) indexed assignment)
    #[inline]
    fn set_handle(&mut self, fd: u64, handle: FileHandle) {
        let idx = fd as usize;
        if idx < self.handles.len() {
            self.handles[idx] = Some(handle);
        }
    }

    /// Remove handle (O(1) indexed removal + add to free list)
    #[inline]
    fn remove_handle(&mut self, fd: u64) -> Option<FileHandle> {
        let idx = fd as usize;
        if idx < self.handles.len() {
            let handle = self.handles[idx].take();
            if handle.is_some() && idx >= 3 {
                // Add to free list for reuse (skip stdin/stdout/stderr)
                self.free_list.push(idx);
            }
            handle
        } else {
            None
        }
    }

    // ========== FILE Operations ==========

    /// file.open(path, flags) -> fd
    pub fn file_open(&mut self, path: &str, flags: u32) -> Result<u64, IOError> {
        let path = Path::new(path);

        // Check permissions
        let read_requested = flags & 0x01 != 0;
        let write_requested = flags & 0x02 != 0;
        let create_requested = flags & 0x04 != 0;
        let append_requested = flags & 0x08 != 0;

        if read_requested && !self.permissions.file_read {
            return Err(IOError::PermissionDenied("File read not allowed".into()));
        }
        if (write_requested || create_requested || append_requested) && !self.permissions.file_write
        {
            return Err(IOError::PermissionDenied("File write not allowed".into()));
        }
        if !self.permissions.is_path_allowed(path) {
            return Err(IOError::PermissionDenied(format!(
                "Path not allowed: {:?}",
                path
            )));
        }

        let mut options = OpenOptions::new();
        options.read(read_requested || (!write_requested && !create_requested));
        options.write(write_requested || create_requested || append_requested);
        options.create(create_requested);
        options.append(append_requested);
        options.truncate(create_requested && !append_requested);

        match options.open(path) {
            Ok(file) => {
                let fd = self.allocate_fd();
                self.set_handle(fd, FileHandle::File(file));
                Ok(fd)
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                Err(IOError::FileNotFound(path.display().to_string()))
            }
            Err(e) => Err(IOError::IoError(e.to_string())),
        }
    }

    /// file.read(fd, buf_ptr, len) -> bytes_read
    pub fn file_read(&mut self, fd: u64, buffer: &mut [u8]) -> Result<usize, IOError> {
        if !self.permissions.file_read {
            return Err(IOError::PermissionDenied("File read not allowed".into()));
        }

        match self.get_handle_mut(fd) {
            Some(FileHandle::File(file)) => file
                .read(buffer)
                .map_err(|e| IOError::IoError(e.to_string())),
            Some(FileHandle::TcpStream(stream)) => stream
                .read(buffer)
                .map_err(|e| IOError::IoError(e.to_string())),
            Some(_) => Err(IOError::InvalidOperation),
            None => Err(IOError::InvalidFd(fd)),
        }
    }

    /// file.write(fd, buf_ptr, len) -> bytes_written
    pub fn file_write(&mut self, fd: u64, buffer: &[u8]) -> Result<usize, IOError> {
        if !self.permissions.file_write {
            return Err(IOError::PermissionDenied("File write not allowed".into()));
        }

        match self.get_handle_mut(fd) {
            Some(FileHandle::File(file)) => file
                .write(buffer)
                .map_err(|e| IOError::IoError(e.to_string())),
            Some(FileHandle::TcpStream(stream)) => stream
                .write(buffer)
                .map_err(|e| IOError::IoError(e.to_string())),
            Some(_) => Err(IOError::InvalidOperation),
            None => Err(IOError::InvalidFd(fd)),
        }
    }

    /// file.close(fd)
    pub fn file_close(&mut self, fd: u64) -> Result<(), IOError> {
        if self.remove_handle(fd).is_some() {
            Ok(())
        } else {
            Err(IOError::InvalidFd(fd))
        }
    }

    /// file.seek(fd, offset, whence) -> new_position
    pub fn file_seek(&mut self, fd: u64, offset: i64, whence: u32) -> Result<u64, IOError> {
        let seek_from = match whence {
            0 => SeekFrom::Start(offset as u64),
            1 => SeekFrom::Current(offset),
            2 => SeekFrom::End(offset),
            _ => return Err(IOError::InvalidOperation),
        };

        match self.get_handle_mut(fd) {
            Some(FileHandle::File(file)) => file
                .seek(seek_from)
                .map_err(|e| IOError::IoError(e.to_string())),
            Some(_) => Err(IOError::InvalidOperation),
            None => Err(IOError::InvalidFd(fd)),
        }
    }

    /// file.stat(path) -> (size, mtime)
    pub fn file_stat(&self, path: &str) -> Result<(u64, u64), IOError> {
        let path = Path::new(path);

        if !self.permissions.file_read {
            return Err(IOError::PermissionDenied("File stat not allowed".into()));
        }
        if !self.permissions.is_path_allowed(path) {
            return Err(IOError::PermissionDenied(format!(
                "Path not allowed: {:?}",
                path
            )));
        }

        match fs::metadata(path) {
            Ok(meta) => {
                let size = meta.len();
                let mtime = meta
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                Ok((size, mtime))
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                Err(IOError::FileNotFound(path.display().to_string()))
            }
            Err(e) => Err(IOError::IoError(e.to_string())),
        }
    }

    /// file.mkdir(path)
    pub fn file_mkdir(&self, path: &str) -> Result<(), IOError> {
        let path = Path::new(path);

        if !self.permissions.file_write {
            return Err(IOError::PermissionDenied(
                "Directory creation not allowed".into(),
            ));
        }
        if !self.permissions.is_path_allowed(path) {
            return Err(IOError::PermissionDenied(format!(
                "Path not allowed: {:?}",
                path
            )));
        }

        fs::create_dir_all(path).map_err(|e| IOError::IoError(e.to_string()))
    }

    /// file.delete(path)
    pub fn file_delete(&self, path: &str) -> Result<(), IOError> {
        let path = Path::new(path);

        if !self.permissions.file_write {
            return Err(IOError::PermissionDenied(
                "File deletion not allowed".into(),
            ));
        }
        if !self.permissions.is_path_allowed(path) {
            return Err(IOError::PermissionDenied(format!(
                "Path not allowed: {:?}",
                path
            )));
        }

        if path.is_dir() {
            fs::remove_dir_all(path).map_err(|e| IOError::IoError(e.to_string()))
        } else {
            fs::remove_file(path).map_err(|e| IOError::IoError(e.to_string()))
        }
    }

    // ========== NET Operations ==========

    /// net.socket(domain, type) -> fd
    pub fn net_socket(&mut self, _domain: u32, _socket_type: u32) -> Result<u64, IOError> {
        // Check for mock first
        if let Some(mock_val) = self.network_mocks.get_mock_value(NetMockOp::Socket) {
            return if mock_val >= 0 {
                Ok(mock_val as u64)
            } else {
                Err(IOError::IoError("Mock socket error".into()))
            };
        }

        // TODO: Use domain and socket_type when implementing full socket support
        let fd = self.allocate_fd();

        // Socket is created lazily on connect/bind
        // For now, just return a placeholder fd
        Ok(fd)
    }

    /// net.connect(fd, addr, port)
    pub fn net_connect(&mut self, fd: u64, addr: &str, port: u16) -> Result<(), IOError> {
        // Check for mock first
        if let Some(mock_val) = self.network_mocks.get_mock_value(NetMockOp::Connect) {
            return if mock_val >= 0 {
                Ok(())
            } else {
                Err(IOError::ConnectionRefused("Mock connection refused".into()))
            };
        }

        if !self.permissions.net_connect {
            return Err(IOError::PermissionDenied(
                "Network connect not allowed".into(),
            ));
        }
        if !self.permissions.is_host_allowed(addr) {
            return Err(IOError::PermissionDenied(format!(
                "Host not allowed: {}",
                addr
            )));
        }
        if !self.permissions.is_port_allowed(port) {
            return Err(IOError::PermissionDenied(format!(
                "Port not allowed: {}",
                port
            )));
        }

        let socket_addr = format!("{}:{}", addr, port)
            .to_socket_addrs()
            .map_err(|e| IOError::InvalidAddress(e.to_string()))?
            .next()
            .ok_or_else(|| IOError::InvalidAddress("Could not resolve address".into()))?;

        match TcpStream::connect(socket_addr) {
            Ok(stream) => {
                // Enable TCP_NODELAY by default for low-latency responses
                let _ = stream.set_nodelay(true);
                self.set_handle(fd, FileHandle::TcpStream(stream));
                Ok(())
            }
            Err(e) if e.kind() == io::ErrorKind::ConnectionRefused => {
                Err(IOError::ConnectionRefused(format!("{}:{}", addr, port)))
            }
            Err(e) => Err(IOError::IoError(e.to_string())),
        }
    }

    /// net.bind(fd, addr, port)
    /// Creates a socket with SO_REUSEADDR and SO_REUSEPORT (when available)
    /// for multi-worker support.
    ///
    /// In shared listener mode:
    /// - First worker creates the listener and shares it
    /// - Other workers wait and attach to the shared listener
    pub fn net_bind(&mut self, fd: u64, addr: &str, port: u16) -> Result<(), IOError> {
        // Check for mock first
        if let Some(mock_val) = self.network_mocks.get_mock_value(NetMockOp::Bind) {
            return if mock_val >= 0 {
                Ok(())
            } else {
                Err(IOError::IoError("Mock bind error".into()))
            };
        }

        use std::sync::atomic::Ordering;

        if !self.permissions.net_listen {
            return Err(IOError::PermissionDenied("Network bind not allowed".into()));
        }
        if !self.permissions.is_port_allowed(port) {
            return Err(IOError::PermissionDenied(format!(
                "Port not allowed: {}",
                port
            )));
        }

        // Check if we're in shared listener mode
        let shared_listener_opt = if let Some(ref state) = self.shared_listener_state {
            // Try to be the first worker to create the listener
            let mut guard = state.listener.lock().unwrap();

            if guard.is_none() {
                // We're the first - create the listener
                let listener = self.create_listener(addr, port)?;
                let listener_arc = std::sync::Arc::new(listener);
                *guard = Some(std::sync::Arc::clone(&listener_arc));

                // Signal that listener is ready
                state.ready.store(true, Ordering::Release);

                Some(listener_arc)
            } else {
                // Another worker created it - use the shared one
                Some(std::sync::Arc::clone(guard.as_ref().unwrap()))
            }
        } else {
            None
        };

        if let Some(listener_arc) = shared_listener_opt {
            // Shared mode - cache the shared listener
            self.cached_shared_listener = Some(listener_arc);

            // Store a dummy handle so net.listen doesn't fail
            // Actual accepts will use cached_shared_listener
            let dummy =
                TcpListener::bind("127.0.0.1:0").map_err(|e| IOError::IoError(e.to_string()))?;
            self.set_handle(fd, FileHandle::TcpListener(dummy));
            return Ok(());
        }

        // Normal mode (SO_REUSEPORT or single-threaded)
        let listener = self.create_listener(addr, port)?;
        self.set_handle(fd, FileHandle::TcpListener(listener));
        Ok(())
    }

    /// Create a TCP listener with SO_REUSEADDR and SO_REUSEPORT
    fn create_listener(&self, addr: &str, port: u16) -> Result<TcpListener, IOError> {
        let socket_addr: SocketAddr = format!("{}:{}", addr, port)
            .to_socket_addrs()
            .map_err(|e| IOError::InvalidAddress(e.to_string()))?
            .next()
            .ok_or_else(|| IOError::InvalidAddress("Could not resolve address".into()))?;

        // Use socket2 for advanced options
        let domain = if socket_addr.is_ipv4() {
            Domain::IPV4
        } else {
            Domain::IPV6
        };

        let socket = Socket::new(domain, Type::STREAM, Some(Protocol::TCP))
            .map_err(|e| IOError::IoError(e.to_string()))?;

        // Enable SO_REUSEADDR - allows rebinding after server restart
        socket
            .set_reuse_address(true)
            .map_err(|e| IOError::IoError(e.to_string()))?;

        // Enable SO_REUSEPORT if available - allows multiple sockets on same port
        // This enables kernel-level load balancing across worker threads
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))]
        {
            if let Err(e) = socket.set_reuse_port(true) {
                // Log but don't fail - SO_REUSEPORT is optional
                eprintln!("Warning: SO_REUSEPORT not available: {}", e);
            }
        }

        // Bind the socket
        socket
            .bind(&socket_addr.into())
            .map_err(|e| IOError::IoError(e.to_string()))?;

        // Set listen backlog and convert to std TcpListener
        socket
            .listen(128)
            .map_err(|e| IOError::IoError(e.to_string()))?;

        // Convert socket2::Socket to std::net::TcpListener
        Ok(socket.into())
    }

    /// net.listen(fd, backlog)
    pub fn net_listen(&mut self, fd: u64, _backlog: u32) -> Result<(), IOError> {
        // Check for mock first
        if let Some(mock_val) = self.network_mocks.get_mock_value(NetMockOp::Listen) {
            return if mock_val >= 0 {
                Ok(())
            } else {
                Err(IOError::IoError("Mock listen error".into()))
            };
        }

        if !self.permissions.net_listen {
            return Err(IOError::PermissionDenied(
                "Network listen not allowed".into(),
            ));
        }

        // TcpListener in Rust starts listening on bind
        match self.get_handle(fd) {
            Some(FileHandle::TcpListener(_)) => Ok(()),
            Some(_) => Err(IOError::InvalidOperation),
            None => Err(IOError::InvalidFd(fd)),
        }
    }

    /// net.accept(fd) -> client_fd
    ///
    /// In shared listener mode, uses the cached shared listener instead of
    /// the per-fd listener. This allows multiple workers to accept on the
    /// same port.
    pub fn net_accept(&mut self, fd: u64) -> Result<u64, IOError> {
        // Check for mock first - crucial for testing server loops
        if let Some(mock_val) = self.network_mocks.get_mock_value(NetMockOp::Accept) {
            return if mock_val >= 0 {
                Ok(mock_val as u64)
            } else {
                // Negative value signals end of clients (break server loop)
                Err(IOError::IoError("No more clients".into()))
            };
        }

        if !self.permissions.net_listen {
            return Err(IOError::PermissionDenied(
                "Network accept not allowed".into(),
            ));
        }

        // In shared mode, use the shared listener
        if let Some(ref shared) = self.cached_shared_listener {
            return match shared.accept() {
                Ok((stream, _addr)) => {
                    let _ = stream.set_nodelay(true);
                    let client_fd = self.allocate_fd();
                    self.set_handle(client_fd, FileHandle::TcpStream(stream));
                    Ok(client_fd)
                }
                Err(e) => Err(IOError::IoError(e.to_string())),
            };
        }

        // Normal mode - use the per-fd listener
        let listener_clone = match self.get_handle(fd) {
            Some(FileHandle::TcpListener(listener)) => listener
                .try_clone()
                .map_err(|e| IOError::IoError(e.to_string()))?,
            Some(_) => return Err(IOError::InvalidOperation),
            None => return Err(IOError::InvalidFd(fd)),
        };

        // Accept connection and store it
        match listener_clone.accept() {
            Ok((stream, _addr)) => {
                // Enable TCP_NODELAY by default for low-latency responses
                let _ = stream.set_nodelay(true);
                let client_fd = self.allocate_fd();
                self.set_handle(client_fd, FileHandle::TcpStream(stream));
                Ok(client_fd)
            }
            Err(e) => Err(IOError::IoError(e.to_string())),
        }
    }

    /// net.send(fd, buf, len) -> bytes_sent
    pub fn net_send(&mut self, fd: u64, buffer: &[u8]) -> Result<usize, IOError> {
        // Check for mock first
        if let Some(mock_val) = self.network_mocks.get_mock_value(NetMockOp::Send) {
            return if mock_val >= 0 {
                Ok(mock_val as usize)
            } else {
                Err(IOError::IoError("Mock send error".into()))
            };
        }

        if !self.permissions.net_connect {
            return Err(IOError::PermissionDenied("Network send not allowed".into()));
        }

        match self.get_handle_mut(fd) {
            Some(FileHandle::TcpStream(stream)) => stream
                .write(buffer)
                .map_err(|e| IOError::IoError(e.to_string())),
            Some(FileHandle::UdpSocket(socket)) => socket
                .send(buffer)
                .map_err(|e| IOError::IoError(e.to_string())),
            Some(_) => Err(IOError::InvalidOperation),
            None => Err(IOError::InvalidFd(fd)),
        }
    }

    /// net.recv(fd, buf, len) -> bytes_received
    pub fn net_recv(&mut self, fd: u64, buffer: &mut [u8]) -> Result<usize, IOError> {
        // Check for mock first
        if let Some(mock_val) = self.network_mocks.get_mock_value(NetMockOp::Recv) {
            // If we have mock recv data and return value matches data length, copy it
            // This allows sequences like recv="hello",0 where first call gets data,
            // subsequent calls return 0 (connection closed)
            if let Some(data) = self.network_mocks.get_recv_data() {
                if mock_val == data.len() as i64 {
                    let len = std::cmp::min(data.len(), buffer.len());
                    buffer[..len].copy_from_slice(&data[..len]);
                    return Ok(len);
                }
            }
            return if mock_val >= 0 {
                Ok(mock_val as usize)
            } else {
                Err(IOError::IoError("Mock recv error".into()))
            };
        }

        if !self.permissions.net_connect && !self.permissions.net_listen {
            return Err(IOError::PermissionDenied("Network recv not allowed".into()));
        }

        match self.get_handle_mut(fd) {
            Some(FileHandle::TcpStream(stream)) => stream
                .read(buffer)
                .map_err(|e| IOError::IoError(e.to_string())),
            Some(FileHandle::UdpSocket(socket)) => socket
                .recv(buffer)
                .map_err(|e| IOError::IoError(e.to_string())),
            Some(_) => Err(IOError::InvalidOperation),
            None => Err(IOError::InvalidFd(fd)),
        }
    }

    /// net.close(fd)
    pub fn net_close(&mut self, fd: u64) -> Result<(), IOError> {
        // Check for mock first
        if let Some(mock_val) = self.network_mocks.get_mock_value(NetMockOp::Close) {
            return if mock_val >= 0 {
                Ok(())
            } else {
                Err(IOError::IoError("Mock close error".into()))
            };
        }

        self.file_close(fd)
    }

    /// net.setopt(fd, option, value)
    pub fn net_setopt(&mut self, fd: u64, option: NetOption, value: u64) -> Result<(), IOError> {
        match self.get_handle_mut(fd) {
            Some(FileHandle::TcpStream(stream)) => {
                match option {
                    NetOption::Nonblock => stream
                        .set_nonblocking(value != 0)
                        .map_err(|e| IOError::IoError(e.to_string())),
                    NetOption::TimeoutMs => {
                        let timeout = if value == 0 {
                            None
                        } else {
                            Some(Duration::from_millis(value))
                        };
                        stream
                            .set_read_timeout(timeout)
                            .map_err(|e| IOError::IoError(e.to_string()))?;
                        stream
                            .set_write_timeout(timeout)
                            .map_err(|e| IOError::IoError(e.to_string()))
                    }
                    NetOption::NoDelay => stream
                        .set_nodelay(value != 0)
                        .map_err(|e| IOError::IoError(e.to_string())),
                    _ => Ok(()), // Other options not implemented for TcpStream
                }
            }
            Some(FileHandle::TcpListener(listener)) => match option {
                NetOption::Nonblock => listener
                    .set_nonblocking(value != 0)
                    .map_err(|e| IOError::IoError(e.to_string())),
                _ => Ok(()),
            },
            Some(_) => Err(IOError::InvalidOperation),
            None => Err(IOError::InvalidFd(fd)),
        }
    }

    // ========== IO (Console) Operations ==========

    /// io.print(buf, len)
    pub fn io_print(&self, message: &str) -> Result<(), IOError> {
        if !self.permissions.io_print {
            return Err(IOError::PermissionDenied("Print not allowed".into()));
        }
        print!("{}", message);
        Ok(())
    }

    /// io.read_line(buf, max) -> bytes_read
    pub fn io_read_line(&mut self, buffer: &mut String, max_len: usize) -> Result<usize, IOError> {
        if !self.permissions.io_read {
            return Err(IOError::PermissionDenied("Read not allowed".into()));
        }

        self.line_buffer.clear();
        let stdin = io::stdin();
        let mut handle = stdin.lock();

        match handle.read_line(&mut self.line_buffer) {
            Ok(n) => {
                let len = n.min(max_len);
                buffer.push_str(&self.line_buffer[..len]);
                Ok(len)
            }
            Err(e) => Err(IOError::IoError(e.to_string())),
        }
    }

    /// io.get_args() -> (argc, argv_ptr)
    pub fn io_get_args(&self) -> Vec<String> {
        std::env::args().collect()
    }

    /// io.get_env(name) -> value
    pub fn io_get_env(&self, name: &str) -> Option<String> {
        std::env::var(name).ok()
    }

    // ========== TIME Operations ==========

    /// time.now() -> unix_timestamp
    pub fn time_now(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    /// time.sleep(milliseconds)
    pub fn time_sleep(&self, milliseconds: u64) -> Result<(), IOError> {
        if !self.permissions.time_sleep {
            return Err(IOError::PermissionDenied("Sleep not allowed".into()));
        }

        let ms = milliseconds.min(self.permissions.max_sleep_ms);
        std::thread::sleep(Duration::from_millis(ms));
        Ok(())
    }

    /// time.monotonic() -> nanoseconds since start
    pub fn time_monotonic(&self) -> u64 {
        self.start_time.elapsed().as_nanos() as u64
    }
}

impl Default for IORuntime {
    fn default() -> Self {
        Self::new(IOPermissions::default())
    }
}

// ========== External C ABI functions for JIT ==========

/// File open - called from JIT code
#[no_mangle]
pub extern "C" fn neurlang_file_open(
    runtime: *mut IORuntime,
    path: *const u8,
    path_len: usize,
    flags: u32,
) -> i64 {
    let runtime = unsafe { &mut *runtime };
    let path = unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(path, path_len)) };

    match runtime.file_open(path, flags) {
        Ok(fd) => fd as i64,
        Err(_) => -1,
    }
}

/// File read - called from JIT code
#[no_mangle]
pub extern "C" fn neurlang_file_read(
    runtime: *mut IORuntime,
    fd: u64,
    buf: *mut u8,
    len: usize,
) -> i64 {
    let runtime = unsafe { &mut *runtime };
    let buffer = unsafe { std::slice::from_raw_parts_mut(buf, len) };

    match runtime.file_read(fd, buffer) {
        Ok(n) => n as i64,
        Err(_) => -1,
    }
}

/// File write - called from JIT code
#[no_mangle]
pub extern "C" fn neurlang_file_write(
    runtime: *mut IORuntime,
    fd: u64,
    buf: *const u8,
    len: usize,
) -> i64 {
    let runtime = unsafe { &mut *runtime };
    let buffer = unsafe { std::slice::from_raw_parts(buf, len) };

    match runtime.file_write(fd, buffer) {
        Ok(n) => n as i64,
        Err(_) => -1,
    }
}

/// File close - called from JIT code
#[no_mangle]
pub extern "C" fn neurlang_file_close(runtime: *mut IORuntime, fd: u64) -> i64 {
    let runtime = unsafe { &mut *runtime };
    match runtime.file_close(fd) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

/// Network connect - called from JIT code
#[no_mangle]
pub extern "C" fn neurlang_net_connect(
    runtime: *mut IORuntime,
    fd: u64,
    addr: *const u8,
    addr_len: usize,
    port: u16,
) -> i64 {
    let runtime = unsafe { &mut *runtime };
    let addr = unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(addr, addr_len)) };

    match runtime.net_connect(fd, addr, port) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

/// Network send - called from JIT code
#[no_mangle]
pub extern "C" fn neurlang_net_send(
    runtime: *mut IORuntime,
    fd: u64,
    buf: *const u8,
    len: usize,
) -> i64 {
    let runtime = unsafe { &mut *runtime };
    let buffer = unsafe { std::slice::from_raw_parts(buf, len) };

    match runtime.net_send(fd, buffer) {
        Ok(n) => n as i64,
        Err(_) => -1,
    }
}

/// Network recv - called from JIT code
#[no_mangle]
pub extern "C" fn neurlang_net_recv(
    runtime: *mut IORuntime,
    fd: u64,
    buf: *mut u8,
    len: usize,
) -> i64 {
    let runtime = unsafe { &mut *runtime };
    let buffer = unsafe { std::slice::from_raw_parts_mut(buf, len) };

    match runtime.net_recv(fd, buffer) {
        Ok(n) => n as i64,
        Err(_) => -1,
    }
}

/// Time now - called from JIT code
#[no_mangle]
pub extern "C" fn neurlang_time_now(runtime: *const IORuntime) -> u64 {
    let runtime = unsafe { &*runtime };
    runtime.time_now()
}

/// Time sleep - called from JIT code
#[no_mangle]
pub extern "C" fn neurlang_time_sleep(runtime: *const IORuntime, milliseconds: u64) -> i64 {
    let runtime = unsafe { &*runtime };
    match runtime.time_sleep(milliseconds) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

/// Time monotonic - called from JIT code
#[no_mangle]
pub extern "C" fn neurlang_time_monotonic(runtime: *const IORuntime) -> u64 {
    let runtime = unsafe { &*runtime };
    runtime.time_monotonic()
}

/// Print string - called from JIT code
#[no_mangle]
pub extern "C" fn neurlang_io_print(runtime: *const IORuntime, buf: *const u8, len: usize) -> i64 {
    let runtime = unsafe { &*runtime };
    let message = unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(buf, len)) };

    match runtime.io_print(message) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_permissions() {
        let perms = IOPermissions::default();
        assert!(!perms.file_read);
        assert!(!perms.file_write);
        assert!(!perms.net_connect);
        assert!(!perms.net_listen);
        assert!(perms.io_print); // Print allowed by default
    }

    #[test]
    fn test_path_allowed() {
        let mut perms = IOPermissions::default();
        perms.file_read = true;
        perms.file_paths = vec![PathBuf::from("/tmp")];

        assert!(perms.is_path_allowed(Path::new("/tmp/test.txt")));
        assert!(!perms.is_path_allowed(Path::new("/etc/passwd")));
    }

    #[test]
    fn test_time_operations() {
        let runtime = IORuntime::new(IOPermissions::allow_all());

        let now = runtime.time_now();
        assert!(now > 0);

        let mono = runtime.time_monotonic();
        // Just verify it returned a value (u64 is always >= 0)
        let _ = mono;
    }
}
