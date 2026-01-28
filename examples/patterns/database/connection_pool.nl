; @name: Connection Pool
; @description: Database connection pooling pattern
; @category: patterns/database
; @difficulty: 4
;
; @prompt: implement database connection pool
; @prompt: reuse database connections
; @prompt: connection pooling pattern
; @prompt: get connection from pool
; @prompt: return connection to pool
; @prompt: manage database connection pool
; @prompt: pool database connections
; @prompt: connection reuse pattern
; @prompt: database pool manager
; @prompt: limit active database connections
;
; @param: action=r0 "Action (0=get, 1=return)"
; @param: pool_size=r1 "Current pool available"
;
; @test: r0=0, r1=5 -> r0=1
; @test: r0=0, r1=0 -> r0=0
; @test: r0=1, r1=4 -> r0=5
;
; @note: Returns connection handle (get) or new pool size (return)
; @note: Returns 0 if pool exhausted

.entry main

.section .data

pool_capacity:      .word 10        ; Maximum connections
pool_available:     .word 10        ; Currently available

.section .text

main:
    ; r0 = action (0=get, 1=return)
    ; r1 = pool_available (for testing)
    mov r10, r0
    mov r11, r1

    ; Store test value
    mov r0, pool_available
    store.d r11, [r0]

    ; Dispatch action
    beq r10, zero, pool_get
    b pool_return

pool_get:
    ; Get connection from pool
    mov r0, pool_available
    load.d r1, [r0]

    ; Check if pool empty
    beq r1, zero, pool_exhausted

    ; Decrement available count
    subi r1, r1, 1
    store.d r1, [r0]

    ; Return mock connection handle
    mov r0, 1                       ; Non-zero = valid handle
    halt

pool_exhausted:
    mov r0, 0                       ; No connection available
    halt

pool_return:
    ; Return connection to pool
    mov r0, pool_available
    load.d r1, [r0]

    ; Increment available count
    addi r1, r1, 1

    ; Cap at capacity
    mov r2, pool_capacity
    load.d r2, [r2]
    blt r1, r2, store_available
    mov r1, r2

store_available:
    store.d r1, [r0]
    mov r0, r1                      ; Return new pool size
    halt
