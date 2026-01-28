# Slot Types Reference

This document describes the 20 slot types that combine to build any protocol implementation.

## Overview

Slot types are organized into 7 categories:

| Category | Types | Use Case |
|----------|-------|----------|
| String/Pattern | 5 | Parsing and formatting text |
| Numeric | 3 | Number conversion and validation |
| Control Flow | 3 | State machines and loops |
| I/O | 3 | Network and file operations |
| Extension | 2 | External function calls |
| Error | 1 | Error response handling |
| Data | 2 | Buffer manipulation |

---

## String/Pattern Operations

### PatternMatch

Match input buffer against a pattern and extract captures.

**Parameters:**
```rust
PatternMatch {
    pattern: String,           // Pattern with {capture} placeholders
    input_reg: String,         // Register containing input buffer
    captures: Vec<Capture>,    // Where to store extracted values
    match_label: String,       // Jump here on match
    no_match_label: String,    // Jump here on no match
}

struct Capture {
    name: String,
    output_reg: String,
    capture_type: CaptureType,  // Word, UntilChar(char), Quoted, Rest
}
```

**Example:**
```yaml
slot_type: PatternMatch
pattern: "HELO {domain}"
input_reg: r0
captures:
  - name: domain
    output_reg: r3
    capture_type: Word
match_label: helo_match
no_match_label: try_next
```

**Generated code (50-80 instructions):**
```asm
; Compare "HELO " prefix
load.b r1, [r0]
mov r2, 0x48          ; 'H'
bne r1, r2, try_next
load.b r1, [r0 + 1]
mov r2, 0x45          ; 'E'
bne r1, r2, try_next
; ... continue for "LO "
; Extract domain until whitespace/newline
addi r3, r0, 5        ; Point to domain start
b helo_match
```

---

### PatternSwitch

Match against multiple patterns (switch statement).

**Parameters:**
```rust
PatternSwitch {
    input_reg: String,
    cases: Vec<(String, String)>,  // (pattern, label) pairs
    default_label: String,
}
```

**Example:**
```yaml
slot_type: PatternSwitch
input_reg: r0
cases:
  - ["HELO", handle_helo]
  - ["EHLO", handle_ehlo]
  - ["MAIL", handle_mail]
  - ["RCPT", handle_rcpt]
  - ["DATA", handle_data]
  - ["QUIT", handle_quit]
default_label: handle_unknown
```

**Generated code (100-200 instructions):**
```asm
; Check first char for quick dispatch
load.b r1, [r0]
mov r2, 0x48          ; 'H' - HELO
beq r1, r2, check_helo
mov r2, 0x45          ; 'E' - EHLO
beq r1, r2, check_ehlo
mov r2, 0x4D          ; 'M' - MAIL
beq r1, r2, check_mail
; ... more cases
b handle_unknown

check_helo:
    ; Full pattern match for HELO
    ...
```

---

### ResponseBuilder

Build formatted response from template and variables.

**Parameters:**
```rust
ResponseBuilder {
    template: String,          // Template with {var} placeholders
    variables: HashMap<String, String>,  // var name -> register
    output_reg: String,        // Register for output buffer
    length_reg: String,        // Register for output length
}
```

**Example:**
```yaml
slot_type: ResponseBuilder
template: "250 Hello {domain}\r\n"
variables:
  domain: r3
output_reg: r6
length_reg: r7
```

**Generated code (50-100 instructions):**
```asm
; Write literal "250 Hello "
mov r0, r6            ; Output buffer
mov r1, 0x32          ; '2'
store.b r1, [r0]
mov r1, 0x35          ; '5'
store.b r1, [r0 + 1]
; ... more literals
; Copy variable {domain}
addi r0, r0, 10       ; After "250 Hello "
mov r1, r3            ; domain source
; Copy loop...
; Append "\r\n" and set length
```

---

### StringCompare

Compare two null-terminated strings.

**Parameters:**
```rust
StringCompare {
    str1_reg: String,
    str2_reg: String,
    result_reg: String,        // 0 if equal, nonzero otherwise
}
```

**Example:**
```yaml
slot_type: StringCompare
str1_reg: r0
str2_reg: r1
result_reg: r2
```

**Generated code (20-30 instructions):**
```asm
strcmp_loop:
    load.b r3, [r0]
    load.b r4, [r1]
    sub r2, r3, r4
    bnez r2, strcmp_done    ; Different
    beqz r3, strcmp_done    ; Both null
    addi r0, r0, 1
    addi r1, r1, 1
    b strcmp_loop
strcmp_done:
```

---

### StringCopy

Copy string to destination buffer with length limit.

**Parameters:**
```rust
StringCopy {
    src_reg: String,
    dst_reg: String,
    max_len: u32,
    copied_len_reg: String,
}
```

**Example:**
```yaml
slot_type: StringCopy
src_reg: r0
dst_reg: r1
max_len: 256
copied_len_reg: r2
```

**Generated code (25-35 instructions):**
```asm
mov r2, 0             ; Counter
mov r3, 256           ; Max length
strcpy_loop:
    bge r2, r3, strcpy_done
    load.b r4, [r0]
    store.b r4, [r1]
    beqz r4, strcpy_done
    addi r0, r0, 1
    addi r1, r1, 1
    addi r2, r2, 1
    b strcpy_loop
strcpy_done:
```

---

## Numeric Operations

### IntToString

Convert integer to decimal string.

**Parameters:**
```rust
IntToString {
    value_reg: String,
    output_reg: String,
    length_reg: String,
}
```

**Example:**
```yaml
slot_type: IntToString
value_reg: r0
output_reg: r1
length_reg: r2
```

**Generated code (40-60 instructions):**
```asm
; Handle zero case
beqz r0, itoa_zero
; Extract digits in reverse
mov r3, r1            ; Save start
itoa_loop:
    beqz r0, itoa_reverse
    rem r4, r0, 10    ; r4 = r0 % 10
    addi r4, r4, 0x30 ; Convert to ASCII
    store.b r4, [r1]
    addi r1, r1, 1
    div r0, r0, 10
    b itoa_loop
itoa_reverse:
    ; Reverse the string
    ...
```

---

### StringToInt

Parse decimal string to integer.

**Parameters:**
```rust
StringToInt {
    input_reg: String,
    result_reg: String,
    success_label: String,
    error_label: String,
}
```

**Example:**
```yaml
slot_type: StringToInt
input_reg: r0
result_reg: r1
success_label: parse_ok
error_label: parse_error
```

**Generated code (30-50 instructions):**
```asm
mov r1, 0             ; Result
atoi_loop:
    load.b r2, [r0]
    beqz r2, parse_ok
    ; Check if digit
    slti r3, r2, 0x30
    bnez r3, parse_error
    slti r3, r2, 0x3A
    beqz r3, parse_error
    ; r1 = r1 * 10 + (r2 - '0')
    muli r1, r1, 10
    subi r2, r2, 0x30
    add r1, r1, r2
    addi r0, r0, 1
    b atoi_loop
```

---

### RangeCheck

Validate number is within range.

**Parameters:**
```rust
RangeCheck {
    value_reg: String,
    min: i64,
    max: i64,
    ok_label: String,
    error_label: String,
}
```

**Example:**
```yaml
slot_type: RangeCheck
value_reg: r0
min: 1
max: 65535
ok_label: port_valid
error_label: port_invalid
```

**Generated code (10-20 instructions):**
```asm
mov r1, 1             ; min
blt r0, r1, port_invalid
mov r1, 65535         ; max
bgt r0, r1, port_invalid
b port_valid
```

---

## Control Flow

### StateCheck

Validate current state is one of valid states.

**Parameters:**
```rust
StateCheck {
    state_reg: String,
    valid_states: Vec<String>,  // State constant names
    ok_label: String,
    error_label: String,
}
```

**Example:**
```yaml
slot_type: StateCheck
state_reg: r20
valid_states: [STATE_MAIL_FROM, STATE_RCPT_TO]
ok_label: state_ok
error_label: state_error
```

**Generated code (15-30 instructions):**
```asm
mov r1, STATE_MAIL_FROM
beq r20, r1, state_ok
mov r1, STATE_RCPT_TO
beq r20, r1, state_ok
b state_error
state_ok:
```

---

### StateTransition

Update state register to new state.

**Parameters:**
```rust
StateTransition {
    state_reg: String,
    new_state: String,
}
```

**Example:**
```yaml
slot_type: StateTransition
state_reg: r20
new_state: STATE_GREETED
```

**Generated code (2-5 instructions):**
```asm
mov r20, STATE_GREETED
```

---

### LoopUntil

Loop until condition met.

**Parameters:**
```rust
LoopUntil {
    condition: LoopCondition,
    body_label: String,
    exit_label: String,
}

enum LoopCondition {
    ByteEquals { reg: String, value: u8 },
    ByteNotEquals { reg: String, value: u8 },
    RegisterZero { reg: String },
    RegisterNonZero { reg: String },
}
```

**Example:**
```yaml
slot_type: LoopUntil
condition:
  type: ByteEquals
  reg: r0
  value: 0x0A  # '\n'
body_label: read_loop_body
exit_label: read_loop_done
```

**Generated code (10-20 instructions):**
```asm
read_loop_check:
    load.b r1, [r0]
    mov r2, 0x0A
    beq r1, r2, read_loop_done
    b read_loop_body
```

---

## I/O Operations

### SendResponse

Send buffer contents over socket.

**Parameters:**
```rust
SendResponse {
    socket_reg: String,
    buffer_reg: String,
    length_reg: String,
}
```

**Example:**
```yaml
slot_type: SendResponse
socket_reg: r10
buffer_reg: r6
length_reg: r7
```

**Generated code (15-25 instructions):**
```asm
; Prepare syscall args
mov r0, r10           ; socket fd
mov r1, r6            ; buffer
mov r2, r7            ; length
mov r3, 0             ; flags
sys.write             ; or ext.call @"socket_send"
```

---

### ReadUntil

Read from socket until delimiter.

**Parameters:**
```rust
ReadUntil {
    socket_reg: String,
    buffer_reg: String,
    delimiter: String,         // e.g., "\r\n"
    max_len: u32,
    length_reg: String,
    eof_label: String,
}
```

**Example:**
```yaml
slot_type: ReadUntil
socket_reg: r10
buffer_reg: r4
delimiter: "\r\n"
max_len: 1024
length_reg: r5
eof_label: client_disconnected
```

**Generated code (50-80 instructions):**
```asm
mov r5, 0             ; Length counter
read_loop:
    mov r2, 1024
    sub r2, r2, r5
    beqz r2, read_done    ; Buffer full
    ; Read one byte
    mov r0, r10
    add r1, r4, r5
    mov r2, 1
    sys.read
    beqz r0, client_disconnected  ; EOF
    ; Check for \r\n
    addi r5, r5, 1
    slti r3, r5, 2
    bnez r3, read_loop
    ; Check last two bytes
    subi r3, r5, 2
    add r3, r4, r3
    load.b r6, [r3]
    mov r7, 0x0D          ; '\r'
    bne r6, r7, read_loop
    load.b r6, [r3 + 1]
    mov r7, 0x0A          ; '\n'
    bne r6, r7, read_loop
read_done:
```

---

### ReadNBytes

Read exactly N bytes.

**Parameters:**
```rust
ReadNBytes {
    socket_reg: String,
    buffer_reg: String,
    count_reg: String,
    eof_label: String,
}
```

**Example:**
```yaml
slot_type: ReadNBytes
socket_reg: r10
buffer_reg: r4
count_reg: r5
eof_label: client_disconnected
```

**Generated code (30-50 instructions):**
```asm
mov r6, 0             ; Bytes read so far
read_exact_loop:
    bge r6, r5, read_exact_done
    mov r0, r10
    add r1, r4, r6
    sub r2, r5, r6
    sys.read
    beqz r0, client_disconnected
    add r6, r6, r0
    b read_exact_loop
read_exact_done:
```

---

## Extension Integration

### ExtensionCall

Call extension with arguments.

**Parameters:**
```rust
ExtensionCall {
    extension: String,         // RAG intent or explicit name
    args: Vec<String>,         // Argument registers
    result_reg: String,
}
```

**Example:**
```yaml
slot_type: ExtensionCall
extension: "parse JSON string"
args: [r0, r1]
result_reg: r2
```

**Generated code (5-15 instructions):**
```asm
ext.call @"parse JSON string", r0, r1
mov r2, r0            ; Result in r0
```

---

### ValidationHook

Validate value using extension (db lookup, regex, etc.).

**Parameters:**
```rust
ValidationHook {
    validation_type: String,   // "db_lookup", "regex", etc.
    value_reg: String,
    ok_label: String,
    error_label: String,
}
```

**Example:**
```yaml
slot_type: ValidationHook
validation_type: db_lookup
value_reg: r3
ok_label: user_valid
error_label: user_not_found
```

**Generated code (15-30 instructions):**
```asm
; Call database extension
ext.call @"db lookup user", r3
beqz r0, user_not_found
b user_valid
```

---

## Error Handling

### ErrorResponse

Send error response and optionally close connection.

**Parameters:**
```rust
ErrorResponse {
    socket_reg: String,
    error_code: u32,
    error_message: String,
    close_after: bool,
}
```

**Example:**
```yaml
slot_type: ErrorResponse
socket_reg: r10
error_code: 550
error_message: "User not found"
close_after: false
```

**Generated code (30-50 instructions):**
```asm
; Build "550 User not found\r\n"
mov r0, resp_buffer
mov r1, 0x35          ; '5'
store.b r1, [r0]
mov r1, 0x35          ; '5'
store.b r1, [r0 + 1]
mov r1, 0x30          ; '0'
store.b r1, [r0 + 2]
; ... rest of message
; Send
mov r0, r10
mov r1, resp_buffer
mov r2, msg_len
sys.write
; Close if requested
; beqz close_flag, skip_close
; sys.close r10
```

---

## Data Structures

### BufferWrite

Write value to buffer at offset.

**Parameters:**
```rust
BufferWrite {
    buffer_reg: String,
    offset: BufferOffset,      // Fixed(u32) or Register(String)
    value_reg: String,
    width: MemWidth,           // Byte, Word, Dword, Qword
}
```

**Example:**
```yaml
slot_type: BufferWrite
buffer_reg: r4
offset:
  type: Fixed
  value: 16
value_reg: r5
width: Dword
```

**Generated code (3-8 instructions):**
```asm
store.d r5, [r4 + 16]
```

---

### BufferRead

Read value from buffer at offset.

**Parameters:**
```rust
BufferRead {
    buffer_reg: String,
    offset: BufferOffset,
    result_reg: String,
    width: MemWidth,
}
```

**Example:**
```yaml
slot_type: BufferRead
buffer_reg: r4
offset:
  type: Register
  reg: r6
value_reg: r5
width: Word
```

**Generated code (3-8 instructions):**
```asm
add r7, r4, r6
load.w r5, [r7]
```

---

## Slot Type Selection Guidelines

| Task | Recommended Slot Type |
|------|----------------------|
| Parse command with arguments | PatternMatch |
| Dispatch to handler by command | PatternSwitch |
| Format response with variables | ResponseBuilder |
| Check if strings match | StringCompare |
| Validate state machine | StateCheck |
| Update state | StateTransition |
| Send response to client | SendResponse |
| Read line from socket | ReadUntil |
| Call external function | ExtensionCall |
| Database validation | ValidationHook |
| Send error and close | ErrorResponse |

## See Also

- [Slot Architecture Overview](./README.md)
- [Protocol Specification Format](./protocol-specs.md)
- [Training Data Format](./training-format.md)
