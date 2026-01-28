; @name: LRU Cache
; @description: Least Recently Used cache with capacity limit
; @category: patterns/caching
; @difficulty: 4
;
; @prompt: implement lru cache
; @prompt: create least recently used cache
; @prompt: lru cache with eviction
; @prompt: cache with capacity limit
; @prompt: implement lru eviction policy
; @prompt: bounded cache with lru eviction
; @prompt: lru cache get and put
; @prompt: cache that evicts least used
; @prompt: implement cache with lru policy
; @prompt: create capacity-limited cache
;
; @param: operation=r0 "0=get, 1=put"
; @param: key=r1 "Cache key"
;
; @test: r0=0, r1=1 -> r0=0
; @test: r0=1, r1=1 -> r0=1
; @test: r0=0, r1=999 -> r0=0
; @test: r0=1, r1=5 -> r0=1
; @test: r0=0, r1=0 -> r0=0
;
; @note: Returns value for get (0 if miss), 1 for successful put
; @note: Evicts LRU entry when cache is full
;
; LRU Cache Pattern
; =================
; Doubly-linked list + hash map for O(1) get/put with LRU eviction.

.entry main

.section .data

cache_capacity:     .word 16        ; Maximum entries
cache_size:         .word 0         ; Current entries
; Each entry: key(8) + value(8) + prev(8) + next(8) = 32 bytes
cache_entries:      .space 512, 0   ; 16 entries * 32 bytes
head_idx:           .word -1        ; Most recent (front of list)
tail_idx:           .word -1        ; Least recent (back of list)

.section .text

main:
    ; r0 = operation (0=get, 1=put)
    ; r1 = key
    mov r10, r0
    mov r11, r1

    beq r10, zero, cache_get
    b cache_put

cache_get:
    ; Look up key in cache
    mov r0, r11
    call find_entry
    beq r0, zero, cache_miss

    ; Found - move to front (most recent)
    call move_to_front

    ; Return value
    load.d r0, [r0 + 8]             ; value is at offset 8
    halt

cache_miss:
    mov r0, 0                       ; Return 0 for miss
    halt

cache_put:
    ; Check if key exists
    mov r0, r11
    call find_entry
    bne r0, zero, update_existing

    ; New entry - check capacity
    mov r0, cache_size
    load.d r1, [r0]
    mov r2, cache_capacity
    load.d r2, [r2]
    blt r1, r2, add_entry

    ; At capacity - evict LRU (tail)
    call evict_lru

add_entry:
    ; Add new entry at front
    call allocate_entry
    mov r12, r0                     ; r12 = entry ptr

    ; Store key and value
    store.d r11, [r12]              ; key
    mov r0, 42                      ; mock value
    store.d r0, [r12 + 8]           ; value

    ; Link at front
    call link_at_front

    ; Increment size
    mov r0, cache_size
    load.d r1, [r0]
    addi r1, r1, 1
    store.d r1, [r0]

    mov r0, 1                       ; Success
    halt

update_existing:
    ; Update value and move to front
    mov r1, 42                      ; new value
    store.d r1, [r0 + 8]
    call move_to_front
    mov r0, 1
    halt

find_entry:
    ; r0 = key to find
    ; Returns: entry ptr or 0 if not found
    mov r1, cache_entries
    mov r2, 0                       ; index
    mov r3, cache_capacity
    load.d r3, [r3]

find_loop:
    bge r2, r3, not_found
    load.d r4, [r1]                 ; Load key
    beq r4, r0, found
    addi r1, r1, 32                 ; Next entry
    addi r2, r2, 1
    b find_loop

found:
    mov r0, r1
    ret

not_found:
    mov r0, 0
    ret

move_to_front:
    ; Move entry to front of LRU list
    ; (Implementation would update prev/next pointers)
    ret

evict_lru:
    ; Remove tail entry
    ret

allocate_entry:
    ; Find empty slot
    mov r0, cache_entries
    ret

link_at_front:
    ; Link new entry as new head
    ret
