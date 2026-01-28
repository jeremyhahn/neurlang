; @name: REST API Server (Dynamic)
; @description: HTTP server with dynamic buffer handling that accepts arbitrary values via POST/PUT
; @category: network/http-server
; @difficulty: 4
;
; @prompt: create a REST API server with dynamic content length handling
; @prompt: build an HTTP server that parses request body and stores arbitrary values
; @prompt: implement a REST server with dynamic buffer lengths for proper data handling
; @prompt: write an API server that extracts and saves POST body content
; @prompt: create a web server with dynamic response generation
; @prompt: build a JSON API that handles variable-length values
; @prompt: implement HTTP body parsing to extract POST data
; @prompt: write a REST API with dynamic content-length calculation
;
; @server: true
; @note: Listens on http://127.0.0.1:8080
; @note: Test with: curl -X POST http://localhost:8080/value -d "my custom value"
; @note: Supports arbitrary value lengths (not fixed to 11 bytes)
;
; Neurlang REST API Server (Dynamic Version)
; =======================================
; A real HTTP server that accepts arbitrary values via POST/PUT.
; Uses dynamic buffer lengths for proper data handling.
;
; Endpoints:
;   GET  /value  - Fetch the current value
;   POST /value  - Set a new value (body is the value)
;   PUT  /value  - Set a new value (body is the value)
;   DELETE /value - Reset to default "hello world"
;
; Test with:
;   curl http://localhost:8080/value
;   curl -X POST http://localhost:8080/value -d "my custom value"
;   curl http://localhost:8080/value

.entry main

; ===================
; DATA SECTION
; ===================
.section .data

; Server configuration
bind_addr:      .asciz "127.0.0.1"

; HTTP response templates
http_200_hdr:   .asciz "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: "
http_404:       .asciz "HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\nConnection: close\r\n\r\nNot Found"
crlf:           .asciz "\r\n\r\n"
json_prefix:    .asciz "{\"value\":\""
json_suffix:    .asciz "\"}"

; File paths and defaults
state_file:     .asciz "state.db"
default_value:  .asciz "hello world"

; Log messages
log_start:      .asciz "Neurlang REST API Server (Dynamic) on http://127.0.0.1:8080\n"
log_get:        .asciz "[GET] "
log_post:       .asciz "[POST] "
log_delete:     .asciz "[DELETE] reset\n"

; Buffers
recv_buffer:    .space 4096, 0    ; HTTP request buffer
value_buffer:   .space 1024, 0    ; Value storage buffer
len_str:        .space 16, 0      ; Content-Length as string

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

    ; Bind to 127.0.0.1:8080
    mov r1, bind_addr
    net.bind r0, r10, r1, 8080

    ; Listen with backlog of 10
    mov r1, 10
    net.listen r0, r10, r1

; -----------------------------------------
; Main server loop
; -----------------------------------------
server_loop:
    ; Accept new connection
    net.accept r11, r10           ; r11 = client socket fd

    ; Check for accept error
    mov r3, -1
    beq r11, r3, server_loop

    ; Receive HTTP request (dynamic max length)
    mov r1, recv_buffer
    mov r12, 4096                 ; max bytes
    net.recv r12, r11, r1, 0      ; DYNAMIC: r12 = bytes received

    ; Check for recv error
    blt r12, zero, close_client

    ; Parse method: check first byte
    mov r0, recv_buffer
    load.b r1, [r0]               ; First char

    ; Check for GET (0x47='G')
    mov r4, 0x47
    beq r1, r4, handle_get

    ; Check for POST (0x50='P')
    mov r4, 0x50
    beq r1, r4, check_post_put

    ; Check for DELETE (0x44='D')
    mov r4, 0x44
    beq r1, r4, handle_delete

    b send_404

check_post_put:
    ; POST and PUT both start with 'P', check second char
    load.b r2, [r0 + 1]
    mov r4, 0x4F                  ; 'O' for POST
    beq r2, r4, handle_post
    mov r4, 0x55                  ; 'U' for PUT
    beq r2, r4, handle_post
    b send_404

send_404:
    mov r0, http_404
    net.send r2, r11, r0, 74
    b close_client

close_client:
    net.close r0, r11
    b server_loop

; -----------------------------------------
; Initialize state file
; -----------------------------------------
init_state_file:
    mov r0, state_file
    mov r1, 8
    file.open r2, r0, r1, 1       ; try read

    mov r3, -1
    bne r2, r3, init_file_exists

    ; Create with default value
    mov r0, state_file
    mov r1, 8
    file.open r2, r0, r1, 6       ; write|create

    mov r0, default_value
    file.write r4, r2, r0, 11

    file.close r0, r2
    ret

init_file_exists:
    file.close r0, r2
    ret

; -----------------------------------------
; Handle GET request
; -----------------------------------------
handle_get:
    ; Log
    mov r0, log_get
    mov r1, 6
    io.print r2, r0, r1

    ; Open state file
    mov r0, state_file
    mov r1, 8
    file.open r5, r0, r1, 1       ; read

    mov r3, -1
    beq r5, r3, get_error

    ; Read value (dynamic max length)
    mov r1, value_buffer
    mov r6, 1024                  ; max bytes
    file.read r6, r5, r1, 0       ; DYNAMIC: r6 = bytes read

    file.close r0, r5

    ; Print value to console
    mov r0, value_buffer
    mov r1, r6
    io.print r2, r0, r1
    mov r0, crlf
    mov r1, 1
    io.print r2, r0, r1

    ; Calculate JSON length: 10 (prefix) + value_len + 2 (suffix)
    addi r7, r6, 12               ; r7 = total JSON length

    ; Convert length to ASCII string
    call int_to_str               ; r7 = number, returns len in r8

    ; Send HTTP header
    mov r0, http_200_hdr
    net.send r2, r11, r0, 84

    ; Send content length (dynamic)
    mov r0, len_str
    mov r3, r8                    ; r8 = length of number string
    net.send r3, r11, r0, 0       ; DYNAMIC

    ; Send CRLF
    mov r0, crlf
    net.send r2, r11, r0, 4

    ; Send JSON prefix
    mov r0, json_prefix
    net.send r2, r11, r0, 10

    ; Send value (dynamic length)
    mov r0, value_buffer
    mov r3, r6                    ; r6 = value length
    net.send r3, r11, r0, 0       ; DYNAMIC

    ; Send JSON suffix
    mov r0, json_suffix
    net.send r2, r11, r0, 2

    b close_client

get_error:
    b send_404

; -----------------------------------------
; Handle POST/PUT request - parse body and save
; -----------------------------------------
handle_post:
    ; Log
    mov r0, log_post
    mov r1, 7
    io.print r2, r0, r1

    ; Find body in request (after \r\n\r\n)
    ; r12 = total bytes received, recv_buffer has the request
    call find_body                ; Returns body_start in r13, body_len in r14

    ; Check if we found body
    beq r14, zero, post_error

    ; Print the value we're saving
    mov r0, r13                   ; body start address
    mov r1, r14                   ; body length
    io.print r2, r0, r1
    mov r0, crlf
    mov r1, 1
    io.print r2, r0, r1

    ; Open state file for writing
    mov r0, state_file
    mov r1, 8
    file.open r5, r0, r1, 6       ; write|create (truncates)

    mov r3, -1
    beq r5, r3, post_error

    ; Write body to file (dynamic length)
    mov r0, r13                   ; body address
    mov r3, r14                   ; body length
    file.write r3, r5, r0, 0      ; DYNAMIC: write r3 bytes

    file.close r0, r5

    ; Send response with the new value
    ; Calculate JSON length
    addi r7, r14, 12              ; JSON len = 10 + body_len + 2

    call int_to_str               ; r7 = number, r8 = string length

    ; Send HTTP header
    mov r0, http_200_hdr
    net.send r2, r11, r0, 84

    ; Send content length (dynamic)
    mov r0, len_str
    mov r3, r8
    net.send r3, r11, r0, 0

    ; Send CRLF
    mov r0, crlf
    net.send r2, r11, r0, 4

    ; Send JSON prefix
    mov r0, json_prefix
    net.send r2, r11, r0, 10

    ; Send body (dynamic length)
    mov r0, r13
    mov r3, r14
    net.send r3, r11, r0, 0

    ; Send JSON suffix
    mov r0, json_suffix
    net.send r2, r11, r0, 2

    b close_client

post_error:
    b send_404

; -----------------------------------------
; Handle DELETE request - reset to default
; -----------------------------------------
handle_delete:
    ; Log
    mov r0, log_delete
    mov r1, 16
    io.print r2, r0, r1

    ; Open state file
    mov r0, state_file
    mov r1, 8
    file.open r5, r0, r1, 6       ; write|create

    ; Write default value
    mov r0, default_value
    file.write r4, r5, r0, 11

    file.close r0, r5

    ; Response: {"value":"hello world"} = 23 bytes
    mov r0, http_200_hdr
    net.send r2, r11, r0, 84

    ; Send "23"
    mov r0, len_str
    mov r1, 0x32                  ; '2'
    store.b r1, [r0]
    mov r1, 0x33                  ; '3'
    store.b r1, [r0 + 1]
    mov r3, 2
    net.send r3, r11, r0, 0

    mov r0, crlf
    net.send r2, r11, r0, 4

    mov r0, json_prefix
    net.send r2, r11, r0, 10

    mov r0, default_value
    net.send r2, r11, r0, 11

    mov r0, json_suffix
    net.send r2, r11, r0, 2

    b close_client

; -----------------------------------------
; Find body in HTTP request
; Input: recv_buffer, r12 = total bytes
; Output: r13 = body start address, r14 = body length
; Uses: r0, r1, r2, r3, r4
; -----------------------------------------
find_body:
    mov r3, recv_buffer           ; base address
    mov r4, 0                     ; current offset

find_loop:
    ; Check if we've reached end (need at least 4 bytes)
    addi r0, r4, 4
    bgt r0, r12, find_not_found

    ; Load current position
    add r0, r3, r4
    load.b r1, [r0]
    mov r2, 0x0D                  ; '\r'
    bne r1, r2, find_next

    load.b r1, [r0 + 1]
    mov r2, 0x0A                  ; '\n'
    bne r1, r2, find_next

    load.b r1, [r0 + 2]
    mov r2, 0x0D                  ; '\r'
    bne r1, r2, find_next

    load.b r1, [r0 + 3]
    mov r2, 0x0A                  ; '\n'
    bne r1, r2, find_next

    ; Found \r\n\r\n! Body starts at offset + 4
    addi r4, r4, 4
    add r13, r3, r4               ; body start address
    sub r14, r12, r4              ; body length = total - offset
    ret

find_next:
    addi r4, r4, 1
    b find_loop

find_not_found:
    mov r13, r3
    mov r14, 0
    ret

; -----------------------------------------
; Convert integer to string
; Input: r7 = number
; Output: len_str buffer, r8 = string length
; -----------------------------------------
int_to_str:
    mov r0, len_str
    mov r8, 0                     ; string length
    mov r9, r7                    ; copy of number

    ; Handle 0 specially
    bne r9, zero, int_convert

    mov r1, 0x30                  ; '0'
    store.b r1, [r0]
    mov r8, 1
    ret

int_convert:
    ; Count digits and extract (simple: handle up to 3 digits for our use case)
    ; Divide by 100 for first digit
    mov r1, 100
    div r2, r9, r1                ; r2 = hundreds digit
    beq r2, zero, skip_hundreds
    addi r3, r2, 0x30             ; convert to ASCII
    store.b r3, [r0]
    addi r0, r0, 1
    addi r8, r8, 1
    mul r4, r2, r1
    sub r9, r9, r4                ; remainder

skip_hundreds:
    ; Tens digit
    mov r1, 10
    div r2, r9, r1
    ; Only skip if we haven't written anything yet AND digit is 0
    bne r8, zero, write_tens
    beq r2, zero, skip_tens
write_tens:
    addi r3, r2, 0x30
    store.b r3, [r0]
    addi r0, r0, 1
    addi r8, r8, 1
    mul r4, r2, r1
    sub r9, r9, r4

skip_tens:
    ; Units digit (always write)
    addi r3, r9, 0x30
    store.b r3, [r0]
    addi r8, r8, 1

    ret
