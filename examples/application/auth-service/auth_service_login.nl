; @name: Auth Service - Login
; @description: Authenticates user and generates session token
; @category: application/auth-service
; @difficulty: 3
;
; @prompt: user login endpoint
; @prompt: authenticate user api
; @prompt: login rest service
; @prompt: user authentication endpoint
; @prompt: login with credentials
; @prompt: auth login api
; @prompt: session login endpoint
; @prompt: user signin api
; @prompt: authentication service login
; @prompt: login and get token
;
; Mock all extensions for testing
; @mock: json_parse=1
; @mock: json_get=12345
; @mock: json_free=0
; @mock: sqlite_open=1
; @mock: sqlite_prepare=1
; @mock: sqlite_bind_text=0
; @mock: sqlite_step=1
; @mock: sqlite_column_text=12346
; @mock: sqlite_finalize=0
; @mock: sqlite_close=0
; @mock: uuid_v7=1
; @mock: uuid_to_string=12347
; @mock: json_new_object=1
; @mock: json_set=0
; @mock: json_stringify=65536
;
; @test: r0=1 -> r0=65536
;
; @note: Composes: json_parse, sqlite_query, uuid_v7 (for session), json_build
; @note: Uses UUID v7 for time-ordered session tokens

.entry main

.section .data
    db_path:      .asciz "/tmp/auth.db"
    select_sql:   .asciz "SELECT id FROM users WHERE email = ?"
    session_sql:  .asciz "INSERT INTO sessions (id, user_id) VALUES (?, ?)"
    key_email:    .asciz "email"
    key_token:    .asciz "token"
    key_user_id:  .asciz "user_id"
    key_success:  .asciz "success"
    val_true:     .asciz "true"

.section .text

main:
    ; r0 = login request JSON body
    ; Parse login credentials
    ext.call r1, json_parse, r0
    beq r1, zero, error

    mov r2, key_email
    ext.call r3, json_get, r1, r2    ; r3 = email

    ext.call r0, json_free, r1

    ; Lookup user in database
    mov r0, db_path
    ext.call r10, sqlite_open, r0
    beq r10, zero, error

    mov r0, select_sql
    ext.call r11, sqlite_prepare, r10, r0
    beq r11, zero, error_close

    mov r0, 1
    ext.call r2, sqlite_bind_text, r11, r0, r3

    ext.call r0, sqlite_step, r11
    beq r0, zero, invalid_credentials

    ; Get user_id
    mov r0, 0
    ext.call r4, sqlite_column_text, r11, r0  ; r4 = user_id

    ext.call r0, sqlite_finalize, r11

    ; Generate session token (UUID v7)
    ext.call r5, uuid_v7
    ext.call r6, uuid_to_string, r5  ; r6 = session token

    ; Store session in database
    mov r0, session_sql
    ext.call r11, sqlite_prepare, r10, r0
    beq r11, zero, error_close

    mov r0, 1
    ext.call r2, sqlite_bind_text, r11, r0, r6  ; session_id
    mov r0, 2
    ext.call r2, sqlite_bind_text, r11, r0, r4  ; user_id

    ext.call r0, sqlite_step, r11
    ext.call r0, sqlite_finalize, r11
    ext.call r0, sqlite_close, r10

    ; Build success response
    ext.call r1, json_new_object

    mov r2, key_success
    mov r7, val_true
    ext.call r0, json_set, r1, r2, r7

    mov r2, key_token
    ext.call r0, json_set, r1, r2, r6

    mov r2, key_user_id
    ext.call r0, json_set, r1, r2, r4

    ext.call r0, json_stringify, r1
    ext.call r8, json_free, r1
    halt

invalid_credentials:
    ext.call r0, sqlite_finalize, r11

error_close:
    ext.call r0, sqlite_close, r10

error:
    mov r0, 0
    halt
