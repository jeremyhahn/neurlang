; @name: Inventory Service - Check Stock
; @description: Queries inventory database and returns stock level as JSON
; @category: application/inventory-service
; @difficulty: 2
;
; @prompt: check inventory stock level
; @prompt: get product stock
; @prompt: inventory query endpoint
; @prompt: check item availability
; @prompt: stock level api
; @prompt: inventory check service
; @prompt: get item quantity
; @prompt: product availability check
; @prompt: stock query endpoint
; @prompt: inventory lookup api
;
; Mock all extensions for testing
; @mock: sqlite_open=1
; @mock: sqlite_prepare=1
; @mock: sqlite_bind_text=0
; @mock: sqlite_step=1
; @mock: sqlite_column_text=12345
; @mock: sqlite_column_int=10
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
; @note: Simple inventory microservice pattern

.entry main

.section .data
    db_path:        .asciz "/tmp/inventory.db"
    select_sql:     .asciz "SELECT product_id, name, stock FROM products WHERE product_id = ?"
    key_product_id: .asciz "product_id"
    key_name:       .asciz "name"
    key_stock:      .asciz "stock"
    key_available:  .asciz "available"
    val_true:       .asciz "true"
    val_false:      .asciz "false"

.section .text

main:
    ; r0 = product_id string
    mov r15, r0

    ; Query inventory database
    mov r0, db_path
    ext.call r10, sqlite_open, r0
    beq r10, zero, error

    mov r0, select_sql
    ext.call r11, sqlite_prepare, r10, r0
    beq r11, zero, error_close

    mov r0, 1
    ext.call r2, sqlite_bind_text, r11, r0, r15

    ext.call r0, sqlite_step, r11
    beq r0, zero, not_found

    ; Get columns
    mov r0, 0
    ext.call r3, sqlite_column_text, r11, r0  ; product_id
    mov r0, 1
    ext.call r4, sqlite_column_text, r11, r0  ; name
    mov r0, 2
    ext.call r5, sqlite_column_int, r11, r0   ; stock

    ext.call r0, sqlite_finalize, r11
    ext.call r0, sqlite_close, r10

    ; Build response JSON
    ext.call r1, json_new_object

    mov r2, key_product_id
    ext.call r0, json_set, r1, r2, r3

    mov r2, key_name
    ext.call r0, json_set, r1, r2, r4

    mov r2, key_stock
    ext.call r0, json_set, r1, r2, r5

    ; Determine availability
    mov r2, key_available
    bgt r5, zero, has_stock
    mov r6, val_false
    b set_available
has_stock:
    mov r6, val_true
set_available:
    ext.call r0, json_set, r1, r2, r6

    ext.call r0, json_stringify, r1
    ext.call r7, json_free, r1
    halt

not_found:
    ext.call r0, sqlite_finalize, r11

error_close:
    ext.call r0, sqlite_close, r10

error:
    mov r0, 0
    halt
