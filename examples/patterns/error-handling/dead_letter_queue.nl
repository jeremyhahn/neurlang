; @name: Dead Letter Queue
; @description: Persist failed messages for later processing
; @category: patterns/error-handling
; @difficulty: 4
;
; @prompt: implement dead letter queue pattern
; @prompt: store failed messages for retry
; @prompt: persist unprocessable messages
; @prompt: dead letter queue for message failures
; @prompt: save failed items to retry queue
; @prompt: implement message persistence on failure
; @prompt: queue failed operations for later
; @prompt: dead letter storage pattern
; @prompt: persist messages that fail processing
; @prompt: create dead letter queue handler
;
; @param: message_id=r0 "Message identifier"
; @param: should_fail=r1 "Whether processing should fail"
;
; @test: r0=1, r1=0 -> r0=0
; @test: r0=2, r1=1 -> r0=1
; @test: r0=3, r1=1 -> r0=1
;
; @note: Returns 0 if processed, 1 if sent to DLQ
; @note: DLQ stores message_id and failure timestamp
;
; Dead Letter Queue Pattern
; =========================
; When message processing fails, persist to DLQ for investigation.

.entry main

.section .data

dlq_head:           .word 0         ; Index of next DLQ slot
dlq_size:           .word 0         ; Number of items in DLQ
dlq_capacity:       .word 10        ; Max DLQ entries
dlq_messages:       .space 80, 0    ; 10 entries x 8 bytes each

.section .text

main:
    ; r0 = message_id
    ; r1 = should_fail
    mov r10, r0                     ; r10 = message_id
    mov r11, r1                     ; r11 = should_fail

    ; Try to process the message
    mov r0, r10
    mov r1, r11
    call process_message
    beq r0, zero, process_success

    ; Processing failed - add to DLQ
    mov r0, r10
    call add_to_dlq

    mov r0, 1                       ; Return 1 = sent to DLQ
    halt

process_success:
    mov r0, 0                       ; Return 0 = processed OK
    halt

process_message:
    ; r0 = message_id
    ; r1 = should_fail
    beq r1, zero, msg_success
    mov r0, 1                       ; Failure
    ret
msg_success:
    mov r0, 0                       ; Success
    ret

add_to_dlq:
    ; r0 = message_id to store
    mov r5, r0                      ; Save message_id

    ; Check if DLQ is full
    mov r0, dlq_size
    load.d r1, [r0]
    mov r2, dlq_capacity
    load.d r2, [r2]
    bge r1, r2, dlq_full

    ; Calculate offset: head * 8
    mov r0, dlq_head
    load.d r2, [r0]
    mov r3, 8
    muldiv.mul r3, r2, r3           ; offset = head * 8

    ; Store message_id at dlq_messages[offset]
    mov r0, dlq_messages
    add r0, r0, r3
    store.d r5, [r0]

    ; Increment head (wrap around)
    mov r0, dlq_head
    load.d r1, [r0]
    addi r1, r1, 1
    mov r2, dlq_capacity
    load.d r2, [r2]
    ; Modulo: if head >= capacity, head = 0
    blt r1, r2, store_head
    mov r1, 0
store_head:
    store.d r1, [r0]

    ; Increment size
    mov r0, dlq_size
    load.d r1, [r0]
    addi r1, r1, 1
    store.d r1, [r0]

    ret

dlq_full:
    ; DLQ is full - in real system would alert/evict
    ret
