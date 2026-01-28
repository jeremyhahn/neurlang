; @name: User Service - Update User
; @description: REST endpoint that updates user in database, returns JSON
; @category: application/user-service
; @difficulty: 3
;
; @prompt: update user api endpoint
; @prompt: rest api update user
; @prompt: modify user in database
; @prompt: user update endpoint
; @prompt: update user by id
; @prompt: user modification api
; @prompt: put user rest service
; @prompt: user service put endpoint
; @prompt: update user data
; @prompt: edit user api
;
; Mock all extensions for testing
; @mock: json_parse=1
; @mock: json_get=12345
; @mock: json_free=0
; @mock: sqlite_open=1
; @mock: sqlite_prepare=1
; @mock: sqlite_bind_text=0
; @mock: sqlite_step=0
; @mock: sqlite_finalize=0
; @mock: sqlite_close=0
; @mock: json_new_object=1
; @mock: json_set=0
; @mock: json_stringify=65536
;
; @test: r0=1, r1=1 -> r0=65536
;
; @note: Composes: json_parse, sqlite_update, json_build
; @note: Demonstrates extension composition for update operation

.entry main

.section .data
    db_path:     .asciz "/tmp/users.db"
    update_sql:  .asciz "UPDATE users SET name = ?, email = ? WHERE id = ?"
    key_id:      .asciz "id"
    key_name:    .asciz "name"
    key_email:   .asciz "email"
    key_status:  .asciz "status"
    status_ok:   .asciz "updated"

.section .text

main:
    ; r0 = user_id, r1 = request body JSON
    mov r14, r0                      ; save user_id
    mov r15, r1                      ; save body ptr

    ; Parse incoming JSON
    ext.call r1, json_parse, r15
    beq r1, zero, error

    ; Extract fields
    mov r2, key_name
    ext.call r3, json_get, r1, r2    ; r3 = name

    mov r2, key_email
    ext.call r4, json_get, r1, r2    ; r4 = email

    ext.call r0, json_free, r1

    ; Open database
    mov r0, db_path
    ext.call r10, sqlite_open, r0
    beq r10, zero, error

    ; Prepare update statement
    mov r0, update_sql
    ext.call r11, sqlite_prepare, r10, r0
    beq r11, zero, error_close

    ; Bind: name, email, id
    mov r0, 1
    ext.call r2, sqlite_bind_text, r11, r0, r3
    mov r0, 2
    ext.call r2, sqlite_bind_text, r11, r0, r4
    mov r0, 3
    ext.call r2, sqlite_bind_text, r11, r0, r14

    ext.call r0, sqlite_step, r11
    ext.call r0, sqlite_finalize, r11
    ext.call r0, sqlite_close, r10

    ; Build response JSON
    ext.call r1, json_new_object

    mov r2, key_id
    ext.call r0, json_set, r1, r2, r14

    mov r2, key_name
    ext.call r0, json_set, r1, r2, r3

    mov r2, key_email
    ext.call r0, json_set, r1, r2, r4

    mov r2, key_status
    mov r5, status_ok
    ext.call r0, json_set, r1, r2, r5

    ext.call r0, json_stringify, r1
    ext.call r6, json_free, r1
    halt

error_close:
    ext.call r0, sqlite_close, r10

error:
    mov r0, 0
    halt
