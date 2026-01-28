# Neurlang RAG Knowledge Base

This document serves as the primary retrieval source for code generation.
The model queries this for patterns, ISA reference, and best practices.

## 1. ISA Quick Reference

### Registers
- `r0-r31`: 64-bit general purpose registers
- `r0`: Function return value, primary accumulator
- `r10-r15`: Conventionally used for socket/connection state
- `zero`: Constant 0 register

### Core Opcodes (32 total)

| Opcode | Mnemonic | Description |
|--------|----------|-------------|
| 0x00 | NOP | No operation |
| 0x01 | MOV | Move register to register |
| 0x02 | LOAD | Load from memory (byte/word/dword) |
| 0x03 | STORE | Store to memory |
| 0x04 | ALU | Arithmetic: ADD, SUB, AND, OR, XOR, SHL, SHR |
| 0x05 | ALUI | ALU with immediate |
| 0x06 | MULDIV | MUL, DIV, MOD |
| 0x07 | BRANCH | Conditional/unconditional branch |
| 0x08 | CALL | Call subroutine |
| 0x09 | RET | Return from subroutine |
| 0x0A | HALT | Stop execution |
| 0x0B | BITS | POPCOUNT, CLZ, CTZ, BSWAP |
| 0x0C | FPU | Float: FADD, FSUB, FMUL, FDIV, FSQRT |
| 0x10 | FILE | File I/O: open, read, write, close |
| 0x11 | NET | Network: socket, bind, listen, accept, send, recv |
| 0x12 | IO | Console print/read |
| 0x13 | TIME | time.now, time.sleep, time.monotonic |
| 0x14 | RAND | rand.u64 |
| 0x18 | CAP | Capabilities: new, restrict, query |
| 0x19 | SPAWN | Spawn concurrent task |
| 0x1A | JOIN | Wait for task completion |
| 0x1B | CHAN | Channel: create, send, recv, close |
| 0x1C | EXT | Extension call via RAG resolver |

## 2. REST API Patterns

### HTTP Server Skeleton
```asm
.entry main
.section .data
    bind_addr:  .asciz "127.0.0.1"

.section .text
main:
    ; Create TCP socket
    mov r1, 2                    ; AF_INET
    mov r2, 1                    ; SOCK_STREAM
    net.socket r10, r1, r2       ; r10 = server socket

    ; Bind to address:port
    mov r1, bind_addr
    net.bind r0, r10, r1, 8080   ; port 8080

    ; Listen
    mov r1, 128                  ; backlog
    net.listen r0, r10, r1

accept_loop:
    net.accept r11, r10          ; r11 = client socket

    ; Read request
    mov r1, request_buf
    net.recv r12, r11, r1, 4096  ; r12 = bytes received

    ; Route request (parse method + path)
    call route_request

    ; Close client
    net.close r0, r11
    b accept_loop
```

### Request Routing Pattern
```asm
route_request:
    ; r1 = request buffer
    ; Check method (first 3-4 chars)
    load.b r2, [r1]              ; First char
    mov r3, 0x47                 ; 'G'
    beq r2, r3, handle_get
    mov r3, 0x50                 ; 'P'
    beq r2, r3, check_post_put
    mov r3, 0x44                 ; 'D'
    beq r2, r3, handle_delete
    b send_404

check_post_put:
    load.b r2, [r1 + 1]
    mov r3, 0x4F                 ; 'O' (POST)
    beq r2, r3, handle_post
    mov r3, 0x55                 ; 'U' (PUT)
    beq r2, r3, handle_put
    b send_404
```

### JSON Response Pattern
```asm
send_json_response:
    ; r0 = status code, r1 = body ptr, r2 = body len
    ; Build response in response_buf

    ; HTTP/1.1 {status}\r\n
    mov r3, response_buf
    call write_status_line

    ; Content-Type: application/json\r\n
    mov r4, content_type_json
    call append_header

    ; Content-Length: {len}\r\n
    call append_content_length

    ; \r\n (end headers)
    call append_crlf

    ; Body
    call append_body

    ; Send
    net.send r0, r11, response_buf, r5
    ret
```

### SQLite Persistence Pattern
```asm
; Initialize database
init_db:
    mov r0, db_path
    ext.call 260, r10, r0        ; sqlite_open -> r10 = db handle

    ; Create table
    mov r1, create_table_sql
    ext.call 262, r0, r10, r1    ; sqlite_execute
    ret

; Insert record
insert_record:
    ; r0 = key, r1 = value
    mov r2, insert_sql
    ext.call 264, r3, r10, r2    ; sqlite_prepare -> r3 = stmt
    ext.call 266, r4, r3, 1, r0  ; sqlite_bind_text(stmt, 1, key)
    ext.call 266, r4, r3, 2, r1  ; sqlite_bind_text(stmt, 2, value)
    ext.call 268, r0, r3         ; sqlite_step
    ext.call 270, r0, r3         ; sqlite_finalize
    ret

; Query record
query_record:
    ; r0 = key -> r1 = value
    mov r2, select_sql
    ext.call 264, r3, r10, r2    ; sqlite_prepare
    ext.call 266, r4, r3, 1, r0  ; sqlite_bind_text(stmt, 1, key)
    ext.call 268, r0, r3         ; sqlite_step
    ext.call 272, r1, r3, 0      ; sqlite_column_text -> r1
    ext.call 270, r0, r3         ; sqlite_finalize
    ret
```

### JSON Handling Pattern
```asm
; Parse JSON request body
parse_json_body:
    ; r0 = body ptr
    ext.call 200, r1, r0         ; json_parse -> r1 = json handle

    ; Get field
    mov r2, field_name
    ext.call 202, r3, r1, r2     ; json_get -> r3 = field value

    ; Free handle when done
    ext.call 209, r0, r1         ; json_free
    ret

; Build JSON response
build_json_object:
    ext.call 210, r1             ; json_new_object -> r1

    mov r2, key_str
    mov r3, value_str
    ext.call 203, r0, r1, r2, r3 ; json_set(obj, key, value)

    ext.call 201, r0, r1         ; json_stringify -> r0 = string
    ext.call 209, r4, r1         ; json_free
    ret
```

## 3. Multi-Service Architecture

### Service Isolation Pattern
Each service runs on its own port with independent database:

```asm
; User Service: port 8080, users.db
; Inventory Service: port 8081, inventory.db

.section .data
    user_db:      .asciz "/tmp/users.db"
    inventory_db: .asciz "/tmp/inventory.db"
```

### User Service Endpoints
| Method | Path | Description |
|--------|------|-------------|
| GET | /users | List all users |
| GET | /users/{id} | Get user by ID |
| POST | /users | Create user |
| PUT | /users/{id} | Update user |
| DELETE | /users/{id} | Delete user |
| POST | /auth/login | Authenticate user |
| POST | /auth/register | Register new user |

### Inventory Service Endpoints
| Method | Path | Description |
|--------|------|-------------|
| GET | /items | List all items |
| GET | /items/{id} | Get item by ID |
| POST | /items | Create item |
| PUT | /items/{id} | Update item |
| DELETE | /items/{id} | Delete item |
| PUT | /items/{id}/stock | Update stock level |
| GET | /items/low-stock | Get items below threshold |

### Cross-Service Communication
Services communicate via HTTP client extensions:

```asm
; Call user service from inventory service
validate_user:
    ; r0 = user_id
    mov r1, user_service_url     ; "http://127.0.0.1:8080/users/"
    ; Append user_id to URL
    call build_url
    ext.call 220, r2, r1         ; http_get -> r2 = response
    ext.call 226, r3, r2         ; http_response_status
    mov r4, 200
    bne r3, r4, user_not_found
    ext.call 227, r0, r2         ; http_response_body
    ext.call 231, r1, r2         ; http_free
    ret
```

## 4. Register Conventions

### Function Calls
- `r0-r3`: Arguments and return values
- `r4-r9`: Caller-saved temporaries
- `r10-r15`: Callee-saved, conventionally for persistent state
- `r16-r31`: General purpose

### Server State
- `r10`: Server socket FD
- `r11`: Client socket FD
- `r12`: Request length / bytes received
- `r13`: Response length
- `r14`: Database handle
- `r15`: Current record ID

## 5. Error Handling

### HTTP Error Responses
```asm
send_400:
    mov r0, 400
    mov r1, error_400_body       ; {"error":"Bad Request"}
    mov r2, 24
    b send_json_response

send_404:
    mov r0, 404
    mov r1, error_404_body       ; {"error":"Not Found"}
    mov r2, 22
    b send_json_response

send_500:
    mov r0, 500
    mov r1, error_500_body       ; {"error":"Internal Server Error"}
    mov r2, 33
    b send_json_response
```

### Database Error Handling
```asm
check_db_error:
    ; r0 = result code
    blt r0, zero, db_error
    ret

db_error:
    ; Log error
    mov r1, db_error_msg
    ext.call 363, r0, r1         ; log_error
    b send_500
```

## 6. Common String Operations

### String Length (null-terminated)
```asm
strlen:
    ; r0 = string ptr -> r0 = length
    mov r1, r0
    mov r2, 0
strlen_loop:
    load.b r3, [r1]
    beq r3, zero, strlen_done
    addi r1, r1, 1
    addi r2, r2, 1
    b strlen_loop
strlen_done:
    mov r0, r2
    ret
```

### String Compare
```asm
strcmp:
    ; r0 = str1, r1 = str2 -> r0 = 0 if equal
strcmp_loop:
    load.b r2, [r0]
    load.b r3, [r1]
    bne r2, r3, strcmp_diff
    beq r2, zero, strcmp_equal
    addi r0, r0, 1
    addi r1, r1, 1
    b strcmp_loop
strcmp_equal:
    mov r0, 0
    ret
strcmp_diff:
    alu.sub r0, r2, r3
    ret
```

### Copy String
```asm
strcpy:
    ; r0 = dest, r1 = src
strcpy_loop:
    load.b r2, [r1]
    store.b r2, [r0]
    beq r2, zero, strcpy_done
    addi r0, r0, 1
    addi r1, r1, 1
    b strcpy_loop
strcpy_done:
    ret
```

## 7. HTTP Parsing

### Parse Request Line
```asm
parse_request:
    ; r0 = request buffer
    ; Extract method (until space)
    mov r1, method_buf
    call copy_until_space

    ; Skip space, extract path
    mov r1, path_buf
    call copy_until_space

    ; Find body (after \r\n\r\n)
    call find_body
    ret

copy_until_space:
    ; r0 = src, r1 = dest, advances r0
copy_space_loop:
    load.b r2, [r0]
    mov r3, 0x20                 ; space
    beq r2, r3, copy_space_done
    store.b r2, [r1]
    addi r0, r0, 1
    addi r1, r1, 1
    b copy_space_loop
copy_space_done:
    store.b zero, [r1]           ; null terminate
    addi r0, r0, 1               ; skip space
    ret
```

### Extract Path Parameter
```asm
extract_path_id:
    ; r0 = path like "/users/123" -> r1 = "123"
    ; Find second '/'
    mov r2, 0
find_second_slash:
    load.b r3, [r0]
    beq r3, zero, no_id
    mov r4, 0x2F                 ; '/'
    bne r3, r4, not_slash
    addi r2, r2, 1
    mov r5, 2
    beq r2, r5, found_id_start
not_slash:
    addi r0, r0, 1
    b find_second_slash
found_id_start:
    addi r0, r0, 1               ; skip the '/'
    mov r1, r0                   ; r1 points to ID string
    ret
no_id:
    mov r1, 0
    ret
```

## 8. Extension Quick Reference

### JSON (200-211)
- `200` json_parse(str) -> handle
- `201` json_stringify(handle) -> str
- `202` json_get(handle, key) -> value
- `203` json_set(handle, key, value)
- `209` json_free(handle)
- `210` json_new_object() -> handle
- `211` json_new_array() -> handle

### HTTP Client (220-231)
- `220` http_get(url) -> response
- `221` http_post(url, body) -> response
- `222` http_put(url, body) -> response
- `223` http_delete(url) -> response
- `226` http_response_status(response) -> code
- `227` http_response_body(response) -> body
- `231` http_free(response)

### SQLite (260-273)
- `260` sqlite_open(path) -> db
- `261` sqlite_close(db)
- `262` sqlite_execute(db, sql)
- `263` sqlite_query(db, sql) -> result
- `264` sqlite_prepare(db, sql) -> stmt
- `265` sqlite_bind_int(stmt, idx, val)
- `266` sqlite_bind_text(stmt, idx, val)
- `268` sqlite_step(stmt)
- `270` sqlite_finalize(stmt)
- `271` sqlite_column_int(stmt, col) -> int
- `272` sqlite_column_text(stmt, col) -> str

### UUID (330-334)
- `330` uuid_v4() -> uuid
- `331` uuid_v7() -> uuid
- `333` uuid_to_string(uuid) -> str
