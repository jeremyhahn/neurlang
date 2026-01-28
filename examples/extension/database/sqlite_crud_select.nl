; @name: SQLite Query Record
; @description: Queries a record from SQLite database using extensions
; @category: extension/database
; @difficulty: 2
;
; @prompt: query record from sqlite
; @prompt: sqlite select row
; @prompt: read data from database
; @prompt: sqlite fetch record
; @prompt: database select operation
; @prompt: retrieve data from sqlite
; @prompt: sqlite query with parameters
; @prompt: get row from table
; @prompt: sqlite parameterized query
; @prompt: read database record
;
; Mock SQLite extensions for testing
; @mock: sqlite_open=1
; @mock: sqlite_prepare=1
; @mock: sqlite_bind_int=0
; @mock: sqlite_step=1
; @mock: sqlite_column_text=12345
; @mock: sqlite_finalize=0
; @mock: sqlite_close=0
;
; @test: -> r0=12345
;
; @note: Uses extensions: sqlite_open(260), sqlite_prepare(264), sqlite_bind_int(265), sqlite_step(268), sqlite_column_text(272), sqlite_finalize(270)
; @note: Returns pointer to name string in r0, or 0 if not found

.entry main

.section .data
    db_path:    .asciz "/tmp/app.db"
    select_sql: .asciz "SELECT name FROM users WHERE id = ?"
    user_id:    .word 1

.section .text

main:
    ; Open database
    mov r0, db_path
    ext.call r10, sqlite_open, r0    ; r10 = db handle
    beq r10, zero, error

    ; Prepare statement
    mov r0, select_sql
    ext.call r11, sqlite_prepare, r10, r0  ; r11 = stmt handle
    beq r11, zero, close_db_error

    ; Bind parameter 1: id
    mov r0, 1
    mov r1, user_id
    load.w r1, [r1]
    ext.call r2, sqlite_bind_int, r11, r0, r1

    ; Execute and check if row exists
    ext.call r0, sqlite_step, r11
    beq r0, zero, no_result

    ; Get column 0 (name)
    mov r0, 0
    ext.call r3, sqlite_column_text, r11, r0  ; r3 = name string

    ; Finalize and close
    ext.call r0, sqlite_finalize, r11
    ext.call r0, sqlite_close, r10

    ; Return the name
    mov r0, r3
    halt

no_result:
    ext.call r0, sqlite_finalize, r11

close_db_error:
    ext.call r0, sqlite_close, r10

error:
    mov r0, 0
    halt
