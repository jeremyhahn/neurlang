# Opcode Reference

Complete reference for all 32 Neurlang opcodes.

## Overview

Neurlang uses a 32-opcode instruction set organized into categories:

| Range     | Category           | Opcodes |
|-----------|--------------------|---------|
| 0x00-0x02 | Arithmetic/Logic   | ALU, ALUI, MULDIV |
| 0x03-0x05 | Memory             | LOAD, STORE, ATOMIC |
| 0x06-0x09 | Control Flow       | BRANCH, CALL, RET, JUMP |
| 0x0A-0x0C | Capabilities       | CAP.NEW, CAP.RESTRICT, CAP.QUERY |
| 0x0D-0x11 | Concurrency        | SPAWN, JOIN, CHAN, FENCE, YIELD |
| 0x12-0x13 | Taint Tracking     | TAINT, SANITIZE |
| 0x14-0x18 | I/O (Sandboxed)    | FILE, NET, NET.SETOPT, IO, TIME |
| 0x19-0x1B | Math Extensions    | FPU, RAND, BITS |
| 0x1C-0x1F | System             | MOV, TRAP, NOP, HALT |
| 0x20      | Extensions         | EXT.CALL |

## Arithmetic Operations

### ALU (0x00) - Arithmetic/Logic Unit

```
Format: ALU.mode rd, rs1, rs2
Effect: rd = rs1 <op> rs2

Mode bits select operation:
┌──────┬────────┬─────────────────────────────────────────────────────┐
│ Mode │ Name   │ Operation                                           │
├──────┼────────┼─────────────────────────────────────────────────────┤
│  0   │ ADD    │ rd = rs1 + rs2 (wrapping)                           │
│  1   │ SUB    │ rd = rs1 - rs2 (wrapping)                           │
│  2   │ AND    │ rd = rs1 & rs2                                      │
│  3   │ OR     │ rd = rs1 | rs2                                      │
│  4   │ XOR    │ rd = rs1 ^ rs2                                      │
│  5   │ SHL    │ rd = rs1 << (rs2 & 63)                              │
│  6   │ SHR    │ rd = rs1 >> (rs2 & 63) (logical)                    │
│  7   │ SAR    │ rd = rs1 >> (rs2 & 63) (arithmetic)                 │
└──────┴────────┴─────────────────────────────────────────────────────┘

Example:
  add r0, r1, r2    ; r0 = r1 + r2
  xor r3, r3, r3    ; r3 = 0 (self-xor clears)
```

### ALUI (0x01) - ALU with Immediate

```
Format: ALUI.mode rd, rs1, imm32
Effect: rd = rs1 <op> sign_extend(imm32)

Same modes as ALU, but rs2 replaced with 32-bit immediate.

Example:
  addi r0, r0, 1    ; r0 = r0 + 1
  andi r1, r1, 0xFF ; r1 = r1 & 0xFF (mask low byte)
```

### MULDIV (0x02) - Multiply/Divide

```
Format: MULDIV.mode rd, rs1, rs2
Effect: rd = rs1 <op> rs2

┌──────┬────────┬─────────────────────────────────────────────────────┐
│ Mode │ Name   │ Operation                                           │
├──────┼────────┼─────────────────────────────────────────────────────┤
│  0   │ MUL    │ rd = (rs1 * rs2) & 0xFFFFFFFFFFFFFFFF (low 64 bits)│
│  1   │ MULH   │ rd = (rs1 * rs2) >> 64 (high 64 bits, signed)       │
│  2   │ DIV    │ rd = rs1 / rs2 (unsigned, traps on div-by-zero)     │
│  3   │ MOD    │ rd = rs1 % rs2 (unsigned, traps on div-by-zero)     │
└──────┴────────┴─────────────────────────────────────────────────────┘

Example:
  mul r0, r1, r2    ; r0 = r1 * r2
  div r3, r4, r5    ; r3 = r4 / r5
```

## Memory Operations

### LOAD (0x03) - Load from Memory

```
Format: LOAD.width rd, [rs1 + offset]
Effect: rd = memory[rs1 + offset]

Width modes:
┌──────┬────────┬─────────────────────────────────────────────────────┐
│ Mode │ Name   │ Operation                                           │
├──────┼────────┼─────────────────────────────────────────────────────┤
│  0   │ BYTE   │ Load 8 bits, zero-extend to 64                      │
│  1   │ HALF   │ Load 16 bits, zero-extend to 64                     │
│  2   │ WORD   │ Load 32 bits, zero-extend to 64                     │
│  3   │ DOUBLE │ Load 64 bits                                        │
└──────┴────────┴─────────────────────────────────────────────────────┘

Security: Automatic bounds checking via fat pointer in rs1.
          Traps on: invalid tag, out of bounds, no read permission.

Example:
  load.d r0, [r1]       ; r0 = 64-bit value at address r1
  load.b r2, [r3 + 4]   ; r2 = byte at r3+4, zero-extended
```

### STORE (0x04) - Store to Memory

```
Format: STORE.width rs, [rs1 + offset]
Effect: memory[rs1 + offset] = rs (truncated to width)

Same width modes as LOAD.
Security: Same checks as LOAD, plus write permission required.

Example:
  store.d r0, [sp]      ; Store r0 at stack pointer
  store.w r1, [r2 + 8]  ; Store low 32 bits of r1 at r2+8
```

### ATOMIC (0x05) - Atomic Memory Operations

```
Format: ATOMIC.op rd, rs1, rs2
Effect: Atomically perform operation, rd = old value

┌──────┬────────┬─────────────────────────────────────────────────────┐
│ Mode │ Name   │ Operation                                           │
├──────┼────────┼─────────────────────────────────────────────────────┤
│  0   │ CAS    │ if *rs1 == rd then *rs1 = rs2; rd = old *rs1        │
│  1   │ XCHG   │ rd = *rs1; *rs1 = rs2                               │
│  2   │ ADD    │ rd = *rs1; *rs1 += rs2                              │
│  3   │ AND    │ rd = *rs1; *rs1 &= rs2                              │
│  4   │ OR     │ rd = *rs1; *rs1 |= rs2                              │
│  5   │ XOR    │ rd = *rs1; *rs1 ^= rs2                              │
│  6   │ MIN    │ rd = *rs1; *rs1 = min(*rs1, rs2)                    │
│  7   │ MAX    │ rd = *rs1; *rs1 = max(*rs1, rs2)                    │
└──────┴────────┴─────────────────────────────────────────────────────┘

Example:
  atomic.add r0, r1, r2   ; Atomically add r2 to memory at r1
  atomic.cas r3, r4, r5   ; CAS: if *r4==r3 then *r4=r5
```

## Control Flow Operations

### BRANCH (0x06) - Conditional Branch

```
Format: BRANCH.cond rs1, rs2, offset
Effect: if condition(rs1, rs2) then PC += offset

┌──────┬────────┬─────────────────────────────────────────────────────┐
│ Mode │ Name   │ Condition                                           │
├──────┼────────┼─────────────────────────────────────────────────────┤
│  0   │ ALWAYS │ Always branch (unconditional)                       │
│  1   │ EQ     │ rs1 == rs2                                          │
│  2   │ NE     │ rs1 != rs2                                          │
│  3   │ LT     │ (signed) rs1 < rs2                                  │
│  4   │ LE     │ (signed) rs1 <= rs2                                 │
│  5   │ GT     │ (signed) rs1 > rs2                                  │
│  6   │ GE     │ (signed) rs1 >= rs2                                 │
│  7   │ LTU    │ (unsigned) rs1 < rs2                                │
└──────┴────────┴─────────────────────────────────────────────────────┘

Example:
  beq r0, zero, done    ; if r0 == 0, jump to done
  blt r1, r2, less      ; if r1 < r2 (signed), jump to less
  b loop                ; unconditional jump to loop
```

### CALL (0x07) - Function Call

```
Format: CALL target
Effect: lr = PC + instruction_size; PC = target

Mode 0: Direct call (target is immediate offset)
Mode 1: Indirect call (target is register)

Example:
  call function         ; Save return addr in lr, jump to function
  call r0               ; Indirect call through r0
```

### RET (0x08) - Return from Function

```
Format: RET
Effect: PC = lr

Example:
  ret                   ; Return to address in link register
```

### JUMP (0x09) - Unconditional Jump

```
Format: JUMP target
Effect: PC = target (no link register save)

Example:
  jump label            ; Direct jump
  jump r0               ; Indirect jump through r0
```

## Capability Operations

### CAP.NEW (0x0A) - Create Capability

```
Format: CAP.NEW rd, rs_base, rs_length
Effect: rd = new_capability(base=rs_base, length=rs_length, perms=full)

Creates a new capability with full permissions.
This is a privileged operation.

Example:
  cap.new r0, r1, r2    ; Create capability: base=r1, length=r2
```

### CAP.RESTRICT (0x0B) - Restrict Capability

```
Format: CAP.RESTRICT rd, rs_cap, rs_mask
Effect: rd = restrict(rs_cap, mask=rs_mask)

Can only shrink bounds or remove permissions, never expand.
Traps if attempting to expand.

Example:
  cap.restrict r0, r1, r2  ; Restrict r1 by mask r2, store in r0
```

### CAP.QUERY (0x0C) - Query Capability

```
Format: CAP.QUERY rd, rs_cap
Effect: rd = query(rs_cap, field=mode)

┌──────┬─────────────────────────────────────────────────────────────┐
│ Mode │ Field                                                       │
├──────┼─────────────────────────────────────────────────────────────┤
│  0   │ Base address                                                │
│  1   │ Length                                                      │
│  2   │ Permissions                                                 │
│  3   │ Current address                                             │
│  4   │ Taint level                                                 │
│  5   │ Valid (1 if valid capability, 0 otherwise)                  │
└──────┴─────────────────────────────────────────────────────────────┘

Example:
  cap.query r0, r1      ; Get base address of capability in r1
```

## Concurrency Operations

### SPAWN (0x0D) - Create Task

```
Format: SPAWN rd, target
Effect: rd = spawn_task(entry=target)

Creates a new lightweight task starting at target.
Returns task ID in rd.

Example:
  spawn r0, worker      ; Spawn task, store ID in r0
```

### JOIN (0x0E) - Wait for Task

```
Format: JOIN rd, rs_task_id
Effect: rd = join_task(rs_task_id)

Blocks until task completes, returns result in rd.

Example:
  join r0, r1           ; Wait for task r1, get result in r0
```

### CHAN (0x0F) - Channel Operations

```
Format: CHAN.op rd, rs1
Effect: Depends on mode

┌──────┬────────┬─────────────────────────────────────────────────────┐
│ Mode │ Name   │ Operation                                           │
├──────┼────────┼─────────────────────────────────────────────────────┤
│  0   │ CREATE │ rd = create_channel()                               │
│  1   │ SEND   │ send(channel=rd, value=rs1)                         │
│  2   │ RECV   │ rd = recv(channel=rs1)                              │
│  3   │ CLOSE  │ close(channel=rd)                                   │
└──────┴────────┴─────────────────────────────────────────────────────┘

Example:
  chan.create r0        ; Create channel, store ID in r0
  chan.send r0, r1      ; Send r1 on channel r0
  chan.recv r2, r0      ; Receive from r0 into r2
```

### FENCE (0x10) - Memory Fence

```
Format: FENCE.mode
Effect: Memory barrier with specified ordering

┌──────┬────────┬─────────────────────────────────────────────────────┐
│ Mode │ Name   │ Ordering                                            │
├──────┼────────┼─────────────────────────────────────────────────────┤
│  0   │ ACQ    │ Acquire: no reads/writes move before               │
│  1   │ REL    │ Release: no reads/writes move after                │
│  2   │ ACQREL │ Acquire-Release: both                              │
│  3   │ SEQCST │ Sequentially consistent                            │
└──────┴────────┴─────────────────────────────────────────────────────┘

Example:
  fence.acquire         ; Acquire barrier
  fence.seqcst          ; Full barrier
```

### YIELD (0x11) - Cooperative Yield

```
Format: YIELD
Effect: Yield execution to scheduler

Example:
  yield                 ; Hint to scheduler
```

## Taint Operations

### TAINT (0x12) - Mark as Tainted

```
Format: TAINT rd
Effect: Mark rd as containing tainted (untrusted) data

Example:
  load.d r0, [r1]       ; Load user input
  taint r0              ; Mark as tainted
```

### SANITIZE (0x13) - Remove Taint

```
Format: SANITIZE rd
Effect: Mark rd as sanitized (trusted) after validation

Must validate data before sanitizing!

Example:
  ; After validating r0...
  sanitize r0           ; Mark as safe
```

## I/O Operations (Sandboxed)

All I/O operations are subject to permission checks via IOPermissions.

### Dynamic vs Static Lengths

I/O operations that transfer data (read, write, send, recv) support two length modes:

**Static Length (immediate):** Length is a constant encoded in the instruction.
```asm
file.read r0, r1, r2, 1024   ; Read up to 1024 bytes (fixed)
net.send r0, r1, r2, 64      ; Send exactly 64 bytes (fixed)
```

**Dynamic Length (register):** When imm=0, the `rd` register specifies the length.
```asm
mov r3, r6                   ; r6 has the actual byte count
file.write r3, r1, r2, 0     ; Write r3 bytes (dynamic, uses rd as length)
net.send r3, r1, r2, 0       ; Send r3 bytes (dynamic, uses rd as length)
```

This enables efficient handling of variable-length data without hardcoded buffer sizes.

### FILE (0x14) - File Operations

```
Format: FILE.op rd, rs1, rs2, imm
Effect: Depends on mode

┌──────┬────────┬─────────────────────────────────────────────────────┐
│ Mode │ Name   │ Operation                                           │
├──────┼────────┼─────────────────────────────────────────────────────┤
│  0   │ OPEN   │ rd = open(path=rs1, path_len=rs2, flags=imm) → fd   │
│  1   │ READ   │ rd = read(fd=rs1, buf=rs2, len=imm|rd) → bytes_read │
│  2   │ WRITE  │ rd = write(fd=rs1, buf=rs2, len=imm|rd) → bytes_out │
│  3   │ CLOSE  │ close(fd=rs1)                                       │
│  4   │ SEEK   │ rd = seek(fd=rs1, offset=rs2, whence=imm)           │
│  5   │ STAT   │ rd = stat(path=rs1) → file_size                     │
│  6   │ MKDIR  │ rd = mkdir(path=rs1) → 0 on success                 │
│  7   │ DELETE │ rd = delete(path=rs1) → 0 on success                │
└──────┴────────┴─────────────────────────────────────────────────────┘

Note: For READ/WRITE, if imm=0, rd is used as the length (dynamic mode).

Requires: file_read or file_write permission, path must be whitelisted.

Example (static length):
  file.open r0, r1, r2, 1     ; Open file (flags=1 for read)
  file.read r3, r0, r4, 1024  ; Read up to 1024 bytes from fd r0
  file.close r0, r0           ; Close fd r0

Example (dynamic length):
  mov r3, r6                  ; r6 = bytes to write
  file.write r3, r0, r4, 0    ; Write r3 bytes (dynamic)
```

### NET (0x15) - Network Operations

```
Format: NET.op rd, rs1, rs2, imm
Effect: Depends on mode

┌──────┬────────┬─────────────────────────────────────────────────────┐
│ Mode │ Name   │ Operation                                           │
├──────┼────────┼─────────────────────────────────────────────────────┤
│  0   │ SOCKET │ rd = socket(domain=rs1, type=rs2) → fd              │
│  1   │ CONNECT│ rd = connect(fd=rs1, addr=rs2, port=imm) → 0        │
│  2   │ BIND   │ rd = bind(fd=rs1, addr=rs2, port=imm) → 0           │
│  3   │ LISTEN │ rd = listen(fd=rs1, backlog=rs2) → 0                │
│  4   │ ACCEPT │ rd = accept(fd=rs1) → client_fd                     │
│  5   │ SEND   │ rd = send(fd=rs1, buf=rs2, len=imm|rd) → bytes_sent │
│  6   │ RECV   │ rd = recv(fd=rs1, buf=rs2, len=imm|rd) → bytes_recv │
│  7   │ CLOSE  │ close(fd=rs1)                                       │
└──────┴────────┴─────────────────────────────────────────────────────┘

Note: For SEND/RECV, if imm=0, rd is used as the length (dynamic mode).

Requires: net_connect or net_listen permission, host must be whitelisted.

Example (static length):
  net.socket r0, r1, r2       ; Create socket, fd in r0
  net.send r3, r0, r4, 64     ; Send 64 bytes from buffer r4

Example (dynamic length):
  ; r6 contains actual byte count to send
  mov r3, r6                  ; Copy length to r3
  net.send r3, r0, r4, 0      ; Send r3 bytes (dynamic)

  ; Receive with dynamic max length
  mov r3, 4096                ; Max buffer size
  net.recv r3, r0, r4, 0      ; Receive up to r3 bytes
  ; r3 now contains bytes actually received
```

### NET.SETOPT (0x16) - Socket Options

```
Format: NET.SETOPT.opt rs1, imm
Effect: Set socket option on fd in rs1

┌──────┬─────────────┬────────────────────────────────────────────────┐
│ Mode │ Option      │ Description                                    │
├──────┼─────────────┼────────────────────────────────────────────────┤
│  0   │ NONBLOCK    │ 0=blocking (default), 1=non-blocking           │
│  1   │ TIMEOUT_MS  │ recv/send timeout in milliseconds (0=infinite) │
│  2   │ KEEPALIVE   │ 0=off, 1=on                                    │
│  3   │ REUSEADDR   │ 0=off, 1=on                                    │
│  4   │ NODELAY     │ 0=off (Nagle on), 1=on (Nagle off)             │
│  5   │ RECVBUFSIZE │ Receive buffer size in bytes                   │
│  6   │ SENDBUFSIZE │ Send buffer size in bytes                      │
│  7   │ LINGER      │ Linger time on close in seconds                │
└──────┴─────────────┴────────────────────────────────────────────────┘

Example:
  net.setopt.nonblock r0, 1    ; Set socket r0 to non-blocking
  net.setopt.timeout r0, 5000  ; Set 5 second timeout
```

### IO (0x17) - Console I/O

```
Format: IO.op rd, rs1, rs2
Effect: Console input/output operations

┌──────┬──────────┬───────────────────────────────────────────────────┐
│ Mode │ Name     │ Operation                                         │
├──────┼──────────┼───────────────────────────────────────────────────┤
│  0   │ PRINT    │ print(buf=rs1, len=rs2) → bytes_written           │
│  1   │ READLINE │ rd = read_line(buf=rs1, max_len=rs2) → len        │
│  2   │ GETARGS  │ rd = get_args() → argc (argv_ptr in r1)           │
│  3   │ GETENV   │ rd = get_env(name=rs1) → value_ptr (null if none) │
└──────┴──────────┴───────────────────────────────────────────────────┘

Example:
  io.print r0, r1         ; Print rs2 bytes from buffer at r0
  io.readline r0, r1, r2  ; Read line into buffer r1, max r2 bytes
```

### TIME (0x18) - Time Operations

```
Format: TIME.op rd
Effect: Time-related operations

┌──────┬───────────┬──────────────────────────────────────────────────┐
│ Mode │ Name      │ Operation                                        │
├──────┼───────────┼──────────────────────────────────────────────────┤
│  0   │ NOW       │ rd = unix_timestamp_seconds()                    │
│  1   │ SLEEP     │ sleep(milliseconds=rs1)                          │
│  2   │ MONOTONIC │ rd = monotonic_nanoseconds()                     │
│  3   │ (reserved)│                                                  │
└──────┴───────────┴──────────────────────────────────────────────────┘

Example:
  time.now r0           ; Get current Unix timestamp in r0
  time.sleep r1         ; Sleep for r1 milliseconds
  time.monotonic r2     ; Get monotonic clock in nanoseconds
```

## Math Extension Operations

### FPU (0x19) - Floating-Point Operations

```
Format: FPU.op rd, rs1, rs2
Effect: Floating-point arithmetic (IEEE 754 double precision)

┌──────┬────────┬─────────────────────────────────────────────────────┐
│ Mode │ Name   │ Operation                                           │
├──────┼────────┼─────────────────────────────────────────────────────┤
│  0   │ FADD   │ rd = rs1 + rs2 (f64)                                │
│  1   │ FSUB   │ rd = rs1 - rs2 (f64)                                │
│  2   │ FMUL   │ rd = rs1 * rs2 (f64)                                │
│  3   │ FDIV   │ rd = rs1 / rs2 (f64)                                │
│  4   │ FSQRT  │ rd = sqrt(rs1)                                      │
│  5   │ FABS   │ rd = abs(rs1)                                       │
│  6   │ FFLOOR │ rd = floor(rs1)                                     │
│  7   │ FCEIL  │ rd = ceil(rs1)                                      │
└──────┴────────┴─────────────────────────────────────────────────────┘

Registers hold 64-bit IEEE 754 doubles (use to_bits/from_bits).

Example:
  fpu.fadd r0, r1, r2   ; r0 = r1 + r2 (floating point)
  fpu.fsqrt r3, r4      ; r3 = sqrt(r4)
```

### RAND (0x1A) - Random Number Generation

```
Format: RAND.op rd, rs1, rs2
Effect: Generate random numbers

┌──────┬───────────┬──────────────────────────────────────────────────┐
│ Mode │ Name      │ Operation                                        │
├──────┼───────────┼──────────────────────────────────────────────────┤
│  0   │ BYTES     │ Fill buffer at rs1 with rs2 random bytes         │
│  1   │ U64       │ rd = random_u64()                                │
└──────┴───────────┴──────────────────────────────────────────────────┘

Note: Uses hardware RNG (RDRAND) when available, otherwise PRNG.

Example:
  rand.u64 r0           ; Get random 64-bit value in r0
  rand.bytes r1, r2     ; Fill buffer at r1 with r2 random bytes
```

### BITS (0x1B) - Bit Manipulation

```
Format: BITS.op rd, rs1
Effect: Bit manipulation operations

┌──────┬──────────┬───────────────────────────────────────────────────┐
│ Mode │ Name     │ Operation                                         │
├──────┼──────────┼───────────────────────────────────────────────────┤
│  0   │ POPCOUNT │ rd = count_ones(rs1) (population count)           │
│  1   │ CLZ      │ rd = count_leading_zeros(rs1)                     │
│  2   │ CTZ      │ rd = count_trailing_zeros(rs1)                    │
│  3   │ BSWAP    │ rd = byte_swap(rs1) (endian conversion)           │
└──────┴──────────┴───────────────────────────────────────────────────┘

Example:
  bits.popcount r0, r1  ; Count set bits in r1, result in r0
  bits.clz r2, r3       ; Count leading zeros in r3
  bits.bswap r4, r5     ; Byte-swap r5 for endian conversion
```

## System Operations

### MOV (0x1C) - Move/Load Immediate

```
Format: MOV rd, rs    (mode 0)
        MOV rd, imm   (mode 1)
Effect: rd = source

Example:
  mov r0, r1            ; r0 = r1
  mov r2, 42            ; r2 = 42
  mov r3, -1            ; r3 = 0xFFFFFFFFFFFFFFFF
```

### TRAP (0x1D) - System Trap

```
Format: TRAP type
Effect: Trigger trap handler

┌──────┬──────────────────┬──────────────────────────────────────────┐
│ Mode │ Name             │ Purpose                                  │
├──────┼──────────────────┼──────────────────────────────────────────┤
│  0   │ SYSCALL          │ System call                              │
│  1   │ BREAKPOINT       │ Debugger breakpoint                      │
│  2   │ BOUNDS           │ Bounds check failed                      │
│  3   │ CAPABILITY       │ Capability violation                     │
│  4   │ TAINT            │ Taint violation                          │
│  5   │ DIVZERO          │ Division by zero                         │
│  6   │ INVALID          │ Invalid instruction                      │
│  7   │ USER             │ User-defined trap                        │
└──────┴──────────────────┴──────────────────────────────────────────┘

Example:
  trap 0                ; Syscall
```

### NOP (0x1E) - No Operation

```
Format: NOP
Effect: None (advance PC only)

Example:
  nop                   ; Do nothing
```

### HALT (0x1F) - Halt Execution

```
Format: HALT
Effect: Stop execution, return r0

Example:
  mov r0, 42            ; Set return value
  halt                  ; Stop execution
```

## Extension Operations

### EXT.CALL (0x20) - Call Rust Extension

```
Format: EXT.CALL rd, ext_id, rs1, rs2
Effect: rd = call_extension(id=ext_id, arg1=rs1, arg2=rs2)

Calls a registered Rust extension function. Extensions provide access to
complex operations (crypto, JSON parsing, etc.) implemented in safe Rust.

┌────────┬─────────────────────────┬──────────────────────────────────────┐
│ ID     │ Name                    │ Operation                            │
├────────┼─────────────────────────┼──────────────────────────────────────┤
│   1    │ sha256                  │ Compute SHA-256 hash                 │
│   2    │ hmac_sha256             │ Compute HMAC-SHA256                  │
│   3    │ aes256_gcm_encrypt      │ AES-256-GCM encryption               │
│   4    │ aes256_gcm_decrypt      │ AES-256-GCM decryption               │
│   5    │ constant_time_eq        │ Constant-time byte comparison        │
│   6    │ secure_random           │ Generate cryptographic random bytes  │
│   7    │ pbkdf2_sha256           │ PBKDF2 key derivation                │
│   8    │ ed25519_sign            │ Ed25519 digital signature            │
│   9    │ ed25519_verify          │ Ed25519 signature verification       │
│  10    │ x25519_derive           │ X25519 key exchange                  │
└────────┴─────────────────────────┴──────────────────────────────────────┘

Extensions receive SafeBuffer wrappers with capability restrictions,
preventing buffer overflows and enforcing read/write permissions.

Example:
  ; Hash data at r1 (len r2), store result at r3
  ext.call r0, sha256, r1, r2      ; r0 = 0 on success

  ; Using symbolic name (assembler resolves to ID)
  ext.call r0, ed25519_sign, r1, r2

  ; Using numeric ID directly
  ext.call r0, 1, r1, r2           ; ID 1 = sha256
```

### Registering Custom Extensions

Extensions are registered at runtime using the `ExtensionRegistry`:

```rust
use neurlang::runtime::{ExtensionRegistry, ExtCategory};

let mut registry = ExtensionRegistry::new();
registry.register(
    "my_extension",
    "Description for AI training context",
    2,  // arg_count
    false,  // taint_propagation
    ExtCategory::Utility,
    Arc::new(|args, outputs| {
        // Implementation
        Ok(0)
    }),
);
```
