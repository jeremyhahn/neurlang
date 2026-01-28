; @name: Order Service - Create Order
; @description: Creates order by storing in database, returns JSON response
; @category: application/order-service
; @difficulty: 3
;
; @prompt: create order api
; @prompt: order service create endpoint
; @prompt: place order api
; @prompt: new order endpoint
; @prompt: order creation api
; @prompt: place order rest api
; @prompt: create order microservice
; @prompt: order api
; @prompt: submit order endpoint
; @prompt: order placement api
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
; @mock: sqlite_bind_int=0
; @mock: sqlite_step=0
; @mock: sqlite_finalize=0
; @mock: sqlite_close=0
; @mock: json_new_object=1
; @mock: json_set=0
; @mock: json_stringify=65536
;
; @test: r0=1 -> r0=65536
;
; @note: Composes: json_parse, sqlite_insert, uuid_v4, json_build
; @note: Demonstrates order creation pattern

.entry main

.section .data
    db_path:        .asciz "/tmp/orders.db"
    insert_sql:     .asciz "INSERT INTO orders (id, user_id, product_id, quantity) VALUES (?, ?, ?, ?)"
    key_id:         .asciz "id"
    key_user_id:    .asciz "user_id"
    key_product_id: .asciz "product_id"
    key_quantity:   .asciz "quantity"
    key_status:     .asciz "status"
    status_created: .asciz "created"

.section .text

main:
    ; r0 = request body JSON
    ; Parse order request
    ext.call r1, json_parse, r0
    beq r1, zero, error

    mov r2, key_user_id
    ext.call r3, json_get, r1, r2    ; r3 = user_id

    mov r2, key_product_id
    ext.call r4, json_get, r1, r2    ; r4 = product_id

    mov r2, key_quantity
    ext.call r5, json_get, r1, r2    ; r5 = quantity

    ext.call r0, json_free, r1

    ; Generate order ID
    ext.call r6, uuid_v4
    ext.call r7, uuid_to_string, r6  ; r7 = order_id

    ; Store order in database
    mov r0, db_path
    ext.call r10, sqlite_open, r0
    beq r10, zero, error

    mov r0, insert_sql
    ext.call r11, sqlite_prepare, r10, r0
    beq r11, zero, error_close

    mov r0, 1
    ext.call r2, sqlite_bind_text, r11, r0, r7   ; order_id
    mov r0, 2
    ext.call r2, sqlite_bind_text, r11, r0, r3   ; user_id
    mov r0, 3
    ext.call r2, sqlite_bind_text, r11, r0, r4   ; product_id
    mov r0, 4
    ext.call r2, sqlite_bind_int, r11, r0, r5    ; quantity

    ext.call r0, sqlite_step, r11
    ext.call r0, sqlite_finalize, r11
    ext.call r0, sqlite_close, r10

    ; Build response JSON
    ext.call r1, json_new_object

    mov r2, key_id
    ext.call r0, json_set, r1, r2, r7

    mov r2, key_user_id
    ext.call r0, json_set, r1, r2, r3

    mov r2, key_product_id
    ext.call r0, json_set, r1, r2, r4

    mov r2, key_quantity
    ext.call r0, json_set, r1, r2, r5

    mov r2, key_status
    mov r8, status_created
    ext.call r0, json_set, r1, r2, r8

    ext.call r0, json_stringify, r1
    ext.call r9, json_free, r1
    halt

error_close:
    ext.call r0, sqlite_close, r10

error:
    mov r0, 0
    halt
