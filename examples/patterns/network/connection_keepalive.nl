; @name: Connection Keep-Alive
; @description: HTTP keep-alive connection reuse
; @category: patterns/network
; @difficulty: 3
;
; @prompt: implement http keep-alive
; @prompt: reuse http connections
; @prompt: connection keep-alive handling
; @prompt: persistent http connections
; @prompt: handle connection: keep-alive header
; @prompt: connection reuse pattern
; @prompt: http connection pooling
; @prompt: keep-alive timeout handling
; @prompt: persistent connection management
; @prompt: connection reuse with keep-alive
;
; @param: connection_header=r0 "0=close, 1=keep-alive"
; @param: idle_time=r1 "Seconds connection has been idle"
;
; @test: r0=1, r1=10 -> r0=1
; @test: r0=0, r1=0 -> r0=0
; @test: r0=1, r1=120 -> r0=0
;
; @note: Returns 1 if keep alive, 0 if close
; @note: Closes if idle > 60 seconds

.entry main

.section .data

keepalive_timeout:  .dword 60       ; Close after 60s idle

.section .text

main:
    ; r0 = connection header (0=close, 1=keep-alive)
    ; r1 = idle time in seconds
    mov r10, r0
    mov r11, r1

    ; If Connection: close, close immediately
    beq r10, zero, close_connection

    ; Check idle timeout
    mov r0, keepalive_timeout
    load.d r0, [r0]
    bge r11, r0, close_connection

    ; Keep alive
    mov r0, 1
    halt

close_connection:
    mov r0, 0
    halt
