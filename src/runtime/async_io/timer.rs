//! Timer wheel for efficient timeout handling
//!
//! Implements a hierarchical timer wheel for managing many concurrent timers
//! with O(1) insertion and deletion.

use super::Token;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Timer entry
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct TimerEntry {
    token: Token,
    deadline: Instant,
    expired: bool,
}

/// Simple timer for use in async runtime
#[derive(Debug)]
pub struct Timer {
    /// Token identifying this timer
    pub token: Token,
    /// When the timer expires
    pub deadline: Instant,
}

impl Timer {
    /// Create a new timer with the given deadline
    pub fn new(token: Token, deadline: Instant) -> Self {
        Self { token, deadline }
    }

    /// Create a timer that expires after the given duration
    pub fn after(token: Token, duration: Duration) -> Self {
        Self::new(token, Instant::now() + duration)
    }

    /// Check if the timer has expired
    pub fn is_expired(&self) -> bool {
        Instant::now() >= self.deadline
    }

    /// Get remaining time until expiry
    pub fn remaining(&self) -> Duration {
        let now = Instant::now();
        if now >= self.deadline {
            Duration::ZERO
        } else {
            self.deadline - now
        }
    }
}

/// Hierarchical timer wheel for efficient timer management
///
/// Uses a simple hash-based approach for small numbers of timers,
/// and can be upgraded to a true wheel for larger workloads.
pub struct TimerWheel {
    /// All timers indexed by token
    timers: HashMap<Token, TimerEntry>,
    /// Recently expired timers (not yet processed)
    expired: Vec<Token>,
    /// Last time we advanced the wheel
    last_tick: Instant,
    /// Resolution of the timer wheel
    #[allow(dead_code)]
    tick_duration: Duration,
}

impl TimerWheel {
    /// Create a new timer wheel with default resolution (1ms)
    pub fn new() -> Self {
        Self::with_resolution(Duration::from_millis(1))
    }

    /// Create a timer wheel with custom resolution
    pub fn with_resolution(tick_duration: Duration) -> Self {
        Self {
            timers: HashMap::new(),
            expired: Vec::new(),
            last_tick: Instant::now(),
            tick_duration,
        }
    }

    /// Insert a timer
    pub fn insert(&mut self, token: Token, deadline: Instant) {
        self.timers.insert(
            token,
            TimerEntry {
                token,
                deadline,
                expired: false,
            },
        );
    }

    /// Insert a timer that expires after a duration
    pub fn insert_after(&mut self, token: Token, duration: Duration) {
        self.insert(token, Instant::now() + duration);
    }

    /// Remove a timer
    pub fn remove(&mut self, token: Token) -> bool {
        self.timers.remove(&token).is_some()
    }

    /// Reset a timer to a new deadline
    pub fn reset(&mut self, token: Token, deadline: Instant) -> bool {
        if let Some(entry) = self.timers.get_mut(&token) {
            entry.deadline = deadline;
            entry.expired = false;
            true
        } else {
            false
        }
    }

    /// Reset a timer to expire after a duration from now
    pub fn reset_after(&mut self, token: Token, duration: Duration) -> bool {
        self.reset(token, Instant::now() + duration)
    }

    /// Advance the timer wheel to the current time
    pub fn advance(&mut self, now: Instant) {
        self.expired.clear();

        for (token, entry) in &mut self.timers {
            if !entry.expired && entry.deadline <= now {
                entry.expired = true;
                self.expired.push(*token);
            }
        }

        self.last_tick = now;
    }

    /// Get all expired timers since last advance
    pub fn expired_timers(&self) -> impl Iterator<Item = Token> + '_ {
        self.expired.iter().copied()
    }

    /// Drain expired timers (removes them from the wheel)
    pub fn drain_expired(&mut self) -> Vec<Token> {
        let expired: Vec<Token> = self
            .timers
            .iter()
            .filter(|(_, e)| e.expired)
            .map(|(t, _)| *t)
            .collect();

        for token in &expired {
            self.timers.remove(token);
        }

        expired
    }

    /// Check if a specific timer has expired
    pub fn is_expired(&self, token: Token) -> bool {
        self.timers.get(&token).map(|e| e.expired).unwrap_or(false)
    }

    /// Get the next expiry time (for calculating poll timeout)
    pub fn next_expiry(&self) -> Option<Instant> {
        self.timers
            .values()
            .filter(|e| !e.expired)
            .map(|e| e.deadline)
            .min()
    }

    /// Calculate timeout until next expiry
    pub fn timeout_until_next(&self) -> Option<Duration> {
        self.next_expiry().map(|deadline| {
            let now = Instant::now();
            if deadline <= now {
                Duration::ZERO
            } else {
                deadline - now
            }
        })
    }

    /// Number of pending timers
    pub fn len(&self) -> usize {
        self.timers.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.timers.is_empty()
    }

    /// Clear all timers
    pub fn clear(&mut self) {
        self.timers.clear();
        self.expired.clear();
    }

    /// Get statistics
    pub fn stats(&self) -> TimerStats {
        let active = self.timers.values().filter(|e| !e.expired).count();
        let expired = self.timers.values().filter(|e| e.expired).count();

        TimerStats {
            total: self.timers.len(),
            active,
            expired,
        }
    }
}

impl Default for TimerWheel {
    fn default() -> Self {
        Self::new()
    }
}

/// Timer wheel statistics
#[derive(Debug, Clone, Copy)]
pub struct TimerStats {
    pub total: usize,
    pub active: usize,
    pub expired: usize,
}

/// A repeating timer that automatically reschedules
#[allow(dead_code)]
pub struct PeriodicTimer {
    token: Token,
    interval: Duration,
    next_fire: Instant,
}

#[allow(dead_code)]
impl PeriodicTimer {
    /// Create a new periodic timer
    pub fn new(token: Token, interval: Duration) -> Self {
        Self {
            token,
            interval,
            next_fire: Instant::now() + interval,
        }
    }

    /// Get the token
    pub fn token(&self) -> Token {
        self.token
    }

    /// Get the interval
    pub fn interval(&self) -> Duration {
        self.interval
    }

    /// Check if it's time to fire
    pub fn should_fire(&self) -> bool {
        Instant::now() >= self.next_fire
    }

    /// Fire the timer and schedule next occurrence
    pub fn fire(&mut self) -> bool {
        if self.should_fire() {
            // Calculate next fire time based on interval (not current time)
            // This prevents drift
            self.next_fire += self.interval;

            // If we're way behind, catch up
            let now = Instant::now();
            while self.next_fire <= now {
                self.next_fire += self.interval;
            }

            true
        } else {
            false
        }
    }

    /// Get time until next fire
    pub fn time_until_fire(&self) -> Duration {
        let now = Instant::now();
        if now >= self.next_fire {
            Duration::ZERO
        } else {
            self.next_fire - now
        }
    }

    /// Change the interval
    pub fn set_interval(&mut self, interval: Duration) {
        self.interval = interval;
    }

    /// Reset to fire after one interval from now
    pub fn reset(&mut self) {
        self.next_fire = Instant::now() + self.interval;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_wheel_insert_remove() {
        let mut wheel = TimerWheel::new();
        let token = Token(1);

        wheel.insert_after(token, Duration::from_millis(100));
        assert_eq!(wheel.len(), 1);
        assert!(!wheel.is_expired(token));

        wheel.remove(token);
        assert_eq!(wheel.len(), 0);
    }

    #[test]
    fn test_timer_wheel_expiry() {
        let mut wheel = TimerWheel::new();
        let token = Token(1);

        wheel.insert_after(token, Duration::from_millis(10));
        assert!(!wheel.is_expired(token));

        std::thread::sleep(Duration::from_millis(20));
        wheel.advance(Instant::now());

        assert!(wheel.is_expired(token));
    }

    #[test]
    fn test_next_expiry() {
        let mut wheel = TimerWheel::new();

        assert!(wheel.next_expiry().is_none());

        let now = Instant::now();
        wheel.insert(Token(1), now + Duration::from_millis(100));
        wheel.insert(Token(2), now + Duration::from_millis(50));
        wheel.insert(Token(3), now + Duration::from_millis(200));

        let next = wheel.next_expiry().unwrap();
        let expected = now + Duration::from_millis(50);

        // Allow 1ms tolerance
        assert!(
            (next - expected) < Duration::from_millis(1)
                || (expected - next) < Duration::from_millis(1)
        );
    }

    #[test]
    fn test_periodic_timer() {
        let mut timer = PeriodicTimer::new(Token(1), Duration::from_millis(10));

        assert!(!timer.should_fire());

        std::thread::sleep(Duration::from_millis(15));
        assert!(timer.should_fire());

        timer.fire();
        assert!(!timer.should_fire());
    }

    #[test]
    fn test_timer_reset() {
        let mut wheel = TimerWheel::new();
        let token = Token(1);

        wheel.insert_after(token, Duration::from_millis(10));
        std::thread::sleep(Duration::from_millis(15));
        wheel.advance(Instant::now());

        assert!(wheel.is_expired(token));

        wheel.reset_after(token, Duration::from_millis(100));
        assert!(!wheel.is_expired(token));
    }
}
