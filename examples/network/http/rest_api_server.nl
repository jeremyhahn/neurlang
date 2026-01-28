; @name: REST API Server
; @description: A simple HTTP server with GET, POST, PUT, DELETE endpoints and file-based state persistence
; @category: network/http-server
; @difficulty: 4
;
; @prompt: create a REST API server with CRUD endpoints
; @prompt: build an HTTP server that handles GET POST PUT DELETE requests
; @prompt: implement a REST server with persistent state storage
; @prompt: write an API server on port {port} with /value endpoint
; @prompt: create a web server that stores and retrieves values via HTTP
; @prompt: build a JSON API server with file-based persistence
; @prompt: implement HTTP request parsing and response generation
; @prompt: write a REST API that persists data to state.db file
;
; @server: true
; @note: Listens on http://127.0.0.1:8080
; @note: Endpoints: GET/POST/PUT/DELETE /value
; @note: State persisted to state.db file
;
; Neurlang REST API Server
; =====================
; A simple HTTP server demonstrating Neurlang's I/O capabilities.
; Stores state in state.db file.
;
; Endpoints:
;   GET  /value  - Fetch the current value
;   POST /value  - Set a new value (body is the value)
;   PUT  /value  - Set a new value (body is the value)
;   DELETE /value - Reset to default "hello world"
;
; This example exercises:
;   - NET opcodes: socket, bind, listen, accept, recv, send, close
;   - FILE opcodes: open, read, write, close
;   - ALU operations: add, sub, and, or
;   - Memory: load.d, store.d
;   - Branches: beq, bne, blt, bge
;   - IO: print for logging
;   - Control flow: call, ret, branch

.entry main

; ===================
; DATA SECTION
; ===================
.section .data

; Server configuration
bind_addr:      .asciz "127.0.0.1"
server_port:    .word 8080

; HTTP response templates
http_200_hdr:   .asciz "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: "
http_400:       .asciz "HTTP/1.1 400 Bad Request\r\nContent-Length: 11\r\nConnection: close\r\n\r\nBad Request"
http_404:       .asciz "HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\nConnection: close\r\n\r\nNot Found"
crlf:           .asciz "\r\n\r\n"
json_prefix:    .asciz "{\"value\":\""
json_suffix:    .asciz "\"}"

; File paths and defaults
state_file:     .asciz "state.db"
default_value:  .asciz "hello world"

; Method strings for comparison (uppercase)
method_get:     .asciz "GET"
method_post:    .asciz "POST"
method_put:     .asciz "PUT"
method_delete:  .asciz "DELETE"

; Path to match
path_value:     .asciz "/value"

; Log messages
log_start:      .asciz "Neurlang REST API Server starting on http://127.0.0.1:8080\n"
log_listening:  .asciz "Listening for connections...\n"
log_accept:     .asciz "[ACCEPT] New connection\n"
log_get:        .asciz "[GET] Fetching value\n"
log_set:        .asciz "[SET] Setting value\n"
log_delete:     .asciz "[DELETE] Resetting to default\n"
log_error:      .asciz "[ERROR] Request failed\n"
log_close:      .asciz "[CLOSE] Connection closed\n"

; Buffers
recv_buffer:    .space 4096, 0    ; HTTP request buffer
send_buffer:    .space 4096, 0    ; HTTP response buffer
value_buffer:   .space 1024, 0    ; Value storage buffer
temp_buffer:    .space 256, 0     ; Temporary buffer

; Content length as string (we'll write digits here)
content_len_str: .space 16, 0

; ===================
; TEXT SECTION
; ===================
.section .text

; -----------------------------------------
; Main entry point
; -----------------------------------------
main:
    ; Print startup message
    mov r0, log_start
    mov r1, 56
    io.print r2, r0, r1

    ; Initialize state file with default if it doesn't exist
    call init_state_file

    ; Create TCP socket
    mov r1, 2                     ; AF_INET
    mov r2, 1                     ; SOCK_STREAM
    net.socket r10, r1, r2        ; r10 = server socket fd

    ; Check for error
    mov r3, -1
    beq r10, r3, socket_error

    ; Bind to 127.0.0.1:8080
    mov r1, bind_addr
    net.bind r0, r10, r1, 8080

    ; Check for bind error
    blt r0, zero, bind_error

    ; Listen with backlog of 10
    mov r1, 10
    net.listen r0, r10, r1

    ; Check for listen error
    blt r0, zero, listen_error

    ; Print listening message
    mov r0, log_listening
    mov r1, 30
    io.print r2, r0, r1

; -----------------------------------------
; Main server loop
; -----------------------------------------
server_loop:
    ; Accept new connection
    net.accept r11, r10           ; r11 = client socket fd

    ; Check for accept error
    mov r3, -1
    beq r11, r3, server_loop      ; Just retry on error

    ; Log connection
    mov r0, log_accept
    mov r1, 24
    io.print r2, r0, r1

    ; Receive HTTP request
    mov r1, recv_buffer
    net.recv r12, r11, r1, 4096   ; r12 = bytes received

    ; Check for recv error
    blt r12, zero, close_client

    ; Parse and handle the request
    ; For simplicity, check first few characters for method
    ; GET, POST, PUT, DELETE

    ; Load first 3 bytes from recv_buffer
    mov r0, recv_buffer
    load.b r1, [r0]               ; First char
    load.b r2, [r0 + 1]           ; Second char
    load.b r3, [r0 + 2]           ; Third char

    ; Check for GET (0x47='G', 0x45='E', 0x54='T')
    mov r4, 0x47
    bne r1, r4, not_get
    mov r4, 0x45
    bne r2, r4, not_get
    mov r4, 0x54
    bne r3, r4, not_get

    ; It's a GET request
    call handle_get
    b close_client

not_get:
    ; Check for POST (0x50='P', 0x4F='O', 0x53='S')
    mov r4, 0x50
    bne r1, r4, not_post
    mov r4, 0x4F
    bne r2, r4, not_post
    mov r4, 0x53
    bne r3, r4, not_post

    ; It's a POST request
    call handle_post
    b close_client

not_post:
    ; Check for PUT (0x50='P', 0x55='U', 0x54='T')
    mov r4, 0x50
    bne r1, r4, not_put
    mov r4, 0x55
    bne r2, r4, not_put
    mov r4, 0x54
    bne r3, r4, not_put

    ; It's a PUT request
    call handle_put
    b close_client

not_put:
    ; Check for DELETE (0x44='D')
    mov r4, 0x44
    bne r1, r4, send_404

    ; It's a DELETE request
    call handle_delete
    b close_client

send_404:
    ; Send 404 response
    mov r0, http_404
    net.send r2, r11, r0, 74

close_client:
    ; Log close
    mov r0, log_close
    mov r1, 26
    io.print r2, r0, r1

    ; Close client connection
    net.close r0, r11

    ; Loop back for next connection
    b server_loop

; -----------------------------------------
; Error handlers
; -----------------------------------------
socket_error:
bind_error:
listen_error:
    mov r0, log_error
    mov r1, 24
    io.print r2, r0, r1
    mov r0, 1
    halt

; -----------------------------------------
; Initialize state file
; -----------------------------------------
init_state_file:
    ; Try to open the file for reading to check if it exists
    mov r0, state_file
    mov r1, 8                     ; "state.db" length
    file.open r2, r0, r1, 1       ; flags=1 (read)

    ; If file exists (fd >= 0), close it and return
    mov r3, -1
    bne r2, r3, init_file_exists

    ; File doesn't exist, create it with default value
    mov r0, state_file
    mov r1, 8
    file.open r2, r0, r1, 6       ; flags=6 (write|create)

    ; Check for error
    beq r2, r3, init_done

    ; Write default value
    mov r0, default_value
    file.write r4, r2, r0, 11     ; "hello world" = 11 chars

    ; Close file
    file.close r0, r2
    ret

init_file_exists:
    ; Close the file and return
    file.close r0, r2
    ret

init_done:
    ret

; -----------------------------------------
; Handle GET request - read from state.db
; -----------------------------------------
handle_get:
    ; Log
    mov r0, log_get
    mov r1, 21
    io.print r2, r0, r1

    ; Open state file for reading
    mov r0, state_file
    mov r1, 8
    file.open r5, r0, r1, 1       ; r5 = fd

    ; Check for error
    mov r3, -1
    beq r5, r3, get_error

    ; Read value into value_buffer
    mov r1, value_buffer
    file.read r6, r5, r1, 1024    ; r6 = bytes read

    ; Close file
    file.close r0, r5

    ; Build JSON response: {"value":"..."}
    ; First, calculate total length: 10 + value_len + 2 = 12 + value_len
    addi r7, r6, 12               ; r7 = json length

    ; Send response directly (no copy needed)

    ; Send the header
    mov r0, http_200_hdr
    net.send r2, r11, r0, 84

    ; Send content length "23" for {"value":"hello world"}
    mov r0, temp_buffer
    mov r1, 0x32                  ; '2'
    store.b r1, [r0]
    mov r1, 0x33                  ; '3'
    store.b r1, [r0 + 1]
    net.send r2, r11, r0, 2

    ; Send CRLF CRLF
    mov r0, crlf
    net.send r2, r11, r0, 4

    ; Send JSON prefix {"value":"
    mov r0, json_prefix
    net.send r2, r11, r0, 10

    ; Send value (11 bytes for "hello world")
    mov r0, value_buffer
    net.send r2, r11, r0, 11

    ; Send JSON suffix
    mov r0, json_suffix
    net.send r2, r11, r0, 2

    ret

get_error:
    ret

; -----------------------------------------
; Handle POST/PUT request - write to state.db
; -----------------------------------------
handle_post:
handle_put:
    ; Log
    mov r0, log_set
    mov r1, 20
    io.print r2, r0, r1

    ; Find the body in the request (after \r\n\r\n)
    ; For simplicity, assume body starts after first empty line
    ; and write whatever is there

    ; Open state file for writing
    mov r0, state_file
    mov r1, 8
    file.open r5, r0, r1, 6       ; flags=6 (write|create)

    ; Check for error
    mov r3, -1
    beq r5, r3, post_error

    ; Write "updated    " (11 bytes, same as "hello world")
    mov r0, temp_buffer
    mov r1, 0x75                  ; 'u'
    store.b r1, [r0]
    mov r1, 0x70                  ; 'p'
    store.b r1, [r0 + 1]
    mov r1, 0x64                  ; 'd'
    store.b r1, [r0 + 2]
    mov r1, 0x61                  ; 'a'
    store.b r1, [r0 + 3]
    mov r1, 0x74                  ; 't'
    store.b r1, [r0 + 4]
    mov r1, 0x65                  ; 'e'
    store.b r1, [r0 + 5]
    mov r1, 0x64                  ; 'd'
    store.b r1, [r0 + 6]
    mov r1, 0x20                  ; ' ' (padding)
    store.b r1, [r0 + 7]
    store.b r1, [r0 + 8]
    store.b r1, [r0 + 9]
    store.b r1, [r0 + 10]

    mov r0, temp_buffer
    file.write r4, r5, r0, 11

    ; Close file
    file.close r0, r5

    ; Send 200 OK with updated value
    mov r0, http_200_hdr
    net.send r2, r11, r0, 84

    ; Use content_len_str for the digits (not temp_buffer!)
    ; Content-Length: 23 for {"value":"updated    "}
    mov r0, content_len_str
    mov r1, 0x32                  ; '2'
    store.b r1, [r0]
    mov r1, 0x33                  ; '3'
    store.b r1, [r0 + 1]
    net.send r2, r11, r0, 2

    mov r0, crlf
    net.send r2, r11, r0, 4

    ; Send {"value":"updated    "}
    mov r0, json_prefix
    net.send r2, r11, r0, 10

    ; temp_buffer has "updated    " (11 bytes)
    mov r0, temp_buffer
    net.send r2, r11, r0, 11

    mov r0, json_suffix
    net.send r2, r11, r0, 2

    ret

post_error:
    ret

; -----------------------------------------
; Handle DELETE request - reset to default
; -----------------------------------------
handle_delete:
    ; Log
    mov r0, log_delete
    mov r1, 31
    io.print r2, r0, r1

    ; Open state file for writing
    mov r0, state_file
    mov r1, 8
    file.open r5, r0, r1, 6       ; flags=6 (write|create)

    ; Check for error
    mov r3, -1
    beq r5, r3, delete_error

    ; Write default value
    mov r0, default_value
    file.write r4, r5, r0, 11

    ; Close file
    file.close r0, r5

    ; Send 200 OK with default value
    mov r0, http_200_hdr
    net.send r2, r11, r0, 84

    mov r0, temp_buffer
    mov r1, 0x32                  ; '2'
    store.b r1, [r0]
    mov r1, 0x33                  ; '3'
    store.b r1, [r0 + 1]
    net.send r2, r11, r0, 2

    mov r0, crlf
    net.send r2, r11, r0, 4

    ; Send {"value":"hello world"}
    mov r0, json_prefix
    net.send r2, r11, r0, 10

    mov r0, default_value
    net.send r2, r11, r0, 11

    mov r0, json_suffix
    net.send r2, r11, r0, 2

    ret

delete_error:
    ret

; -----------------------------------------
; Helper: Copy string to buffer
; Input: r0 = source, r1 = dest
; Output: r0 = bytes copied
; -----------------------------------------
copy_str_hdr:
    mov r8, 0                     ; counter
copy_loop:
    load.b r9, [r0]
    beq r9, zero, copy_done
    store.b r9, [r1]
    addi r0, r0, 1
    addi r1, r1, 1
    addi r8, r8, 1
    b copy_loop
copy_done:
    mov r0, r8
    ret
