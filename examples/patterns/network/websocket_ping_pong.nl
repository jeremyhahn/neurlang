; @name: WebSocket Ping Pong
; @description: WebSocket heartbeat with ping/pong frames
; @category: patterns/network
; @difficulty: 3
;
; @prompt: handle websocket ping pong frames
; @prompt: websocket heartbeat implementation
; @prompt: respond to websocket ping with pong
; @prompt: websocket keepalive mechanism
; @prompt: implement websocket ping handler
; @prompt: websocket connection heartbeat
; @prompt: send websocket ping frames
; @prompt: handle websocket pong response
; @prompt: websocket liveness check
; @prompt: websocket ping pong protocol
;
; @param: frame_type=r0 "Frame type (9=ping, 10=pong)"
;
; @test: r0=9 -> r0=10
; @test: r0=10 -> r0=0
; @test: r0=1 -> r0=0
;
; @note: Ping (opcode 9) triggers Pong (opcode 10)
; @note: Returns pong opcode (10) for ping, 0 otherwise

.entry main

.section .data

ping_opcode:        .dword 9
pong_opcode:        .dword 10

.section .text

main:
    ; r0 = frame opcode
    mov r10, r0

    ; Check if ping frame received
    mov r0, ping_opcode
    load.d r0, [r0]
    beq r10, r0, send_pong

    ; Check if pong frame received
    mov r0, pong_opcode
    load.d r0, [r0]
    beq r10, r0, handle_pong

    ; Other frame type
    mov r0, 0
    halt

send_pong:
    ; Respond to ping with pong
    mov r0, pong_opcode
    load.d r0, [r0]
    halt

handle_pong:
    ; Pong received - no response needed
    mov r0, 0
    halt
