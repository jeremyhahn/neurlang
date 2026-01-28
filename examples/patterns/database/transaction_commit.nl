; @name: Transaction Commit
; @description: Database transaction with BEGIN/COMMIT/ROLLBACK
; @category: patterns/database
; @difficulty: 4
;
; @prompt: implement database transaction with commit
; @prompt: wrap database operations in transaction
; @prompt: begin commit transaction pattern
; @prompt: rollback on transaction failure
; @prompt: atomic database update
; @prompt: transaction with rollback on error
; @prompt: database transaction handling
; @prompt: commit or rollback transaction
; @prompt: wrap queries in transaction
; @prompt: acid transaction pattern
;
; @param: should_fail=r0 "Whether operation should fail (0=success, 1=fail)"
;
; @test: r0=0 -> r0=1
; @test: r0=1 -> r0=0
;
; @note: Returns 1 if committed, 0 if rolled back
; @note: Demonstrates transaction pattern
;
; Transaction Pattern
; ===================
; BEGIN -> operations -> COMMIT (or ROLLBACK on error)

.entry main

.section .data

in_transaction:     .word 0

.section .text

main:
    ; r0 = should_fail
    mov r10, r0

    ; Begin transaction
    ; In real impl: ext.call r0, sqlite_begin, db_handle
    mov r0, in_transaction
    mov r1, 1
    store.d r1, [r0]

    ; Execute operation
    call do_database_work
    bne r0, zero, rollback

    ; Success - commit
    ; In real impl: ext.call r0, sqlite_commit, db_handle
    mov r0, in_transaction
    store.d zero, [r0]

    mov r0, 1                       ; Committed
    halt

rollback:
    ; In real impl: ext.call r0, sqlite_rollback, db_handle
    mov r0, in_transaction
    store.d zero, [r0]

    mov r0, 0                       ; Rolled back
    halt

do_database_work:
    ; Simulate work that may fail
    beq r10, zero, work_success
    mov r0, 1                       ; Failure
    ret
work_success:
    mov r0, 0                       ; Success
    ret
