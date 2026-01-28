; @name: TCP Echo Server
; @description: A simple server that echoes back anything it receives on a TCP connection
; @category: network/tcp
; @difficulty: 2
;
; @prompt: create a TCP echo server that sends back received data
; @prompt: build a server on port {port} that echoes client messages
; @prompt: implement a simple TCP server using socket bind listen accept recv send
; @prompt: write an echo server that reflects all received data back to client
; @prompt: create a network server that echoes input on port 9000
; @prompt: demonstrate TCP server loop with multiple client connections
; @prompt: build a basic socket server that mirrors received data
; @prompt: implement echo protocol server using neurlang network opcodes
;
; @server: true
; @note: Listens on 0.0.0.0:9000
; @note: Test with: echo "hello" | nc localhost 9000
; @note: Handles multiple sequential connections
;
; Network mocks for testing (one client, echo once, then halt)
; @net_mock: socket=3
; @net_mock: bind=0
; @net_mock: listen=0
; @net_mock: accept=5,-1
; @net_mock: recv="hello",0
; @net_mock: send=5
; @net_mock: close=0
;
; @test: -> r0=0
;
; TCP Echo Server
; ===============
; A simple server that echoes back anything it receives.
; Demonstrates: socket, bind, listen, accept, recv, send, close
;
; Usage: Receives data from clients and sends it back unchanged.
; Test with: echo "hello" | nc localhost 9000

.entry main

.section .data

bind_addr:      .asciz "0.0.0.0"
log_start:      .asciz "Echo server on port 9000\n"
recv_buf:       .space 1024, 0

.section .text

main:
    ; Print startup message
    mov r0, log_start
    mov r1, 26
    io.print r2, r0, r1

    ; Create TCP socket: socket(AF_INET, SOCK_STREAM)
    mov r1, 2                     ; AF_INET
    mov r2, 1                     ; SOCK_STREAM
    net.socket r10, r1, r2        ; r10 = server socket

    ; Check for error
    mov r3, -1
    beq r10, r3, error_exit

    ; Bind to 0.0.0.0:9000
    mov r1, bind_addr
    net.bind r0, r10, r1, 9000

    ; Check for bind error
    blt r0, zero, error_exit

    ; Listen with backlog of 5
    mov r1, 5
    net.listen r0, r10, r1

    ; Check for listen error
    blt r0, zero, error_exit

accept_loop:
    ; Accept new connection
    net.accept r11, r10           ; r11 = client socket

    ; Skip if error
    mov r3, -1
    beq r11, r3, accept_loop

echo_loop:
    ; Receive data from client
    mov r1, recv_buf
    mov r2, 1024
    net.recv r12, r11, r1, 0      ; r12 = bytes received

    ; If recv returns <= 0, close connection
    blt r12, zero, close_client
    beq r12, zero, close_client

    ; Echo back the received data
    mov r1, recv_buf
    net.send r0, r11, r1, 0       ; Send r12 bytes back

    ; Continue receiving
    b echo_loop

close_client:
    net.close r0, r11
    b accept_loop

error_exit:
    mov r0, 1
    halt
