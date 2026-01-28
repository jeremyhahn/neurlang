# Security Documentation

Capability-based security and taint tracking.

## Security Model

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Security Layers                                  │
└─────────────────────────────────────────────────────────────────────┘

  Layer 3: Taint Tracking
  ┌─────────────────────────────────────────────────────────────────┐
  │ Track untrusted data flow, require sanitization before use      │
  └─────────────────────────────────────────────────────────────────┘

  Layer 2: Capability Permissions
  ┌─────────────────────────────────────────────────────────────────┐
  │ Control read/write/execute access per memory region             │
  └─────────────────────────────────────────────────────────────────┘

  Layer 1: Bounds Checking
  ┌─────────────────────────────────────────────────────────────────┐
  │ Automatic bounds validation on every memory access              │
  └─────────────────────────────────────────────────────────────────┘
```

## Fat Pointers

### Structure

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Fat Pointer (128 bits)                           │
└─────────────────────────────────────────────────────────────────────┘

  Metadata (64 bits):
  ┌──────────┬──────────┬──────────┬─────────────────┬───────────────┐
  │   TAG    │  TAINT   │  PERMS   │     LENGTH      │  BASE (low)   │
  │  8 bits  │  8 bits  │  8 bits  │    32 bits      │    8 bits     │
  └──────────┴──────────┴──────────┴─────────────────┴───────────────┘

  Address (64 bits):
  ┌──────────────────────────────────────────────────┬───────────────┐
  │              CURRENT ADDRESS                      │ BASE (high)   │
  │                  56 bits                          │    8 bits     │
  └──────────────────────────────────────────────────┴───────────────┘

  Fields:
  • TAG: Magic value (0xCA) indicating valid capability
  • TAINT: Taint level (0 = clean, higher = more untrusted)
  • PERMS: Permission bits (R=1, W=2, X=4, CAP=8, ...)
  • LENGTH: Size of accessible region in bytes
  • BASE: Start of accessible region
  • CURRENT: Current address (must be in [BASE, BASE+LENGTH))
```

### Permission Bits

```rust
pub struct CapPerms(pub u8);

impl CapPerms {
    pub const READ: u8   = 0b0000_0001;  // Can load
    pub const WRITE: u8  = 0b0000_0010;  // Can store
    pub const EXEC: u8   = 0b0000_0100;  // Can execute
    pub const CAP: u8    = 0b0000_1000;  // Can store/load capabilities
    pub const SEAL: u8   = 0b0001_0000;  // Can seal
    pub const UNSEAL: u8 = 0b0010_0000;  // Can unseal
}
```

### Operations

```
┌─────────────────────────────────────────────────────────────────────┐
│                  Capability Operations                               │
└─────────────────────────────────────────────────────────────────────┘

  CAP.NEW (privileged)
  ┌─────────────────────────────────────────────────────────────────┐
  │ Create new capability with full permissions                     │
  │ Only allowed in privileged mode                                  │
  └─────────────────────────────────────────────────────────────────┘

  CAP.RESTRICT (monotonic)
  ┌─────────────────────────────────────────────────────────────────┐
  │ Can ONLY:                                                        │
  │   • Shrink bounds (narrow address range)                        │
  │   • Remove permissions (less access)                            │
  │ Can NEVER:                                                       │
  │   • Expand bounds                                                │
  │   • Add permissions                                              │
  └─────────────────────────────────────────────────────────────────┘

  CAP.QUERY
  ┌─────────────────────────────────────────────────────────────────┐
  │ Read capability properties:                                      │
  │   • Base address                                                 │
  │   • Length                                                       │
  │   • Permissions                                                  │
  │   • Taint level                                                  │
  │   • Validity                                                     │
  └─────────────────────────────────────────────────────────────────┘
```

## Bounds Checking

```
┌─────────────────────────────────────────────────────────────────────┐
│               Implicit Bounds Check Flow                             │
└─────────────────────────────────────────────────────────────────────┘

  Every LOAD/STORE automatically performs:

  ┌─────────────────────────────────────────────────────────────────┐
  │                                                                 │
  │  1. Check TAG == 0xCA (valid capability)                        │
  │     └─▶ If not: TRAP(InvalidTag)                               │
  │                                                                 │
  │  2. Check ADDRESS >= BASE                                       │
  │     └─▶ If not: TRAP(OutOfBounds)                              │
  │                                                                 │
  │  3. Check ADDRESS + SIZE <= BASE + LENGTH                       │
  │     └─▶ If not: TRAP(OutOfBounds)                              │
  │                                                                 │
  │  4. Check required permissions                                  │
  │     └─▶ LOAD requires READ                                     │
  │     └─▶ STORE requires WRITE                                   │
  │     └─▶ If missing: TRAP(PermissionDenied)                     │
  │                                                                 │
  │  5. Proceed with memory access                                  │
  │                                                                 │
  └─────────────────────────────────────────────────────────────────┘
```

## Taint Tracking

### Taint Levels

```rust
pub enum TaintLevel {
    Clean = 0,       // Untainted - safe to use
    UserInput = 1,   // From user - needs validation
    NetworkData = 2, // From network - needs sanitization
    FileData = 3,    // From file - needs validation
    Toxic = 255,     // Maximum taint
}
```

### Taint Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Taint Propagation                                │
└─────────────────────────────────────────────────────────────────────┘

  Unary operations:
  ┌─────────────────────────────────────────────────────────────────┐
  │   taint(result) = taint(operand)                                │
  └─────────────────────────────────────────────────────────────────┘

  Binary operations:
  ┌─────────────────────────────────────────────────────────────────┐
  │   taint(result) = max(taint(operand1), taint(operand2))        │
  └─────────────────────────────────────────────────────────────────┘

  Example:
  ┌─────────────────────────────────────────────────────────────────┐
  │   r0 = load(user_input)  ; r0 is tainted (UserInput)           │
  │   r1 = 10                ; r1 is clean                          │
  │   r2 = r0 + r1           ; r2 is tainted (UserInput)           │
  │   sanitize r0            ; r0 is now clean                      │
  └─────────────────────────────────────────────────────────────────┘
```

### Usage Pattern

```asm
; Correct: validate then sanitize
load.d r0, [user_input]     ; Load untrusted data
taint r0                     ; Mark as tainted

; ... validation logic ...
blt r0, zero, error          ; Validate: must be >= 0
bgt r0, r1, error            ; Validate: must be <= max

sanitize r0                  ; Mark as safe after validation
store.d r0, [trusted_buffer] ; Now safe to use

; WRONG: using tainted data directly
; load.d r0, [user_input]
; taint r0
; call dangerous_function    ; ERROR: r0 is tainted!
```

## Security Context

```rust
pub struct SecurityContext {
    pub taint: TaintTracker,       // Per-register taint state
    pub trap_on_violation: bool,   // Whether to trap or continue
    pub violation_count: u64,      // Security violation counter
}

// Runtime check
fn check_capability(cap: &FatPointer, size: usize, perms: u8) -> CapCheckResult {
    if !cap.is_valid() {
        return CapCheckResult::InvalidTag;
    }
    if !cap.check_bounds(size) {
        return CapCheckResult::OutOfBounds;
    }
    if (cap.perms.0 & perms) != perms {
        return CapCheckResult::PermissionDenied;
    }
    CapCheckResult::Ok
}
```

## FFI Functions

```rust
// Called from generated code for bounds checking
#[no_mangle]
pub extern "C" fn neurlang_bounds_check(
    cap_meta: u64,
    cap_addr: u64,
    access_size: u64,
    required_perms: u64,
) -> u64;  // 0 = OK, non-zero = violation type

// Capability operations
#[no_mangle]
pub extern "C" fn neurlang_cap_new(base: u64, length: u32, perms: u8) -> (u64, u64);

#[no_mangle]
pub extern "C" fn neurlang_cap_restrict(...) -> (u64, u64, u64);

#[no_mangle]
pub extern "C" fn neurlang_cap_query(cap_meta: u64, cap_addr: u64, query_type: u8) -> u64;
```
