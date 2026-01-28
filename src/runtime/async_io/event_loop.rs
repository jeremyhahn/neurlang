//! Platform-specific event loop implementations
//!
//! Provides a unified interface over:
//! - Linux: epoll
//! - macOS/BSD: kqueue
//! - Windows: IOCP (via polling crate fallback)

use std::io;
use std::time::Duration;

/// Unique token identifying a registered resource
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Token(pub u64);

/// Interest flags for I/O events
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Interest(pub u8);

impl Interest {
    pub const READABLE: Interest = Interest(0b0001);
    pub const WRITABLE: Interest = Interest(0b0010);
    pub const ERROR: Interest = Interest(0b0100);
    pub const HUP: Interest = Interest(0b1000);

    pub fn is_readable(&self) -> bool {
        self.0 & Self::READABLE.0 != 0
    }

    pub fn is_writable(&self) -> bool {
        self.0 & Self::WRITABLE.0 != 0
    }

    pub fn is_error(&self) -> bool {
        self.0 & Self::ERROR.0 != 0
    }

    pub fn is_hup(&self) -> bool {
        self.0 & Self::HUP.0 != 0
    }
}

impl std::ops::BitOr for Interest {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Interest(self.0 | rhs.0)
    }
}

impl std::ops::BitAnd for Interest {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Interest(self.0 & rhs.0)
    }
}

impl std::ops::BitOrAssign for Interest {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// An I/O event from the event loop
#[derive(Debug, Clone, Copy)]
pub struct Event {
    pub token: Token,
    pub interest: Interest,
}

/// Platform-specific event loop
pub struct EventLoop {
    #[cfg(target_os = "linux")]
    inner: LinuxEventLoop,

    #[cfg(any(
        target_os = "macos",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd"
    ))]
    inner: KqueueEventLoop,

    #[cfg(target_os = "windows")]
    inner: WindowsEventLoop,
}

impl EventLoop {
    /// Create a new event loop
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            #[cfg(target_os = "linux")]
            inner: LinuxEventLoop::new()?,

            #[cfg(any(
                target_os = "macos",
                target_os = "freebsd",
                target_os = "openbsd",
                target_os = "netbsd"
            ))]
            inner: KqueueEventLoop::new()?,

            #[cfg(target_os = "windows")]
            inner: WindowsEventLoop::new()?,
        })
    }

    /// Register a file descriptor for events
    pub fn register(&mut self, fd: i32, token: Token, interest: Interest) -> io::Result<()> {
        self.inner.register(fd, token, interest)
    }

    /// Modify interest for an existing registration
    pub fn modify(&mut self, fd: i32, token: Token, interest: Interest) -> io::Result<()> {
        self.inner.modify(fd, token, interest)
    }

    /// Deregister a file descriptor
    pub fn deregister(&mut self, fd: i32) -> io::Result<()> {
        self.inner.deregister(fd)
    }

    /// Poll for events
    pub fn poll(
        &mut self,
        events: &mut Vec<Event>,
        timeout: Option<Duration>,
    ) -> io::Result<usize> {
        self.inner.poll(events, timeout)
    }
}

// =============================================================================
// Linux: epoll implementation
// =============================================================================

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use std::os::unix::io::RawFd;

    pub struct LinuxEventLoop {
        epoll_fd: RawFd,
        events: Vec<libc::epoll_event>,
    }

    impl LinuxEventLoop {
        pub fn new() -> io::Result<Self> {
            let epoll_fd = unsafe { libc::epoll_create1(libc::EPOLL_CLOEXEC) };
            if epoll_fd < 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(Self {
                epoll_fd,
                events: vec![unsafe { std::mem::zeroed() }; 1024],
            })
        }

        pub fn register(&mut self, fd: i32, token: Token, interest: Interest) -> io::Result<()> {
            let mut event = libc::epoll_event {
                events: interest_to_epoll(interest),
                u64: token.0,
            };

            let result =
                unsafe { libc::epoll_ctl(self.epoll_fd, libc::EPOLL_CTL_ADD, fd, &mut event) };

            if result < 0 {
                Err(io::Error::last_os_error())
            } else {
                Ok(())
            }
        }

        pub fn modify(&mut self, fd: i32, token: Token, interest: Interest) -> io::Result<()> {
            let mut event = libc::epoll_event {
                events: interest_to_epoll(interest),
                u64: token.0,
            };

            let result =
                unsafe { libc::epoll_ctl(self.epoll_fd, libc::EPOLL_CTL_MOD, fd, &mut event) };

            if result < 0 {
                Err(io::Error::last_os_error())
            } else {
                Ok(())
            }
        }

        pub fn deregister(&mut self, fd: i32) -> io::Result<()> {
            let result = unsafe {
                libc::epoll_ctl(self.epoll_fd, libc::EPOLL_CTL_DEL, fd, std::ptr::null_mut())
            };

            if result < 0 {
                Err(io::Error::last_os_error())
            } else {
                Ok(())
            }
        }

        pub fn poll(
            &mut self,
            events: &mut Vec<Event>,
            timeout: Option<Duration>,
        ) -> io::Result<usize> {
            let timeout_ms = timeout.map(|d| d.as_millis() as i32).unwrap_or(-1);

            let count = unsafe {
                libc::epoll_wait(
                    self.epoll_fd,
                    self.events.as_mut_ptr(),
                    self.events.len() as i32,
                    timeout_ms,
                )
            };

            if count < 0 {
                let err = io::Error::last_os_error();
                if err.kind() == io::ErrorKind::Interrupted {
                    return Ok(0);
                }
                return Err(err);
            }

            for i in 0..count as usize {
                let epoll_event = &self.events[i];
                events.push(Event {
                    token: Token(epoll_event.u64),
                    interest: epoll_to_interest(epoll_event.events),
                });
            }

            Ok(count as usize)
        }
    }

    impl Drop for LinuxEventLoop {
        fn drop(&mut self) {
            unsafe { libc::close(self.epoll_fd) };
        }
    }

    fn interest_to_epoll(interest: Interest) -> u32 {
        let mut events = libc::EPOLLET as u32; // Edge-triggered
        if interest.is_readable() {
            events |= libc::EPOLLIN as u32;
        }
        if interest.is_writable() {
            events |= libc::EPOLLOUT as u32;
        }
        events |= libc::EPOLLERR as u32;
        events |= libc::EPOLLHUP as u32;
        events
    }

    fn epoll_to_interest(events: u32) -> Interest {
        let mut interest = Interest(0);
        if events & libc::EPOLLIN as u32 != 0 {
            interest |= Interest::READABLE;
        }
        if events & libc::EPOLLOUT as u32 != 0 {
            interest |= Interest::WRITABLE;
        }
        if events & libc::EPOLLERR as u32 != 0 {
            interest |= Interest::ERROR;
        }
        if events & libc::EPOLLHUP as u32 != 0 {
            interest |= Interest::HUP;
        }
        interest
    }
}

#[cfg(target_os = "linux")]
use linux::LinuxEventLoop;

// =============================================================================
// macOS/BSD: kqueue implementation
// =============================================================================

#[cfg(any(
    target_os = "macos",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd"
))]
mod kqueue {
    use super::*;
    use std::os::unix::io::RawFd;

    pub struct KqueueEventLoop {
        kqueue_fd: RawFd,
        events: Vec<libc::kevent>,
    }

    impl KqueueEventLoop {
        pub fn new() -> io::Result<Self> {
            let kqueue_fd = unsafe { libc::kqueue() };
            if kqueue_fd < 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(Self {
                kqueue_fd,
                events: vec![unsafe { std::mem::zeroed() }; 1024],
            })
        }

        pub fn register(&mut self, fd: i32, token: Token, interest: Interest) -> io::Result<()> {
            let mut changes = Vec::new();

            if interest.is_readable() {
                changes.push(libc::kevent {
                    ident: fd as usize,
                    filter: libc::EVFILT_READ,
                    flags: libc::EV_ADD | libc::EV_CLEAR,
                    fflags: 0,
                    data: 0,
                    udata: token.0 as *mut libc::c_void,
                });
            }

            if interest.is_writable() {
                changes.push(libc::kevent {
                    ident: fd as usize,
                    filter: libc::EVFILT_WRITE,
                    flags: libc::EV_ADD | libc::EV_CLEAR,
                    fflags: 0,
                    data: 0,
                    udata: token.0 as *mut libc::c_void,
                });
            }

            if changes.is_empty() {
                return Ok(());
            }

            let result = unsafe {
                libc::kevent(
                    self.kqueue_fd,
                    changes.as_ptr(),
                    changes.len() as i32,
                    std::ptr::null_mut(),
                    0,
                    std::ptr::null(),
                )
            };

            if result < 0 {
                Err(io::Error::last_os_error())
            } else {
                Ok(())
            }
        }

        pub fn modify(&mut self, fd: i32, token: Token, interest: Interest) -> io::Result<()> {
            // For kqueue, we delete and re-add
            self.deregister(fd)?;
            self.register(fd, token, interest)
        }

        pub fn deregister(&mut self, fd: i32) -> io::Result<()> {
            let changes = [
                libc::kevent {
                    ident: fd as usize,
                    filter: libc::EVFILT_READ,
                    flags: libc::EV_DELETE,
                    fflags: 0,
                    data: 0,
                    udata: std::ptr::null_mut(),
                },
                libc::kevent {
                    ident: fd as usize,
                    filter: libc::EVFILT_WRITE,
                    flags: libc::EV_DELETE,
                    fflags: 0,
                    data: 0,
                    udata: std::ptr::null_mut(),
                },
            ];

            // Ignore errors - the filter might not be registered
            unsafe {
                libc::kevent(
                    self.kqueue_fd,
                    changes.as_ptr(),
                    changes.len() as i32,
                    std::ptr::null_mut(),
                    0,
                    std::ptr::null(),
                );
            }

            Ok(())
        }

        pub fn poll(
            &mut self,
            events: &mut Vec<Event>,
            timeout: Option<Duration>,
        ) -> io::Result<usize> {
            let timeout_spec = timeout.map(|d| libc::timespec {
                tv_sec: d.as_secs() as libc::time_t,
                tv_nsec: d.subsec_nanos() as libc::c_long,
            });

            let timeout_ptr = match &timeout_spec {
                Some(ts) => ts as *const libc::timespec,
                None => std::ptr::null(),
            };

            let count = unsafe {
                libc::kevent(
                    self.kqueue_fd,
                    std::ptr::null(),
                    0,
                    self.events.as_mut_ptr(),
                    self.events.len() as i32,
                    timeout_ptr,
                )
            };

            if count < 0 {
                let err = io::Error::last_os_error();
                if err.kind() == io::ErrorKind::Interrupted {
                    return Ok(0);
                }
                return Err(err);
            }

            for i in 0..count as usize {
                let kevent = &self.events[i];
                let token = Token(kevent.udata as u64);

                let mut interest = Interest(0);
                if kevent.filter == libc::EVFILT_READ {
                    interest |= Interest::READABLE;
                }
                if kevent.filter == libc::EVFILT_WRITE {
                    interest |= Interest::WRITABLE;
                }
                if kevent.flags & libc::EV_ERROR != 0 {
                    interest |= Interest::ERROR;
                }
                if kevent.flags & libc::EV_EOF != 0 {
                    interest |= Interest::HUP;
                }

                events.push(Event { token, interest });
            }

            Ok(count as usize)
        }
    }

    impl Drop for KqueueEventLoop {
        fn drop(&mut self) {
            unsafe { libc::close(self.kqueue_fd) };
        }
    }
}

#[cfg(any(
    target_os = "macos",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd"
))]
use kqueue::KqueueEventLoop;

// =============================================================================
// Windows: IOCP implementation (simplified)
// =============================================================================

#[cfg(target_os = "windows")]
mod windows {
    use super::*;

    pub struct WindowsEventLoop {
        // Windows IOCP implementation would go here
        // For now, using a polling fallback
        poll_fds: std::collections::HashMap<i32, (Token, Interest)>,
    }

    impl WindowsEventLoop {
        pub fn new() -> io::Result<Self> {
            Ok(Self {
                poll_fds: std::collections::HashMap::new(),
            })
        }

        pub fn register(&mut self, fd: i32, token: Token, interest: Interest) -> io::Result<()> {
            self.poll_fds.insert(fd, (token, interest));
            Ok(())
        }

        pub fn modify(&mut self, fd: i32, token: Token, interest: Interest) -> io::Result<()> {
            self.poll_fds.insert(fd, (token, interest));
            Ok(())
        }

        pub fn deregister(&mut self, fd: i32) -> io::Result<()> {
            self.poll_fds.remove(&fd);
            Ok(())
        }

        pub fn poll(
            &mut self,
            events: &mut Vec<Event>,
            timeout: Option<Duration>,
        ) -> io::Result<usize> {
            // Simplified polling for Windows
            // A real implementation would use IOCP
            if let Some(t) = timeout {
                std::thread::sleep(t.min(Duration::from_millis(10)));
            }

            // Return all registered as ready (simplified)
            for (&_fd, &(token, interest)) in &self.poll_fds {
                events.push(Event { token, interest });
            }

            Ok(events.len())
        }
    }
}

#[cfg(target_os = "windows")]
use windows::WindowsEventLoop;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interest_flags() {
        let interest = Interest::READABLE | Interest::WRITABLE;
        assert!(interest.is_readable());
        assert!(interest.is_writable());
        assert!(!interest.is_error());
    }

    #[test]
    fn test_event_loop_creation() {
        let loop_result = EventLoop::new();
        assert!(loop_result.is_ok());
    }
}
