# Memory Architecture

Memory model, address spaces, and allocation strategies.

## Memory Layout

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Virtual Address Space                            │
└─────────────────────────────────────────────────────────────────────┘

  High addresses
  ┌─────────────────────────────────────────────────────────────────┐
  │                        Stack                                      │
  │                     (grows down)                                  │
  │                         ↓                                         │
  ├─────────────────────────────────────────────────────────────────┤
  │                                                                   │
  │                        Free                                       │
  │                                                                   │
  ├─────────────────────────────────────────────────────────────────┤
  │                         ↑                                         │
  │                     (grows up)                                    │
  │                        Heap                                       │
  ├─────────────────────────────────────────────────────────────────┤
  │                        Data                                       │
  │                  (globals, constants)                             │
  ├─────────────────────────────────────────────────────────────────┤
  │                        Code                                       │
  │                  (read-only, executable)                          │
  └─────────────────────────────────────────────────────────────────┘
  Low addresses
```

## Segment Capabilities

Each memory region is protected by a capability:

```
┌─────────────────────────────────────────────────────────────────────┐
│                   Segment Capabilities                               │
└─────────────────────────────────────────────────────────────────────┘

  ┌──────────┬──────────────────┬────────────────────────────────────┐
  │ Segment  │ Permissions      │ Capability                         │
  ├──────────┼──────────────────┼────────────────────────────────────┤
  │ Code     │ R-X              │ cap_code: read + execute           │
  │ Data     │ RW-              │ cap_data: read + write             │
  │ Heap     │ RW-              │ cap_heap: read + write             │
  │ Stack    │ RW-              │ cap_stack: read + write            │
  └──────────┴──────────────────┴────────────────────────────────────┘
```

## Stack Frame Layout

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Stack Frame                                      │
└─────────────────────────────────────────────────────────────────────┘

  Higher addresses
  ┌─────────────────────────────────────────────────────────────────┐
  │                     Arguments                                     │
  │               (spilled from registers)                            │
  ├─────────────────────────────────────────────────────────────────┤
  │                   Return Address                                  │
  ├─────────────────────────────────────────────────────────────────┤
  │                    Saved FP (r30)                                │
  ├─────────────────────────────────────────────────────────────────┤  ← FP (r30)
  │                   Local Variables                                 │
  ├─────────────────────────────────────────────────────────────────┤
  │                  Saved Registers                                  │
  ├─────────────────────────────────────────────────────────────────┤
  │                   Callee Arguments                                │
  └─────────────────────────────────────────────────────────────────┘  ← SP (r31)
  Lower addresses
```

## Memory Access Patterns

### Load Operations

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Load Widths                                      │
└─────────────────────────────────────────────────────────────────────┘

  LOAD.B (byte, 8-bit):
  ┌────┐
  │ B0 │ → zero-extended to 64 bits
  └────┘

  LOAD.H (half, 16-bit):
  ┌────┬────┐
  │ B0 │ B1 │ → zero-extended to 64 bits
  └────┴────┘

  LOAD.W (word, 32-bit):
  ┌────┬────┬────┬────┐
  │ B0 │ B1 │ B2 │ B3 │ → zero-extended to 64 bits
  └────┴────┴────┴────┘

  LOAD.D (double, 64-bit):
  ┌────┬────┬────┬────┬────┬────┬────┬────┐
  │ B0 │ B1 │ B2 │ B3 │ B4 │ B5 │ B6 │ B7 │
  └────┴────┴────┴────┴────┴────┴────┴────┘
```

### Store Operations

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Store Widths                                     │
└─────────────────────────────────────────────────────────────────────┘

  STORE.B: Write low 8 bits
  STORE.H: Write low 16 bits (little-endian)
  STORE.W: Write low 32 bits (little-endian)
  STORE.D: Write full 64 bits (little-endian)
```

### Addressing Modes

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Addressing Modes                                  │
└─────────────────────────────────────────────────────────────────────┘

  Register Indirect:
    load.d r0, [r1]       ; addr = r1

  Register + Offset:
    load.d r0, [r1 + 16]  ; addr = r1 + 16
    load.d r0, [r1 - 8]   ; addr = r1 - 8

  RISC-style syntax (alternative):
    ld r0, 16(r1)         ; same as load.d r0, [r1 + 16]
```

## Alignment

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Alignment Requirements                            │
└─────────────────────────────────────────────────────────────────────┘

  Neurlang supports unaligned access (with performance cost):

  ┌──────────────┬────────────────┬────────────────────────────────┐
  │ Access Type  │ Natural Align  │ Unaligned Penalty              │
  ├──────────────┼────────────────┼────────────────────────────────┤
  │ Byte         │ 1              │ None                           │
  │ Half         │ 2              │ ~2x slower                     │
  │ Word         │ 4              │ ~2x slower                     │
  │ Double       │ 8              │ ~2x slower                     │
  │ Capability   │ 16             │ ~3x slower                     │
  └──────────────┴────────────────┴────────────────────────────────┘

  Recommendation: Align data to natural boundaries for performance.
```

## Atomic Memory Operations

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Atomic Operations                                 │
└─────────────────────────────────────────────────────────────────────┘

  All atomic operations are 64-bit and naturally aligned:

  ATOMIC.CAS [addr], expected, new
    ┌─────────────────────────────────────────────────────────────┐
    │ if *addr == expected:                                        │
    │     *addr = new                                              │
    │     return 1 (success)                                       │
    │ else:                                                        │
    │     return 0 (failure)                                       │
    └─────────────────────────────────────────────────────────────┘

  ATOMIC.XCHG [addr], value
    ┌─────────────────────────────────────────────────────────────┐
    │ old = *addr                                                  │
    │ *addr = value                                                │
    │ return old                                                   │
    └─────────────────────────────────────────────────────────────┘

  ATOMIC.ADD [addr], value
    ┌─────────────────────────────────────────────────────────────┐
    │ old = *addr                                                  │
    │ *addr = old + value                                          │
    │ return old                                                   │
    └─────────────────────────────────────────────────────────────┘
```

## Memory Ordering

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Memory Ordering Model                             │
└─────────────────────────────────────────────────────────────────────┘

  Default: Relaxed ordering (like ARM/RISC-V)

  Explicit fences for ordering:

  FENCE.ACQUIRE
    ┌─────────────────────────────────────────────────────────────┐
    │ All loads after fence see effects of stores before          │
    │ Used after: lock acquisition, channel receive               │
    └─────────────────────────────────────────────────────────────┘

  FENCE.RELEASE
    ┌─────────────────────────────────────────────────────────────┐
    │ All stores before fence visible before stores after         │
    │ Used before: lock release, channel send                     │
    └─────────────────────────────────────────────────────────────┘

  FENCE.SEQ_CST
    ┌─────────────────────────────────────────────────────────────┐
    │ Full sequential consistency (most expensive)                 │
    │ Use sparingly                                                │
    └─────────────────────────────────────────────────────────────┘
```

## Heap Allocation

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Heap Management                                   │
└─────────────────────────────────────────────────────────────────────┘

  Neurlang provides capability-aware allocation:

  ALLOC rd, size
    ┌─────────────────────────────────────────────────────────────┐
    │ Allocates 'size' bytes from heap                             │
    │ Returns capability with exact bounds                         │
    │ rd.base = allocated address                                  │
    │ rd.length = size                                             │
    │ rd.perms = RW                                                │
    └─────────────────────────────────────────────────────────────┘

  FREE rd
    ┌─────────────────────────────────────────────────────────────┐
    │ Deallocates memory pointed by rd                             │
    │ Invalidates capability (revocation)                          │
    └─────────────────────────────────────────────────────────────┘
```

## Runtime Memory Pool

```
┌─────────────────────────────────────────────────────────────────────┐
│                   Executable Buffer Pool                             │
└─────────────────────────────────────────────────────────────────────┘

  Pre-allocated RWX memory for JIT compilation:

  Startup:
    mmap(256KB, PROT_READ | PROT_WRITE | PROT_EXEC)
    Divide into 64 × 4KB buffers
    Push all indices to free list

  Acquire (lock-free):
    Pop index from free list (atomic)
    Return ExecutableBuffer handle
    Time: ~200ns

  Release:
    Fill with INT3 (0xCC)
    Push index back to free list
    Time: ~100ns

  Benefits:
    • No mprotect() syscalls on hot path
    • Lock-free allocation/deallocation
    • Predictable memory layout
```

## Memory Safety Checks

```
┌─────────────────────────────────────────────────────────────────────┐
│                   Implicit Safety Checks                             │
└─────────────────────────────────────────────────────────────────────┘

  Every LOAD/STORE performs (automatically):

  1. Tag Validation
     ┌─────────────────────────────────────────────────────────────┐
     │ if ptr.tag != 0xCA: TRAP(InvalidTag)                        │
     └─────────────────────────────────────────────────────────────┘

  2. Bounds Check
     ┌─────────────────────────────────────────────────────────────┐
     │ if addr < ptr.base: TRAP(OutOfBounds)                       │
     │ if addr + size > ptr.base + ptr.length: TRAP(OutOfBounds)   │
     └─────────────────────────────────────────────────────────────┘

  3. Permission Check
     ┌─────────────────────────────────────────────────────────────┐
     │ LOAD requires: ptr.perms & READ                             │
     │ STORE requires: ptr.perms & WRITE                           │
     │ CAP LOAD/STORE requires: ptr.perms & CAP                    │
     └─────────────────────────────────────────────────────────────┘

  4. Taint Propagation
     ┌─────────────────────────────────────────────────────────────┐
     │ Loaded data inherits capability's taint level               │
     │ Stored data does not reduce memory's taint                  │
     └─────────────────────────────────────────────────────────────┘
```

## Example: Array Traversal

```asm
; Safe array sum with automatic bounds checking
; r0 = array capability, r1 = length (from cap.query)

    mov r2, 0            ; sum = 0
    mov r3, 0            ; i = 0

loop:
    bge r3, r1, done     ; while i < length

    ; This load is bounds-checked against r0's capability
    ; If r3*8 exceeds bounds, TRAP occurs
    mul r4, r3, 8        ; offset = i * 8
    add r5, r0, r4       ; ptr = array + offset
    load.d r6, [r5]      ; value = *ptr (bounds-checked!)

    add r2, r2, r6       ; sum += value
    addi r3, r3, 1       ; i++
    b loop

done:
    mov r0, r2           ; return sum
    halt
```
