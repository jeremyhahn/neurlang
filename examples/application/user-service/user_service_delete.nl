; @name: User Service - Delete User
; @description: REST endpoint that deletes user from database
; @category: application/user-service
; @difficulty: 2
;
; @prompt: delete user api endpoint
; @prompt: rest api delete user
; @prompt: remove user from database
; @prompt: user deletion endpoint
; @prompt: delete user by id
; @prompt: user removal api
; @prompt: delete user rest service
; @prompt: user service delete endpoint
; @prompt: remove user data
; @prompt: delete user api
;
; Mock all extensions for testing
; @mock: sqlite_open=1
; @mock: sqlite_prepare=1
; @mock: sqlite_bind_text=0
; @mock: sqlite_step=0
; @mock: sqlite_finalize=0
; @mock: sqlite_close=0
; @mock: json_new_object=1
; @mock: json_set=0
; @mock: json_stringify=65536
; @mock: json_free=0
;
; @test: r0=1 -> r0=65536
;
; @note: Composes: sqlite_delete, json_build
; @note: Demonstrates extension composition for delete operation
; @note: Input: user_id, Output: JSON confirmation

.entry main

.section .data
    db_path:    .asciz "/tmp/users.db"
    delete_sql: .asciz "DELETE FROM users WHERE id = ?"
    key_id:     .asciz "id"
    key_status: .asciz "status"
    status_ok:  .asciz "deleted"

.section .text

main:
    ; r0 = user_id string
    mov r14, r0                      ; save user_id

    ; === Delete from database ===
    mov r0, db_path
    ext.call r10, sqlite_open, r0
    beq r10, zero, error

    mov r0, delete_sql
    ext.call r11, sqlite_prepare, r10, r0
    beq r11, zero, error_close

    ; Bind user_id
    mov r0, 1
    ext.call r2, sqlite_bind_text, r11, r0, r14

    ext.call r0, sqlite_step, r11
    ext.call r0, sqlite_finalize, r11
    ext.call r0, sqlite_close, r10

    ; === Build response JSON ===
    ext.call r1, json_new_object

    mov r2, key_id
    ext.call r0, json_set, r1, r2, r14

    mov r2, key_status
    mov r3, status_ok
    ext.call r0, json_set, r1, r2, r3

    ext.call r0, json_stringify, r1
    ext.call r4, json_free, r1
    halt

error_close:
    ext.call r0, sqlite_close, r10

error:
    mov r0, 0
    halt
