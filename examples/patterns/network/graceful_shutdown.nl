; @name: Graceful Shutdown
; @description: Drain active connections on server shutdown
; @category: patterns/network
; @difficulty: 4
;
; @prompt: implement graceful server shutdown
; @prompt: drain connections before shutdown
; @prompt: graceful shutdown with connection draining
; @prompt: stop accepting new connections on shutdown
; @prompt: wait for active requests before exit
; @prompt: graceful termination pattern
; @prompt: shutdown server gracefully
; @prompt: drain active connections then exit
; @prompt: orderly server shutdown
; @prompt: connection draining on shutdown
;
; @param: active_connections=r0 "Number of active connections"
; @param: new_request=r1 "Is there a new request (0/1)"
;
; @test: r0=0, r1=0 -> r0=0
; @test: r0=5, r1=0 -> r0=5
; @test: r0=0, r1=1 -> r0=1
;
; @note: Returns remaining connections after handling
; @note: Rejects new connections during shutdown
; @note: Testable shutdown state transitions
;
; Graceful Shutdown Pattern
; =========================
; Stop accepting, wait for active to complete, then exit.

.entry main

.section .data

shutdown_flag:      .dword 0        ; 0=running, 1=shutting down
active_count:       .dword 0        ; Active connections

.section .text

main:
    ; r0 = active connections
    ; r1 = new request incoming
    mov r10, r0
    mov r11, r1

    ; Store active count
    mov r0, active_count
    store.d r10, [r0]

    ; Check if shutting down
    mov r0, shutdown_flag
    load.d r0, [r0]
    bne r0, zero, draining_mode

    ; Normal operation - handle new request
    bne r11, zero, accept_new

    ; No new request, return current count
    mov r0, r10
    halt

accept_new:
    ; Accept and process
    addi r10, r10, 1
    mov r0, active_count
    store.d r10, [r0]
    mov r0, r10
    halt

draining_mode:
    ; Shutting down - reject new connections
    beq r11, zero, check_drained

    ; Reject new connection (would send 503)
    mov r0, r10
    halt

check_drained:
    ; Check if all connections drained
    beq r10, zero, shutdown_complete

    ; Still have active connections
    mov r0, r10
    halt

shutdown_complete:
    ; All connections drained - can exit
    mov r0, 0
    halt

; Signal handler for shutdown
handle_shutdown_signal:
    ; Set shutdown flag
    mov r0, shutdown_flag
    mov r1, 1
    store.d r1, [r0]

    ; Stop accepting new connections
    ; (In real impl, would close listening socket)
    ret

; Called when request completes
connection_completed:
    ; Decrement active count
    mov r0, active_count
    load.d r1, [r0]
    subi r1, r1, 1
    store.d r1, [r0]

    ; Check if can shutdown now
    mov r0, shutdown_flag
    load.d r0, [r0]
    beq r0, zero, conn_done

    ; Shutting down - check if drained
    mov r0, active_count
    load.d r0, [r0]
    bne r0, zero, conn_done

    ; All drained - trigger shutdown
    call complete_shutdown

conn_done:
    ret

complete_shutdown:
    ; Final cleanup and exit
    halt
