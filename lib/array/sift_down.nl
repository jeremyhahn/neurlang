; @name: Sift Down
; @description: Sift down operation for heapsort.
; @category: array
; @difficulty: 3
;
; @prompt: sift down element at {start} in heap {arr} with {end} elements
; @prompt: heapsort sift down {arr} from {start} to {end}
; @prompt: restore heap property at index {start} in {arr} up to {end}
; @prompt: sift down operation on {arr} starting at {start} ending at {end}
; @prompt: bubble down element at {start} in heap array {arr} to {end}
; @prompt: max-heapify {arr} at position {start} with heap size {end}
; @prompt: perform sift down on {arr} from index {start} to {end}
; @prompt: heap sift down {arr} {start} {end}
; @prompt: fix heap at index {start} in {arr} up to {end}
; @prompt: down-heap operation on {arr} at {start} with bound {end}
;
; @param: arr=r0 "Pointer to array of u64 elements (mutable)"
; @param: start=r1 "Index to start sifting from"
; @param: end=r2 "Last valid index in heap"
;
; @test: r0=0, r1=0, r2=0 -> r0=0
; @note: Modifies array in-place by sifting element at start down
;
; @export: sift_down
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r3, r1  ; start
.loop_0:
    nop
    mov r4, 2  ; 2
    mov r15, r3  ; root
    muldiv.Mul r4, r4, r15
    mov r15, 1  ; 1
    alu.Add r4, r4, r15
    mov r15, r4  ; left_child
    mov r14, r2  ; end
    bgt r15, r14, .set_4
    mov r15, 0
    b .cmp_end_5
.set_4:
    nop
    mov r15, 1
.cmp_end_5:
    nop
    beq r15, zero, .endif_3
    b .endloop_1
.endif_3:
    nop
    mov r5, r4  ; left_child
    mov r15, 1  ; 1
    alu.Add r5, r5, r15
    mov r6, r3  ; root
    mov r15, r0  ; ptr
    mov r14, r6  ; swap
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    load.Double r15, [r15]
    mov r14, r0  ; ptr
    mov r15, r4  ; left_child
    alui.Shl r15, r15, 3
    alu.Add r14, r14, r15
    load.Double r14, [r14]
    blt r15, r14, .set_8
    mov r15, 0
    b .cmp_end_9
.set_8:
    nop
    mov r15, 1
.cmp_end_9:
    nop
    beq r15, zero, .endif_7
    mov r6, r4  ; left_child
.endif_7:
    nop
    mov r15, r5  ; right_child
    mov r14, r2  ; end
    ble r15, r14, .set_12
    mov r15, 0
    b .cmp_end_13
.set_12:
    nop
    mov r15, 1
.cmp_end_13:
    nop
    mov r14, r0  ; ptr
    mov r15, r6  ; swap
    alui.Shl r15, r15, 3
    alu.Add r14, r14, r15
    load.Double r14, [r14]
    mov r15, r0  ; ptr
    mov r14, r5  ; right_child
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    load.Double r15, [r15]
    blt r14, r15, .set_14
    mov r14, 0
    b .cmp_end_15
.set_14:
    nop
    mov r14, 1
.cmp_end_15:
    nop
    bne r15, zero, .set_16
    mov r15, 0
    b .cmp_end_17
.set_16:
    nop
    mov r15, 1
.cmp_end_17:
    nop
    mov r13, r14
    bne r13, zero, .set_18
    mov r13, 0
    b .cmp_end_19
.set_18:
    nop
    mov r13, 1
.cmp_end_19:
    nop
    alu.And r15, r15, r13
    beq r15, zero, .endif_11
    mov r6, r5  ; right_child
.endif_11:
    nop
    mov r15, r6  ; swap
    mov r14, r3  ; root
    beq r15, r14, .set_22
    mov r15, 0
    b .cmp_end_23
.set_22:
    nop
    mov r15, 1
.cmp_end_23:
    nop
    beq r15, zero, .endif_21
    b .endloop_1
.endif_21:
    nop
    mov r7, r0  ; ptr
    mov r15, r3  ; root
    alui.Shl r15, r15, 3
    alu.Add r7, r7, r15
    load.Double r7, [r7]
    mov r15, r0  ; ptr
    mov r14, r3  ; root
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, r0  ; ptr
    mov r15, r6  ; swap
    alui.Shl r15, r15, 3
    alu.Add r14, r14, r15
    load.Double r14, [r14]
    store.Double r14, [r15]
    mov r15, r0  ; ptr
    mov r14, r6  ; swap
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, r7  ; tmp
    store.Double r14, [r15]
    mov r3, r6  ; swap
    b .loop_0
.endloop_1:
    nop
    halt
