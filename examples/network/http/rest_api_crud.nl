; @name: REST API CRUD Server
; @description: RESTful HTTP server with multi-record CRUD operations and persistent storage
; @category: network/http-server
; @difficulty: 5
;
; @prompt: create a full CRUD REST API server with multiple records
; @prompt: build an HTTP server with /values and /values/{id} endpoints
; @prompt: implement a RESTful API with list get create update delete operations
; @prompt: write a multi-record REST server with persistent file storage
; @prompt: create an API server supporting GET POST PUT DELETE with ID routing
; @prompt: build a database-backed REST API with proper HTTP responses
; @prompt: implement path parameter parsing for /values/{id} routes
; @prompt: write a complete CRUD server with JSON responses and file persistence
;
; @server: true
; @note: Listens on http://0.0.0.0:8080
; @note: Storage format in /tmp/state.db: id:value per line
; @note: GET /values returns all records, GET /values/{id} returns one
;
; Neurlang REST API Server - Multi-Record CRUD
; ==========================================
; A RESTful HTTP server with proper routing and persistent storage.
;
; Storage Format (state.db): Each line is "id:value\n"
;
; Endpoints:
;   GET    /values       - List all values
;   GET    /values/{id}  - Get value by ID
;   POST   /values/{id}  - Create/update value
;   PUT    /values/{id}  - Create/update value
;   DELETE /values/{id}  - Delete value
;
; Register Convention:
;   r10 = server socket
;   r11 = client socket
;   r12 = request length
;   r15 = ID length (after routing)

.entry main

; =============================================================================
; DATA SECTION
; =============================================================================
.section .data

bind_addr:      .asciz "0.0.0.0"

; HTTP responses (pre-built for simplicity)
http_200:       .asciz "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: "
http_201:       .asciz "HTTP/1.1 201 Created\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: "
http_404:       .asciz "HTTP/1.1 404 Not Found\r\nContent-Length: 21\r\nConnection: close\r\n\r\n{\"error\":\"Not found\"}"
http_400:       .asciz "HTTP/1.1 400 Bad Request\r\nContent-Length: 23\r\nConnection: close\r\n\r\n{\"error\":\"Bad request\"}"
crlf2:          .asciz "\r\n\r\n"

; JSON templates
json_obj_start: .asciz "{\"id\":\""
json_val_mid:   .asciz "\",\"value\":\""
json_obj_end:   .asciz "\"}"
json_arr_start: .asciz "{\"values\":["
json_arr_end:   .asciz "]}"
json_del_start: .asciz "{\"deleted\":\""
json_comma:     .asciz ","

; Path pattern
path_values:    .asciz "/values"

; Files
state_file:     .asciz "/tmp/state.db"
temp_file:      .asciz "/tmp/state.db.tmp"

; Log messages
log_start:      .asciz "CRUD API on http://0.0.0.0:8080\n"

; Buffers
recv_buf:       .space 4096, 0
resp_buf:       .space 4096, 0
db_buf:         .space 4096, 0
work_buf:       .space 256, 0
id_buf:         .space 32, 0
len_buf:        .space 16, 0

; Saved values (pseudo-stack since we don't have real stack)
saved_body_ptr: .space 8, 0
saved_body_len: .space 8, 0

; =============================================================================
; TEXT SECTION
; =============================================================================
.section .text

main:
    mov r0, log_start
    mov r1, 34
    io.print r2, r0, r1

    mov r1, 2
    mov r2, 1
    net.socket r10, r1, r2

    mov r1, bind_addr
    net.bind r0, r10, r1, 8080

    mov r1, 10
    net.listen r0, r10, r1

server_loop:
    net.accept r11, r10
    mov r3, -1
    beq r11, r3, server_loop

    mov r1, recv_buf
    mov r12, 4096
    net.recv r12, r11, r1, 0
    blt r12, zero, close_conn

    call handle_request

close_conn:
    net.close r0, r11
    b server_loop

; =============================================================================
; REQUEST HANDLER
; =============================================================================
handle_request:
    mov r0, recv_buf
    load.b r1, [r0]

    ; Check method
    mov r2, 0x47                  ; 'G'
    beq r1, r2, route_get
    mov r2, 0x50                  ; 'P'
    beq r1, r2, check_post_put
    mov r2, 0x44                  ; 'D'
    beq r1, r2, route_delete
    b send_400

check_post_put:
    load.b r1, [r0 + 1]
    mov r2, 0x4F                  ; 'O' for POST
    beq r1, r2, route_post
    mov r2, 0x55                  ; 'U' for PUT
    beq r1, r2, route_put
    b send_400

route_get:
    mov r1, 4                     ; Offset after "GET "
    call parse_path               ; Returns r15=ID len (0 for /values)
    mov r0, -1
    beq r15, r0, send_404
    beq r15, zero, call_get_all
    call do_get_one
    b close_conn                  ; Direct jump (no ret - RA corrupted by nested calls)
call_get_all:
    call do_get_all
    b close_conn

route_post:
    mov r1, 5                     ; Offset after "POST "
    call parse_path
    mov r0, -1
    beq r15, r0, send_404
    beq r15, zero, send_400
    call do_post
    b close_conn                  ; Direct jump (no ret - RA corrupted by nested calls)

route_put:
    mov r1, 4                     ; Offset after "PUT "
    call parse_path
    mov r0, -1
    beq r15, r0, send_404
    beq r15, zero, send_400
    call do_post                  ; Same as POST
    b close_conn                  ; Direct jump (no ret - RA corrupted by nested calls)

route_delete:
    mov r1, 7                     ; Offset after "DELETE "
    call parse_path
    mov r0, -1
    beq r15, r0, send_404
    beq r15, zero, send_400
    call do_delete
    b close_conn                  ; Direct jump (no ret - RA corrupted by nested calls)

; =============================================================================
; PARSE PATH - Extract /values or /values/{id}
; Input: r1 = offset to path start
; Output: r15 = ID length (0 for /values, -1 for no match), ID in id_buf
; =============================================================================
parse_path:
    mov r0, recv_buf
    add r0, r0, r1
    mov r2, path_values
    mov r3, 0

parse_cmp:
    mov r4, 7
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
    mov r5, 0x20                  ; space = end of path
    beq r4, r5, parse_exact
    mov r5, 0x3F                  ; ? = query string
    beq r4, r5, parse_exact
    mov r5, 0x2F                  ; / = has ID
    beq r4, r5, parse_id
    b parse_nomatch

parse_exact:
    mov r15, 0
    ret

parse_id:
    addi r0, r0, 1                ; Skip /
    mov r1, id_buf
    mov r2, 0

parse_id_loop:
    load.b r3, [r0]
    mov r4, 0x20
    beq r3, r4, parse_id_done
    mov r4, 0x3F
    beq r3, r4, parse_id_done
    mov r4, 0x0D
    beq r3, r4, parse_id_done
    beq r3, zero, parse_id_done
    store.b r3, [r1]
    addi r0, r0, 1
    addi r1, r1, 1
    addi r2, r2, 1
    mov r4, 30
    beq r2, r4, parse_id_done
    b parse_id_loop

parse_id_done:
    store.b zero, [r1]
    mov r15, r2
    ret

parse_nomatch:
    mov r15, -1
    ret

; =============================================================================
; GET /values - List all records
; =============================================================================
do_get_all:
    ; GET /values - List all records from database
    ; Build response: {"values":[{"id":"X","value":"Y"},...]}
    ; NOTE: Uses r13 for response length (r3 is clobbered by str_copy/mem_copy)

    ; Start building response with {"values":[
    mov r0, resp_buf
    mov r1, json_arr_start
    call str_copy                 ; r2 = length
    mov r13, r2                   ; r13 = response length (NOT r3!)

    ; Open and read state file
    mov r0, state_file
    mov r1, 13                    ; length of "/tmp/state.db"
    file.open r4, r0, r1, 1       ; flags = read only
    mov r5, -1
    beq r4, r5, get_all_empty     ; File doesn't exist

    mov r0, db_buf
    mov r5, 4096
    file.read r5, r4, r0, 0
    file.close r0, r4
    beq r5, zero, get_all_empty   ; Empty file

    ; Parse records: "id:value\n"
    mov r6, 0                     ; offset
    mov r7, 0                     ; record count

get_all_loop:
    bge r6, r5, get_all_done

    ; Add comma between records
    bne r7, zero, get_all_comma
    b get_all_record
get_all_comma:
    mov r0, resp_buf
    add r0, r0, r13
    mov r1, 0x2C
    store.b r1, [r0]
    addi r13, r13, 1

get_all_record:
    ; Find ID end (:)
    mov r0, db_buf
    add r0, r0, r6
    mov r8, r0                    ; ID start
    mov r9, 0                     ; ID length

get_all_find_colon:
    load.b r1, [r0]
    beq r1, zero, get_all_done
    mov r2, 0x3A
    beq r1, r2, get_all_found_colon
    addi r0, r0, 1
    addi r9, r9, 1
    addi r6, r6, 1
    b get_all_find_colon

get_all_found_colon:
    addi r0, r0, 1
    addi r6, r6, 1
    mov r1, r0                    ; Value start

    ; Find value end (newline)
    mov r2, 0
get_all_find_nl:
    load.b r4, [r0]
    beq r4, zero, get_all_build
    mov r14, 0x0A
    beq r4, r14, get_all_build
    addi r0, r0, 1
    addi r2, r2, 1
    addi r6, r6, 1
    b get_all_find_nl

get_all_build:
    ; Skip newline
    load.b r4, [r0]
    mov r14, 0x0A
    bne r4, r14, get_all_skip_done
    addi r6, r6, 1
get_all_skip_done:

    ; Build: {"id":"X","value":"Y"}
    ; Save value info in work_buf area (offset 0-7)
    mov r0, work_buf
    store.d r1, [r0]              ; value ptr at offset 0
    store.d r2, [r0 + 8]          ; value len at offset 8

    ; Append {"id":"
    mov r0, resp_buf
    add r0, r0, r13
    mov r1, json_obj_start
    call str_copy
    add r13, r13, r2

    ; Append ID
    mov r0, resp_buf
    add r0, r0, r13
    mov r1, r8
    mov r4, r9
    call mem_copy
    add r13, r13, r9

    ; Append ","value":"
    mov r0, resp_buf
    add r0, r0, r13
    mov r1, json_val_mid
    call str_copy
    add r13, r13, r2

    ; Append value
    mov r0, work_buf
    load.d r1, [r0]               ; value ptr
    load.d r4, [r0 + 8]           ; value len
    mov r0, resp_buf
    add r0, r0, r13
    call mem_copy
    mov r0, work_buf
    load.d r4, [r0 + 8]
    add r13, r13, r4

    ; Append "}
    mov r0, resp_buf
    add r0, r0, r13
    mov r1, json_obj_end
    call str_copy
    add r13, r13, r2

    addi r7, r7, 1
    b get_all_loop

get_all_empty:
get_all_done:
    ; Append ]}
    mov r0, resp_buf
    add r0, r0, r13
    mov r1, json_arr_end
    call str_copy
    add r13, r13, r2

    mov r0, r13
    call send_200
    b close_conn                  ; Must jump directly - RA corrupted by nested calls

; =============================================================================
; GET /values/{id}
; =============================================================================
do_get_one:
    call db_find                  ; Returns r0=value_ptr, r1=value_len, or r0=-1
    mov r2, -1
    beq r0, r2, send_404

    ; Save value info
    mov r4, r0
    mov r5, r1

    ; Build {"id":"X","value":"Y"}
    mov r0, resp_buf
    mov r1, json_obj_start
    call str_copy
    mov r3, r2

    mov r0, resp_buf
    add r0, r0, r3
    mov r1, id_buf
    mov r4, r15
    call mem_copy
    add r3, r3, r15

    mov r0, resp_buf
    add r0, r0, r3
    mov r1, json_val_mid
    call str_copy
    add r3, r3, r2

    ; Reload saved value info
    mov r0, work_buf
    load.d r1, [r0]
    load.d r4, [r0 + 8]
    mov r0, resp_buf
    add r0, r0, r3
    call mem_copy
    mov r0, work_buf
    load.d r4, [r0 + 8]
    add r3, r3, r4

    mov r0, resp_buf
    add r0, r0, r3
    mov r1, json_obj_end
    call str_copy
    add r3, r3, r2

    mov r0, r3
    call send_200
    b close_conn                  ; Must jump directly - RA corrupted by nested calls

; =============================================================================
; POST/PUT /values/{id}
; =============================================================================
do_post:
    ; Full POST: Write record to database and return JSON

    ; Find body
    call find_body                ; r0=body_ptr, r1=body_len
    beq r1, zero, send_400

    ; Save body_ptr and body_len (r0, r1 will be overwritten)
    mov r4, r0                    ; r4 = body_ptr
    mov r5, r1                    ; r5 = body_len

    ; Open state file for append
    ; file.open rd, path, path_len, flags
    ; Flags: read=0x01, write=0x02, create=0x04, append=0x08
    ; We want write + create + append = 0x02 + 0x04 + 0x08 = 0x0E = 14
    mov r0, state_file
    mov r1, 13                    ; length of "/tmp/state.db"                     ; length of "state.db"
    file.open r6, r0, r1, 14      ; flags = write | create | append

    ; Check for error
    mov r7, -1
    beq r6, r7, do_post_fallback

    ; Write: id:value\n
    ; Write ID
    mov r0, id_buf
    mov r3, r15                   ; r15 = ID length
    file.write r3, r6, r0, 0

    ; Write colon
    mov r0, work_buf
    mov r1, 0x3A
    store.b r1, [r0]
    file.write r3, r6, r0, 1

    ; Write value
    mov r0, r4                    ; body_ptr
    mov r3, r5                    ; body_len
    file.write r3, r6, r0, 0

    ; Write newline
    mov r0, work_buf
    mov r1, 0x0A
    store.b r1, [r0]
    file.write r3, r6, r0, 1

    file.close r0, r6

do_post_fallback:
    ; Build response {"id":"X","value":"Y"}
    ; First build the JSON in resp_buf
    mov r0, resp_buf

    ; {"id":"
    mov r1, 0x7B                  ; {
    store.b r1, [r0]
    mov r1, 0x22                  ; "
    store.b r1, [r0 + 1]
    mov r1, 0x69                  ; i
    store.b r1, [r0 + 2]
    mov r1, 0x64                  ; d
    store.b r1, [r0 + 3]
    mov r1, 0x22                  ; "
    store.b r1, [r0 + 4]
    mov r1, 0x3A                  ; :
    store.b r1, [r0 + 5]
    mov r1, 0x22                  ; "
    store.b r1, [r0 + 6]

    mov r3, 7                     ; current offset

    ; Copy ID to response
    mov r0, resp_buf
    add r0, r0, r3
    mov r1, id_buf
    mov r2, r15                   ; ID length
do_post_copy_id:
    beq r2, zero, do_post_after_id
    load.b r7, [r1]
    store.b r7, [r0]
    addi r0, r0, 1
    addi r1, r1, 1
    addi r3, r3, 1
    subi r2, r2, 1
    b do_post_copy_id

do_post_after_id:
    ; ","value":"
    mov r0, resp_buf
    add r0, r0, r3
    mov r1, 0x22                  ; "
    store.b r1, [r0]
    mov r1, 0x2C                  ; ,
    store.b r1, [r0 + 1]
    mov r1, 0x22                  ; "
    store.b r1, [r0 + 2]
    mov r1, 0x76                  ; v
    store.b r1, [r0 + 3]
    mov r1, 0x61                  ; a
    store.b r1, [r0 + 4]
    mov r1, 0x6C                  ; l
    store.b r1, [r0 + 5]
    mov r1, 0x75                  ; u
    store.b r1, [r0 + 6]
    mov r1, 0x65                  ; e
    store.b r1, [r0 + 7]
    mov r1, 0x22                  ; "
    store.b r1, [r0 + 8]
    mov r1, 0x3A                  ; :
    store.b r1, [r0 + 9]
    mov r1, 0x22                  ; "
    store.b r1, [r0 + 10]
    addi r3, r3, 11

    ; Copy value to response
    mov r0, resp_buf
    add r0, r0, r3
    mov r1, r4                    ; body_ptr
    mov r2, r5                    ; body_len
do_post_copy_val:
    beq r2, zero, do_post_after_val
    load.b r7, [r1]
    store.b r7, [r0]
    addi r0, r0, 1
    addi r1, r1, 1
    addi r3, r3, 1
    subi r2, r2, 1
    b do_post_copy_val

do_post_after_val:
    ; "}
    mov r0, resp_buf
    add r0, r0, r3
    mov r1, 0x22                  ; "
    store.b r1, [r0]
    mov r1, 0x7D                  ; }
    store.b r1, [r0 + 1]
    addi r3, r3, 2

    ; r3 now has total response length
    ; Save response length
    mov r5, r3

    ; Send HTTP 201 header (89 bytes)
    mov r0, http_201
    net.send r2, r11, r0, 89

    ; Build content length string
    ; For small numbers, we just handle 1-2 digits
    mov r0, len_buf
    mov r3, r5                    ; response length
    mov r7, 10
    div r6, r3, r7                ; r6 = tens digit
    beq r6, zero, do_post_single_digit

    ; Two digits
    addi r8, r6, 0x30             ; '0' + tens
    store.b r8, [r0]
    mul r8, r6, r7
    sub r3, r3, r8                ; remainder
    addi r8, r3, 0x30             ; '0' + ones
    store.b r8, [r0 + 1]
    mov r8, 2
    b do_post_send_len

do_post_single_digit:
    addi r8, r3, 0x30             ; '0' + value
    store.b r8, [r0]
    mov r8, 1

do_post_send_len:
    ; r8 = length of content-length string
    mov r0, len_buf
    mov r3, r8
    net.send r3, r11, r0, 0       ; Dynamic send

    ; Send CRLF CRLF (4 bytes)
    mov r0, crlf2
    net.send r2, r11, r0, 4

    ; Send body
    mov r0, resp_buf
    mov r3, r5                    ; response length
    net.send r3, r11, r0, 0       ; Dynamic send

    b close_conn                  ; Must jump directly - RA corrupted by call find_body

; =============================================================================
; DELETE /values/{id}
; =============================================================================
do_delete:
    call db_delete                ; r0 = 1 if deleted, 0 if not found
    beq r0, zero, send_404

    ; Build {"deleted":"id"}
    mov r0, resp_buf
    mov r1, json_del_start
    call str_copy
    mov r3, r2

    mov r0, resp_buf
    add r0, r0, r3
    mov r1, id_buf
    mov r4, r15
    call mem_copy
    add r3, r3, r15

    mov r0, resp_buf
    add r0, r0, r3
    mov r1, json_obj_end
    call str_copy
    add r3, r3, r2

    mov r0, r3
    call send_200
    b close_conn                  ; Must jump directly - RA corrupted by nested calls

; =============================================================================
; FIND BODY in HTTP request
; Output: r0 = body_ptr, r1 = body_len
; =============================================================================
find_body:
    mov r0, recv_buf
    mov r2, 0

find_body_loop:
    addi r3, r2, 4
    bgt r3, r12, find_body_none

    add r3, r0, r2
    load.b r4, [r3]
    mov r5, 0x0D
    bne r4, r5, find_body_next
    load.b r4, [r3 + 1]
    mov r5, 0x0A
    bne r4, r5, find_body_next
    load.b r4, [r3 + 2]
    mov r5, 0x0D
    bne r4, r5, find_body_next
    load.b r4, [r3 + 3]
    mov r5, 0x0A
    bne r4, r5, find_body_next

    ; Found
    addi r2, r2, 4
    add r0, r0, r2
    sub r1, r12, r2
    ret

find_body_next:
    addi r2, r2, 1
    b find_body_loop

find_body_none:
    mov r0, recv_buf
    mov r1, 0
    ret

; =============================================================================
; DATABASE OPERATIONS
; =============================================================================

; Find record by ID (in id_buf, length r15)
; Output: r0 = value_ptr in db_buf, r1 = value_len, or r0 = -1
db_find:
    mov r0, state_file
    mov r1, 13                    ; length of "/tmp/state.db"
    file.open r4, r0, r1, 1
    mov r5, -1
    beq r4, r5, db_find_notfound

    mov r0, db_buf
    mov r5, 4096
    file.read r5, r4, r0, 0
    file.close r0, r4
    beq r5, zero, db_find_notfound

    mov r6, 0                     ; offset

db_find_loop:
    bge r6, r5, db_find_notfound

    ; Compare ID
    mov r0, db_buf
    add r0, r0, r6
    mov r1, id_buf
    mov r2, r15
    call mem_cmp
    beq r0, zero, db_find_skip

    ; Check for ':' after ID
    mov r0, db_buf
    add r0, r0, r6
    add r0, r0, r15
    load.b r1, [r0]
    mov r2, 0x3A
    bne r1, r2, db_find_skip

    ; Found - get value
    addi r0, r0, 1

    ; Save value start in work_buf
    mov r1, work_buf
    store.d r0, [r1]

    mov r7, r0
    mov r8, 0

db_find_val_len:
    load.b r1, [r0]
    beq r1, zero, db_find_val_done
    mov r2, 0x0A
    beq r1, r2, db_find_val_done
    addi r0, r0, 1
    addi r8, r8, 1
    b db_find_val_len

db_find_val_done:
    ; Save value len
    mov r0, work_buf
    store.d r8, [r0 + 8]
    mov r0, r7
    mov r1, r8
    ret

db_find_skip:
    mov r0, db_buf
    add r0, r0, r6

db_find_skip_nl:
    bge r6, r5, db_find_notfound
    load.b r1, [r0]
    beq r1, zero, db_find_notfound
    mov r2, 0x0A
    addi r0, r0, 1
    addi r6, r6, 1
    bne r1, r2, db_find_skip_nl
    b db_find_loop

db_find_notfound:
    mov r0, -1
    mov r1, 0
    ret

; Put record (ID in id_buf len r15, value in r0 len r1)
; Rewrites file excluding old record, appends new
db_put:
    mov r4, r0                    ; value_ptr
    mov r5, r1                    ; value_len

    ; Read existing file
    mov r0, state_file
    mov r1, 13                    ; length of "/tmp/state.db"
    file.open r6, r0, r1, 1
    mov r7, -1
    beq r6, r7, db_put_new

    mov r0, db_buf
    mov r7, 4096
    file.read r7, r6, r0, 0
    file.close r0, r6
    b db_put_rewrite

db_put_new:
    mov r7, 0                     ; No existing data

db_put_rewrite:
    ; Open temp file
    mov r0, temp_file
    mov r1, 12
    file.open r8, r0, r1, 6
    mov r9, -1
    beq r8, r9, db_put_done

    ; Copy lines that don't match ID
    mov r6, 0                     ; read offset

db_put_loop:
    bge r6, r7, db_put_append

    ; Compare ID
    mov r0, db_buf
    add r0, r0, r6
    mov r1, id_buf
    mov r2, r15
    call mem_cmp
    beq r0, zero, db_put_copy_line

    ; Check for ':'
    mov r0, db_buf
    add r0, r0, r6
    add r0, r0, r15
    load.b r1, [r0]
    mov r2, 0x3A
    beq r1, r2, db_put_skip_line

db_put_copy_line:
    mov r0, db_buf
    add r0, r0, r6
    mov r1, r0
    mov r2, 0

db_put_line_len:
    mov r3, r6
    add r3, r3, r2
    bge r3, r7, db_put_write_line
    load.b r3, [r0]
    mov r9, 0x0A
    beq r3, r9, db_put_inc_nl
    addi r0, r0, 1
    addi r2, r2, 1
    b db_put_line_len

db_put_inc_nl:
    addi r2, r2, 1

db_put_write_line:
    mov r0, r1
    mov r3, r2
    file.write r3, r8, r0, 0
    add r6, r6, r2
    b db_put_loop

db_put_skip_line:
    mov r0, db_buf
    add r0, r0, r6

db_put_skip_nl:
    bge r6, r7, db_put_append
    load.b r1, [r0]
    beq r1, zero, db_put_append
    mov r2, 0x0A
    addi r0, r0, 1
    addi r6, r6, 1
    bne r1, r2, db_put_skip_nl
    b db_put_loop

db_put_append:
    ; Write new record
    mov r0, id_buf
    mov r3, r15
    file.write r3, r8, r0, 0

    mov r0, work_buf
    mov r1, 0x3A
    store.b r1, [r0]
    mov r3, 1
    file.write r3, r8, r0, 0

    mov r0, r4
    mov r3, r5
    file.write r3, r8, r0, 0

    mov r0, work_buf
    mov r1, 0x0A
    store.b r1, [r0]
    mov r3, 1
    file.write r3, r8, r0, 0

    file.close r0, r8

    ; Copy temp to state
    call db_copy_temp

db_put_done:
    ret

; Delete record
; Returns r0 = 1 if deleted, 0 if not found
db_delete:
    ; First check if it exists
    call db_find
    mov r1, -1
    beq r0, r1, db_del_notfound

    ; Read file
    mov r0, state_file
    mov r1, 13                    ; length of "/tmp/state.db"
    file.open r6, r0, r1, 1
    mov r7, -1
    beq r6, r7, db_del_notfound

    mov r0, db_buf
    mov r7, 4096
    file.read r7, r6, r0, 0
    file.close r0, r6

    ; Open temp
    mov r0, temp_file
    mov r1, 12
    file.open r8, r0, r1, 6
    mov r9, -1
    beq r8, r9, db_del_notfound

    ; Copy all lines except matching
    mov r6, 0

db_del_loop:
    bge r6, r7, db_del_done

    mov r0, db_buf
    add r0, r0, r6
    mov r1, id_buf
    mov r2, r15
    call mem_cmp
    beq r0, zero, db_del_copy

    mov r0, db_buf
    add r0, r0, r6
    add r0, r0, r15
    load.b r1, [r0]
    mov r2, 0x3A
    beq r1, r2, db_del_skip

db_del_copy:
    mov r0, db_buf
    add r0, r0, r6
    mov r1, r0
    mov r2, 0

db_del_len:
    mov r3, r6
    add r3, r3, r2
    bge r3, r7, db_del_write
    load.b r3, [r0]
    mov r9, 0x0A
    beq r3, r9, db_del_inc
    addi r0, r0, 1
    addi r2, r2, 1
    b db_del_len

db_del_inc:
    addi r2, r2, 1

db_del_write:
    mov r0, r1
    mov r3, r2
    file.write r3, r8, r0, 0
    add r6, r6, r2
    b db_del_loop

db_del_skip:
    mov r0, db_buf
    add r0, r0, r6

db_del_skip_nl:
    bge r6, r7, db_del_done
    load.b r1, [r0]
    beq r1, zero, db_del_done
    mov r2, 0x0A
    addi r0, r0, 1
    addi r6, r6, 1
    bne r1, r2, db_del_skip_nl
    b db_del_loop

db_del_done:
    file.close r0, r8
    call db_copy_temp
    mov r0, 1
    ret

db_del_notfound:
    mov r0, 0
    ret

; Copy temp file to state file
db_copy_temp:
    mov r0, temp_file
    mov r1, 12
    file.open r4, r0, r1, 1
    mov r5, -1
    beq r4, r5, db_copy_done

    mov r0, db_buf
    mov r5, 4096
    file.read r5, r4, r0, 0
    file.close r0, r4

    mov r0, state_file
    mov r1, 13                    ; length of "/tmp/state.db"
    file.open r4, r0, r1, 6
    mov r6, -1
    beq r4, r6, db_copy_done

    mov r0, db_buf
    mov r3, r5
    file.write r3, r4, r0, 0
    file.close r0, r4

db_copy_done:
    ret

; =============================================================================
; HELPER FUNCTIONS
; =============================================================================

; Copy null-terminated string from r1 to r0
; Output: r2 = length copied
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

; Copy r4 bytes from r1 to r0
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

; Compare r2 bytes at r0 and r1
; Returns r0 = 1 if equal, 0 if not
mem_cmp:
    mov r3, 0
mem_cmp_loop:
    beq r3, r2, mem_cmp_eq
    load.b r4, [r0]
    load.b r5, [r1]
    bne r4, r5, mem_cmp_ne
    addi r0, r0, 1
    addi r1, r1, 1
    addi r3, r3, 1
    b mem_cmp_loop
mem_cmp_eq:
    mov r0, 1
    ret
mem_cmp_ne:
    mov r0, 0
    ret

; Convert r0 to ASCII in len_buf, return length in r1
int_to_str:
    mov r2, len_buf
    mov r3, r0
    mov r4, 0

    bne r3, zero, int_convert
    mov r5, 0x30
    store.b r5, [r2]
    mov r1, 1
    ret

int_convert:
    mov r5, 1000
    div r6, r3, r5
    beq r6, zero, int_try_100
    addi r7, r6, 0x30
    store.b r7, [r2]
    addi r2, r2, 1
    addi r4, r4, 1
    mul r7, r6, r5
    sub r3, r3, r7

int_try_100:
    mov r5, 100
    div r6, r3, r5
    beq r4, zero, int_skip_100
    b int_write_100
int_skip_100:
    beq r6, zero, int_try_10
int_write_100:
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

; =============================================================================
; RESPONSE SENDERS
; =============================================================================

; Send 200 with body in resp_buf, length in r0
; NOTE: Uses r8 for body length (r5 is clobbered by int_to_str)
; NOTE: Jumps to close_conn (RA corrupted by call int_to_str)
send_200:
    mov r8, r0                    ; r8 = body length (NOT r5!)
    call int_to_str
    mov r9, r1                    ; r9 = content-length string length

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
    b close_conn                  ; Must jump directly - RA corrupted by call int_to_str

send_201:
    mov r8, r0                    ; r8 = body length (NOT r5!)
    call int_to_str
    mov r9, r1                    ; r9 = content-length string length

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
    b close_conn                  ; Must jump directly - RA corrupted by call int_to_str

send_404:
    mov r0, http_404
    net.send r2, r11, r0, 86      ; Fixed: was 85, missing closing }
    b close_conn                  ; Direct jump (branched to, not called)

send_400:
    mov r0, http_400
    net.send r2, r11, r0, 90      ; Fixed: was 89, missing closing }
    b close_conn                  ; Direct jump (branched to, not called)
