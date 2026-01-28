; @name: REST DELETE Endpoint
; @description: HTTP DELETE endpoint that removes a resource
; @category: network/rest
; @difficulty: 3
;
; @prompt: create DELETE endpoint
; @prompt: implement HTTP DELETE handler
; @prompt: REST API DELETE request handler
; @prompt: handle DELETE /resource/{id} request
; @prompt: remove resource with DELETE
; @prompt: HTTP DELETE response 204 No Content
; @prompt: DELETE /items/{id} to remove item
; @prompt: destroy resource via DELETE
; @prompt: REST delete endpoint
; @prompt: remove existing resource with DELETE
;
; @server: true
; @note: Listens on http://127.0.0.1:8093
; @note: DELETE / returns 204 No Content
;
; Network mocks for testing
; @net_mock: socket=3
; @net_mock: bind=0
; @net_mock: listen=0
; @net_mock: accept=5,-1
; @net_mock: recv="DELETE /item/1 HTTP/1.1\r\nHost: localhost\r\n\r\n"
; @net_mock: send=47
; @net_mock: close=0
;
; @test: -> r0=0

.entry main

.section .data
bind_addr:   .asciz "127.0.0.1"
response:    .asciz "HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n"
bad_method:  .asciz "HTTP/1.1 405 Method Not Allowed\r\nContent-Length: 0\r\n\r\n"
recv_buf:    .space 1024, 0

.section .text

main:
    ; Create TCP socket
    mov r1, 2
    mov r2, 1
    net.socket r10, r1, r2

    ; Bind to port 8093
    mov r1, bind_addr
    net.bind r0, r10, r1, 8093

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

    ; Check if DELETE request (first char = 'D')
    mov r3, recv_buf
    load.b r1, [r3]
    mov r2, 0x44                 ; 'D'
    bne r1, r2, send_405

    ; Send 204 No Content response
    mov r0, response
    net.send r1, r11, r0, 47
    b close_client

send_405:
    mov r0, bad_method
    net.send r1, r11, r0, 55

close_client:
    net.close r0, r11
    b accept_loop
