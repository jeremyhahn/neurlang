; @name: Queue Dequeue
; @description: Dequeue a value.
; @category: collections/queue
; @difficulty: 2
;
; @prompt: dequeue value from queue at {ptr}
; @prompt: remove front element from queue {ptr}
; @prompt: dequeue from queue at address {ptr}
; @prompt: get and remove front of queue {ptr}
; @prompt: queue dequeue from {ptr}
; @prompt: remove front item from FIFO queue {ptr}
; @prompt: dequeue element from queue at {ptr}
; @prompt: take value from front of queue {ptr}
; @prompt: extract front element from queue at {ptr}
; @prompt: pop the queue located at {ptr}
; @prompt: remove and return front of queue {ptr}
; @prompt: get value from front of queue {ptr} and remove it
; @prompt: dequeue item from queue structure at {ptr}
;
; @param: ptr=r0 "Memory address of the queue"
;
; @test: r0=0 -> r0=0
;
; @export: queue_dequeue
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r1, r0  ; ptr
    load.Double r1, [r1]
    mov r2, r0  ; ptr
    mov r15, 1  ; 1
    alui.Shl r15, r15, 3
    alu.Add r2, r2, r15
    load.Double r2, [r2]
    mov r3, r0  ; ptr
    mov r15, 3  ; 3
    alui.Shl r15, r15, 3
    alu.Add r3, r3, r15
    load.Double r3, [r3]
    mov r15, r3  ; count
    mov r14, 0  ; 0
    beq r15, r14, .set_2
    mov r15, 0
    b .cmp_end_3
.set_2:
    nop
    mov r15, 1
.cmp_end_3:
    nop
    beq r15, zero, .endif_1
    mov r0, 0  ; 0
    halt
.endif_1:
    nop
    mov r4, r0  ; ptr
    mov r14, r2  ; head
    mov r15, 4  ; 4
    alu.Add r15, r15, r14
    alui.Shl r15, r15, 3
    alu.Add r4, r4, r15
    load.Double r4, [r4]
    mov r15, r2  ; head
    mov r14, 1  ; 1
    alu.Add r15, r15, r14
    mov r14, r1  ; capacity
    bge r15, r14, .set_6
    mov r15, 0
    b .cmp_end_7
.set_6:
    nop
    mov r15, 1
.cmp_end_7:
    nop
    beq r15, zero, .cond_else_4
    mov r5, 0  ; 0
    b .cond_end_5
.cond_else_4:
    nop
    mov r5, r2  ; head
    mov r15, 1  ; 1
    alu.Add r5, r5, r15
.cond_end_5:
    nop
    mov r15, r0  ; ptr
    mov r14, 1  ; 1
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, r5  ; new_head
    store.Double r14, [r15]
    mov r15, r0  ; ptr
    mov r14, 3  ; 3
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, r3  ; count
    mov r15, 1  ; 1
    alu.Sub r14, r14, r15
    store.Double r14, [r15]
    mov r0, r4  ; value
    halt
