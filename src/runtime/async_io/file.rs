//! Async file operations
//!
//! Provides non-blocking file I/O wrappers.

use super::Token;
use std::io::{self, SeekFrom};
use std::os::unix::io::{AsRawFd, RawFd};
use std::path::Path;

/// File state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileState {
    /// File is open and ready
    Open,
    /// File operation in progress
    Busy,
    /// File is closed
    Closed,
    /// File encountered an error
    Error,
}

/// Open flags for async files
#[derive(Debug, Clone, Copy)]
pub struct OpenOptions {
    read: bool,
    write: bool,
    create: bool,
    truncate: bool,
    append: bool,
}

impl Default for OpenOptions {
    fn default() -> Self {
        Self {
            read: true,
            write: false,
            create: false,
            truncate: false,
            append: false,
        }
    }
}

impl OpenOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read(mut self, read: bool) -> Self {
        self.read = read;
        self
    }

    pub fn write(mut self, write: bool) -> Self {
        self.write = write;
        self
    }

    pub fn create(mut self, create: bool) -> Self {
        self.create = create;
        self
    }

    pub fn truncate(mut self, truncate: bool) -> Self {
        self.truncate = truncate;
        self
    }

    pub fn append(mut self, append: bool) -> Self {
        self.append = append;
        self
    }

    fn to_libc_flags(&self) -> i32 {
        let mut flags = libc::O_NONBLOCK;

        if self.read && self.write {
            flags |= libc::O_RDWR;
        } else if self.write {
            flags |= libc::O_WRONLY;
        } else {
            flags |= libc::O_RDONLY;
        }

        if self.create {
            flags |= libc::O_CREAT;
        }
        if self.truncate {
            flags |= libc::O_TRUNC;
        }
        if self.append {
            flags |= libc::O_APPEND;
        }

        flags
    }
}

/// Async file wrapper
pub struct AsyncFile {
    /// Raw file descriptor
    fd: RawFd,
    /// Token for event registration
    token: Token,
    /// Current state
    state: FileState,
    /// Current position
    position: u64,
    /// File size (cached)
    size: Option<u64>,
}

impl AsyncFile {
    /// Create a new async file wrapper from an existing fd
    pub fn new(fd: RawFd, token: Token) -> Self {
        // Ensure non-blocking mode
        unsafe {
            let flags = libc::fcntl(fd, libc::F_GETFL);
            libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
        }

        Self {
            fd,
            token,
            state: FileState::Open,
            position: 0,
            size: None,
        }
    }

    /// Open a file with the given options
    pub fn open<P: AsRef<Path>>(path: P, options: OpenOptions) -> io::Result<Self> {
        let path_cstr = std::ffi::CString::new(path.as_ref().to_str().unwrap())?;
        let flags = options.to_libc_flags();
        let mode: libc::mode_t = 0o644; // rw-r--r--

        let fd = unsafe { libc::open(path_cstr.as_ptr(), flags, mode) };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }

        let token = Token(0); // Will be set on registration
        Ok(Self::new(fd, token))
    }

    /// Create a new file (write mode, truncate if exists)
    pub fn create<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        Self::open(
            path,
            OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(true),
        )
    }

    /// Open a file for reading
    pub fn open_read<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        Self::open(path, OpenOptions::new().read(true))
    }

    /// Open a file for appending
    pub fn open_append<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        Self::open(
            path,
            OpenOptions::new().write(true).create(true).append(true),
        )
    }

    /// Get the file descriptor
    pub fn fd(&self) -> RawFd {
        self.fd
    }

    /// Get the token
    pub fn token(&self) -> Token {
        self.token
    }

    /// Get the current state
    pub fn state(&self) -> FileState {
        self.state
    }

    /// Get the current position
    pub fn position(&self) -> u64 {
        self.position
    }

    /// Get the file size
    pub fn size(&mut self) -> io::Result<u64> {
        if let Some(size) = self.size {
            return Ok(size);
        }

        let mut stat: libc::stat = unsafe { std::mem::zeroed() };
        let result = unsafe { libc::fstat(self.fd, &mut stat) };
        if result < 0 {
            return Err(io::Error::last_os_error());
        }

        let size = stat.st_size as u64;
        self.size = Some(size);
        Ok(size)
    }

    /// Non-blocking read
    pub fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let result =
            unsafe { libc::read(self.fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };

        if result < 0 {
            let err = io::Error::last_os_error();
            if err.kind() == io::ErrorKind::WouldBlock {
                return Err(err);
            }
            self.state = FileState::Error;
            return Err(err);
        }

        self.position += result as u64;
        Ok(result as usize)
    }

    /// Non-blocking write
    pub fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let result =
            unsafe { libc::write(self.fd, buf.as_ptr() as *const libc::c_void, buf.len()) };

        if result < 0 {
            let err = io::Error::last_os_error();
            if err.kind() == io::ErrorKind::WouldBlock {
                return Err(err);
            }
            self.state = FileState::Error;
            return Err(err);
        }

        self.position += result as u64;
        // Invalidate cached size after write
        self.size = None;
        Ok(result as usize)
    }

    /// Read at a specific offset (pread)
    pub fn read_at(&self, buf: &mut [u8], offset: u64) -> io::Result<usize> {
        let result = unsafe {
            libc::pread(
                self.fd,
                buf.as_mut_ptr() as *mut libc::c_void,
                buf.len(),
                offset as libc::off_t,
            )
        };

        if result < 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(result as usize)
    }

    /// Write at a specific offset (pwrite)
    pub fn write_at(&mut self, buf: &[u8], offset: u64) -> io::Result<usize> {
        let result = unsafe {
            libc::pwrite(
                self.fd,
                buf.as_ptr() as *const libc::c_void,
                buf.len(),
                offset as libc::off_t,
            )
        };

        if result < 0 {
            return Err(io::Error::last_os_error());
        }

        // Invalidate cached size after write
        self.size = None;
        Ok(result as usize)
    }

    /// Seek to a position
    pub fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let (whence, offset) = match pos {
            SeekFrom::Start(n) => (libc::SEEK_SET, n as libc::off_t),
            SeekFrom::End(n) => (libc::SEEK_END, n as libc::off_t),
            SeekFrom::Current(n) => (libc::SEEK_CUR, n as libc::off_t),
        };

        let result = unsafe { libc::lseek(self.fd, offset, whence) };
        if result < 0 {
            return Err(io::Error::last_os_error());
        }

        self.position = result as u64;
        Ok(self.position)
    }

    /// Sync all data to disk
    pub fn sync_all(&self) -> io::Result<()> {
        let result = unsafe { libc::fsync(self.fd) };
        if result < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    /// Sync data (not metadata) to disk
    pub fn sync_data(&self) -> io::Result<()> {
        let result = unsafe { libc::fdatasync(self.fd) };
        if result < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    /// Truncate or extend the file to the specified length
    pub fn set_len(&mut self, len: u64) -> io::Result<()> {
        let result = unsafe { libc::ftruncate(self.fd, len as libc::off_t) };
        if result < 0 {
            return Err(io::Error::last_os_error());
        }
        self.size = Some(len);
        Ok(())
    }

    /// Close the file
    pub fn close(&mut self) -> io::Result<()> {
        if self.state == FileState::Closed {
            return Ok(());
        }

        let result = unsafe { libc::close(self.fd) };
        self.state = FileState::Closed;

        if result < 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(())
    }

    /// Set the token (called when registering with runtime)
    pub fn set_token(&mut self, token: Token) {
        self.token = token;
    }
}

impl Drop for AsyncFile {
    fn drop(&mut self) {
        if self.state != FileState::Closed {
            unsafe { libc::close(self.fd) };
        }
    }
}

impl AsRawFd for AsyncFile {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_file_create_and_write() {
        let path = "/tmp/neurlang_async_test.txt";

        // Create and write
        let mut file = AsyncFile::create(path).unwrap();
        assert_eq!(file.state(), FileState::Open);

        let written = file.write(b"hello world").unwrap();
        assert_eq!(written, 11);

        file.close().unwrap();
        assert_eq!(file.state(), FileState::Closed);

        // Cleanup
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_file_read() {
        let path = "/tmp/neurlang_async_read_test.txt";

        // Create test file
        std::fs::write(path, "test content").unwrap();

        // Read
        let mut file = AsyncFile::open_read(path).unwrap();
        let mut buf = [0u8; 64];
        let read = file.read(&mut buf).unwrap();

        assert_eq!(read, 12);
        assert_eq!(&buf[..read], b"test content");

        // Cleanup
        std::fs::remove_file(path).ok();
    }

    #[test]
    fn test_file_seek() {
        let path = "/tmp/neurlang_async_seek_test.txt";

        // Create test file
        std::fs::write(path, "abcdefghij").unwrap();

        // Seek and read
        let mut file = AsyncFile::open_read(path).unwrap();
        file.seek(SeekFrom::Start(5)).unwrap();

        let mut buf = [0u8; 5];
        let read = file.read(&mut buf).unwrap();

        assert_eq!(read, 5);
        assert_eq!(&buf[..read], b"fghij");

        // Cleanup
        std::fs::remove_file(path).ok();
    }
}
