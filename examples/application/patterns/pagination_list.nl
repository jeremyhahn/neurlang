; @name: Paginated List Query
; @description: Queries database with pagination, returns JSON with items and metadata
; @category: application/pattern
; @difficulty: 3
;
; @prompt: paginated list endpoint
; @prompt: list with pagination
; @prompt: paginated query api
; @prompt: get items with page and limit
; @prompt: pagination pattern
; @prompt: list resources with paging
; @prompt: paginated database query
; @prompt: offset limit query
; @prompt: page through results
; @prompt: paginated list response
;
; Mock all extensions for testing
; @mock: sqlite_open=1
; @mock: sqlite_prepare=1
; @mock: sqlite_bind_int=0
; @mock: sqlite_step=0
; @mock: sqlite_column_text=12345
; @mock: sqlite_finalize=0
; @mock: sqlite_close=0
; @mock: json_new_array=1
; @mock: json_new_object=1
; @mock: json_set=0
; @mock: json_array_push=0
; @mock: json_stringify=65536
; @mock: json_free=0
;
; @test: r0=1, r1=10 -> r0=65536
;
; @note: Composes: sqlite_query, json_build with array
; @note: Standard pagination pattern: page, limit

.entry main

.section .data
    db_path:      .asciz "/tmp/app.db"
    select_sql:   .asciz "SELECT id, name FROM users LIMIT ? OFFSET ?"
    key_items:    .asciz "items"
    key_page:     .asciz "page"
    key_limit:    .asciz "limit"
    key_id:       .asciz "id"
    key_name:     .asciz "name"

.section .text

main:
    ; r0 = page number, r1 = limit
    mov r14, r0                      ; page
    mov r15, r1                      ; limit

    ; Calculate offset: (page - 1) * limit
    alui.sub r2, r14, 1
    muldiv.mul r2, r2, r15           ; r2 = offset

    ; Open database
    mov r0, db_path
    ext.call r10, sqlite_open, r0
    beq r10, zero, error

    ; Query page of items
    mov r0, select_sql
    ext.call r11, sqlite_prepare, r10, r0
    beq r11, zero, error_close

    mov r0, 1
    ext.call r3, sqlite_bind_int, r11, r0, r15  ; limit
    mov r0, 2
    ext.call r3, sqlite_bind_int, r11, r0, r2   ; offset

    ; Build items array
    ext.call r12, json_new_array

items_loop:
    ext.call r0, sqlite_step, r11
    beq r0, zero, items_done

    ; Get row values
    mov r0, 0
    ext.call r4, sqlite_column_text, r11, r0  ; id
    mov r0, 1
    ext.call r5, sqlite_column_text, r11, r0  ; name

    ; Build item object
    ext.call r1, json_new_object

    mov r2, key_id
    ext.call r0, json_set, r1, r2, r4
    mov r2, key_name
    ext.call r0, json_set, r1, r2, r5

    ; Push to array
    ext.call r0, json_array_push, r12, r1
    ext.call r0, json_free, r1

    b items_loop

items_done:
    ext.call r0, sqlite_finalize, r11
    ext.call r0, sqlite_close, r10

    ; Build response JSON
    ext.call r1, json_new_object

    mov r2, key_items
    ext.call r0, json_set, r1, r2, r12

    mov r2, key_page
    ext.call r0, json_set, r1, r2, r14

    mov r2, key_limit
    ext.call r0, json_set, r1, r2, r15

    ext.call r0, json_stringify, r1
    ext.call r3, json_free, r12
    ext.call r3, json_free, r1
    halt

error_close:
    ext.call r0, sqlite_close, r10

error:
    mov r0, 0
    halt
