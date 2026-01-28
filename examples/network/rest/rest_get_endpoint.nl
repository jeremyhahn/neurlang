; @name: REST GET Endpoint
; @description: Simple HTTP GET endpoint returning JSON data
; @category: network/rest
; @difficulty: 3
;
; @prompt: create GET endpoint
; @prompt: implement HTTP GET handler
; @prompt: REST API GET request handler
; @prompt: handle GET /resource request
; @prompt: return JSON from GET endpoint
; @prompt: simple GET API endpoint
; @prompt: HTTP GET response with JSON
; @prompt: read-only REST endpoint
; @prompt: fetch resource via GET
; @prompt: GET /items returning JSON
;
; @server: true
; @note: Listens on http://127.0.0.1:8090
; @note: GET / returns {"status":"ok","method":"GET"}
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

.section .data
bind_addr:   .asciz "127.0.0.1"
response:    .asciz "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 32\r\n\r\n{\"status\":\"ok\",\"method\":\"GET\"}"
recv_buf:    .space 1024, 0

.section .text

main:
    ; Create TCP socket
    mov r1, 2                    ; AF_INET
    mov r2, 1                    ; SOCK_STREAM
    net.socket r10, r1, r2

    ; Bind to port 8090
    mov r1, bind_addr
    net.bind r0, r10, r1, 8090

    ; Listen
    mov r1, 128
    net.listen r0, r10, r1

accept_loop:
    ; Accept connection
    net.accept r11, r10
    mov r1, -1
    beq r11, r1, accept_loop

    ; Receive request
    mov r1, recv_buf
    net.recv r12, r11, r1, 1024
    blt r12, zero, close_client

    ; Check if GET request (first char = 'G')
    mov r3, recv_buf
    load.b r1, [r3]
    mov r2, 0x47                 ; 'G'
    bne r1, r2, close_client

    ; Send JSON response
    mov r0, response
    net.send r1, r11, r0, 109

close_client:
    net.close r0, r11
    b accept_loop
