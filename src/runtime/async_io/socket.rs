//! Async socket operations
//!
//! Provides non-blocking socket wrappers for TCP and UDP.

use super::Token;
use std::io;
use std::net::SocketAddr;
use std::os::unix::io::{AsRawFd, RawFd};

/// Socket state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketState {
    /// Socket is idle
    Idle,
    /// Socket is connecting
    Connecting,
    /// Socket is connected and ready
    Connected,
    /// Socket is listening for connections
    Listening,
    /// Socket encountered an error
    Error,
    /// Socket is closed
    Closed,
}

/// Async socket wrapper
pub struct AsyncSocket {
    /// Raw file descriptor
    fd: RawFd,
    /// Token for event registration
    token: Token,
    /// Current state
    state: SocketState,
    /// Local address (if bound)
    local_addr: Option<SocketAddr>,
    /// Remote address (if connected)
    remote_addr: Option<SocketAddr>,
}

impl AsyncSocket {
    /// Create a new async socket wrapper
    pub fn new(fd: RawFd, token: Token) -> Self {
        // Set non-blocking mode
        unsafe {
            let flags = libc::fcntl(fd, libc::F_GETFL);
            libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
        }

        Self {
            fd,
            token,
            state: SocketState::Idle,
            local_addr: None,
            remote_addr: None,
        }
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
    pub fn state(&self) -> SocketState {
        self.state
    }

    /// Get the local address
    pub fn local_addr(&self) -> Option<SocketAddr> {
        self.local_addr
    }

    /// Get the remote address
    pub fn remote_addr(&self) -> Option<SocketAddr> {
        self.remote_addr
    }

    /// Create a new TCP socket
    pub fn tcp() -> io::Result<Self> {
        let fd = unsafe { libc::socket(libc::AF_INET, libc::SOCK_STREAM | libc::SOCK_NONBLOCK, 0) };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }

        // Generate a temporary token (will be replaced on registration)
        let token = Token(0);
        Ok(Self::new(fd, token))
    }

    /// Create a new UDP socket
    pub fn udp() -> io::Result<Self> {
        let fd = unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM | libc::SOCK_NONBLOCK, 0) };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }

        let token = Token(0);
        Ok(Self::new(fd, token))
    }

    /// Bind to an address
    pub fn bind(&mut self, addr: SocketAddr) -> io::Result<()> {
        let (sockaddr, len) = socket_addr_to_raw(&addr);

        let result =
            unsafe { libc::bind(self.fd, &sockaddr as *const _ as *const libc::sockaddr, len) };

        if result < 0 {
            return Err(io::Error::last_os_error());
        }

        self.local_addr = Some(addr);
        Ok(())
    }

    /// Listen for connections (TCP only)
    pub fn listen(&mut self, backlog: i32) -> io::Result<()> {
        let result = unsafe { libc::listen(self.fd, backlog) };

        if result < 0 {
            return Err(io::Error::last_os_error());
        }

        self.state = SocketState::Listening;
        Ok(())
    }

    /// Start async connect (returns WouldBlock if in progress)
    pub fn connect(&mut self, addr: SocketAddr) -> io::Result<()> {
        let (sockaddr, len) = socket_addr_to_raw(&addr);

        let result =
            unsafe { libc::connect(self.fd, &sockaddr as *const _ as *const libc::sockaddr, len) };

        if result < 0 {
            let err = io::Error::last_os_error();
            if err.kind() == io::ErrorKind::WouldBlock
                || err.raw_os_error() == Some(libc::EINPROGRESS)
            {
                self.state = SocketState::Connecting;
                self.remote_addr = Some(addr);
                return Err(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "connection in progress",
                ));
            }
            return Err(err);
        }

        self.state = SocketState::Connected;
        self.remote_addr = Some(addr);
        Ok(())
    }

    /// Check if connect completed (call after socket becomes writable)
    pub fn check_connect(&mut self) -> io::Result<()> {
        let mut error: libc::c_int = 0;
        let mut len: libc::socklen_t = std::mem::size_of::<libc::c_int>() as libc::socklen_t;

        let result = unsafe {
            libc::getsockopt(
                self.fd,
                libc::SOL_SOCKET,
                libc::SO_ERROR,
                &mut error as *mut _ as *mut libc::c_void,
                &mut len,
            )
        };

        if result < 0 {
            return Err(io::Error::last_os_error());
        }

        if error != 0 {
            self.state = SocketState::Error;
            return Err(io::Error::from_raw_os_error(error));
        }

        self.state = SocketState::Connected;
        Ok(())
    }

    /// Accept a connection (TCP only)
    pub fn accept(&self) -> io::Result<(AsyncSocket, SocketAddr)> {
        let mut addr: libc::sockaddr_in = unsafe { std::mem::zeroed() };
        let mut len: libc::socklen_t = std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t;

        let fd = unsafe {
            libc::accept4(
                self.fd,
                &mut addr as *mut _ as *mut libc::sockaddr,
                &mut len,
                libc::SOCK_NONBLOCK,
            )
        };

        if fd < 0 {
            return Err(io::Error::last_os_error());
        }

        let socket_addr = raw_to_socket_addr(&addr)?;
        let mut socket = AsyncSocket::new(fd, Token(0));
        socket.state = SocketState::Connected;
        socket.remote_addr = Some(socket_addr);

        Ok((socket, socket_addr))
    }

    /// Non-blocking read
    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        let result =
            unsafe { libc::read(self.fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };

        if result < 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(result as usize)
    }

    /// Non-blocking write
    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        let result =
            unsafe { libc::write(self.fd, buf.as_ptr() as *const libc::c_void, buf.len()) };

        if result < 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(result as usize)
    }

    /// Non-blocking recv (with flags)
    pub fn recv(&self, buf: &mut [u8], flags: i32) -> io::Result<usize> {
        let result = unsafe {
            libc::recv(
                self.fd,
                buf.as_mut_ptr() as *mut libc::c_void,
                buf.len(),
                flags,
            )
        };

        if result < 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(result as usize)
    }

    /// Non-blocking send (with flags)
    pub fn send(&self, buf: &[u8], flags: i32) -> io::Result<usize> {
        let result = unsafe {
            libc::send(
                self.fd,
                buf.as_ptr() as *const libc::c_void,
                buf.len(),
                flags,
            )
        };

        if result < 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(result as usize)
    }

    /// Non-blocking recvfrom (UDP)
    pub fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        let mut addr: libc::sockaddr_in = unsafe { std::mem::zeroed() };
        let mut len: libc::socklen_t = std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t;

        let result = unsafe {
            libc::recvfrom(
                self.fd,
                buf.as_mut_ptr() as *mut libc::c_void,
                buf.len(),
                0,
                &mut addr as *mut _ as *mut libc::sockaddr,
                &mut len,
            )
        };

        if result < 0 {
            return Err(io::Error::last_os_error());
        }

        let socket_addr = raw_to_socket_addr(&addr)?;
        Ok((result as usize, socket_addr))
    }

    /// Non-blocking sendto (UDP)
    pub fn send_to(&self, buf: &[u8], addr: SocketAddr) -> io::Result<usize> {
        let (sockaddr, len) = socket_addr_to_raw(&addr);

        let result = unsafe {
            libc::sendto(
                self.fd,
                buf.as_ptr() as *const libc::c_void,
                buf.len(),
                0,
                &sockaddr as *const _ as *const libc::sockaddr,
                len,
            )
        };

        if result < 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(result as usize)
    }

    /// Shutdown the socket
    pub fn shutdown(&mut self, how: std::net::Shutdown) -> io::Result<()> {
        let how = match how {
            std::net::Shutdown::Read => libc::SHUT_RD,
            std::net::Shutdown::Write => libc::SHUT_WR,
            std::net::Shutdown::Both => libc::SHUT_RDWR,
        };

        let result = unsafe { libc::shutdown(self.fd, how) };

        if result < 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(())
    }

    /// Close the socket
    pub fn close(&mut self) -> io::Result<()> {
        if self.state == SocketState::Closed {
            return Ok(());
        }

        let result = unsafe { libc::close(self.fd) };
        self.state = SocketState::Closed;

        if result < 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(())
    }

    /// Set socket option
    pub fn set_option<T>(&self, level: i32, name: i32, value: &T) -> io::Result<()> {
        let result = unsafe {
            libc::setsockopt(
                self.fd,
                level,
                name,
                value as *const T as *const libc::c_void,
                std::mem::size_of::<T>() as libc::socklen_t,
            )
        };

        if result < 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(())
    }

    /// Enable SO_REUSEADDR
    pub fn set_reuse_addr(&self, reuse: bool) -> io::Result<()> {
        let val: libc::c_int = if reuse { 1 } else { 0 };
        self.set_option(libc::SOL_SOCKET, libc::SO_REUSEADDR, &val)
    }

    /// Enable TCP_NODELAY
    pub fn set_nodelay(&self, nodelay: bool) -> io::Result<()> {
        let val: libc::c_int = if nodelay { 1 } else { 0 };
        self.set_option(libc::IPPROTO_TCP, libc::TCP_NODELAY, &val)
    }

    /// Set the token (called when registering with runtime)
    pub fn set_token(&mut self, token: Token) {
        self.token = token;
    }
}

impl Drop for AsyncSocket {
    fn drop(&mut self) {
        if self.state != SocketState::Closed {
            unsafe { libc::close(self.fd) };
        }
    }
}

impl AsRawFd for AsyncSocket {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

// Helper functions for address conversion

fn socket_addr_to_raw(addr: &SocketAddr) -> (libc::sockaddr_in, libc::socklen_t) {
    match addr {
        SocketAddr::V4(v4) => {
            let mut sockaddr: libc::sockaddr_in = unsafe { std::mem::zeroed() };
            sockaddr.sin_family = libc::AF_INET as libc::sa_family_t;
            sockaddr.sin_port = v4.port().to_be();
            sockaddr.sin_addr.s_addr = u32::from_ne_bytes(v4.ip().octets());
            (
                sockaddr,
                std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t,
            )
        }
        SocketAddr::V6(_) => {
            // For simplicity, only IPv4 for now
            panic!("IPv6 not yet supported");
        }
    }
}

fn raw_to_socket_addr(addr: &libc::sockaddr_in) -> io::Result<SocketAddr> {
    let ip = std::net::Ipv4Addr::from(addr.sin_addr.s_addr.to_ne_bytes());
    let port = u16::from_be(addr.sin_port);
    Ok(SocketAddr::V4(std::net::SocketAddrV4::new(ip, port)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp_socket_creation() {
        let socket = AsyncSocket::tcp();
        assert!(socket.is_ok());
        let socket = socket.unwrap();
        assert!(socket.fd() >= 0);
        assert_eq!(socket.state(), SocketState::Idle);
    }

    #[test]
    fn test_udp_socket_creation() {
        let socket = AsyncSocket::udp();
        assert!(socket.is_ok());
        let socket = socket.unwrap();
        assert!(socket.fd() >= 0);
        assert_eq!(socket.state(), SocketState::Idle);
    }
}
