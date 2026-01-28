//! Security stencils for capability-based memory access
//!
//! Implements bounds checking and capability enforcement in generated code.

use crate::ir::{CapPerms, FatPointer};

/// FFI-safe result for capability creation
#[repr(C)]
pub struct CapNewResult {
    /// Capability metadata
    pub meta: u64,
    /// Capability address
    pub addr: u64,
}

/// FFI-safe result for capability restriction
#[repr(C)]
pub struct CapRestrictResult {
    /// 0 = success, 1 = failure
    pub status: u64,
    /// Capability metadata (if success)
    pub meta: u64,
    /// Capability address (if success)
    pub addr: u64,
}

/// Result of a capability check
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapCheckResult {
    /// Access is allowed
    Ok,
    /// Invalid capability tag
    InvalidTag,
    /// Access out of bounds
    OutOfBounds,
    /// Permission denied
    PermissionDenied,
    /// Taint violation (accessing tainted data without sanitization)
    TaintViolation,
}

/// Check capability for memory access
#[inline(always)]
pub fn check_capability(
    cap: &FatPointer,
    access_size: usize,
    required_perms: u8,
) -> CapCheckResult {
    // 1. Check tag validity
    if !cap.is_valid() {
        return CapCheckResult::InvalidTag;
    }

    // 2. Check bounds
    if !cap.check_bounds(access_size) {
        return CapCheckResult::OutOfBounds;
    }

    // 3. Check permissions
    if (cap.perms.0 & required_perms) != required_perms {
        return CapCheckResult::PermissionDenied;
    }

    CapCheckResult::Ok
}

/// Check capability for read access
#[inline(always)]
pub fn check_read(cap: &FatPointer, size: usize) -> CapCheckResult {
    check_capability(cap, size, CapPerms::READ)
}

/// Check capability for write access
#[inline(always)]
pub fn check_write(cap: &FatPointer, size: usize) -> CapCheckResult {
    check_capability(cap, size, CapPerms::WRITE)
}

/// Check capability for execute access
#[inline(always)]
pub fn check_exec(cap: &FatPointer) -> CapCheckResult {
    check_capability(cap, 0, CapPerms::EXEC)
}

/// Bounds check result for inline checking
#[repr(C)]
pub struct BoundsCheckResult {
    pub valid: bool,
    pub trap_code: u8,
}

/// Inline bounds check function called from generated code
///
/// Arguments passed via registers:
/// - rdi: capability metadata (high 64 bits)
/// - rsi: capability address (low 64 bits)
/// - rdx: access size
/// - rcx: required permissions
#[no_mangle]
pub extern "C" fn neurlang_bounds_check(
    cap_meta: u64,
    cap_addr: u64,
    access_size: u64,
    required_perms: u64,
) -> u64 {
    let cap = FatPointer::decode(cap_meta, cap_addr);
    let result = check_capability(&cap, access_size as usize, required_perms as u8);

    match result {
        CapCheckResult::Ok => 0,
        CapCheckResult::InvalidTag => 1,
        CapCheckResult::OutOfBounds => 2,
        CapCheckResult::PermissionDenied => 3,
        CapCheckResult::TaintViolation => 4,
    }
}

/// Create a new capability (privileged operation)
#[no_mangle]
pub extern "C" fn neurlang_cap_new(base: u64, length: u32, perms: u8) -> CapNewResult {
    let cap = FatPointer::new(base, length, CapPerms::new(perms));
    let (meta, addr) = cap.encode();
    CapNewResult { meta, addr }
}

/// Restrict a capability (can only shrink bounds/permissions)
#[no_mangle]
pub extern "C" fn neurlang_cap_restrict(
    cap_meta: u64,
    cap_addr: u64,
    new_base: u64,
    new_length: u32,
    new_perms: u8,
) -> CapRestrictResult {
    let cap = FatPointer::decode(cap_meta, cap_addr);

    match cap.restrict(new_base, new_length, CapPerms::new(new_perms)) {
        Some(new_cap) => {
            let (meta, addr) = new_cap.encode();
            CapRestrictResult {
                status: 0,
                meta,
                addr,
            } // Success
        }
        None => CapRestrictResult {
            status: 1,
            meta: 0,
            addr: 0,
        }, // Failure
    }
}

/// Query capability properties
#[no_mangle]
pub extern "C" fn neurlang_cap_query(cap_meta: u64, cap_addr: u64, query_type: u8) -> u64 {
    let cap = FatPointer::decode(cap_meta, cap_addr);

    match query_type {
        0 => cap.base,
        1 => cap.length as u64,
        2 => cap.perms.0 as u64,
        3 => cap.address,
        4 => cap.taint as u64,
        5 => cap.is_valid() as u64,
        _ => 0,
    }
}

/// Taint tracking state for a value
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TaintLevel {
    /// Untainted - safe to use
    Clean = 0,
    /// User input - needs validation
    UserInput = 1,
    /// Network data - needs sanitization
    NetworkData = 2,
    /// File data - needs validation
    FileData = 3,
    /// Maximum taint level
    Toxic = 255,
}

/// Taint tracking context
pub struct TaintTracker {
    /// Taint levels for each register
    register_taint: [TaintLevel; 32],
    /// Whether taint checking is enabled (for runtime toggle)
    #[allow(dead_code)]
    enabled: bool,
}

impl TaintTracker {
    pub fn new() -> Self {
        Self {
            register_taint: [TaintLevel::Clean; 32],
            enabled: true,
        }
    }

    /// Mark a register as tainted
    pub fn taint(&mut self, reg: u8, level: TaintLevel) {
        if reg < 32 {
            self.register_taint[reg as usize] = level;
        }
    }

    /// Remove taint from a register (after validation)
    pub fn sanitize(&mut self, reg: u8) {
        if reg < 32 {
            self.register_taint[reg as usize] = TaintLevel::Clean;
        }
    }

    /// Check if a register is tainted
    pub fn is_tainted(&self, reg: u8) -> bool {
        if reg < 32 {
            self.register_taint[reg as usize] != TaintLevel::Clean
        } else {
            false
        }
    }

    /// Get taint level
    pub fn get_taint(&self, reg: u8) -> TaintLevel {
        if reg < 32 {
            self.register_taint[reg as usize]
        } else {
            TaintLevel::Clean
        }
    }

    /// Propagate taint from src to dst
    pub fn propagate(&mut self, dst: u8, src: u8) {
        if dst < 32 && src < 32 {
            let src_taint = self.register_taint[src as usize];
            self.register_taint[dst as usize] = src_taint;
        }
    }

    /// Propagate taint from two sources (binary operation)
    pub fn propagate_binary(&mut self, dst: u8, src1: u8, src2: u8) {
        if dst < 32 && src1 < 32 && src2 < 32 {
            let t1 = self.register_taint[src1 as usize] as u8;
            let t2 = self.register_taint[src2 as usize] as u8;
            // Max taint wins
            self.register_taint[dst as usize] = if t1 > t2 {
                self.register_taint[src1 as usize]
            } else {
                self.register_taint[src2 as usize]
            };
        }
    }
}

impl Default for TaintTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Runtime security context
pub struct SecurityContext {
    /// Taint tracking
    pub taint: TaintTracker,
    /// Whether to trap on security violations
    pub trap_on_violation: bool,
    /// Count of security violations
    pub violation_count: u64,
}

impl SecurityContext {
    pub fn new() -> Self {
        Self {
            taint: TaintTracker::new(),
            trap_on_violation: true,
            violation_count: 0,
        }
    }

    /// Record a security violation
    pub fn record_violation(&mut self, kind: CapCheckResult) {
        self.violation_count += 1;
        if self.trap_on_violation {
            // In a real implementation, this would trigger a trap
            panic!("Security violation: {:?}", kind);
        }
    }
}

impl Default for SecurityContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_check() {
        let cap = FatPointer::new(0x1000, 256, CapPerms::new(CapPerms::READ | CapPerms::WRITE));

        assert_eq!(check_read(&cap, 1), CapCheckResult::Ok);
        assert_eq!(check_read(&cap, 256), CapCheckResult::Ok);
        assert_eq!(check_read(&cap, 257), CapCheckResult::OutOfBounds);
        assert_eq!(check_exec(&cap), CapCheckResult::PermissionDenied);
    }

    #[test]
    fn test_taint_tracking() {
        let mut tracker = TaintTracker::new();

        tracker.taint(0, TaintLevel::UserInput);
        assert!(tracker.is_tainted(0));
        assert!(!tracker.is_tainted(1));

        tracker.propagate(1, 0);
        assert!(tracker.is_tainted(1));

        tracker.sanitize(0);
        assert!(!tracker.is_tainted(0));
    }

    #[test]
    fn test_taint_propagation() {
        let mut tracker = TaintTracker::new();

        tracker.taint(1, TaintLevel::UserInput);
        tracker.taint(2, TaintLevel::NetworkData);

        tracker.propagate_binary(0, 1, 2);

        // Should have higher taint level (NetworkData > UserInput)
        assert_eq!(tracker.get_taint(0), TaintLevel::NetworkData);
    }
}
