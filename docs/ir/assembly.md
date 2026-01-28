# Assembly Language Guide

Text assembly syntax for Neurlang programs.

## Basic Syntax

```
; This is a comment
# This is also a comment

label:                  ; Label definition
    instruction         ; Instruction (indentation optional)
    instruction arg1, arg2, arg3
```

## Dot-Notation Syntax

Neurlang supports dot-notation for opcode variants:

```asm
; Arithmetic with dot-notation
alu.add rd, rs1, rs2    ; Same as: add rd, rs1, rs2
alu.sub rd, rs1, rs2    ; Same as: sub rd, rs1, rs2
alui.add rd, rs1, imm   ; Same as: addi rd, rs1, imm

; Memory with dot-notation
load.d rd, [rs1]        ; Load double (64-bit)
store.w rs, [rd]        ; Store word (32-bit)

; Branches with dot-notation
branch.eq rs1, rs2, label   ; Same as: beq rs1, rs2, label
branch.lt rs1, rs2, label   ; Same as: blt rs1, rs2, label
branch.always label         ; Unconditional branch

; All I/O uses dot-notation
file.open rd, rs1, rs2
net.connect rd, rs1, rs2
io.print rs1, rs2
time.now rd
fpu.sqrt rd, rs1
rand.u64 rd
bits.popcount rd, rs1
```

## Directives

```asm
; Entry point
.entry main             ; Set entry point to label 'main'

; Data section
.section data           ; Switch to data section
mydata:
    .word 10, 20, 30    ; 64-bit words
    .byte 0x41, 0x42    ; Individual bytes
    .space 16           ; Reserve 16 bytes (zero-filled)
    .ascii "Hello"      ; ASCII string (no null terminator)

; Section switching
.section code           ; Switch to code section (default)
.section data           ; Switch to data section
```

## Registers

```
General Purpose:
  r0-r15              ; 16 general purpose registers
  a0-a5               ; Aliases for r0-r5 (arguments)
  ret                 ; Alias for r0 (return value)

Special Purpose:
  sp                  ; Stack pointer
  fp                  ; Frame pointer
  lr                  ; Link register (return address)
  pc                  ; Program counter (read-only)
  csp                 ; Capability stack pointer
  cfp                 ; Capability frame pointer
  zero                ; Always zero (read-only)
```

## Instruction Formats

### Arithmetic

```asm
; Three-register format
add rd, rs1, rs2        ; rd = rs1 + rs2
sub rd, rs1, rs2        ; rd = rs1 - rs2
and rd, rs1, rs2        ; rd = rs1 & rs2
or rd, rs1, rs2         ; rd = rs1 | rs2
xor rd, rs1, rs2        ; rd = rs1 ^ rs2
shl rd, rs1, rs2        ; rd = rs1 << rs2
shr rd, rs1, rs2        ; rd = rs1 >> rs2 (logical)
sar rd, rs1, rs2        ; rd = rs1 >> rs2 (arithmetic)

; Register-immediate format
addi rd, rs1, imm       ; rd = rs1 + imm
subi rd, rs1, imm       ; rd = rs1 - imm
andi rd, rs1, imm       ; rd = rs1 & imm
ori rd, rs1, imm        ; rd = rs1 | imm
xori rd, rs1, imm       ; rd = rs1 ^ imm
shli rd, rs1, imm       ; rd = rs1 << imm
shri rd, rs1, imm       ; rd = rs1 >> imm
sari rd, rs1, imm       ; rd = rs1 >> imm (arithmetic)

; Multiply/Divide
mul rd, rs1, rs2        ; rd = rs1 * rs2 (low 64 bits)
mulh rd, rs1, rs2       ; rd = rs1 * rs2 (high 64 bits)
div rd, rs1, rs2        ; rd = rs1 / rs2
mod rd, rs1, rs2        ; rd = rs1 % rs2
rem rd, rs1, rs2        ; Alias for mod
```

### Memory

```asm
; Load with width suffix
load.b rd, [rs1]        ; Load byte
load.h rd, [rs1]        ; Load half (16-bit)
load.w rd, [rs1]        ; Load word (32-bit)
load.d rd, [rs1]        ; Load double (64-bit)

; Load with offset
load.d rd, [rs1 + 8]    ; Load from rs1 + 8
load.w rd, [rs1 - 4]    ; Load from rs1 - 4

; Alternative syntax
ld rd, [rs1]            ; Same as load.d
lw rd, [rs1]            ; Same as load.w
lh rd, [rs1]            ; Same as load.h
lb rd, [rs1]            ; Same as load.b

; Store
store.b rs, [rd]        ; Store byte
store.h rs, [rd]        ; Store half
store.w rs, [rd]        ; Store word
store.d rs, [rd]        ; Store double

store.d rs, [rd + 16]   ; Store with offset

; Alternative syntax
sd rs, [rd]             ; Same as store.d
sw rs, [rd]             ; Same as store.w

; Offset in parentheses (RISC-V style)
ld rd, 8(rs1)           ; Load from rs1 + 8
sd rs, 16(rd)           ; Store to rd + 16
```

### Control Flow

```asm
; Unconditional branch
b label                 ; Jump to label
j label                 ; Alias for b
jmp label               ; Alias for b

; Conditional branches
beq rs1, rs2, label     ; Branch if rs1 == rs2
bne rs1, rs2, label     ; Branch if rs1 != rs2
blt rs1, rs2, label     ; Branch if rs1 < rs2 (signed)
ble rs1, rs2, label     ; Branch if rs1 <= rs2 (signed)
bgt rs1, rs2, label     ; Branch if rs1 > rs2 (signed)
bge rs1, rs2, label     ; Branch if rs1 >= rs2 (signed)
bltu rs1, rs2, label    ; Branch if rs1 < rs2 (unsigned)

; Function calls
call label              ; Call function at label
call rs                 ; Indirect call through register
ret                     ; Return (jump to lr)

; Unconditional jumps
jump label              ; Direct jump (no link save)
jump rs                 ; Indirect jump
```

### System

```asm
; Move operations
mov rd, rs              ; rd = rs
mov rd, imm             ; rd = immediate
li rd, imm              ; Load immediate (alias)

; Control
nop                     ; No operation
halt                    ; Stop execution
hlt                     ; Alias for halt

; Traps
trap imm                ; Trigger trap
syscall                 ; System call (trap 0)
break                   ; Breakpoint (trap 1)
bkpt                    ; Alias for break
```

### Atomics

```asm
; Atomic operations
atomic.cas rd, rs1, rs2   ; Compare-and-swap
atomic.xchg rd, rs1, rs2  ; Exchange
atomic.add rd, rs1, rs2   ; Atomic add
atomic.and rd, rs1, rs2   ; Atomic and
atomic.or rd, rs1, rs2    ; Atomic or
atomic.xor rd, rs1, rs2   ; Atomic xor
atomic.min rd, rs1, rs2   ; Atomic min
atomic.max rd, rs1, rs2   ; Atomic max

; Short forms
cas rd, rs1, rs2          ; Compare-and-swap
xchg rd, rs1, rs2         ; Exchange
```

### Concurrency

```asm
; Task management
spawn rd, label         ; Spawn task at label
join rd, rs             ; Wait for task, get result

; Channels
chan.create rd          ; Create channel
chan.send rd, rs        ; Send value on channel
chan.recv rd, rs        ; Receive from channel
chan.close rd           ; Close channel

send rd, rs             ; Alias for chan.send
recv rd, rs             ; Alias for chan.recv

; Synchronization
fence.acquire           ; Acquire fence
fence.release           ; Release fence
fence.acqrel            ; Acquire-release fence
fence.seqcst            ; Sequential consistency fence
fence                   ; Alias for fence.seqcst

yield                   ; Cooperative yield
```

### Security

```asm
; Capabilities
cap.new rd, rs1, rs2    ; Create capability
cap.restrict rd, rs1, rs2  ; Restrict capability
cap.query rd, rs, mode  ; Query capability (mode: 0=base, 1=len, 2=perms)

; Taint tracking
taint rd                ; Mark register as tainted
sanitize rd             ; Remove taint after validation
untaint rd              ; Alias for sanitize
```

### File I/O (Sandboxed)

```asm
; File operations (requires permissions)
file.open rd, rs1, rs2    ; Open file: rd = open(path=rs1, flags=rs2)
file.read rd, rs1, rs2    ; Read: rd = read(fd=rs1, buf=rs2, len)
file.write rd, rs1, rs2   ; Write: rd = write(fd=rs1, buf=rs2, len)
file.close rd             ; Close file descriptor
file.seek rd, rs1, rs2    ; Seek: rd = seek(fd=rs1, offset=rs2)
file.stat rd, rs1         ; Stat: rd = file_size(path=rs1)
file.mkdir rd, rs1        ; Create directory
file.delete rd, rs1       ; Delete file/directory
```

### Network I/O (Sandboxed)

```asm
; Socket operations (requires permissions)
net.socket rd, rs1, rs2   ; Create socket: rd = socket(domain, type)
net.connect rd, rs1, rs2  ; Connect: connect(fd=rs1, addr=rs2, port)
net.bind rd, rs1, rs2     ; Bind: bind(fd=rs1, addr=rs2, port)
net.listen rd, rs1, rs2   ; Listen: listen(fd=rs1, backlog=rs2)
net.accept rd, rs1        ; Accept: rd = accept(fd=rs1)
net.send rd, rs1, rs2     ; Send: rd = send(fd=rs1, buf=rs2, len)
net.recv rd, rs1, rs2     ; Recv: rd = recv(fd=rs1, buf=rs2, len)
net.close rd              ; Close socket

; Socket options
net.setopt.nonblock rs1, imm    ; Set non-blocking mode
net.setopt.timeout rs1, imm     ; Set timeout in ms
net.setopt.keepalive rs1, imm   ; Set keepalive
net.setopt.reuseaddr rs1, imm   ; Set reuseaddr
net.setopt.nodelay rs1, imm     ; Set TCP_NODELAY
```

### Console I/O

```asm
; Console operations
io.print rs1, rs2         ; Print buffer at rs1, length rs2
io.readline rd, rs1, rs2  ; Read line into rs1, max len rs2, returns len
io.getargs rd             ; Get argc in rd, argv pointer in r1
io.getenv rd, rs1         ; Get env var rs1, result ptr in rd
```

### Time Operations

```asm
; Time operations
time.now rd               ; Get Unix timestamp in rd
time.sleep rs1            ; Sleep for rs1 milliseconds
time.monotonic rd         ; Get monotonic time in nanoseconds
```

### Floating-Point (FPU)

```asm
; IEEE 754 double-precision operations
fpu.add rd, rs1, rs2      ; rd = rs1 + rs2 (f64)
fpu.sub rd, rs1, rs2      ; rd = rs1 - rs2 (f64)
fpu.mul rd, rs1, rs2      ; rd = rs1 * rs2 (f64)
fpu.div rd, rs1, rs2      ; rd = rs1 / rs2 (f64)
fpu.sqrt rd, rs1          ; rd = sqrt(rs1)
fpu.abs rd, rs1           ; rd = abs(rs1)
fpu.floor rd, rs1         ; rd = floor(rs1)
fpu.ceil rd, rs1          ; rd = ceil(rs1)

; Alternative f-prefix syntax
fpu.fadd rd, rs1, rs2     ; Same as fpu.add
fpu.fsqrt rd, rs1         ; Same as fpu.sqrt
```

### Random Numbers

```asm
; Random number generation
rand.u64 rd               ; rd = random 64-bit value
rand.bytes rs1, rs2       ; Fill buffer at rs1 with rs2 random bytes
```

### Bit Manipulation

```asm
; Bit operations
bits.popcount rd, rs1     ; rd = count of set bits in rs1
bits.clz rd, rs1          ; rd = count leading zeros in rs1
bits.ctz rd, rs1          ; rd = count trailing zeros in rs1
bits.bswap rd, rs1        ; rd = byte-swapped rs1 (endian conversion)
```

### Extension Calls

```asm
; Call registered Rust extensions (Tier 2)
ext.call rd, sha256, rs1, rs2       ; Call SHA-256 extension
ext.call rd, ed25519_sign, rs1, rs2 ; Call Ed25519 signing
ext.call rd, 42, rs1, rs2           ; Call extension by numeric ID

; Built-in crypto extensions:
; sha256, hmac_sha256, aes256_gcm_encrypt, aes256_gcm_decrypt,
; constant_time_eq, secure_random, pbkdf2_sha256,
; ed25519_sign, ed25519_verify, x25519_derive
```

## Intrinsics (Tier 1)

Intrinsics are macro-like constructs that expand to optimized Neurlang IR
at assembly time. They provide zero-cost abstractions for common algorithms.

```asm
; Intrinsic syntax: @name arg1, arg2, ...
; Result is typically placed in r0

; Memory operations
@memcpy dst, src, len     ; Copy len bytes from src to dst
@memset dst, val, len     ; Fill len bytes at dst with val
@memcmp ptr1, ptr2, len   ; Compare memory, result in r0

; String operations
@strlen str               ; String length, result in r0
@strcmp str1, str2        ; Compare strings, result in r0

; Math operations
@abs val                  ; Absolute value, result in r0
@min a, b                 ; Minimum of two values, result in r0
@max a, b                 ; Maximum of two values, result in r0
@clamp val, lo, hi        ; Clamp value between lo and hi
@gcd a, b                 ; Greatest common divisor, result in r0

; Array operations
@sum arr, len             ; Sum array elements, result in r0
@reverse arr, len         ; Reverse array in place
@find_min arr, len        ; Find minimum element index

; Bitwise operations
@popcount val             ; Population count, result in r0
@clz val                  ; Count leading zeros
@ctz val                  ; Count trailing zeros
```

### Why Use Intrinsics?

1. **Zero runtime overhead**: Expand at assembly time
2. **Guaranteed correctness**: Thoroughly tested implementations
3. **Minimal tokens**: AI outputs `@memcpy r0, r1, 256` instead of a loop
4. **Same security**: Capability checking and taint tracking still apply

## Immediates

```asm
; Decimal
mov r0, 42              ; Decimal: 42
mov r0, -1              ; Negative: -1

; Hexadecimal
mov r0, 0xFF            ; Hex: 255
mov r0, 0xDEADBEEF      ; Hex: 3735928559

; Binary
mov r0, 0b1010          ; Binary: 10
mov r0, 0b11110000      ; Binary: 240
```

## Labels

```asm
; Simple labels
start:
    mov r0, 0

loop:
    addi r0, r0, 1
    b loop

; Labels can be on same line as instruction
done: halt
```

## Example Programs

### Hello World (Return 42)

```asm
; Return the answer to life, universe, and everything
main:
    mov r0, 42          ; Set return value
    halt                ; Stop execution
```

### Sum 1 to N

```asm
; Calculate sum of 1 to N
; Input: r0 = N
; Output: r0 = sum

sum_to_n:
    mov r1, 0           ; sum = 0
    mov r2, 1           ; i = 1
loop:
    bgt r2, r0, done    ; if i > n, exit
    add r1, r1, r2      ; sum += i
    addi r2, r2, 1      ; i++
    b loop
done:
    mov r0, r1          ; return sum
    halt
```

### Fibonacci

```asm
; Calculate Fibonacci(n)
; Input: n in r0
; Output: fib(n) in r0

fibonacci:
    mov r1, 0           ; fib(n-2)
    mov r2, 1           ; fib(n-1)
    beq r0, zero, return_r1
    subi r0, r0, 1
    beq r0, zero, return_r2
loop:
    add r3, r1, r2      ; next = fib(n-2) + fib(n-1)
    mov r1, r2          ; fib(n-2) = fib(n-1)
    mov r2, r3          ; fib(n-1) = next
    subi r0, r0, 1
    bne r0, zero, loop
return_r2:
    mov r0, r2
    b done
return_r1:
    mov r0, r1
done:
    halt
```

### Array Sum with Memory

```asm
; Sum array elements
; r0 = base address of array
; r1 = count
; Returns sum in r0

array_sum:
    mov r2, 0           ; sum = 0
    mov r3, 0           ; i = 0
    mov r4, 8           ; element size
loop:
    bge r3, r1, done    ; if i >= count, exit
    mul r5, r3, r4      ; offset = i * 8
    add r5, r0, r5      ; address = base + offset
    load.d r6, [r5]     ; element = array[i]
    add r2, r2, r6      ; sum += element
    addi r3, r3, 1      ; i++
    b loop
done:
    mov r0, r2          ; return sum
    halt
```
