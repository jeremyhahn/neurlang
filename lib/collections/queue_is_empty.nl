; @name: Queue Is Empty
; @description: Check if queue is empty.
; @category: collections/queue
; @difficulty: 1
;
; @prompt: check if queue at {ptr} is empty
; @prompt: is queue {ptr} empty
; @prompt: test if queue at {ptr} has no elements
; @prompt: determine if queue {ptr} is empty
; @prompt: check queue {ptr} for emptiness
; @prompt: is FIFO queue at {ptr} empty
; @prompt: verify queue at address {ptr} is empty
; @prompt: see if queue {ptr} contains no items
; @prompt: check for empty queue at {ptr}
; @prompt: test queue {ptr} emptiness
; @prompt: is queue at memory {ptr} empty
; @prompt: check if queue {ptr} has zero elements
; @prompt: query if queue at {ptr} is empty
;
; @param: ptr=r0 "Memory address of the queue"
;
; @test: r0=0 -> r0=1
; @note: Returns 1 if empty (count is 0), 0 otherwise. With ptr=0, reads count from address 24 which is 0
;
; @export: queue_is_empty
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r15, r0  ; ptr
    mov r14, 3  ; 3
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    load.Double r15, [r15]
    mov r14, 0  ; 0
    beq r15, r14, .set_2
    mov r15, 0
    b .cmp_end_3
.set_2:
    nop
    mov r15, 1
.cmp_end_3:
    nop
    beq r15, zero, .else_0
    mov r0, 1  ; 1
    b .endif_1
.else_0:
    nop
    mov r0, 0  ; 0
.endif_1:
    nop
    halt
