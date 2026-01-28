; @name: Product Search Service
; @description: Searches products by name pattern
; @category: application/search-service
; @difficulty: 3
;
; @prompt: product search api
; @prompt: search products endpoint
; @prompt: product catalog search
; @prompt: find products by query
; @prompt: search items service
; @prompt: product lookup api
; @prompt: search inventory
; @prompt: product search rest api
; @prompt: search products database
; @prompt: product query endpoint
;
; Mock all extensions for testing
; @mock: sqlite_open=1
; @mock: sqlite_prepare=1
; @mock: sqlite_bind_text=0
; @mock: sqlite_step=0
; @mock: sqlite_column_text=12345
; @mock: sqlite_column_int=100
; @mock: sqlite_finalize=0
; @mock: sqlite_close=0
; @mock: json_new_array=1
; @mock: json_new_object=1
; @mock: json_set=0
; @mock: json_array_push=0
; @mock: json_stringify=65536
; @mock: json_free=0
;
; @test: r0=1 -> r0=65536
;
; @note: Composes: sqlite_query (with LIKE), json_build with array
; @note: Returns search results as JSON

.entry main

.section .data
    db_path:     .asciz "/tmp/products.db"
    search_sql:  .asciz "SELECT id, name, price FROM products WHERE name LIKE ? LIMIT 10"
    key_results: .asciz "results"
    key_query:   .asciz "query"
    key_id:      .asciz "id"
    key_name:    .asciz "name"
    key_price:   .asciz "price"

.section .text

main:
    ; r0 = search query (already formatted with % wildcards)
    mov r14, r0                      ; query

    ; Open database
    mov r0, db_path
    ext.call r10, sqlite_open, r0
    beq r10, zero, error

    ; Execute search query
    mov r0, search_sql
    ext.call r11, sqlite_prepare, r10, r0
    beq r11, zero, error_close

    ; Bind search pattern
    mov r0, 1
    ext.call r2, sqlite_bind_text, r11, r0, r14

    ; Build results array
    ext.call r12, json_new_array

results_loop:
    ext.call r0, sqlite_step, r11
    beq r0, zero, results_done

    ; Get row values
    mov r0, 0
    ext.call r3, sqlite_column_text, r11, r0  ; id
    mov r0, 1
    ext.call r4, sqlite_column_text, r11, r0  ; name
    mov r0, 2
    ext.call r5, sqlite_column_int, r11, r0   ; price

    ; Build result object
    ext.call r1, json_new_object

    mov r2, key_id
    ext.call r0, json_set, r1, r2, r3
    mov r2, key_name
    ext.call r0, json_set, r1, r2, r4
    mov r2, key_price
    ext.call r0, json_set, r1, r2, r5

    ext.call r0, json_array_push, r12, r1
    ext.call r0, json_free, r1

    b results_loop

results_done:
    ext.call r0, sqlite_finalize, r11
    ext.call r0, sqlite_close, r10

    ; Build response JSON
    ext.call r1, json_new_object

    mov r2, key_query
    ext.call r0, json_set, r1, r2, r14

    mov r2, key_results
    ext.call r0, json_set, r1, r2, r12

    ext.call r0, json_stringify, r1
    ext.call r6, json_free, r12
    ext.call r6, json_free, r1
    halt

error_close:
    ext.call r0, sqlite_close, r10

error:
    mov r0, 0
    halt
