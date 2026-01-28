; @name: Partial Failure Handling
; @description: Handle partial success in batch operations
; @category: patterns/error-handling
; @difficulty: 4
;
; @prompt: handle partial failure in batch operation
; @prompt: process batch with partial success tracking
; @prompt: track successful and failed items in batch
; @prompt: handle partial batch failures
; @prompt: continue processing after partial failure
; @prompt: batch processing with error collection
; @prompt: partial success handling pattern
; @prompt: track which items failed in batch
; @prompt: aggregate errors from batch operation
; @prompt: handle batch with some failures
;
; @param: item_count=r0 "Number of items in batch"
; @param: fail_mask=r1 "Bitmap of which items fail"
;
; @test: r0=3, r1=0 -> r0=3
; @test: r0=3, r1=1 -> r0=2
; @test: r0=3, r1=7 -> r0=0
; @test: r0=4, r1=5 -> r0=2
;
; @note: Returns count of successfully processed items
; @note: fail_mask bit i means item i fails
;
; Partial Failure Handling Pattern
; ================================
; Process batch, track successes/failures, report partial results.

.entry main

.section .data

success_count:      .word 0
failure_count:      .word 0
items_processed:    .word 0

.section .text

main:
    ; r0 = number of items
    ; r1 = failure mask (bit i = item i fails)
    mov r10, r0                     ; r10 = total items
    mov r11, r1                     ; r11 = failure mask
    mov r12, 0                      ; r12 = current index
    mov r13, 0                      ; r13 = success count
    mov r14, 0                      ; r14 = failure count

process_loop:
    bge r12, r10, done

    ; Check if this item should fail (bit test)
    mov r0, 1
    mov r1, r12
    ; Shift 1 left by index
    beq r1, zero, check_mask
shift_loop:
    add r0, r0, r0                  ; r0 <<= 1
    subi r1, r1, 1
    bgt r1, zero, shift_loop

check_mask:
    ; r0 = 1 << index
    alu.And r1, r11, r0             ; r1 = mask & (1 << index)
    bne r1, zero, item_failed

    ; Item succeeded
    addi r13, r13, 1
    b next_item

item_failed:
    addi r14, r14, 1

next_item:
    addi r12, r12, 1
    b process_loop

done:
    ; Store results
    mov r0, success_count
    store.d r13, [r0]
    mov r0, failure_count
    store.d r14, [r0]
    mov r0, items_processed
    store.d r10, [r0]

    ; Return success count
    mov r0, r13
    halt
