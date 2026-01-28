# Neurlang Testing Guide

This document describes the testing framework for Neurlang programs, including test annotations, extension mocking, and memory setup.

## Overview

Neurlang uses annotation-based testing embedded in `.nl` source files. Tests are defined using special comment annotations that specify inputs, expected outputs, and any required mocks or memory setup.

## Test Annotations

### Basic Test Format

```asm
; @test: inputs -> outputs
```

**Examples:**
```asm
; @test: r0=5 -> r0=120           ; Single register input/output
; @test: r0=10, r1=3 -> r0=1      ; Multiple inputs
; @test: r0=0 -> r0=0, r1=1       ; Multiple outputs
```

### Register Values

Register values can be specified in decimal or hexadecimal:
```asm
; @test: r0=255 -> r0=255         ; Decimal
; @test: r0=0xFF -> r0=0xFF       ; Hexadecimal
; @test: r0=0x10000 -> r0=65536   ; DATA_BASE address
```

### Memory Setup

Tests can pre-populate memory at specific addresses:
```asm
; @test: r0=0x10000 [0x10000]="hello" -> r0=104
```

Format: `[address]="string"` or `[address]=bytes`

**Important:** Memory addresses in the data section start at `DATA_BASE` (0x10000 = 65536). To write to a data label's location, calculate: `DATA_BASE + label_offset`.

### Float Values

For floating-point tests, use the raw 64-bit representation:
```asm
; @test: r0=4614253070214989087 -> r0=4614253070214989087  ; 3.14159...
```

Use an IEEE 754 converter to get the bit representation.

## Extension Mocking

### The `@mock` Annotation

Mock external extension calls to test code that uses `ext.call`:

```asm
; @mock: extension_name=return_value       ; Use extension name (recommended)
; @mock: extension_id=return_value         ; Or use numeric ID
; @mock: extension_name=return_value,out1,out2  ; With output values
```

**Examples with extension names (recommended):**
```asm
; @mock: tls_connect=1       ; returns handle 1
; @mock: http_get=1          ; returns handle 1
; @mock: http_response_status=200  ; returns 200 OK
; @mock: json_parse=1        ; returns handle 1
```

**Examples with numeric IDs:**
```asm
; @mock: 500=1       ; tls_connect returns handle 1
; @mock: 190=1       ; http_get returns handle 1
```

### Extension Name Resolution

Extension names in `@mock` annotations are resolved via the RAG system to their
corresponding numeric IDs. This ensures:

- **Single source of truth**: IDs come from `ext_ids` module
- **Future-proof**: New extensions work automatically
- **Self-documenting**: `http_get=1` is clearer than `190=1`

### Extension ID Reference

All IDs are defined in `src/runtime/extensions/mod.rs` (ext_ids module):

| Range | Category | Common Extensions |
|-------|----------|-------------------|
| 1-99 | Crypto | sha256, hmac_sha256, sha1, aes256_gcm_* |
| 100-119 | Vec | vec_new, vec_push, vec_pop, vec_get |
| 120-139 | HashMap | hashmap_new, hashmap_get, hashmap_insert |
| 140-169 | String | string_new, string_len, string_concat |
| 170-189 | JSON | json_parse, json_get, json_stringify |
| 190-209 | HTTP | http_get, http_post, http_response_status |
| 400-419 | Compression | zlib_compress, gzip_compress |
| 420-439 | Encoding | base64_encode, hex_encode, url_encode |
| 440-459 | DateTime | datetime_now, datetime_parse |
| 460-479 | Regex | regex_match, regex_find, regex_replace |
| 480-499 | FileSystem | fs_read, fs_write, fs_exists |
| 500-519 | TLS | tls_connect, tls_send, tls_recv, tls_close |
| 520-539 | X509 | x509_parse, x509_verify |

### Stateful Mocks (Return Value Sequences)

For testing code with loops that call the same extension multiple times (like server accept loops), use semicolon-separated return values:

```asm
; @mock: tcp_accept=5;0    ; Returns 5 first call, then 0 forever
```

**How it works:**
- First call returns `5` (client FD)
- Second and subsequent calls return `0` (no more clients, exit loop)

**Example: Testing a server that handles one connection:**
```asm
; @mock: tcp_bind=3
; @mock: tcp_listen=0
; @mock: tcp_accept=5;0      ; Accept one client, then exit
; @mock: tcp_recv=100;0      ; Receive data, then connection closed
; @mock: tcp_send=100
; @mock: tcp_close=0
;
; @test: r0=8080 -> r0=0
```

**Longer sequences:**
```asm
; @mock: tcp_accept=5;6;7;0  ; Accept 3 clients, then exit
```

When the sequence is exhausted, the last value repeats forever.

### Mock + Memory Setup Combined

For tests that need both mocked extensions AND specific memory content:

```asm
; Mock the TLS extensions
; @mock: 500=1       ; tls_connect returns handle
; @mock: 501=64      ; tls_write returns bytes written
; @mock: 502=64      ; tls_read returns bytes read
; @mock: 503=0       ; tls_close returns success
;
; Set up response buffer with HTTP response (at response_buf address)
; @test: r0=1 [0x1004c]="HTTP/1.1 200 OK\r\n" -> r0=200
```

### Calculating Data Section Addresses

To mock memory at a data label's address:

1. Count bytes of each data item before your target label
2. Add `DATA_BASE` (0x10000 = 65536)

**Example data section:**
```asm
.section .data
host:       .asciz "example.com"   ; 12 bytes (11 + null)
port:       .dword 443             ; 8 bytes
buffer:     .space 1024, 0         ; buffer starts at offset 20
```

`buffer` address = 0x10000 + 12 + 8 = 0x10014 = 65556

**Data directive sizes:**
- `.asciz "str"` = length + 1 (null terminator)
- `.dword value` = 8 bytes
- `.word value` = 4 bytes
- `.byte value` = 1 byte
- `.space N, V` = N bytes

## File-Level Annotations

### `@name`
Human-readable test name:
```asm
; @name: Factorial Calculator
```

### `@server` (Deprecated)
Previously used to skip tests for server code. **Now replaced by `@mock`**.

```asm
; OLD (skips tests):
; @server: true

; NEW (tests with mocks):
; @mock: 500=1
; @test: r0=0 -> r0=200
```

### `@note`
Document test behavior or limitations:
```asm
; @note: Returns HTTP status code from response
; @note: TLS operations are mocked for testing
```

## Running Tests

### Basic Usage
```bash
# Test all examples
nl test -p examples/

# Test specific file
nl test -p examples/patterns/network/https_client.nl

# Verbose output (show each test)
nl test -p examples/ -v

# With coverage tracking
nl test -p examples/ --coverage

# Stop on first failure
nl test -p examples/ --fail-fast
```

### Test Output
```
Running tests for 109 examples...

  [PASS] Factorial (4 tests)
  [PASS] HTTPS Client (2 tests)
  [FAIL] Broken Example[0]: r0=5 -> r0=120
         r0 = 0 (expected 120)

==================================================
Test Results:
  Total:   189
  Passed:  188 (99%)
  Failed:  1
  Skipped: 0 examples
==================================================
```

## Best Practices

### 1. Always Include Tests
Every `.nl` file should have at least one `@test` annotation:
```asm
; @test: r0=0 -> r0=0     ; Basic smoke test
```

### 2. Test Edge Cases
```asm
; @test: r0=0 -> r0=1     ; Zero input
; @test: r0=1 -> r0=1     ; Minimum positive
; @test: r0=5 -> r0=120   ; Typical case
```

### 3. Mock All External Dependencies
If your code uses `ext.call`, mock every extension:
```asm
; @mock: 170=1       ; json_parse returns handle
; @mock: 171=0       ; json_get returns success
; @mock: 172=0       ; json_free returns success
```

### 4. Document Complex Mocks
```asm
; Mock HTTP client for testing:
; - 190 (http_get): Returns 200 OK
; - Response body is set up in memory at 0x10100
; @mock: 190=200
; @test: r0=0 [0x10100]='{"status":"ok"}' -> r0=1
```

### 5. Use Meaningful Test Names
```asm
; @name: JWT Token Validator
; @description: Validates JWT tokens and extracts claims
```

## Converting `@server: true` to Mocks

### Before (Untestable)
```asm
; @server: true
; @note: Requires network - cannot unit test
```

### After (Testable)
```asm
; @mock: 190=200     ; http_get returns 200
; @mock: 170=1       ; json_parse returns handle
; @test: r0=0 -> r0=200
; @note: HTTP operations mocked for testing
```

### Conversion Steps

1. **Identify all `ext.call` instructions** in the file
2. **Determine expected return values** for each extension
3. **Add `@mock` annotations** for each extension ID
4. **Set up memory** if the code reads from buffers
5. **Add `@test` annotations** with expected behavior
6. **Remove `@server: true`** annotation

## Troubleshooting

### "Extension X failed: Not found"
Add a mock for that extension ID:
```asm
; @mock: X=0
```

### Wrong output value
1. Check if data section addresses are correct
2. Verify mock return values match expected behavior
3. Use verbose mode to see actual vs expected

### Memory access issues
1. Ensure memory setup address is correct
2. Check DATA_BASE offset calculation
3. Verify string includes null terminator

## Network Mocking for Server Testing

Server examples (those with `@server: true`) contain infinite loops that accept connections and process requests. To test these without real network I/O, Neurlang provides **network mocking** via the `@net_mock:` annotation.

### How Network Mocking Works

Network mocking intercepts all network operations (`net.socket`, `net.bind`, `net.listen`, `net.accept`, `net.recv`, `net.send`, `net.close`) and returns pre-configured values instead of performing real I/O.

**Key mechanism:** When `net.accept` fails in mock mode (returns -1), the interpreter **halts gracefully** instead of looping forever. This allows testing a single client interaction.

```
┌─────────────────────────────────────────────────────────────┐
│  Server Code Flow with Network Mocking                       │
├─────────────────────────────────────────────────────────────┤
│  1. net.socket  →  Mock returns fd (e.g., 3)                │
│  2. net.bind    →  Mock returns 0 (success)                 │
│  3. net.listen  →  Mock returns 0 (success)                 │
│  4. net.accept  →  Mock returns client_fd (e.g., 5)         │
│  5. net.recv    →  Mock returns data + length               │
│  6. [process request]                                        │
│  7. net.send    →  Mock returns bytes sent                  │
│  8. net.close   →  Mock returns 0 (success)                 │
│  9. net.accept  →  Mock returns -1 → INTERPRETER HALTS      │
└─────────────────────────────────────────────────────────────┘
```

### The `@net_mock` Annotation

```asm
; @net_mock: operation=return_value
; @net_mock: operation=value1,value2    ; Sequence of return values
; @net_mock: recv="data"                ; String data for recv
; @net_mock: recv="data",0              ; Data first, then 0 on subsequent calls
```

### Supported Operations

| Operation | Description | Example |
|-----------|-------------|---------|
| `socket` | Create socket | `@net_mock: socket=3` |
| `bind` | Bind to address/port | `@net_mock: bind=0` |
| `listen` | Start listening | `@net_mock: listen=0` |
| `accept` | Accept connection | `@net_mock: accept=5,-1` |
| `connect` | Connect to server | `@net_mock: connect=0` |
| `recv` | Receive data | `@net_mock: recv="GET /\r\n"` |
| `send` | Send data | `@net_mock: send=100` |
| `close` | Close connection | `@net_mock: close=0` |

### Return Value Sequences

Use comma-separated values for operations called multiple times:

```asm
; @net_mock: accept=5,-1    ; First call: return 5, then -1 forever
```

**Behavior:** After the sequence is exhausted, the **last value repeats** forever.

This is critical for testing server loops:
- First `accept` returns client fd (5)
- Second `accept` returns -1 → triggers interpreter halt

### Recv with Data

For `recv`, you can specify the actual data to return:

```asm
; Simple string data
; @net_mock: recv="GET / HTTP/1.1\r\nHost: localhost\r\n\r\n"

; Data + subsequent return values (for echo servers)
; @net_mock: recv="hello",0    ; First recv gets "hello", second returns 0
```

**Escape sequences supported:** `\n`, `\r`, `\t`, `\0`, `\\`, `\"`, `\xHH`

### Complete Server Test Example

```asm
; @name: REST GET Endpoint
; @description: Simple HTTP GET endpoint returning JSON
; @server: true
;
; Network mocks for testing (one client, then halt)
; @net_mock: socket=3
; @net_mock: bind=0
; @net_mock: listen=0
; @net_mock: accept=5,-1
; @net_mock: recv="GET / HTTP/1.1\r\nHost: localhost\r\n\r\n"
; @net_mock: send=109
; @net_mock: close=0
;
; @test: -> r0=0

.entry main
; ... server code ...
```

### Testing Echo Servers

Echo servers have a nested loop that receives and sends until the connection closes:

```asm
; @net_mock: socket=3
; @net_mock: bind=0
; @net_mock: listen=0
; @net_mock: accept=5,-1
; @net_mock: recv="hello",0      ; First recv: data, second: connection closed
; @net_mock: send=5
; @net_mock: close=0
;
; @test: -> r0=0
```

The `recv="hello",0` pattern:
1. First `recv` call: returns "hello" (5 bytes)
2. Second `recv` call: returns 0 (connection closed, exits echo loop)
3. Code proceeds to close client
4. Next `accept` returns -1 → interpreter halts

### Running Server Tests

Server tests are skipped by default. Use `--include-servers` to run them:

```bash
# Skip server tests (default)
nl test -p examples

# Include server tests
nl test -p examples --include-servers

# Server tests with coverage
nl test -p examples --include-servers --coverage
```

---

## Code Coverage

Neurlang tracks instruction-level coverage during test execution.

### Enabling Coverage

```bash
nl test -p examples --coverage
```

### Coverage Report

```
==================================================
Test Results:
  Total:   248
  Passed:  248 (100%)
  Failed:  0
  Skipped: 10 examples
  Time:    81.028682ms

Coverage Report:
  Instructions: 1987/2840 (70.0%)
==================================================
```

### What Coverage Measures

- **Instruction Coverage:** Percentage of instructions executed during all tests
- Each unique instruction address is counted once
- Branch targets and fall-through paths are tracked separately

### Achieving Higher Coverage

Coverage may be less than 100% due to:

1. **Error handling paths:** Code that handles errors (like `blt r0, zero, error_exit`) may not be exercised in normal tests
2. **Dead code:** Helper functions not called from `main()`
3. **Skipped examples:** Files without `@test` annotations
4. **Untested branches:** Conditional paths not covered by test inputs

**Strategies to improve coverage:**

```asm
; 1. Test error paths
; @test: r0=-1 -> r0=0          ; Test error input

; 2. Test multiple branches
; @test: r0=1 -> r0=10          ; Path A
; @test: r0=2 -> r0=20          ; Path B

; 3. Add tests to all helper functions
; @test: r0=5 -> r0=120         ; Test factorial helper
```

### Coverage in CI/CD

```bash
# Fail if coverage drops below threshold
nl test -p examples --coverage | grep "Instructions:" | \
  awk -F'[(/]' '{if ($2/$3 < 0.80) exit 1}'
```

---

## Implementation Details

### How Extension Mocking Works

1. Test runner parses `@mock` annotations at file level
2. Before execution, `ExtensionRegistry` is put in mock mode
3. Each mock is registered with its return value
4. When `ext.call` executes, mocked extensions return configured values
5. Real extension code is never called

### How Network Mocking Works

1. Test runner parses `@net_mock` annotations at file level
2. `IORuntime.network_mocks` is configured with mock operations
3. When network opcodes execute, mocks are checked first
4. If mocked, the pre-configured value is returned
5. Real network I/O is never performed
6. When `net.accept` fails in mock mode, interpreter halts gracefully

### Execution Order

1. Extension mocks are configured
2. **Network mocks are configured**
3. Data section is loaded into memory
4. Test memory setup overwrites specific addresses
5. Input registers are set
6. Program executes
7. Output registers are verified
8. **Coverage data is collected** (if enabled)

This order ensures test memory can override data section values.

### Files Modified for Network Mocking

| File | Changes |
|------|---------|
| `src/stencil/io.rs` | Added `NetworkMocks`, `NetMock`, `NetMockOp` |
| `src/interp/dispatch.rs` | Halt on accept failure in mock mode |
| `src/main.rs` | Parse `@net_mock:` annotations, apply to interpreter |
