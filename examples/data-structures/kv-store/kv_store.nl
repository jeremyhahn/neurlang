; @name: In-Memory Key-Value Store
; @description: A simple key-value store with GET, SET, DELETE operations using hash-based lookup
; @category: data-structures/kv-store
; @difficulty: 4
;
; @prompt: implement an in-memory key-value store with get set delete
; @prompt: create a hash-based key-value storage system
; @prompt: write a kv store that supports set get and delete operations
; @prompt: implement a simple database with string keys and values
; @prompt: build a key-value store using djb2 hash function
; @prompt: create a storage system with {max_entries} slots for key-value pairs
; @prompt: demonstrate hash table implementation with linear storage
; @prompt: write a kv store with hash lookup and value pool allocation
;
; @server: true
;
; @note: Returns kv_count (3) - delete marks hash=0 but doesn't decrement count
; @note: KNOWN BUG: Memory read out of bounds - data section addressing issue
; @note: TODO: Fix memory bounds checking for data section access
; @note: Uses djb2 hash algorithm for key hashing
; @note: Maximum 16 entries supported
;
; In-Memory Key-Value Store
; =========================
; A simple key-value store with GET, SET, DELETE operations.
; Demonstrates: memory management, string operations, data structures
;
; Storage: Linear array of (key_hash, value_ptr, value_len) entries
; Max entries: 16

.entry main

.section .data

; Storage for 16 entries: each is 24 bytes (hash:8, ptr:8, len:8)
kv_entries:     .space 384, 0     ; 16 * 24 bytes
kv_count:       .word 0           ; Current number of entries
kv_max:         .word 16          ; Maximum entries

; Value storage pool
value_pool:     .space 4096, 0    ; Pool for storing values
pool_offset:    .word 0           ; Next free offset in pool

; Test keys and values
test_key1:      .asciz "name"
test_val1:      .asciz "Alice"
test_key2:      .asciz "age"
test_val2:      .asciz "30"
test_key3:      .asciz "city"
test_val3:      .asciz "NYC"

; Result buffer
result_buf:     .space 256, 0

; Log messages
log_set:        .asciz "SET: "
log_get:        .asciz "GET: "
log_del:        .asciz "DEL: "
log_found:      .asciz " = "
log_notfound:   .asciz " (not found)\n"
log_newline:    .asciz "\n"

.section .text

main:
    ; Test SET operations
    mov r0, test_key1
    mov r1, 4                     ; "name" length
    mov r2, test_val1
    mov r3, 5                     ; "Alice" length
    call kv_set

    mov r0, test_key2
    mov r1, 3                     ; "age" length
    mov r2, test_val2
    mov r3, 2                     ; "30" length
    call kv_set

    mov r0, test_key3
    mov r1, 4                     ; "city" length
    mov r2, test_val3
    mov r3, 3                     ; "NYC" length
    call kv_set

    ; Test GET operations
    mov r0, test_key1
    mov r1, 4
    call kv_get                   ; Should return "Alice"

    mov r0, test_key2
    mov r1, 3
    call kv_get                   ; Should return "30"

    ; Test DELETE
    mov r0, test_key2
    mov r1, 3
    call kv_delete

    ; Try to GET deleted key
    mov r0, test_key2
    mov r1, 3
    call kv_get                   ; Should return not found

    ; Return count
    mov r0, kv_count
    load.d r0, [r0]
    halt

; ============================================================
; KV_SET: Store a key-value pair
; Input: r0=key_ptr, r1=key_len, r2=val_ptr, r3=val_len
; ============================================================
kv_set:
    ; Save parameters
    mov r10, r0                   ; key_ptr
    mov r11, r1                   ; key_len
    mov r12, r2                   ; val_ptr
    mov r13, r3                   ; val_len

    ; Compute key hash
    mov r0, r10
    mov r1, r11
    call hash_string              ; r0 = hash

    mov r14, r0                   ; r14 = hash

    ; Check if key exists (update) or new entry
    mov r0, r14
    call find_by_hash             ; r0 = entry_ptr or 0

    bne r0, zero, update_entry

    ; New entry - check if space available
    mov r4, kv_count
    load.d r4, [r4]
    mov r5, kv_max
    load.d r5, [r5]
    bge r4, r5, set_full

    ; Allocate entry
    ; entry_ptr = kv_entries + (count * 24)
    mov r5, 24
    mul r4, r4, r5
    mov r0, kv_entries
    add r0, r0, r4                ; r0 = new entry ptr

    ; Increment count
    mov r4, kv_count
    load.d r5, [r4]
    addi r5, r5, 1
    store.d r5, [r4]

update_entry:
    ; r0 = entry ptr, store hash
    store.d r14, [r0]             ; entry.hash = hash

    ; Copy value to pool
    mov r4, pool_offset
    load.d r5, [r4]               ; r5 = current pool offset
    mov r6, value_pool
    add r6, r6, r5                ; r6 = dest ptr

    ; Store value ptr in entry
    store.d r6, [r0 + 8]          ; entry.val_ptr = pool + offset
    store.d r13, [r0 + 16]        ; entry.val_len = val_len

    ; Copy value
    mov r7, 0
copy_val:
    beq r7, r13, copy_val_done
    load.b r8, [r12]
    store.b r8, [r6]
    addi r12, r12, 1
    addi r6, r6, 1
    addi r7, r7, 1
    b copy_val

copy_val_done:
    ; Update pool offset
    add r5, r5, r13
    store.d r5, [r4]

    ret

set_full:
    ret

; ============================================================
; KV_GET: Retrieve value by key
; Input: r0=key_ptr, r1=key_len
; Output: r0=val_ptr (0 if not found), r1=val_len
; ============================================================
kv_get:
    ; Compute hash
    call hash_string              ; r0 = hash

    ; Find entry
    call find_by_hash             ; r0 = entry_ptr or 0

    beq r0, zero, get_not_found

    ; Return value ptr and len
    load.d r1, [r0 + 16]          ; r1 = val_len
    load.d r0, [r0 + 8]           ; r0 = val_ptr
    ret

get_not_found:
    mov r0, 0
    mov r1, 0
    ret

; ============================================================
; KV_DELETE: Remove entry by key
; Input: r0=key_ptr, r1=key_len
; ============================================================
kv_delete:
    ; Compute hash
    call hash_string              ; r0 = hash

    ; Find entry
    call find_by_hash             ; r0 = entry_ptr or 0

    beq r0, zero, del_not_found

    ; Mark as deleted (hash = 0)
    store.d zero, [r0]

    ret

del_not_found:
    ret

; ============================================================
; HASH_STRING: Compute simple hash of string
; Input: r0=str_ptr, r1=str_len
; Output: r0=hash
; ============================================================
hash_string:
    mov r2, 5381                  ; djb2 initial value
    mov r3, 0                     ; index

hash_loop:
    beq r3, r1, hash_done
    load.b r4, [r0]
    ; hash = hash * 33 + c
    mov r5, 33
    mul r2, r2, r5
    add r2, r2, r4
    addi r0, r0, 1
    addi r3, r3, 1
    b hash_loop

hash_done:
    mov r0, r2
    ret

; ============================================================
; FIND_BY_HASH: Find entry by hash
; Input: r0=hash
; Output: r0=entry_ptr (0 if not found)
; ============================================================
find_by_hash:
    mov r2, r0                    ; r2 = target hash
    mov r3, kv_count
    load.d r3, [r3]               ; r3 = count
    mov r4, 0                     ; index
    mov r5, kv_entries            ; r5 = base ptr

find_loop:
    beq r4, r3, find_not_found
    load.d r6, [r5]               ; r6 = entry.hash
    beq r6, r2, find_found
    addi r5, r5, 24               ; next entry
    addi r4, r4, 1
    b find_loop

find_found:
    mov r0, r5
    ret

find_not_found:
    mov r0, 0
    ret
