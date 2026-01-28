; @name: REST POST Endpoint
; @description: HTTP POST endpoint that accepts JSON and creates resource
; @category: network/rest
; @difficulty: 3
;
; @prompt: create POST endpoint
; @prompt: implement HTTP POST handler
; @prompt: REST API POST request handler
; @prompt: handle POST /resource request
; @prompt: accept JSON via POST endpoint
; @prompt: create resource with POST
; @prompt: HTTP POST response 201 Created
; @prompt: POST /items to create new item
; @prompt: handle form submission via POST
; @prompt: REST create endpoint
;
; @server: true
; @note: Listens on http://127.0.0.1:8091
; @note: POST / returns 201 Created with {"id":"new","created":true}
;
; Network mocks for testing
; @net_mock: socket=3
; @net_mock: bind=0
; @net_mock: listen=0
; @net_mock: accept=5,-1
; @net_mock: recv="POST / HTTP/1.1\r\nContent-Length: 10\r\n\r\n{\"x\":1}"
; @net_mock: send=109
; @net_mock: close=0
;
; @test: -> r0=0

.entry main

.section .data
bind_addr:   .asciz "127.0.0.1"
response:    .asciz "HTTP/1.1 201 Created\r\nContent-Type: application/json\r\nContent-Length: 26\r\n\r\n{\"id\":\"new\",\"created\":true}"
bad_method:  .asciz "HTTP/1.1 405 Method Not Allowed\r\nContent-Length: 0\r\n\r\n"
recv_buf:    .space 1024, 0

.section .text

main:
    ; Create TCP socket
    mov r1, 2
    mov r2, 1
    net.socket r10, r1, r2

    ; Bind to port 8091
    mov r1, bind_addr
    net.bind r0, r10, r1, 8091

    ; Listen
    mov r1, 128
    net.listen r0, r10, r1

accept_loop:
    net.accept r11, r10
    mov r1, -1
    beq r11, r1, accept_loop

    ; Receive request
    mov r1, recv_buf
    net.recv r12, r11, r1, 1024
    blt r12, zero, close_client

    ; Check if POST request (first char = 'P', second = 'O')
    mov r3, recv_buf
    load.b r1, [r3]
    mov r2, 0x50                 ; 'P'
    bne r1, r2, send_405

    load.b r1, [r3 + 1]
    mov r2, 0x4F                 ; 'O'
    bne r1, r2, send_405

    ; Send 201 Created response
    mov r0, response
    net.send r1, r11, r0, 109
    b close_client

send_405:
    mov r0, bad_method
    net.send r1, r11, r0, 55

close_client:
    net.close r0, r11
    b accept_loop
