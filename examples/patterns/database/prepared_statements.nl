; @name: Prepared Statements
; @description: Use parameterized queries to prevent SQL injection
; @category: patterns/database
; @difficulty: 3
;
; @prompt: use prepared statement with parameters
; @prompt: parameterized sql query
; @prompt: bind parameters to prepared statement
; @prompt: prevent sql injection with prepared stmt
; @prompt: execute prepared statement safely
; @prompt: sql prepared statement pattern
; @prompt: bind values to sql query
; @prompt: parameterized database query
; @prompt: safe sql with bound parameters
; @prompt: prepared statement with placeholders
;
; @param: param_count=r0 "Number of parameters to bind"
;
; @test: r0=1 -> r0=1
; @test: r0=3 -> r0=1
; @test: r0=0 -> r0=1
;
; @note: Returns 1 on success
; @note: Demonstrates prepared statement pattern
;
; Prepared Statements Pattern
; ===========================
; Prepare query once, bind parameters safely, execute.

.entry main

.section .data

param_count:        .word 0

.section .text

main:
    ; r0 = number of parameters
    mov r10, r0

    ; Store param count
    mov r0, param_count
    store.d r10, [r0]

    ; Prepared statement workflow:
    ; 1. Prepare: sqlite_prepare(db, sql) -> stmt handle
    ; 2. Bind: sqlite_bind(stmt, index, value) for each param
    ; 3. Step: sqlite_step(stmt) to execute
    ; 4. Finalize: sqlite_finalize(stmt) to cleanup

    ; Simulate binding parameters
    mov r11, 1                      ; param index (1-based)

bind_loop:
    bgt r11, r10, execute_stmt

    ; Bind parameter r11
    ; In real impl: ext.call sqlite_bind, stmt, r11, value

    addi r11, r11, 1
    b bind_loop

execute_stmt:
    ; Execute prepared statement
    ; In real impl: ext.call sqlite_step, stmt

    ; Finalize
    ; In real impl: ext.call sqlite_finalize, stmt

    mov r0, 1                       ; Success
    halt
