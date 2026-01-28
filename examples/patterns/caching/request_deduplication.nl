; @name: Request Deduplication
; @description: Deduplicate concurrent requests for same resource
; @category: patterns/caching
; @difficulty: 4
;
; @prompt: deduplicate concurrent requests
; @prompt: coalesce duplicate requests
; @prompt: single flight pattern
; @prompt: prevent thundering herd
; @prompt: dedupe parallel requests
; @prompt: request coalescing pattern
; @prompt: avoid duplicate concurrent fetches
; @prompt: single inflight request per key
; @prompt: request deduplication cache
; @prompt: collapse concurrent requests
;
; @param: key=r0 "Request key"
; @param: in_flight=r1 "Is request already in flight"
;
; @test: r0=1, r1=0 -> r0=1
; @test: r0=1, r1=1 -> r0=2
; @test: r0=2, r1=0 -> r0=1
;
; @note: Returns 1 if new request, 2 if waiting on existing
; @note: Prevents duplicate work for concurrent requests
;
; Request Deduplication Pattern
; =============================
; If request for key is in flight, wait for it instead of duplicating.

.entry main

.section .data

inflight_key:       .word 0         ; Currently in-flight request key
inflight_result:    .word 0         ; Result when complete
request_count:      .word 0         ; How many waiting on this key

.section .text

main:
    ; r0 = key
    ; r1 = in_flight (is this key already being fetched)
    mov r10, r0
    mov r11, r1

    ; Check if request is already in flight
    bne r11, zero, wait_for_existing

    ; New request - mark as in flight
    mov r0, inflight_key
    store.d r10, [r0]

    ; Do the actual fetch
    call fetch_resource

    ; Clear in-flight marker
    mov r0, inflight_key
    store.d zero, [r0]

    mov r0, 1                       ; New request
    halt

wait_for_existing:
    ; Request already in flight - wait for result
    ; Increment waiting count
    mov r0, request_count
    load.d r1, [r0]
    addi r1, r1, 1
    store.d r1, [r0]

    ; In real impl, would wait for result
    mov r0, 2                       ; Waited for existing
    halt

fetch_resource:
    ; Simulate fetching (would be actual I/O)
    ret

; Full implementation with callback
request_with_dedup:
    ; r0 = key, r1 = callback ptr
    mov r5, r0
    mov r6, r1

    ; Check in-flight map
    mov r0, r5
    call is_inflight
    bne r0, zero, add_waiter

    ; Not in flight - start new request
    mov r0, r5
    call mark_inflight

    ; Do fetch
    mov r0, r5
    call fetch_resource
    mov r7, r0                      ; r7 = result

    ; Notify all waiters
    mov r0, r5
    mov r1, r7
    call notify_waiters

    ; Clear in-flight
    mov r0, r5
    call clear_inflight

    ; Return result
    mov r0, r7
    ret

add_waiter:
    ; Add callback to waiter list
    mov r0, r5
    mov r1, r6
    call add_to_waiters
    ; Will be called when request completes
    mov r0, 0                       ; Pending
    ret

is_inflight:
    mov r0, 0                       ; Stub
    ret

mark_inflight:
    ret

clear_inflight:
    ret

notify_waiters:
    ret

add_to_waiters:
    ret
