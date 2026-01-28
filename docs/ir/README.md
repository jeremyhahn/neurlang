# IR Specification

The Neurlang Intermediate Representation is a 32-opcode binary format optimized for AI code generation.

## Documentation

| Document | Description |
|----------|-------------|
| [Opcodes](./opcodes.md) | Complete opcode reference (32 opcodes) |
| [Encoding](./encoding.md) | Binary instruction format |
| [Assembly](./assembly.md) | Text assembly syntax (including intrinsics) |
| [Fat Pointers](./fat-pointers.md) | Capability-based memory |

## Design Philosophy

```
┌─────────────────────────────────────────────────────────────────────┐
│                      Design Principles                               │
└─────────────────────────────────────────────────────────────────────┘

  1. MINIMAL VOCABULARY
     ┌─────────────────────────────────────────────────────────────┐
     │ 32 opcodes × 8 modes = 264 distinct operations              │
     │ (vs. 1000+ in x86-64)                                       │
     └─────────────────────────────────────────────────────────────┘

  2. FIXED-SIZE INSTRUCTIONS
     ┌─────────────────────────────────────────────────────────────┐
     │ 4 bytes (base) or 8 bytes (with immediate)                  │
     │ Predictable decoding, no variable-length complexity         │
     └─────────────────────────────────────────────────────────────┘

  3. IMPLICIT SECURITY
     ┌─────────────────────────────────────────────────────────────┐
     │ Every LOAD/STORE automatically bounds-checked               │
     │ No separate check instructions needed                       │
     └─────────────────────────────────────────────────────────────┘

  4. SANDBOXED I/O
     ┌─────────────────────────────────────────────────────────────┐
     │ File, network, console I/O with permission system           │
     │ Deny-by-default security model                              │
     └─────────────────────────────────────────────────────────────┘

  5. AI-FRIENDLY ENCODING
     ┌─────────────────────────────────────────────────────────────┐
     │ One token per opcode (33-token vocabulary)                  │
     │ Regular structure for pattern learning                      │
     └─────────────────────────────────────────────────────────────┘

  6. THREE-TIER CODE REUSE
     ┌─────────────────────────────────────────────────────────────┐
     │ Tier 0: Core opcodes (AI writes from scratch)               │
     │ Tier 1: Intrinsics (~30 zero-cost macros)                   │
     │ Tier 2: Extensions (Rust FFI for crypto)                    │
     └─────────────────────────────────────────────────────────────┘
```

## Opcode Categories

```
┌─────────────────────────────────────────────────────────────────────┐
│                      32 Opcodes by Category                          │
└─────────────────────────────────────────────────────────────────────┘

  ARITHMETIC (3)          MEMORY (3)            CONTROL (4)
  ┌─────────────┐        ┌─────────────┐       ┌─────────────┐
  │ 0x00 ALU    │        │ 0x03 LOAD   │       │ 0x06 BRANCH │
  │ 0x01 ALUI   │        │ 0x04 STORE  │       │ 0x07 CALL   │
  │ 0x02 MULDIV │        │ 0x05 ATOMIC │       │ 0x08 RET    │
  └─────────────┘        └─────────────┘       │ 0x09 JUMP   │
                                               └─────────────┘

  CAPABILITIES (3)        CONCURRENCY (5)       TAINT (2)
  ┌─────────────┐        ┌─────────────┐       ┌─────────────┐
  │ 0x0A CAP.NEW│        │ 0x0D SPAWN  │       │ 0x12 TAINT  │
  │ 0x0B CAP.RST│        │ 0x0E JOIN   │       │ 0x13 SANIT. │
  │ 0x0C CAP.QRY│        │ 0x0F CHAN   │       └─────────────┘
  └─────────────┘        │ 0x10 FENCE  │
                         │ 0x11 YIELD  │
                         └─────────────┘

  I/O (5)                 MATH (3)              SYSTEM (4)
  ┌─────────────┐        ┌─────────────┐       ┌─────────────┐
  │ 0x14 FILE   │        │ 0x19 FPU    │       │ 0x1C MOV    │
  │ 0x15 NET    │        │ 0x1A RAND   │       │ 0x1D TRAP   │
  │ 0x16 SETOPT │        │ 0x1B BITS   │       │ 0x1E NOP    │
  │ 0x17 IO     │        └─────────────┘       │ 0x1F HALT   │
  │ 0x18 TIME   │                              └─────────────┘
  └─────────────┘

  EXTENSIONS (1)
  ┌─────────────┐
  │ 0x20 EXT.CALL│  Call registered Rust extensions
  └─────────────┘
```

## Instruction Format

```
┌─────────────────────────────────────────────────────────────────────┐
│                   4-Byte Base Instruction                            │
└─────────────────────────────────────────────────────────────────────┘

  Bit:  31      26 25    21 20    16 15    11 10       8 7          0
       ┌─────────┬────────┬────────┬────────┬──────────┬─────────────┐
       │ OPCODE  │   RD   │  RS1   │  RS2   │   MODE   │  Reserved   │
       │ 6 bits  │ 5 bits │ 5 bits │ 5 bits │  3 bits  │   8 bits    │
       └─────────┴────────┴────────┴────────┴──────────┴─────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│                   8-Byte Extended Instruction                        │
└─────────────────────────────────────────────────────────────────────┘

  Bytes 0-3: Same as base instruction
  Bytes 4-7: 32-bit signed immediate value

       ┌─────────────────────────────────────────────────────────────┐
       │                    32-bit Immediate                          │
       │                    (sign-extended to 64-bit)                 │
       └─────────────────────────────────────────────────────────────┘
```

## Quick Reference

| Range | Category | Opcodes | Examples |
|-------|----------|---------|----------|
| 0x00-0x02 | Arithmetic | ALU, ALUI, MULDIV | `add r0, r1, r2`, `mul r0, r1, r2` |
| 0x03-0x05 | Memory | LOAD, STORE, ATOMIC | `load.d r0, [r1]`, `atomic.cas` |
| 0x06-0x09 | Control | BRANCH, CALL, RET, JUMP | `beq r0, r1, label`, `call func` |
| 0x0A-0x0C | Capabilities | CAP.NEW, CAP.RESTRICT, CAP.QUERY | `cap.new r0, r1, r2` |
| 0x0D-0x11 | Concurrency | SPAWN, JOIN, CHAN, FENCE, YIELD | `spawn r0, task`, `chan.send` |
| 0x12-0x13 | Taint | TAINT, SANITIZE | `taint r0`, `sanitize r0` |
| 0x14-0x18 | I/O | FILE, NET, NET.SETOPT, IO, TIME | `file.open`, `net.connect`, `io.print` |
| 0x19-0x1B | Math | FPU, RAND, BITS | `fpu.sqrt r0, r1`, `rand.u64 r0` |
| 0x1C-0x1F | System | MOV, TRAP, NOP, HALT | `mov r0, 42`, `halt` |
| 0x20 | Extensions | EXT.CALL | `ext.call r0, sha256, r1, r2` |

## Intrinsics (Tier 1)

Intrinsics are zero-cost macros that expand to optimized Neurlang IR at assembly time:

```asm
@memcpy r0, r1, 256    ; Copy 256 bytes
@strlen r1             ; String length, result in r0
@gcd r1, r2            ; Greatest common divisor
@min r1, r2            ; Minimum of two values
```

See [Assembly Guide](./assembly.md) for the complete list.

## Extensions (Tier 2)

Call Rust functions for complex operations:

```asm
ext.call r0, sha256, r1, r2           ; SHA-256 hash
ext.call r0, aes256_gcm_encrypt, ...  ; AES-256-GCM
ext.call r0, ed25519_sign, r1, r2     ; Ed25519 signature
```

Built-in crypto: `sha256`, `hmac_sha256`, `aes256_gcm_encrypt/decrypt`,
`ed25519_sign/verify`, `x25519_derive`, `pbkdf2_sha256`, `secure_random`, `constant_time_eq`

## I/O Permissions

All I/O operations are sandboxed with deny-by-default permissions:

```rust
pub struct IOPermissions {
    pub file_read: bool,           // FILE.read
    pub file_write: bool,          // FILE.write
    pub file_paths: Vec<PathBuf>,  // Allowed paths
    pub net_connect: bool,         // NET.connect
    pub net_listen: bool,          // NET.listen
    pub net_hosts: Vec<String>,    // Allowed hosts
    pub io_print: bool,            // IO.print (default: true)
    pub io_read: bool,             // IO.readline
}
```
