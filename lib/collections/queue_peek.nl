; @name: Queue Peek
; @description: Peek at the front of the queue without dequeuing.
; @category: collections/queue
; @difficulty: 1
;
; @prompt: peek at front of queue {ptr}
; @prompt: get front value from queue {ptr} without removing
; @prompt: look at front element of queue at {ptr}
; @prompt: read front of queue {ptr}
; @prompt: peek queue at address {ptr}
; @prompt: view front item in queue {ptr}
; @prompt: get queue front from {ptr} without dequeuing
; @prompt: examine front of queue at {ptr}
; @prompt: check what is at front of queue {ptr}
; @prompt: peek at queue {ptr} front element
; @prompt: read front value from FIFO queue {ptr}
; @prompt: inspect front of queue at memory {ptr}
; @prompt: see front element of queue {ptr} without removal
;
; @param: ptr=r0 "Memory address of the queue"
;
; @test: r0=0 -> r0=0
;
; @export: queue_peek
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r1, r0  ; ptr
    mov r15, 1  ; 1
    alui.Shl r15, r15, 3
    alu.Add r1, r1, r15
    load.Double r1, [r1]
    mov r2, r0  ; ptr
    mov r15, 3  ; 3
    alui.Shl r15, r15, 3
    alu.Add r2, r2, r15
    load.Double r2, [r2]
    mov r15, r2  ; count
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
    mov r14, r1  ; head
    mov r15, 4  ; 4
    alu.Add r15, r15, r14
    alui.Shl r15, r15, 3
    alu.Add r0, r0, r15
    load.Double r0, [r0]
    halt
