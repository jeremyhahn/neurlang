; @name: User Service - Create User
; @description: REST endpoint that creates user in database, returns JSON response
; @category: application/user-service
; @difficulty: 3
;
; @prompt: create user api endpoint
; @prompt: rest api create user
; @prompt: post user to database
; @prompt: user creation endpoint
; @prompt: add new user to system
; @prompt: user registration api
; @prompt: create user rest service
; @prompt: user service post endpoint
; @prompt: add user with json response
; @prompt: new user api
;
; Mock all extensions for testing
; @mock: json_parse=1
; @mock: json_get=12345
; @mock: json_free=0
; @mock: uuid_v4=1
; @mock: uuid_to_string=12346
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
; @test: r0=1 -> r0=65536
;
; @note: Composes: json_parse, sqlite_insert, json_build, uuid_v4
; @note: Demonstrates extension composition for CRUD operation

.entry main

.section .data
    db_path:      .asciz "/tmp/users.db"
    insert_sql:   .asciz "INSERT INTO users (id, name, email) VALUES (?, ?, ?)"
    key_id:       .asciz "id"
    key_name:     .asciz "name"
    key_email:    .asciz "email"
    key_status:   .asciz "status"
    status_ok:    .asciz "created"

.section .text

main:
    ; r0 = request body JSON pointer (input)
    ; Parse incoming JSON
    ext.call r1, json_parse, r0      ; r1 = json handle
    beq r1, zero, error

    ; Extract name field
    mov r2, key_name
    ext.call r3, json_get, r1, r2    ; r3 = name value

    ; Extract email field
    mov r2, key_email
    ext.call r4, json_get, r1, r2    ; r4 = email value

    ; Free input json
    ext.call r0, json_free, r1

    ; Generate UUID for new user
    ext.call r5, uuid_v4
    ext.call r6, uuid_to_string, r5  ; r6 = uuid string

    ; Open database
    mov r0, db_path
    ext.call r10, sqlite_open, r0
    beq r10, zero, error

    ; Prepare insert statement
    mov r0, insert_sql
    ext.call r11, sqlite_prepare, r10, r0
    beq r11, zero, error_close

    ; Bind parameters: id, name, email
    mov r0, 1
    ext.call r2, sqlite_bind_text, r11, r0, r6
    mov r0, 2
    ext.call r2, sqlite_bind_text, r11, r0, r3
    mov r0, 3
    ext.call r2, sqlite_bind_text, r11, r0, r4

    ; Execute and finalize
    ext.call r0, sqlite_step, r11
    ext.call r0, sqlite_finalize, r11
    ext.call r0, sqlite_close, r10

    ; Build response JSON
    ext.call r1, json_new_object

    mov r2, key_id
    ext.call r0, json_set, r1, r2, r6

    mov r2, key_name
    ext.call r0, json_set, r1, r2, r3

    mov r2, key_email
    ext.call r0, json_set, r1, r2, r4

    mov r2, key_status
    mov r7, status_ok
    ext.call r0, json_set, r1, r2, r7

    ; Stringify response
    ext.call r0, json_stringify, r1
    ext.call r8, json_free, r1
    halt

error_close:
    ext.call r0, sqlite_close, r10

error:
    mov r0, 0
    halt
