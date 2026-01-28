; @name: Static HTTP Server
; @description: Minimal HTTP server for benchmarking - returns fixed static response with no parsing
; @category: network/http-server
; @difficulty: 2
;
; @prompt: create a minimal HTTP server for benchmarking
; @prompt: build a static response server with no request parsing
; @prompt: write the fastest possible HTTP server in neurlang
; @prompt: implement a benchmark server that returns fixed JSON response
; @prompt: create a high-performance static HTTP server
; @prompt: build an HTTP server optimized for maximum throughput
; @prompt: write a server that just accepts receives sends closes in a loop
; @prompt: implement a minimal web server for performance testing
;
; @server: true
; @note: Listens on http://0.0.0.0:8080
; @note: Returns identical response to nginx for fair benchmarking
; @note: No parsing, no routing, no file I/O - maximum performance
;
; Network mocks for testing
; @net_mock: socket=3
; @net_mock: bind=0
; @net_mock: listen=0
; @net_mock: accept=5,-1
; @net_mock: recv="GET / HTTP/1.1\r\n\r\n",0
; @net_mock: send=225
; @net_mock: close=0
;
; @test: -> r0=0
;
; Static Server - Minimal HTTP server for benchmarking
; Returns the exact same static response as nginx for fair comparison
; No parsing, no routing, no file I/O - just accept, recv, send, close

.data
    bind_addr:      .asciz "0.0.0.0"
    log_start:      .asciz "Static server on http://0.0.0.0:8080\n"

    ; Pre-built HTTP response - identical to nginx (134 byte body)
    ; "Connection: close" = 17 chars + CRLF = 19 bytes
    ; Total: 17 + 32 + 21 + 19 + 2 + 134 = 225 bytes
    http_response:  .asciz "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 134\r\nConnection: close\r\n\r\n{\"values\":[{\"id\":\"user1\",\"value\":\"hello\"},{\"id\":\"user2\",\"value\":\"world\"},{\"id\":\"user3\",\"value\":\"test\"},{\"id\":\"user4\",\"value\":\"data\"}]}"

    ; Receive buffer
    recv_buf:       .space 4096

.text
main:
    ; Print startup message
    mov r0, log_start
    mov r1, 34
    io.print r2, r0, r1

    ; Create socket
    mov r1, 2
    mov r2, 1
    net.socket r10, r1, r2

    ; Bind
    mov r1, bind_addr
    net.bind r0, r10, r1, 8080

    ; Listen
    mov r1, 128
    net.listen r0, r10, r1

server_loop:
    ; Accept connection
    net.accept r11, r10
    mov r3, -1
    beq r11, r3, server_loop

    ; Read request (just drain it)
    mov r1, recv_buf
    mov r12, 4096
    net.recv r12, r11, r1, 0
    blt r12, zero, close_conn

    ; Send static response (225 bytes)
    mov r0, http_response
    net.send r2, r11, r0, 225

close_conn:
    net.close r0, r11
    b server_loop
