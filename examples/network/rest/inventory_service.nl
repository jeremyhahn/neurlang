; @name: Inventory Service REST API
; @description: Inventory management REST API demonstrating HTTP server patterns
; @category: network/microservice
; @difficulty: 5
;
; @prompt: create an inventory service REST API on port 8081
; @prompt: implement inventory CRUD endpoints
; @prompt: build inventory management API with stock tracking
; @prompt: create REST API for item management
; @prompt: implement /items endpoints
; @prompt: build inventory microservice with low stock alerts
; @prompt: create RESTful inventory service with JSON responses
; @prompt: implement item API with list get create update delete
; @prompt: create GET /items endpoint for inventory service
; @prompt: implement PUT /items/{id}/stock endpoint to update inventory quantity
;
; @server: true
; @note: Listens on http://127.0.0.1:8081
; @note: Endpoints: GET /items, GET /items/{id}, POST /items, PUT /items/{id}, DELETE /items/{id}
;
; Network mocks for testing GET /items
; @net_mock: socket=3
; @net_mock: bind=0
; @net_mock: listen=0
; @net_mock: accept=5,-1
; @net_mock: recv="GET /items HTTP/1.1\r\nHost: localhost\r\n\r\n",0
; @net_mock: send=100
; @net_mock: close=0
;
; @test: -> r0=0
;
; Inventory Service REST API
; ==========================
; Demonstrates inventory management patterns:
; - HTTP server on different port (8081)
; - Same routing patterns as user service
; - Item-specific JSON responses
;
; Register Convention:
;   r10 = server socket
;   r11 = client socket
;   r12 = request length
;   r15 = current ID length

.entry main

.section .data

; Network
bind_addr:      .asciz "127.0.0.1"

; HTTP responses
http_200:       .asciz "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: "
http_201:       .asciz "HTTP/1.1 201 Created\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: "
http_204:       .asciz "HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n"
http_400:       .asciz "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nContent-Length: 24\r\n\r\n{\"error\":\"Bad Request\"}"
http_404:       .asciz "HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\nContent-Length: 22\r\n\r\n{\"error\":\"Not Found\"}"
crlf2:          .asciz "\r\n\r\n"

; Paths
path_items:     .asciz "/items"

; JSON templates
json_start:     .asciz "{\"items\":["
json_end:       .asciz "]}"
json_item:      .asciz "{\"id\":\""
json_name:      .asciz "\",\"name\":\""
json_qty:       .asciz "\",\"quantity\":"
json_close:     .asciz "}"

; Log messages
log_start:      .asciz "Inventory Service on http://127.0.0.1:8081\n"

; Buffers
recv_buf:       .space 4096, 0
resp_buf:       .space 4096, 0
id_buf:         .space 64, 0
len_buf:        .space 16, 0

.section .text

main:
    ; Log startup
    mov r0, log_start
    mov r1, 45
    io.print r2, r0, r1

    ; Create TCP socket
    mov r1, 2                    ; AF_INET
    mov r2, 1                    ; SOCK_STREAM
    net.socket r10, r1, r2

    ; Bind to port 8081 (different from user service)
    mov r1, bind_addr
    net.bind r0, r10, r1, 8081

    ; Listen
    mov r1, 128
    net.listen r0, r10, r1

accept_loop:
    net.accept r11, r10
    mov r1, -1
    beq r11, r1, accept_loop

    mov r1, recv_buf
    mov r12, 4096
    net.recv r12, r11, r1, 0
    blt r12, zero, close_client

    call route_request

close_client:
    net.close r0, r11
    b accept_loop

; ============================================================
; REQUEST ROUTING
; ============================================================
route_request:
    mov r0, recv_buf
    load.b r1, [r0]

    mov r2, 0x47                 ; 'G'
    beq r1, r2, route_get
    mov r2, 0x50                 ; 'P'
    beq r1, r2, check_post_put
    mov r2, 0x44                 ; 'D'
    beq r1, r2, route_delete
    b send_400

check_post_put:
    load.b r1, [r0 + 1]
    mov r2, 0x4F                 ; 'O'
    beq r1, r2, route_post
    mov r2, 0x55                 ; 'U'
    beq r1, r2, route_put
    b send_400

route_get:
    mov r1, 4
    call parse_path
    mov r0, -1
    beq r15, r0, send_404
    beq r15, zero, do_get_all
    b do_get_one

route_post:
    mov r1, 5
    call parse_path
    b do_create_item

route_put:
    mov r1, 4
    call parse_path
    beq r15, zero, send_400
    b do_update_item

route_delete:
    mov r1, 7
    call parse_path
    beq r15, zero, send_400
    b do_delete_item

; ============================================================
; PATH PARSING
; ============================================================
parse_path:
    mov r0, recv_buf
    add r0, r0, r1

    mov r2, path_items
    mov r3, 0

parse_cmp:
    mov r4, 6                    ; length of "/items"
    beq r3, r4, parse_matched
    load.b r4, [r0]
    load.b r5, [r2]
    bne r4, r5, parse_nomatch
    addi r0, r0, 1
    addi r2, r2, 1
    addi r3, r3, 1
    b parse_cmp

parse_matched:
    load.b r4, [r0]
    mov r5, 0x20
    beq r4, r5, parse_exact
    mov r5, 0x2F
    beq r4, r5, parse_id
    b parse_nomatch

parse_exact:
    mov r15, 0
    ret

parse_id:
    addi r0, r0, 1
    mov r1, id_buf
    mov r2, 0

parse_id_loop:
    load.b r3, [r0]
    mov r4, 0x20
    beq r3, r4, parse_id_done
    mov r4, 0x3F
    beq r3, r4, parse_id_done
    mov r4, 0x2F                 ; stop at /stock
    beq r3, r4, parse_id_done
    beq r3, zero, parse_id_done
    store.b r3, [r1]
    addi r0, r0, 1
    addi r1, r1, 1
    addi r2, r2, 1
    b parse_id_loop

parse_id_done:
    store.b zero, [r1]
    mov r15, r2
    ret

parse_nomatch:
    mov r15, -1
    ret

; ============================================================
; GET /items - List all items
; ============================================================
do_get_all:
    mov r0, resp_buf
    mov r1, json_start
    call str_copy
    mov r3, r2

    ; Empty array for demo
    mov r0, resp_buf
    add r0, r0, r3
    mov r1, json_end
    call str_copy
    add r3, r3, r2

    mov r0, r3
    call send_200
    ret

; ============================================================
; GET /items/{id} - Get single item
; ============================================================
do_get_one:
    mov r0, resp_buf
    mov r1, json_item
    call str_copy
    mov r3, r2

    ; Append ID
    mov r0, resp_buf
    add r0, r0, r3
    mov r1, id_buf
    mov r4, r15
    call mem_copy
    add r3, r3, r15

    ; Append quantity (demo: 100)
    mov r0, resp_buf
    add r0, r0, r3
    mov r1, json_qty
    call str_copy
    add r3, r3, r2

    ; Add "100}"
    mov r0, resp_buf
    add r0, r0, r3
    mov r2, 0x31                 ; '1'
    store.b r2, [r0]
    mov r2, 0x30                 ; '0'
    store.b r2, [r0 + 1]
    store.b r2, [r0 + 2]
    mov r2, 0x7D                 ; '}'
    store.b r2, [r0 + 3]
    addi r3, r3, 4

    mov r0, r3
    call send_200
    ret

; ============================================================
; POST /items - Create item
; ============================================================
do_create_item:
    mov r0, resp_buf
    mov r1, json_item
    call str_copy
    mov r3, r2

    ; Placeholder ID "new"
    mov r0, resp_buf
    add r0, r0, r3
    mov r2, 0x6E
    store.b r2, [r0]
    mov r2, 0x65
    store.b r2, [r0 + 1]
    mov r2, 0x77
    store.b r2, [r0 + 2]
    addi r3, r3, 3

    mov r0, resp_buf
    add r0, r0, r3
    mov r1, json_qty
    call str_copy
    add r3, r3, r2

    mov r0, resp_buf
    add r0, r0, r3
    mov r2, 0x30                 ; '0'
    store.b r2, [r0]
    mov r2, 0x7D
    store.b r2, [r0 + 1]
    addi r3, r3, 2

    mov r0, r3
    call send_201
    ret

; ============================================================
; PUT /items/{id} - Update item
; ============================================================
do_update_item:
    b do_get_one

; ============================================================
; DELETE /items/{id}
; ============================================================
do_delete_item:
    b send_204

; ============================================================
; HELPERS
; ============================================================
str_copy:
    mov r3, 0
str_copy_loop:
    load.b r4, [r1]
    beq r4, zero, str_copy_done
    store.b r4, [r0]
    addi r0, r0, 1
    addi r1, r1, 1
    addi r3, r3, 1
    b str_copy_loop
str_copy_done:
    mov r2, r3
    ret

mem_copy:
    mov r2, 0
mem_copy_loop:
    beq r2, r4, mem_copy_done
    load.b r3, [r1]
    store.b r3, [r0]
    addi r0, r0, 1
    addi r1, r1, 1
    addi r2, r2, 1
    b mem_copy_loop
mem_copy_done:
    ret

int_to_str:
    mov r2, len_buf
    mov r3, r0
    mov r4, 0
    bne r3, zero, int_conv
    mov r5, 0x30
    store.b r5, [r2]
    mov r1, 1
    ret
int_conv:
    mov r5, 100
    div r6, r3, r5
    beq r6, zero, int_try_10
    addi r7, r6, 0x30
    store.b r7, [r2]
    addi r2, r2, 1
    addi r4, r4, 1
    mul r7, r6, r5
    sub r3, r3, r7
int_try_10:
    mov r5, 10
    div r6, r3, r5
    beq r4, zero, int_skip_10
    b int_write_10
int_skip_10:
    beq r6, zero, int_write_1
int_write_10:
    addi r7, r6, 0x30
    store.b r7, [r2]
    addi r2, r2, 1
    addi r4, r4, 1
    mul r7, r6, r5
    sub r3, r3, r7
int_write_1:
    addi r7, r3, 0x30
    store.b r7, [r2]
    addi r4, r4, 1
    mov r1, r4
    ret

; ============================================================
; RESPONSE SENDERS
; ============================================================
send_200:
    mov r8, r0
    call int_to_str
    mov r9, r1
    mov r0, http_200
    net.send r2, r11, r0, 84
    mov r0, len_buf
    mov r3, r9
    net.send r3, r11, r0, 0
    mov r0, crlf2
    net.send r2, r11, r0, 4
    mov r0, resp_buf
    mov r3, r8
    net.send r3, r11, r0, 0
    b close_client

send_201:
    mov r8, r0
    call int_to_str
    mov r9, r1
    mov r0, http_201
    net.send r2, r11, r0, 89
    mov r0, len_buf
    mov r3, r9
    net.send r3, r11, r0, 0
    mov r0, crlf2
    net.send r2, r11, r0, 4
    mov r0, resp_buf
    mov r3, r8
    net.send r3, r11, r0, 0
    b close_client

send_204:
    mov r0, http_204
    net.send r1, r11, r0, 47
    b close_client

send_400:
    mov r0, http_400
    net.send r1, r11, r0, 106
    b close_client

send_404:
    mov r0, http_404
    net.send r1, r11, r0, 104
    b close_client
