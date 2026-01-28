//! Async I/O Runtime for Neurlang
//!
//! Provides non-blocking I/O operations with platform-specific event loops:
//! - Linux: epoll (+ io_uring when available)
//! - macOS/BSD: kqueue
//! - Windows: IOCP
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    ASYNC I/O RUNTIME                            │
//! ├─────────────────────────────────────────────────────────────────┤
//! │  EventLoop: Platform-specific event notification                │
//! │  Reactor: Manages I/O resources and their readiness             │
//! │  Executor: Runs tasks and handles completions                   │
//! │  Waker: Signals tasks when I/O is ready                         │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Usage
//!
//! ```ignore
//! let mut runtime = AsyncRuntime::new()?;
//!
//! // Register a socket for async operations
//! let token = runtime.register_socket(socket_fd, Interest::READABLE)?;
//!
//! // Poll for events
//! runtime.poll(Duration::from_millis(100))?;
//!
//! // Check if socket is ready
//! if runtime.is_ready(token, Interest::READABLE) {
//!     // Perform non-blocking read
//! }
//! ```

mod event_loop;
mod executor;
mod file;
mod reactor;
mod socket;
mod timer;

pub use event_loop::{Event, EventLoop, Interest, Token};
pub use executor::{Executor, Task, TaskId, TaskState};
pub use file::{AsyncFile, FileState};
pub use reactor::Reactor;
pub use socket::{AsyncSocket, SocketState};
pub use timer::{Timer, TimerWheel};

use std::io;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// Global token counter for unique identification
static TOKEN_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Generate a unique token
pub fn next_token() -> Token {
    Token(TOKEN_COUNTER.fetch_add(1, Ordering::Relaxed))
}

/// Async I/O runtime combining event loop, reactor, and executor
pub struct AsyncRuntime {
    /// Platform-specific event loop
    event_loop: EventLoop,
    /// I/O resource management
    reactor: Reactor,
    /// Task executor
    executor: Executor,
    /// Timer wheel for timeouts
    timer_wheel: TimerWheel,
    /// Pending events from last poll
    events: Vec<Event>,
}

impl AsyncRuntime {
    /// Create a new async runtime
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            event_loop: EventLoop::new()?,
            reactor: Reactor::new(),
            executor: Executor::new(),
            timer_wheel: TimerWheel::new(),
            events: Vec::with_capacity(1024),
        })
    }

    /// Register a raw file descriptor for async I/O
    pub fn register_fd(&mut self, fd: i32, interest: Interest) -> io::Result<Token> {
        let token = next_token();
        self.event_loop.register(fd, token, interest)?;
        self.reactor.insert(token, fd, interest);
        Ok(token)
    }

    /// Modify interest for an existing registration
    pub fn modify_interest(&mut self, token: Token, interest: Interest) -> io::Result<()> {
        if let Some(fd) = self.reactor.get_fd(token) {
            self.event_loop.modify(fd, token, interest)?;
            self.reactor.set_interest(token, interest);
        }
        Ok(())
    }

    /// Deregister a file descriptor
    pub fn deregister(&mut self, token: Token) -> io::Result<()> {
        if let Some(fd) = self.reactor.remove(token) {
            self.event_loop.deregister(fd)?;
        }
        Ok(())
    }

    /// Poll for I/O events with timeout
    pub fn poll(&mut self, timeout: Option<Duration>) -> io::Result<usize> {
        // Process any expired timers first
        let now = std::time::Instant::now();
        self.timer_wheel.advance(now);

        // Calculate effective timeout based on next timer
        let effective_timeout = match self.timer_wheel.next_expiry() {
            Some(expiry) if expiry > now => {
                let timer_timeout = expiry - now;
                match timeout {
                    Some(t) => Some(t.min(timer_timeout)),
                    None => Some(timer_timeout),
                }
            }
            Some(_) => Some(Duration::ZERO), // Timer already expired
            None => timeout,
        };

        // Poll the event loop
        self.events.clear();
        let count = self.event_loop.poll(&mut self.events, effective_timeout)?;

        // Mark resources as ready in the reactor
        for event in &self.events {
            self.reactor.set_ready(event.token, event.interest);
        }

        Ok(count)
    }

    /// Check if a token is ready for the given interest
    pub fn is_ready(&self, token: Token, interest: Interest) -> bool {
        self.reactor.is_ready(token, interest)
    }

    /// Clear ready state for a token
    pub fn clear_ready(&mut self, token: Token, interest: Interest) {
        self.reactor.clear_ready(token, interest);
    }

    /// Schedule a timer
    pub fn schedule_timer(&mut self, duration: Duration) -> Token {
        let token = next_token();
        let deadline = std::time::Instant::now() + duration;
        self.timer_wheel.insert(token, deadline);
        token
    }

    /// Cancel a timer
    pub fn cancel_timer(&mut self, token: Token) {
        self.timer_wheel.remove(token);
    }

    /// Check if a timer has expired
    pub fn timer_expired(&self, token: Token) -> bool {
        self.timer_wheel.is_expired(token)
    }

    /// Create an async socket wrapper
    pub fn create_async_socket(&mut self, fd: i32) -> io::Result<AsyncSocket> {
        let token = self.register_fd(fd, Interest::READABLE | Interest::WRITABLE)?;
        Ok(AsyncSocket::new(fd, token))
    }

    /// Create an async file wrapper
    pub fn create_async_file(&mut self, fd: i32) -> io::Result<AsyncFile> {
        let token = self.register_fd(fd, Interest::READABLE | Interest::WRITABLE)?;
        Ok(AsyncFile::new(fd, token))
    }

    /// Spawn a new task
    pub fn spawn(&mut self, task: Task) -> TaskId {
        self.executor.spawn(task)
    }

    /// Run the event loop until all tasks complete or timeout
    pub fn run(&mut self, timeout: Option<Duration>) -> io::Result<()> {
        let start = std::time::Instant::now();

        loop {
            // Check timeout
            if let Some(t) = timeout {
                if start.elapsed() >= t {
                    break;
                }
            }

            // Check if there are any pending tasks
            if self.executor.is_empty() {
                break;
            }

            // Poll for events
            let remaining = timeout.map(|t| t.saturating_sub(start.elapsed()));
            self.poll(remaining)?;

            // Run ready tasks
            self.executor.run_ready(&self.reactor);
        }

        Ok(())
    }

    /// Get statistics about the runtime
    pub fn stats(&self) -> RuntimeStats {
        RuntimeStats {
            registered_fds: self.reactor.len(),
            pending_timers: self.timer_wheel.len(),
            pending_tasks: self.executor.len(),
        }
    }
}

/// Runtime statistics
#[derive(Debug, Clone, Copy)]
pub struct RuntimeStats {
    pub registered_fds: usize,
    pub pending_timers: usize,
    pub pending_tasks: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let runtime = AsyncRuntime::new();
        assert!(runtime.is_ok());
    }

    #[test]
    fn test_token_generation() {
        let t1 = next_token();
        let t2 = next_token();
        assert_ne!(t1, t2);
    }

    #[test]
    fn test_timer_scheduling() {
        let mut runtime = AsyncRuntime::new().unwrap();
        let token = runtime.schedule_timer(Duration::from_millis(10));
        assert!(!runtime.timer_expired(token));

        std::thread::sleep(Duration::from_millis(20));
        runtime.timer_wheel.advance(std::time::Instant::now());
        assert!(runtime.timer_expired(token));
    }
}
