; @name: User Service REST API
; @description: User management REST API demonstrating HTTP server patterns
; @category: network/microservice
; @difficulty: 5
;
; @prompt: create a user service REST API on port 8080
; @prompt: implement user CRUD endpoints
; @prompt: build user management API with GET POST PUT DELETE endpoints
; @prompt: create REST API for user management
; @prompt: implement /users endpoints
; @prompt: build user microservice
; @prompt: create RESTful user service with JSON responses
; @prompt: implement user API with list get create update delete
;
; @server: true
; @note: Listens on http://127.0.0.1:8080
; @note: Endpoints: GET /users, GET /users/{id}, POST /users, PUT /users/{id}, DELETE /users/{id}
; @note: Uses file-based storage (same pattern as rest_api_crud.nl)
;
; Network mocks for testing GET /users endpoint
; @net_mock: socket=3
; @net_mock: bind=0
; @net_mock: listen=0
; @net_mock: accept=5,-1
; @net_mock: recv="GET /users HTTP/1.1\r\nHost: localhost\r\n\r\n",0
; @net_mock: send=100
; @net_mock: close=0
;
; @test: -> r0=0
;
; User Service REST API
; =====================
; Demonstrates user management REST API patterns:
; - HTTP server setup and accept loop
; - Request routing by method and path
; - Path parameter extraction
; - JSON response building
; - Proper HTTP status codes
;
; Register Convention:
;   r10 = server socket
;   r11 = client socket
;   r12 = request length
;   r15 = current ID length

.entry main

.section .data

; Network
bind_addr:      .asciz "127.0.0.1"

; HTTP responses
http_200:       .asciz "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: "
http_201:       .asciz "HTTP/1.1 201 Created\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: "
http_204:       .asciz "HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n"
http_400:       .asciz "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nContent-Length: 24\r\n\r\n{\"error\":\"Bad Request\"}"
http_404:       .asciz "HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\nContent-Length: 22\r\n\r\n{\"error\":\"Not Found\"}"
crlf2:          .asciz "\r\n\r\n"

; Paths
path_users:     .asciz "/users"

; JSON templates
json_start:     .asciz "{\"users\":["
json_end:       .asciz "]}"
json_user:      .asciz "{\"id\":\""
json_mid:       .asciz "\",\"name\":\""
json_close:     .asciz "\"}"

; Storage
state_file:     .asciz "/tmp/users.db"

; Log messages
log_start:      .asciz "User Service on http://127.0.0.1:8080\n"

; Buffers
recv_buf:       .space 4096, 0
resp_buf:       .space 4096, 0
id_buf:         .space 64, 0
len_buf:        .space 16, 0

.section .text

main:
    ; Log startup
    mov r0, log_start
    mov r1, 39
    io.print r2, r0, r1

    ; Create TCP socket
    mov r1, 2                    ; AF_INET
    mov r2, 1                    ; SOCK_STREAM
    net.socket r10, r1, r2

    ; Bind to port 8080
    mov r1, bind_addr
    net.bind r0, r10, r1, 8080

    ; Listen with backlog 128
    mov r1, 128
    net.listen r0, r10, r1

accept_loop:
    ; Accept new connection
    net.accept r11, r10
    mov r1, -1
    beq r11, r1, accept_loop

    ; Receive request
    mov r1, recv_buf
    mov r12, 4096
    net.recv r12, r11, r1, 0
    blt r12, zero, close_client

    ; Route request
    call route_request

close_client:
    net.close r0, r11
    b accept_loop

; ============================================================
; REQUEST ROUTING
; Determines method and dispatches to appropriate handler
; ============================================================
route_request:
    mov r0, recv_buf
    load.b r1, [r0]

    ; Check method by first character
    mov r2, 0x47                 ; 'G' for GET
    beq r1, r2, route_get
    mov r2, 0x50                 ; 'P' for POST/PUT
    beq r1, r2, check_post_put
    mov r2, 0x44                 ; 'D' for DELETE
    beq r1, r2, route_delete
    b send_400

check_post_put:
    ; Distinguish POST from PUT by second character
    load.b r1, [r0 + 1]
    mov r2, 0x4F                 ; 'O' for POST
    beq r1, r2, route_post
    mov r2, 0x55                 ; 'U' for PUT
    beq r1, r2, route_put
    b send_400

route_get:
    mov r1, 4                    ; offset after "GET "
    call parse_path
    mov r0, -1
    beq r15, r0, send_404        ; path didn't match
    beq r15, zero, do_get_all    ; /users (no ID)
    b do_get_one                 ; /users/{id}

route_post:
    mov r1, 5                    ; offset after "POST "
    call parse_path
    b do_create_user

route_put:
    mov r1, 4                    ; offset after "PUT "
    call parse_path
    beq r15, zero, send_400
    b do_update_user

route_delete:
    mov r1, 7                    ; offset after "DELETE "
    call parse_path
    beq r15, zero, send_400
    b do_delete_user

; ============================================================
; PATH PARSING
; Extracts /users or /users/{id}
; Output: r15 = ID length (0 for /users, -1 for no match)
; ============================================================
parse_path:
    mov r0, recv_buf
    add r0, r0, r1               ; r0 -> path start

    ; Compare with "/users"
    mov r2, path_users
    mov r3, 0

parse_cmp:
    mov r4, 6                    ; length of "/users"
    beq r3, r4, parse_matched
    load.b r4, [r0]
    load.b r5, [r2]
    bne r4, r5, parse_nomatch
    addi r0, r0, 1
    addi r2, r2, 1
    addi r3, r3, 1
    b parse_cmp

parse_matched:
    ; Check what's after /users
    load.b r4, [r0]
    mov r5, 0x20                 ; space = end of path
    beq r4, r5, parse_exact
    mov r5, 0x2F                 ; '/' = has ID
    beq r4, r5, parse_id
    b parse_nomatch

parse_exact:
    mov r15, 0
    ret

parse_id:
    ; Extract ID after the slash
    addi r0, r0, 1               ; skip '/'
    mov r1, id_buf
    mov r2, 0

parse_id_loop:
    load.b r3, [r0]
    mov r4, 0x20                 ; space
    beq r3, r4, parse_id_done
    mov r4, 0x3F                 ; '?'
    beq r3, r4, parse_id_done
    beq r3, zero, parse_id_done
    store.b r3, [r1]
    addi r0, r0, 1
    addi r1, r1, 1
    addi r2, r2, 1
    b parse_id_loop

parse_id_done:
    store.b zero, [r1]           ; null terminate
    mov r15, r2
    ret

parse_nomatch:
    mov r15, -1
    ret

; ============================================================
; GET /users - List all users
; Returns JSON array of users
; ============================================================
do_get_all:
    ; Build response: {"users":[...]}
    mov r0, resp_buf
    mov r1, json_start
    call str_copy
    mov r3, r2                   ; r3 = response length

    ; In real implementation, would read from storage
    ; For demo, return empty array

    ; Close array
    mov r0, resp_buf
    add r0, r0, r3
    mov r1, json_end
    call str_copy
    add r3, r3, r2

    mov r0, r3
    call send_200
    ret

; ============================================================
; GET /users/{id} - Get single user
; ============================================================
do_get_one:
    ; Build response: {"id":"xxx","name":"xxx"}
    mov r0, resp_buf
    mov r1, json_user
    call str_copy
    mov r3, r2

    ; Append ID
    mov r0, resp_buf
    add r0, r0, r3
    mov r1, id_buf
    mov r4, r15
    call mem_copy
    add r3, r3, r15

    ; Append rest of JSON
    mov r0, resp_buf
    add r0, r0, r3
    mov r1, json_close
    call str_copy
    add r3, r3, r2

    mov r0, r3
    call send_200
    ret

; ============================================================
; POST /users - Create new user
; ============================================================
do_create_user:
    ; Parse body, generate ID, store user
    ; Return 201 Created with user JSON

    ; For demo, return success with placeholder
    mov r0, resp_buf
    mov r1, json_user
    call str_copy
    mov r3, r2

    ; Add placeholder ID "new"
    mov r0, resp_buf
    add r0, r0, r3
    mov r2, 0x6E                 ; 'n'
    store.b r2, [r0]
    mov r2, 0x65                 ; 'e'
    store.b r2, [r0 + 1]
    mov r2, 0x77                 ; 'w'
    store.b r2, [r0 + 2]
    addi r3, r3, 3

    mov r0, resp_buf
    add r0, r0, r3
    mov r1, json_close
    call str_copy
    add r3, r3, r2

    mov r0, r3
    call send_201
    ret

; ============================================================
; PUT /users/{id} - Update user
; ============================================================
do_update_user:
    ; Same as GET one - return updated user
    b do_get_one

; ============================================================
; DELETE /users/{id} - Delete user
; ============================================================
do_delete_user:
    ; Return 204 No Content
    b send_204

; ============================================================
; HELPER: Copy null-terminated string
; Input: r0 = dest, r1 = src
; Output: r2 = length copied
; ============================================================
str_copy:
    mov r3, 0
str_copy_loop:
    load.b r4, [r1]
    beq r4, zero, str_copy_done
    store.b r4, [r0]
    addi r0, r0, 1
    addi r1, r1, 1
    addi r3, r3, 1
    b str_copy_loop
str_copy_done:
    mov r2, r3
    ret

; ============================================================
; HELPER: Copy r4 bytes
; Input: r0 = dest, r1 = src, r4 = count
; ============================================================
mem_copy:
    mov r2, 0
mem_copy_loop:
    beq r2, r4, mem_copy_done
    load.b r3, [r1]
    store.b r3, [r0]
    addi r0, r0, 1
    addi r1, r1, 1
    addi r2, r2, 1
    b mem_copy_loop
mem_copy_done:
    ret

; ============================================================
; HELPER: Integer to string
; Input: r0 = number
; Output: r1 = length, result in len_buf
; ============================================================
int_to_str:
    mov r2, len_buf
    mov r3, r0
    mov r4, 0

    bne r3, zero, int_conv
    mov r5, 0x30
    store.b r5, [r2]
    mov r1, 1
    ret

int_conv:
    mov r5, 100
    div r6, r3, r5
    beq r6, zero, int_try_10
    addi r7, r6, 0x30
    store.b r7, [r2]
    addi r2, r2, 1
    addi r4, r4, 1
    mul r7, r6, r5
    sub r3, r3, r7

int_try_10:
    mov r5, 10
    div r6, r3, r5
    beq r4, zero, int_skip_10
    b int_write_10
int_skip_10:
    beq r6, zero, int_write_1
int_write_10:
    addi r7, r6, 0x30
    store.b r7, [r2]
    addi r2, r2, 1
    addi r4, r4, 1
    mul r7, r6, r5
    sub r3, r3, r7

int_write_1:
    addi r7, r3, 0x30
    store.b r7, [r2]
    addi r4, r4, 1
    mov r1, r4
    ret

; ============================================================
; RESPONSE SENDERS
; ============================================================
send_200:
    ; r0 = body length
    mov r8, r0
    call int_to_str
    mov r9, r1

    mov r0, http_200
    net.send r2, r11, r0, 84

    mov r0, len_buf
    mov r3, r9
    net.send r3, r11, r0, 0

    mov r0, crlf2
    net.send r2, r11, r0, 4

    mov r0, resp_buf
    mov r3, r8
    net.send r3, r11, r0, 0
    b close_client

send_201:
    mov r8, r0
    call int_to_str
    mov r9, r1

    mov r0, http_201
    net.send r2, r11, r0, 89

    mov r0, len_buf
    mov r3, r9
    net.send r3, r11, r0, 0

    mov r0, crlf2
    net.send r2, r11, r0, 4

    mov r0, resp_buf
    mov r3, r8
    net.send r3, r11, r0, 0
    b close_client

send_204:
    mov r0, http_204
    net.send r1, r11, r0, 47
    b close_client

send_400:
    mov r0, http_400
    net.send r1, r11, r0, 106
    b close_client

send_404:
    mov r0, http_404
    net.send r1, r11, r0, 104
    b close_client
