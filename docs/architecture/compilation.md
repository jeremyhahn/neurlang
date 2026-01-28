# Compilation Pipeline

The Neurlang compiler uses copy-and-patch compilation for sub-5μs compile times.

## Pipeline Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Compilation Pipeline                              │
└─────────────────────────────────────────────────────────────────────┘

  Input                                                     Output
    │                                                          │
    ▼                                                          ▼
┌────────┐    ┌────────┐    ┌────────┐    ┌────────┐    ┌────────┐
│ Binary │───▶│Decode  │───▶│Stencil │───▶│ Patch  │───▶│Execut- │
│   IR   │    │Instruct│    │Lookup  │    │Operands│    │able    │
│        │    │        │    │        │    │        │    │Buffer  │
└────────┘    └────────┘    └────────┘    └────────┘    └────────┘
    4-8B        O(1)         O(1)          O(1)          Ready!
   /instr      decode       lookup        patch
```

## Traditional JIT vs Copy-and-Patch

```
Traditional JIT (~1ms):
┌──────────────────────────────────────────────────────────────────┐
│ Parse │ Build IR │ Optimize │ RegAlloc │ Encode │ Emit │ Link   │
│ 50μs  │  100μs   │  500μs   │  200μs   │ 100μs  │ 50μs │        │
└──────────────────────────────────────────────────────────────────┘

Copy-and-Patch (~5μs):
┌────────────────────────────────────┐
│ Decode │ Lookup │ memcpy │ Patch  │
│  1μs   │  1μs   │  2μs   │  1μs   │
└────────────────────────────────────┘
```

## Stencil Structure

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Stencil Format                               │
└─────────────────────────────────────────────────────────────────────┘

  Pre-compiled at build time:

  ┌────────────────────────────────────────────────────────────────┐
  │  Stencil for ALU_ADD                                           │
  ├────────────────────────────────────────────────────────────────┤
  │  Machine Code:                                                  │
  │  48 b8 [PLACEHOLDER_SRC1]    ; movabs rax, <src1_reg>          │
  │  48 8b 04 c7                 ; mov rax, [rdi + rax*8]          │
  │  48 b9 [PLACEHOLDER_SRC2]    ; movabs rcx, <src2_reg>          │
  │  48 8b 0c cf                 ; mov rcx, [rdi + rcx*8]          │
  │  48 01 c8                    ; add rax, rcx                     │
  │  48 b9 [PLACEHOLDER_DST]     ; movabs rcx, <dst_reg>           │
  │  48 89 04 cf                 ; mov [rdi + rcx*8], rax          │
  │  c3                          ; ret                              │
  ├────────────────────────────────────────────────────────────────┤
  │  Patch Info:                                                    │
  │  - Offset 2:  8 bytes, SRC1_REG                                │
  │  - Offset 16: 8 bytes, SRC2_REG                                │
  │  - Offset 33: 8 bytes, DST_REG                                 │
  └────────────────────────────────────────────────────────────────┘
```

## Compilation Steps

### Step 1: Buffer Acquisition

```rust
// Lock-free buffer acquisition from pool
let buffer = pool.acquire()?;  // ~200ns

// Buffer is pre-allocated with RWX permissions
// No mprotect() syscall needed!
```

### Step 2: Instruction Decoding

```rust
// Decode 4-byte instruction
let opcode = (word >> 26) & 0x3F;  // 6 bits
let rd = (word >> 21) & 0x1F;      // 5 bits
let rs1 = (word >> 16) & 0x1F;     // 5 bits
let mode = (word >> 8) & 0x07;     // 3 bits
```

### Step 3: Stencil Lookup

```rust
// O(1) table lookup
let idx = (opcode << 3) | mode;
let stencil = &STENCIL_TABLE[idx];
```

### Step 4: Copy and Patch

```rust
// Copy stencil code
output[..stencil.len].copy_from_slice(&stencil.code);

// Patch operands (3-5 patches per instruction)
for patch in &stencil.patches {
    match patch.kind {
        DstReg => write_u64(&mut output[patch.offset..], rd as u64),
        Src1Reg => write_u64(&mut output[patch.offset..], rs1 as u64),
        // ...
    }
}
```

## Buffer Pool Operation

```
┌─────────────────────────────────────────────────────────────────────┐
│                       Buffer Pool                                    │
└─────────────────────────────────────────────────────────────────────┘

  Initialization (once at startup):

  ┌──────────────────────────────────────────────────────────────┐
  │                   256KB RWX Memory Region                     │
  ├────────┬────────┬────────┬────────┬─────────────────────────┤
  │ Buf 0  │ Buf 1  │ Buf 2  │  ...   │        Buf 63           │
  │  4KB   │  4KB   │  4KB   │        │         4KB             │
  └────────┴────────┴────────┴────────┴─────────────────────────┘
       │
       ▼
  Free List (lock-free queue):
  [0, 1, 2, 3, ..., 63]

  Acquire (O(1)):
  ┌────────────────────────┐
  │ idx = free_list.pop() │ ──▶ Return Buffer[idx]
  └────────────────────────┘

  Release (O(1)):
  ┌────────────────────────┐
  │ free_list.push(idx)   │ ──▶ Buffer available again
  └────────────────────────┘
```

## Compilation Modes

### JIT Mode (default)
- Compiles to executable buffer
- Executes immediately
- Best for: Programs with >10 instructions

### AOT Mode
- Compiles to raw bytes or ELF
- Saves to file
- Best for: Deployment, embedding

### Interpreter Mode
- No compilation
- Direct interpretation
- Best for: Programs with <10 instructions

## Performance Breakdown

```
32-instruction program compile:
┌──────────────────────────────────────────────────────────────────┐
│ Phase              │ Time    │ % of Total                       │
├────────────────────┼─────────┼──────────────────────────────────┤
│ Buffer acquire     │  200ns  │   5%                             │
│ Instruction decode │  500ns  │  12%                             │
│ Stencil lookup     │  300ns  │   7%                             │
│ memcpy + patch     │ 2800ns  │  70%                             │
│ Epilogue           │  200ns  │   5%                             │
├────────────────────┼─────────┼──────────────────────────────────┤
│ Total              │ 4000ns  │ 100% (4μs)                       │
└──────────────────────────────────────────────────────────────────┘
```

## Error Handling

| Error | Cause | Recovery |
|-------|-------|----------|
| `MissingStencil` | Unknown opcode/mode | Fall back to interpreter |
| `BufferAllocationFailed` | Pool exhausted | Wait and retry |
| `ProgramTooLarge` | >64KB code | Split into functions |
