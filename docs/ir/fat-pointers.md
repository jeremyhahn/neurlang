# Fat Pointer Specification

Capability-based memory safety through 128-bit fat pointers.

## Overview

```
┌───────────────────────────────────────────────────────────────────┐
│                     Fat Pointer (128 bits)                        │
└───────────────────────────────────────────────────────────────────┘

  Every pointer in Neurlang carries its own bounds and permissions.
  This enables hardware-enforced memory safety without runtime overhead.

  ┌─────────────────────────────────────────────────────────────────┐
  │   Regular Pointer (64-bit)    vs    Fat Pointer (128-bit)       │
  │                                                                 │
  │   ┌────────────────────┐           ┌────────────────────────┐  │
  │   │      Address       │           │ Metadata   │  Address  │  │
  │   │      64 bits       │           │  64 bits   │  64 bits  │  │
  │   └────────────────────┘           └────────────────────────┘  │
  │                                                                 │
  │   Can access ANY memory            Can ONLY access bounds       │
  └─────────────────────────────────────────────────────────────────┘
```

## Bit Layout

### Metadata Word (64 bits)

```
┌───────────────────────────────────────────────────────────────────┐
│                        Metadata Word                              │
└───────────────────────────────────────────────────────────────────┘

  Bits 63-56: TAG (magic value)
  Bits 55-48: TAINT (taint level)
  Bits 47-40: PERMS (permission bits)
  Bits 39-8:  LENGTH (32 bits, size in bytes)
  Bits 7-0:   BASE_LOW (low 8 bits of base address)

  ┌──────────┬──────────┬──────────┬────────────────┬──────────────┐
  │   TAG    │  TAINT   │  PERMS   │     LENGTH     │  BASE (low)  │
  │  8 bits  │  8 bits  │  8 bits  │    32 bits     │    8 bits    │
  └──────────┴──────────┴──────────┴────────────────┴──────────────┘
   63     56  55     48  47     40  39             8  7            0
```

### Address Word (64 bits)

```
┌───────────────────────────────────────────────────────────────────┐
│                         Address Word                              │
└───────────────────────────────────────────────────────────────────┘

  Bits 63-8:  CURRENT (current address, 56 bits)
  Bits 7-0:   BASE_HIGH (high 8 bits of base address)

  Note: Full base address = BASE_HIGH:BASE_LOW (16 bits visible)
        For larger bases, see Address Reconstruction below.

  ┌─────────────────────────────────────────────────┬──────────────┐
  │              CURRENT ADDRESS                    │  BASE (high) │
  │                  56 bits                        │    8 bits    │
  └─────────────────────────────────────────────────┴──────────────┘
   63                                              8  7            0
```

## Field Specifications

### TAG Field

```rust
pub const TAG_VALID: u8 = 0xCA;  // Magic value for valid capability
pub const TAG_NULL: u8 = 0x00;   // Null/invalid capability
```

A valid capability MUST have TAG = 0xCA. Any other value traps.

### TAINT Field

```rust
pub enum TaintLevel {
    Clean = 0,       // Untainted - safe to use anywhere
    UserInput = 1,   // From user input - needs validation
    NetworkData = 2, // From network - needs sanitization
    FileData = 3,    // From file - needs validation
    // ...
    Toxic = 255,     // Maximum taint - forbidden
}
```

Taint propagation: `result_taint = max(operand1_taint, operand2_taint)`

### PERMS Field

```rust
pub struct CapPerms(pub u8);

impl CapPerms {
    pub const READ: u8   = 0b0000_0001;  // Can load from memory
    pub const WRITE: u8  = 0b0000_0010;  // Can store to memory
    pub const EXEC: u8   = 0b0000_0100;  // Can execute (jump to)
    pub const CAP: u8    = 0b0000_1000;  // Can store/load capabilities
    pub const SEAL: u8   = 0b0001_0000;  // Can seal capabilities
    pub const UNSEAL: u8 = 0b0010_0000;  // Can unseal capabilities
}

// Common combinations
pub const RW: u8 = READ | WRITE;           // Data pointer
pub const RX: u8 = READ | EXEC;            // Code pointer
pub const RWX: u8 = READ | WRITE | EXEC;   // JIT buffer (dangerous)
```

### LENGTH Field

32-bit unsigned length in bytes. Maximum addressable region: 4GB per capability.

### BASE Field

16 bits of base address are stored directly. The full base is reconstructed from the current address (see below).

## Address Reconstruction

Since only 16 bits of base are stored, the full base address is derived:

```rust
fn reconstruct_base(meta: u64, addr: u64) -> u64 {
    let base_low = (meta & 0xFF) as u64;
    let base_high = (addr & 0xFF) as u64;
    let base_partial = (base_high << 8) | base_low;

    // Current address should be within [base, base+length]
    // Use current address's high bits + stored base bits
    let current = addr >> 8;
    let base_full = (current & !0xFFFF) | base_partial;

    // Handle wrap-around
    if base_full > current {
        base_full - (1 << 16)
    } else {
        base_full
    }
}
```

## Capability Operations

### CAP.NEW (Privileged)

```
┌─────────────────────────────────────────────────────────────────────┐
│ Create new capability with specified bounds and permissions          │
│ Only allowed in privileged mode (kernel/runtime)                     │
└─────────────────────────────────────────────────────────────────────┘

Inputs:
  - base: u64     (start address)
  - length: u32   (size in bytes)
  - perms: u8     (permission bits)

Output:
  - (meta: u64, addr: u64)  (fat pointer)
```

### CAP.RESTRICT (Monotonic)

```
┌─────────────────────────────────────────────────────────────────────┐
│ Restrict an existing capability (can ONLY shrink, never expand)      │
└─────────────────────────────────────────────────────────────────────┘

Allowed operations:
  ✓ Shrink bounds (new_base >= old_base, new_end <= old_end)
  ✓ Remove permissions (new_perms ⊆ old_perms)
  ✓ Increase taint level

Forbidden operations:
  ✗ Expand bounds
  ✗ Add permissions
  ✗ Decrease taint level
```

### CAP.QUERY

```
┌─────────────────────────────────────────────────────────────────────┐
│ Read capability properties (non-modifying)                           │
└─────────────────────────────────────────────────────────────────────┘

Query types:
  0: Get base address
  1: Get length
  2: Get permissions
  3: Get taint level
  4: Check validity (returns 0 or 1)
```

## Bounds Checking

Every memory access automatically performs:

```rust
fn check_access(cap: FatPointer, offset: usize, size: usize, required_perms: u8) -> Result<()> {
    // 1. Check tag
    if cap.tag() != TAG_VALID {
        return Err(Trap::InvalidTag);
    }

    // 2. Check lower bound
    let addr = cap.current();
    let base = cap.base();
    if addr < base {
        return Err(Trap::OutOfBounds);
    }

    // 3. Check upper bound
    let end = addr + offset + size;
    let limit = base + cap.length() as u64;
    if end > limit {
        return Err(Trap::OutOfBounds);
    }

    // 4. Check permissions
    if (cap.perms() & required_perms) != required_perms {
        return Err(Trap::PermissionDenied);
    }

    Ok(())
}
```

## Usage Examples

### Creating a Stack Capability

```asm
; Allocate 64KB stack with RW permissions
cap.new r1, 0x10000, 0x10000, 0x03  ; base=64K, len=64K, perms=RW

; Use as stack pointer
mov r31, r1

; Push value (bounds-checked automatically)
subi r31, r31, 8
store.d r0, [r31]

; Pop value
load.d r2, [r31]
addi r31, r31, 8
```

### Passing Restricted Capability

```asm
; Full buffer capability (1KB, RW)
cap.new r1, 0x1000, 0x400, 0x03

; Create read-only view for callee
cap.restrict r2, r1, 0, 0x400, 0x01  ; same bounds, READ only

; Pass to function (cannot modify)
mov r0, r2
call untrusted_fn
```

### Checking Capability Properties

```asm
; Get length of buffer
cap.query r2, r1, 1  ; query type 1 = length

; Loop over buffer
mov r3, 0
loop:
    bge r3, r2, done
    load.b r4, [r1]
    ; ... process byte ...
    addi r1, r1, 1
    addi r3, r3, 1
    b loop
done:
```

## Security Properties

```
┌─────────────────────────────────────────────────────────────────────┐
│                   Security Guarantees                                │
└─────────────────────────────────────────────────────────────────────┘

  1. Spatial Safety
     ┌─────────────────────────────────────────────────────────────┐
     │ Cannot access memory outside capability bounds               │
     │ Buffer overflows impossible without corrupting tag           │
     └─────────────────────────────────────────────────────────────┘

  2. Capability Monotonicity
     ┌─────────────────────────────────────────────────────────────┐
     │ Capabilities can only be weakened, never strengthened        │
     │ Prevents privilege escalation                                │
     └─────────────────────────────────────────────────────────────┘

  3. Unforgeable Tags
     ┌─────────────────────────────────────────────────────────────┐
     │ Only privileged code can create new capabilities             │
     │ Random bit-flips unlikely to produce valid tag (1/256)       │
     └─────────────────────────────────────────────────────────────┘

  4. Taint Tracking
     ┌─────────────────────────────────────────────────────────────┐
     │ Untrusted data marked and tracked through operations         │
     │ Must be sanitized before use in sensitive contexts           │
     └─────────────────────────────────────────────────────────────┘
```

## Comparison with Other Systems

| System | Pointer Size | Bounds Storage | Performance |
|--------|--------------|----------------|-------------|
| C/C++ | 64-bit | None | Fast, unsafe |
| CHERI | 128-bit | In pointer | ~5% overhead |
| **Neurlang** | 128-bit | In pointer | Similar to CHERI |
| Rust (safe) | 64-bit | Per-allocation | Runtime checks |
| AddressSanitizer | 64-bit | Shadow memory | 2-3x overhead |
