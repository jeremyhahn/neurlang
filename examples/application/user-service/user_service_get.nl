; @name: User Service - Get User
; @description: REST endpoint that retrieves user from database, returns JSON
; @category: application/user-service
; @difficulty: 2
;
; @prompt: get user api endpoint
; @prompt: rest api get user
; @prompt: fetch user from database
; @prompt: user retrieval endpoint
; @prompt: get user by id
; @prompt: user lookup api
; @prompt: get user rest service
; @prompt: user service get endpoint
; @prompt: retrieve user as json
; @prompt: fetch user api
;
; Mock all extensions for testing
; @mock: sqlite_open=1
; @mock: sqlite_prepare=1
; @mock: sqlite_bind_text=0
; @mock: sqlite_step=1
; @mock: sqlite_column_text=12345
; @mock: sqlite_finalize=0
; @mock: sqlite_close=0
; @mock: json_new_object=1
; @mock: json_set=0
; @mock: json_stringify=65536
; @mock: json_free=0
;
; @test: r0=1 -> r0=65536
;
; @note: Composes: sqlite_query, json_build
; @note: Demonstrates extension composition for read operation

.entry main

.section .data
    db_path:     .asciz "/tmp/users.db"
    select_sql:  .asciz "SELECT id, name, email FROM users WHERE id = ?"
    key_id:      .asciz "id"
    key_name:    .asciz "name"
    key_email:   .asciz "email"

.section .text

main:
    ; r0 = user_id string (from path parameter)
    mov r15, r0                      ; save user_id

    ; Open database
    mov r0, db_path
    ext.call r10, sqlite_open, r0
    beq r10, zero, error

    ; Prepare select statement
    mov r0, select_sql
    ext.call r11, sqlite_prepare, r10, r0
    beq r11, zero, error_close

    ; Bind user_id parameter
    mov r0, 1
    ext.call r2, sqlite_bind_text, r11, r0, r15

    ; Execute and check for result
    ext.call r0, sqlite_step, r11
    beq r0, zero, not_found

    ; Get column values
    mov r0, 0
    ext.call r3, sqlite_column_text, r11, r0  ; r3 = id
    mov r0, 1
    ext.call r4, sqlite_column_text, r11, r0  ; r4 = name
    mov r0, 2
    ext.call r5, sqlite_column_text, r11, r0  ; r5 = email

    ext.call r0, sqlite_finalize, r11
    ext.call r0, sqlite_close, r10

    ; Build response JSON
    ext.call r1, json_new_object

    mov r2, key_id
    ext.call r0, json_set, r1, r2, r3

    mov r2, key_name
    ext.call r0, json_set, r1, r2, r4

    mov r2, key_email
    ext.call r0, json_set, r1, r2, r5

    ; Stringify and return
    ext.call r0, json_stringify, r1
    ext.call r6, json_free, r1
    halt

not_found:
    ext.call r0, sqlite_finalize, r11

error_close:
    ext.call r0, sqlite_close, r10

error:
    mov r0, 0
    halt
