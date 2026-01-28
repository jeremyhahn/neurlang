; @name: Queue Enqueue
; @description: Enqueue a value.
; @category: collections/queue
; @difficulty: 2
;
; @prompt: enqueue {value} to queue at {ptr}
; @prompt: add {value} to back of queue {ptr}
; @prompt: put {value} in queue at address {ptr}
; @prompt: enqueue value {value} to queue {ptr}
; @prompt: add element {value} to queue at {ptr}
; @prompt: insert {value} at end of queue {ptr}
; @prompt: queue enqueue {value} to {ptr}
; @prompt: append {value} to queue at memory {ptr}
; @prompt: place {value} at back of queue {ptr}
; @prompt: add item {value} to FIFO queue at {ptr}
; @prompt: add {value} to the queue located at {ptr}
; @prompt: store {value} in queue {ptr}
; @prompt: enqueue {value} to queue structure at {ptr}
; @prompt: push {value} to end of queue {ptr}
;
; @param: ptr=r0 "Memory address of the queue"
; @param: value=r1 "Value to enqueue"
;
; @test: r0=0, r1=0 -> r0=0
;
; @export: queue_enqueue
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r2, r0  ; ptr
    load.Double r2, [r2]
    mov r3, r0  ; ptr
    mov r15, 2  ; 2
    alui.Shl r15, r15, 3
    alu.Add r3, r3, r15
    load.Double r3, [r3]
    mov r4, r0  ; ptr
    mov r15, 3  ; 3
    alui.Shl r15, r15, 3
    alu.Add r4, r4, r15
    load.Double r4, [r4]
    mov r15, r4  ; count
    mov r14, r2  ; capacity
    bge r15, r14, .set_2
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
    mov r15, r0  ; ptr
    mov r15, r3  ; tail
    mov r14, 4  ; 4
    alu.Add r14, r14, r15
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, r1  ; value
    store.Double r14, [r15]
    mov r15, r3  ; tail
    mov r14, 1  ; 1
    alu.Add r15, r15, r14
    mov r14, r2  ; capacity
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
    mov r5, r3  ; tail
    mov r15, 1  ; 1
    alu.Add r5, r5, r15
.cond_end_5:
    nop
    mov r15, r0  ; ptr
    mov r14, 2  ; 2
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, r5  ; new_tail
    store.Double r14, [r15]
    mov r15, r0  ; ptr
    mov r14, 3  ; 3
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, r4  ; count
    mov r15, 1  ; 1
    alu.Add r14, r14, r15
    store.Double r14, [r15]
    mov r0, 1  ; 1
    halt
