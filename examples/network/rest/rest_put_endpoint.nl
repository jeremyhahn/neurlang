; @name: REST PUT Endpoint
; @description: HTTP PUT endpoint that updates a resource
; @category: network/rest
; @difficulty: 3
;
; @prompt: create PUT endpoint
; @prompt: implement HTTP PUT handler
; @prompt: REST API PUT request handler
; @prompt: handle PUT /resource/{id} request
; @prompt: update resource with PUT
; @prompt: HTTP PUT response 200 OK
; @prompt: PUT /items/{id} to update item
; @prompt: replace resource via PUT
; @prompt: REST update endpoint
; @prompt: modify existing resource with PUT
;
; @server: true
; @note: Listens on http://127.0.0.1:8092
; @note: PUT / returns 200 OK with {"updated":true}
;
; Network mocks for testing
; @net_mock: socket=3
; @net_mock: bind=0
; @net_mock: listen=0
; @net_mock: accept=5,-1
; @net_mock: recv="PUT /item/1 HTTP/1.1\r\nContent-Length: 10\r\n\r\n{\"x\":2}"
; @net_mock: send=89
; @net_mock: close=0
;
; @test: -> r0=0

.entry main

.section .data
bind_addr:   .asciz "127.0.0.1"
response:    .asciz "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 16\r\n\r\n{\"updated\":true}"
bad_method:  .asciz "HTTP/1.1 405 Method Not Allowed\r\nContent-Length: 0\r\n\r\n"
recv_buf:    .space 1024, 0

.section .text

main:
    ; Create TCP socket
    mov r1, 2
    mov r2, 1
    net.socket r10, r1, r2

    ; Bind to port 8092
    mov r1, bind_addr
    net.bind r0, r10, r1, 8092

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

    ; Check if PUT request (first char = 'P', second = 'U')
    mov r3, recv_buf
    load.b r1, [r3]
    mov r2, 0x50                 ; 'P'
    bne r1, r2, send_405

    load.b r1, [r3 + 1]
    mov r2, 0x55                 ; 'U'
    bne r1, r2, send_405

    ; Send 200 OK response
    mov r0, response
    net.send r1, r11, r0, 89
    b close_client

send_405:
    mov r0, bad_method
    net.send r1, r11, r0, 55

close_client:
    net.close r0, r11
    b accept_loop
