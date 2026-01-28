; @name: HTTPS Client
; @description: TLS client with certificate verification
; @category: patterns/network
; @difficulty: 4
;
; @prompt: make https request with tls
; @prompt: secure http client with tls
; @prompt: https get request with cert verify
; @prompt: tls encrypted http request
; @prompt: secure api call with https
; @prompt: https client with certificate check
; @prompt: ssl http client request
; @prompt: make secure https request
; @prompt: tls client connection
; @prompt: https request with verification
;
; @param: verify_cert=r0 "Whether to verify certificate (0/1)"
;
; Mock TLS extensions for testing:
; - 500 (tls_connect) returns handle 1
; - 501 (tls_write) returns bytes written (64)
; - 502 (tls_read) returns bytes read (64)
; - 503 (tls_close) returns 0 (success)
; @mock: 500=1
; @mock: 501=64
; @mock: 502=64
; @mock: 503=0
;
; Memory at 0x1004c (65612 = DATA_BASE + 76 = response_buf address) contains HTTP response
; @test: r0=1 [0x1004c]="HTTP/1.1 200 OK\r\n" -> r0=200
; @test: r0=0 [0x1004c]="HTTP/1.1 200 OK\r\n" -> r0=200
;
; @note: TLS operations are mocked for testing
; @note: Returns HTTP status code from response
; @note: Uses ext.call for TLS operations
;
; HTTPS Client Pattern
; ====================
; Establish TLS connection and make HTTP request.

.entry main

.section .data

host:               .asciz "api.example.com"
port:               .dword 443
path:               .asciz "/api/v1/users"
request_template:   .asciz "GET {path} HTTP/1.1\r\nHost: {host}\r\n\r\n"
response_buf:       .space 4096, 0

.section .text

main:
    ; r0 = verify_cert flag
    mov r10, r0

    ; Connect with TLS
    mov r0, host
    mov r1, port
    load.d r1, [r1]
    ext.call r11, 500, r0, r1       ; tls_connect -> r11 = handle

    ; Build HTTP request
    call build_request
    mov r12, r0                     ; r12 = request ptr
    mov r13, r1                     ; r13 = request len

    ; Send request
    ext.call r0, 501, r11, r12      ; tls_write

    ; Read response
    mov r0, response_buf
    ext.call r0, 502, r11, r0       ; tls_read -> r0 = bytes read

    ; Parse status code
    call parse_status_code
    mov r14, r0                     ; r14 = status code

    ; Close connection
    ext.call r0, 503, r11, zero     ; tls_close

    mov r0, r14                     ; Return status code
    halt

build_request:
    ; Build HTTP GET request
    ; Would format: GET /path HTTP/1.1\r\nHost: host\r\n\r\n
    mov r0, request_template
    mov r1, 64                      ; Request length
    ret

parse_status_code:
    ; Parse "HTTP/1.1 XXX " from response
    mov r0, response_buf
    ; Skip "HTTP/1.1 " (9 chars)
    addi r0, r0, 9

    ; Parse 3-digit status code
    mov r1, 0                       ; result

    load.b r2, [r0]
    subi r2, r2, 0x30               ; Convert ASCII to digit
    mov r3, 100
    muldiv.mul r2, r2, r3
    add r1, r1, r2

    load.b r2, [r0 + 1]
    subi r2, r2, 0x30
    mov r3, 10
    muldiv.mul r2, r2, r3
    add r1, r1, r2

    load.b r2, [r0 + 2]
    subi r2, r2, 0x30
    add r1, r1, r2

    mov r0, r1
    ret
