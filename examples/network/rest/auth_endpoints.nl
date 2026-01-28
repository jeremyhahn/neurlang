; @name: Authentication Endpoints
; @description: User authentication patterns with login and register
; @category: network/auth
; @difficulty: 4
;
; @prompt: implement POST /auth/login endpoint that validates credentials
; @prompt: create POST /auth/register endpoint for new user signup
; @prompt: build authentication API with login and register
; @prompt: implement user authentication REST endpoints
; @prompt: create login endpoint that checks credentials
; @prompt: implement user registration endpoint
; @prompt: build auth service with token generation
;
; @server: true
; @note: Part of User Service on port 8080
; @note: Endpoints: POST /auth/login, POST /auth/register
;
; Network mocks for testing POST /auth/login
; @net_mock: socket=3
; @net_mock: bind=0
; @net_mock: listen=0
; @net_mock: accept=5,-1
; @net_mock: recv="POST /auth/login HTTP/1.1\r\nContent-Type: application/json\r\n\r\n{\"username\":\"admin\",\"password\":\"secret\"}",0
; @net_mock: send=200
; @net_mock: close=0
;
; @test: -> r0=0
;
; Authentication Endpoints
; ========================
; Demonstrates authentication patterns:
; - POST /auth/login: Validate credentials, return token
; - POST /auth/register: Create new user account
; - Password checking patterns
; - Token generation (simplified)
;
; Register Convention:
;   r10 = server socket
;   r11 = client socket
;   r12 = request length

.entry main

.section .data

; Network
bind_addr:      .asciz "127.0.0.1"

; HTTP responses
http_200:       .asciz "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: "
http_201:       .asciz "HTTP/1.1 201 Created\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: "
http_400:       .asciz "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nContent-Length: 24\r\n\r\n{\"error\":\"Bad Request\"}"
http_401:       .asciz "HTTP/1.1 401 Unauthorized\r\nContent-Type: application/json\r\nContent-Length: 30\r\n\r\n{\"error\":\"Invalid credentials\"}"
crlf2:          .asciz "\r\n\r\n"

; Paths
path_login:     .asciz "/auth/login"
path_register:  .asciz "/auth/register"

; JSON templates
json_token:     .asciz "{\"token\":\""
json_id:        .asciz "{\"id\":\""
json_close:     .asciz "\"}"

; Demo credentials
demo_user:      .asciz "admin"
demo_pass:      .asciz "password"

; Log messages
log_start:      .asciz "Auth Service on http://127.0.0.1:8080/auth\n"

; Buffers
recv_buf:       .space 4096, 0
resp_buf:       .space 4096, 0
len_buf:        .space 16, 0

.section .text

main:
    mov r0, log_start
    mov r1, 44
    io.print r2, r0, r1

    ; Create TCP socket
    mov r1, 2
    mov r2, 1
    net.socket r10, r1, r2

    mov r1, bind_addr
    net.bind r0, r10, r1, 8080

    mov r1, 128
    net.listen r0, r10, r1

accept_loop:
    net.accept r11, r10
    mov r1, -1
    beq r11, r1, accept_loop

    mov r1, recv_buf
    mov r12, 4096
    net.recv r12, r11, r1, 0
    blt r12, zero, close_client

    call route_request

close_client:
    net.close r0, r11
    b accept_loop

; ============================================================
; REQUEST ROUTING
; ============================================================
route_request:
    ; Only handle POST for auth endpoints
    mov r0, recv_buf
    load.b r1, [r0]
    mov r2, 0x50                 ; 'P'
    bne r1, r2, send_400

    ; Check for /auth/login or /auth/register
    call check_login_path
    bne r0, zero, do_login

    call check_register_path
    bne r0, zero, do_register

    b send_400

; Check if path is /auth/login
check_login_path:
    mov r0, recv_buf
    addi r0, r0, 5               ; skip "POST "
    mov r1, path_login
    mov r2, 0

check_login_loop:
    mov r3, 11                   ; length of "/auth/login"
    beq r2, r3, check_login_match
    load.b r3, [r0]
    load.b r4, [r1]
    bne r3, r4, check_login_fail
    addi r0, r0, 1
    addi r1, r1, 1
    addi r2, r2, 1
    b check_login_loop

check_login_match:
    mov r0, 1
    ret

check_login_fail:
    mov r0, 0
    ret

; Check if path is /auth/register
check_register_path:
    mov r0, recv_buf
    addi r0, r0, 5
    mov r1, path_register
    mov r2, 0

check_reg_loop:
    mov r3, 14                   ; length of "/auth/register"
    beq r2, r3, check_reg_match
    load.b r3, [r0]
    load.b r4, [r1]
    bne r3, r4, check_reg_fail
    addi r0, r0, 1
    addi r1, r1, 1
    addi r2, r2, 1
    b check_reg_loop

check_reg_match:
    mov r0, 1
    ret

check_reg_fail:
    mov r0, 0
    ret

; ============================================================
; POST /auth/login
; Validates credentials and returns token
; ============================================================
do_login:
    ; In real implementation:
    ; 1. Parse JSON body for username/password
    ; 2. Query database for user
    ; 3. Hash password and compare
    ; 4. Generate JWT token on success

    ; For demo, return a token
    mov r0, resp_buf
    mov r1, json_token
    call str_copy
    mov r3, r2

    ; Add demo token
    mov r0, resp_buf
    add r0, r0, r3
    mov r2, 0x74                 ; 't'
    store.b r2, [r0]
    mov r2, 0x6F                 ; 'o'
    store.b r2, [r0 + 1]
    mov r2, 0x6B                 ; 'k'
    store.b r2, [r0 + 2]
    mov r2, 0x65                 ; 'e'
    store.b r2, [r0 + 3]
    mov r2, 0x6E                 ; 'n'
    store.b r2, [r0 + 4]
    mov r2, 0x31                 ; '1'
    store.b r2, [r0 + 5]
    mov r2, 0x32                 ; '2'
    store.b r2, [r0 + 6]
    mov r2, 0x33                 ; '3'
    store.b r2, [r0 + 7]
    addi r3, r3, 8

    mov r0, resp_buf
    add r0, r0, r3
    mov r1, json_close
    call str_copy
    add r3, r3, r2

    mov r0, r3
    call send_200
    ret

; ============================================================
; POST /auth/register
; Creates new user account
; ============================================================
do_register:
    ; In real implementation:
    ; 1. Parse JSON body for username/password/email
    ; 2. Check if username exists
    ; 3. Hash password
    ; 4. Generate UUID for user ID
    ; 5. Insert into database

    ; For demo, return success with placeholder ID
    mov r0, resp_buf
    mov r1, json_id
    call str_copy
    mov r3, r2

    ; Add demo ID "user123"
    mov r0, resp_buf
    add r0, r0, r3
    mov r2, 0x75                 ; 'u'
    store.b r2, [r0]
    mov r2, 0x73                 ; 's'
    store.b r2, [r0 + 1]
    mov r2, 0x65                 ; 'e'
    store.b r2, [r0 + 2]
    mov r2, 0x72                 ; 'r'
    store.b r2, [r0 + 3]
    mov r2, 0x31                 ; '1'
    store.b r2, [r0 + 4]
    mov r2, 0x32                 ; '2'
    store.b r2, [r0 + 5]
    mov r2, 0x33                 ; '3'
    store.b r2, [r0 + 6]
    addi r3, r3, 7

    mov r0, resp_buf
    add r0, r0, r3
    mov r1, json_close
    call str_copy
    add r3, r3, r2

    mov r0, r3
    call send_201
    ret

; ============================================================
; HELPERS
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
    mov r5, 10
    div r6, r3, r5
    beq r6, zero, int_write_1
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

send_400:
    mov r0, http_400
    net.send r1, r11, r0, 106
    b close_client

send_401:
    mov r0, http_401
    net.send r1, r11, r0, 112
    b close_client
