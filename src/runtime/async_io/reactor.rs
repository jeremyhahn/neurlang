//! Reactor for managing I/O resource readiness
//!
//! The reactor tracks:
//! - Registered file descriptors and their tokens
//! - Current interest flags for each resource
//! - Ready state (which resources are ready for I/O)

use super::{Interest, Token};
use std::collections::HashMap;

/// Resource entry in the reactor
#[derive(Debug, Clone)]
struct Resource {
    fd: i32,
    interest: Interest,
    ready: Interest,
}

/// Manages I/O resources and their readiness state
pub struct Reactor {
    resources: HashMap<Token, Resource>,
    fd_to_token: HashMap<i32, Token>,
}

impl Reactor {
    /// Create a new reactor
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
            fd_to_token: HashMap::new(),
        }
    }

    /// Insert a new resource
    pub fn insert(&mut self, token: Token, fd: i32, interest: Interest) {
        self.resources.insert(
            token,
            Resource {
                fd,
                interest,
                ready: Interest(0),
            },
        );
        self.fd_to_token.insert(fd, token);
    }

    /// Remove a resource
    pub fn remove(&mut self, token: Token) -> Option<i32> {
        if let Some(resource) = self.resources.remove(&token) {
            self.fd_to_token.remove(&resource.fd);
            Some(resource.fd)
        } else {
            None
        }
    }

    /// Get the file descriptor for a token
    pub fn get_fd(&self, token: Token) -> Option<i32> {
        self.resources.get(&token).map(|r| r.fd)
    }

    /// Get the token for a file descriptor
    pub fn get_token(&self, fd: i32) -> Option<Token> {
        self.fd_to_token.get(&fd).copied()
    }

    /// Set interest for a resource
    pub fn set_interest(&mut self, token: Token, interest: Interest) {
        if let Some(resource) = self.resources.get_mut(&token) {
            resource.interest = interest;
        }
    }

    /// Get interest for a resource
    pub fn get_interest(&self, token: Token) -> Option<Interest> {
        self.resources.get(&token).map(|r| r.interest)
    }

    /// Mark a resource as ready
    pub fn set_ready(&mut self, token: Token, interest: Interest) {
        if let Some(resource) = self.resources.get_mut(&token) {
            resource.ready |= interest;
        }
    }

    /// Check if a resource is ready
    pub fn is_ready(&self, token: Token, interest: Interest) -> bool {
        self.resources
            .get(&token)
            .map(|r| (r.ready & interest).0 != 0)
            .unwrap_or(false)
    }

    /// Clear ready state
    pub fn clear_ready(&mut self, token: Token, interest: Interest) {
        if let Some(resource) = self.resources.get_mut(&token) {
            resource.ready = Interest(resource.ready.0 & !interest.0);
        }
    }

    /// Get all ready tokens
    pub fn ready_tokens(&self) -> impl Iterator<Item = (Token, Interest)> + '_ {
        self.resources
            .iter()
            .filter(|(_, r)| r.ready.0 != 0)
            .map(|(&token, r)| (token, r.ready))
    }

    /// Number of registered resources
    pub fn len(&self) -> usize {
        self.resources.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }
}

impl Default for Reactor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reactor_insert_remove() {
        let mut reactor = Reactor::new();
        let token = Token(1);

        reactor.insert(token, 5, Interest::READABLE);
        assert_eq!(reactor.get_fd(token), Some(5));
        assert_eq!(reactor.get_token(5), Some(token));

        let fd = reactor.remove(token);
        assert_eq!(fd, Some(5));
        assert_eq!(reactor.get_fd(token), None);
    }

    #[test]
    fn test_reactor_ready_state() {
        let mut reactor = Reactor::new();
        let token = Token(1);

        reactor.insert(token, 5, Interest::READABLE | Interest::WRITABLE);

        assert!(!reactor.is_ready(token, Interest::READABLE));

        reactor.set_ready(token, Interest::READABLE);
        assert!(reactor.is_ready(token, Interest::READABLE));
        assert!(!reactor.is_ready(token, Interest::WRITABLE));

        reactor.clear_ready(token, Interest::READABLE);
        assert!(!reactor.is_ready(token, Interest::READABLE));
    }
}
