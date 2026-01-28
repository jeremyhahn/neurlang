; @name: SQLite Insert Record
; @description: Inserts a record into SQLite database using extensions
; @category: extension/database
; @difficulty: 2
;
; @prompt: insert record into sqlite
; @prompt: sqlite insert row
; @prompt: add data to database
; @prompt: sqlite create record
; @prompt: database insert operation
; @prompt: store data in sqlite
; @prompt: sqlite insert with parameters
; @prompt: add row to table
; @prompt: sqlite parameterized insert
; @prompt: create database record
;
; Mock SQLite extensions for testing
; @mock: sqlite_open=1
; @mock: sqlite_prepare=1
; @mock: sqlite_bind_text=0
; @mock: sqlite_step=0
; @mock: sqlite_finalize=0
; @mock: sqlite_close=0
;
; @test: -> r0=1
;
; @note: Uses extensions: sqlite_open(260), sqlite_prepare(264), sqlite_bind_text(266), sqlite_step(268), sqlite_finalize(270)
; @note: Returns 1 on success, 0 on error

.entry main

.section .data
    db_path:    .asciz "/tmp/app.db"
    insert_sql: .asciz "INSERT INTO users (name, email) VALUES (?, ?)"
    name_val:   .asciz "Alice"
    email_val:  .asciz "alice@example.com"

.section .text

main:
    ; Open database
    mov r0, db_path
    ext.call r10, sqlite_open, r0    ; r10 = db handle
    beq r10, zero, error

    ; Prepare statement
    mov r0, insert_sql
    ext.call r11, sqlite_prepare, r10, r0  ; r11 = stmt handle
    beq r11, zero, close_db_error

    ; Bind parameter 1: name
    mov r0, 1
    mov r1, name_val
    ext.call r2, sqlite_bind_text, r11, r0, r1

    ; Bind parameter 2: email
    mov r0, 2
    mov r1, email_val
    ext.call r2, sqlite_bind_text, r11, r0, r1

    ; Execute statement
    ext.call r0, sqlite_step, r11

    ; Finalize statement
    ext.call r0, sqlite_finalize, r11

    ; Close database
    ext.call r0, sqlite_close, r10

    ; Return success
    mov r0, 1
    halt

close_db_error:
    ext.call r0, sqlite_close, r10

error:
    mov r0, 0
    halt
