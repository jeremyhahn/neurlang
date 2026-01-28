; @name: Network Echo Server Test
; @description: Tests basic network operations - TCP server that accepts one connection and echoes data
; @category: network/tcp
; @difficulty: 3
;
; @prompt: create a TCP echo server for testing network operations
; @prompt: write a network test that binds to port {port} and echoes data
; @prompt: demonstrate net.socket net.bind net.listen net.accept net.recv net.send
; @prompt: build a simple TCP server that receives and echoes back data
; @prompt: test network opcodes with a single-connection echo server
; @prompt: create a server on 127.0.0.1:{port} that echoes received data
; @prompt: implement a basic TCP server for network opcode testing
; @prompt: write a network test server with socket bind listen accept recv send close
;
; @server: true
; @note: Test with: nc localhost 8088
; @note: Server accepts one connection, echoes data, then exits
;
; Neurlang Network Test Program
; ===========================
; Tests basic network operations - creates a TCP server and waits for one connection
;
; This program:
; 1. Creates a TCP socket
; 2. Binds to 127.0.0.1:8088
; 3. Listens for connections
; 4. Accepts one connection
; 5. Receives data and echoes it back
; 6. Closes and exits

.entry main

.section .data

; Bind address (must be null-terminated string)
bind_addr:      .asciz "127.0.0.1"

; Messages
msg_start:      .asciz "Neurlang Network Test - Echo Server on port 8088\n"
msg_socket:     .asciz "Creating socket...\n"
msg_bind:       .asciz "Binding to 127.0.0.1:8088...\n"
msg_listen:     .asciz "Listening for connections...\n"
msg_accept:     .asciz "Waiting for connection (run: nc localhost 8088)...\n"
msg_connected:  .asciz "Client connected!\n"
msg_recv:       .asciz "Received: "
msg_send:       .asciz "Echoing back...\n"
msg_done:       .asciz "\nConnection closed. Test complete!\n"
msg_error:      .asciz "Error occurred!\n"
newline:        .asciz "\n"

; Receive buffer
recv_buffer:    .space 1024, 0

.section .text

main:
    ; Print start message
    mov r0, msg_start
    mov r1, 47
    io.print r2, r0, r1

    ; Print socket message
    mov r0, msg_socket
    mov r1, 19
    io.print r2, r0, r1

    ; Create TCP socket: socket(AF_INET=2, SOCK_STREAM=1)
    mov r1, 2                     ; AF_INET
    mov r2, 1                     ; SOCK_STREAM
    net.socket r10, r1, r2        ; r10 = server socket fd

    ; Check for error
    mov r3, -1
    beq r10, r3, error

    ; Print bind message
    mov r0, msg_bind
    mov r1, 29
    io.print r2, r0, r1

    ; Bind to 127.0.0.1:8088
    ; net.bind rd, rs1, rs2, imm
    ; rd = result, rs1 = fd, rs2 = addr_ptr, imm = port
    mov r1, bind_addr             ; address string pointer
    net.bind r0, r10, r1, 8088

    ; Check for error
    blt r0, zero, error

    ; Print listen message
    mov r0, msg_listen
    mov r1, 30
    io.print r2, r0, r1

    ; Listen with backlog of 1
    mov r1, 1
    net.listen r0, r10, r1

    ; Check for error
    blt r0, zero, error

    ; Print accept message
    mov r0, msg_accept
    mov r1, 50
    io.print r2, r0, r1

    ; Accept a connection
    net.accept r11, r10           ; r11 = client socket fd

    ; Check for error
    mov r3, -1
    beq r11, r3, error

    ; Print connected message
    mov r0, msg_connected
    mov r1, 18
    io.print r2, r0, r1

    ; Receive data from client
    ; net.recv rd, rs1, rs2, imm
    ; rd = bytes received, rs1 = fd, rs2 = buf_ptr, imm = max_len
    mov r1, recv_buffer
    net.recv r12, r11, r1, 1024   ; r12 = bytes received

    ; Check for error
    blt r12, zero, error

    ; Print "Received: "
    mov r0, msg_recv
    mov r1, 10
    io.print r2, r0, r1

    ; Print received data
    mov r0, recv_buffer
    mov r1, r12                   ; length
    io.print r2, r0, r1

    ; Print send message
    mov r0, msg_send
    mov r1, 16
    io.print r2, r0, r1

    ; Echo the data back (up to 256 bytes)
    ; net.send rd, rs1, rs2, imm
    ; For now, we use a fixed length since imm is the length
    mov r0, recv_buffer
    net.send r2, r11, r0, 256     ; Echo up to 256 bytes

    ; Close client connection
    net.close r0, r11

    ; Close server socket
    net.close r0, r10

    ; Print done message
    mov r0, msg_done
    mov r1, 36
    io.print r2, r0, r1

    ; Return success
    mov r0, 0
    halt

error:
    ; Print error message
    mov r0, msg_error
    mov r1, 17
    io.print r2, r0, r1

    ; Return error
    mov r0, 1
    halt
