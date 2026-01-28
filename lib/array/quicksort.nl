; @name: Quicksort
; @description: Quicksort implementation using heapsort (in-place, O(n log n) guaranteed).
; @category: array
; @difficulty: 3
;
; @prompt: quicksort array {arr} with {len} elements
; @prompt: sort {arr} of size {len} using quicksort
; @prompt: perform quicksort on {arr} containing {len} items
; @prompt: in-place sort {len} elements in {arr} with quicksort
; @prompt: sort {arr} of length {len} efficiently
; @prompt: apply quicksort to {arr} with {len} values
; @prompt: sort {len} element array {arr} using heap-based quicksort
; @prompt: fast sort {arr} array of {len} entries
; @prompt: efficient sort for {arr} with {len} items
; @prompt: use quicksort on {arr} containing {len} u64 values
; @prompt: sort {len} numbers in {arr} with O(n log n) guarantee
; @prompt: heapsort {arr} with {len} elements for guaranteed performance
;
; @param: arr=r0 "Pointer to array of u64 elements (mutable)"
; @param: len=r1 "Number of elements to sort"
;
; @test: r0=0, r1=0 -> r0=0
;
; @export: quicksort
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r15, r1  ; len
    mov r14, 2  ; 2
    blt r15, r14, .set_2
    mov r15, 0
    b .cmp_end_3
.set_2:
    nop
    mov r15, 1
.cmp_end_3:
    nop
    beq r15, zero, .endif_1
    halt
.endif_1:
    nop
    mov r2, r1  ; len
    mov r15, 2  ; 2
    alu.Sub r2, r2, r15
    mov r15, 2  ; 2
    muldiv.Div r2, r2, r15
.loop_4:
    nop
    mov r3, r1  ; len
    mov r15, 1  ; 1
    alu.Sub r3, r3, r15
    mov r4, r2  ; start
.loop_6:
    nop
    mov r5, 2  ; 2
    mov r15, r4  ; root
    muldiv.Mul r5, r5, r15
    mov r15, 1  ; 1
    alu.Add r5, r5, r15
    mov r15, r5  ; left_child
    mov r14, r3  ; heap_end
    bgt r15, r14, .set_10
    mov r15, 0
    b .cmp_end_11
.set_10:
    nop
    mov r15, 1
.cmp_end_11:
    nop
    beq r15, zero, .endif_9
    b .endloop_7
.endif_9:
    nop
    mov r6, r5  ; left_child
    mov r15, 1  ; 1
    alu.Add r6, r6, r15
    mov r7, r4  ; root
    mov r14, r0  ; ptr
    mov r15, r5  ; left_child
    alui.Shl r15, r15, 3
    alu.Add r14, r14, r15
    load.Double r14, [r14]
    mov r15, r0  ; ptr
    mov r14, r7  ; swap
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    load.Double r15, [r15]
    blt r15, r14, .set_14
    mov r15, 0
    b .cmp_end_15
.set_14:
    nop
    mov r15, 1
.cmp_end_15:
    nop
    beq r15, zero, .endif_13
    mov r7, r5  ; left_child
.endif_13:
    nop
    mov r15, r0  ; ptr
    mov r14, r6  ; right_child
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    load.Double r15, [r15]
    mov r14, r0  ; ptr
    mov r15, r7  ; swap
    alui.Shl r15, r15, 3
    alu.Add r14, r14, r15
    load.Double r14, [r14]
    blt r14, r15, .set_18
    mov r14, 0
    b .cmp_end_19
.set_18:
    nop
    mov r14, 1
.cmp_end_19:
    nop
    mov r15, r6  ; right_child
    mov r14, r3  ; heap_end
    ble r15, r14, .set_20
    mov r15, 0
    b .cmp_end_21
.set_20:
    nop
    mov r15, 1
.cmp_end_21:
    nop
    bne r15, zero, .set_22
    mov r15, 0
    b .cmp_end_23
.set_22:
    nop
    mov r15, 1
.cmp_end_23:
    nop
    mov r13, r14
    bne r13, zero, .set_24
    mov r13, 0
    b .cmp_end_25
.set_24:
    nop
    mov r13, 1
.cmp_end_25:
    nop
    alu.And r15, r15, r13
    beq r15, zero, .endif_17
    mov r7, r6  ; right_child
.endif_17:
    nop
    mov r15, r7  ; swap
    mov r14, r4  ; root
    beq r15, r14, .set_28
    mov r15, 0
    b .cmp_end_29
.set_28:
    nop
    mov r15, 1
.cmp_end_29:
    nop
    beq r15, zero, .endif_27
    b .endloop_7
.endif_27:
    nop
    mov r8, r0  ; ptr
    mov r15, r4  ; root
    alui.Shl r15, r15, 3
    alu.Add r8, r8, r15
    load.Double r8, [r8]
    mov r15, r0  ; ptr
    mov r14, r4  ; root
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, r0  ; ptr
    mov r15, r7  ; swap
    alui.Shl r15, r15, 3
    alu.Add r14, r14, r15
    load.Double r14, [r14]
    store.Double r14, [r15]
    mov r15, r0  ; ptr
    mov r14, r7  ; swap
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, r8  ; tmp
    store.Double r14, [r15]
    mov r4, r7  ; swap
    b .loop_6
.endloop_7:
    nop
    mov r15, r2  ; start
    mov r14, 0  ; 0
    beq r15, r14, .set_32
    mov r15, 0
    b .cmp_end_33
.set_32:
    nop
    mov r15, 1
.cmp_end_33:
    nop
    beq r15, zero, .endif_31
    b .endloop_5
.endif_31:
    nop
    mov r15, 1  ; 1
    alu.Sub r2, r2, r15
    b .loop_4
.endloop_5:
    nop
    mov r9, r1  ; len
    mov r15, 1  ; 1
    alu.Sub r9, r9, r15
.while_34:
    nop
    mov r15, r9  ; end
    mov r14, 0  ; 0
    bgt r15, r14, .set_36
    mov r15, 0
    b .cmp_end_37
.set_36:
    nop
    mov r15, 1
.cmp_end_37:
    nop
    beq r15, zero, .endwhile_35
    mov r10, r0  ; ptr
    mov r15, 0  ; 0
    alui.Shl r15, r15, 3
    alu.Add r10, r10, r15
    load.Double r10, [r10]
    mov r15, r0  ; ptr
    mov r14, 0  ; 0
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, r0  ; ptr
    mov r15, r9  ; end
    alui.Shl r15, r15, 3
    alu.Add r14, r14, r15
    load.Double r14, [r14]
    store.Double r14, [r15]
    mov r15, r0  ; ptr
    mov r14, r9  ; end
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, r10  ; tmp
    store.Double r14, [r15]
    mov r15, 1  ; 1
    alu.Sub r9, r9, r15
    mov r11, 0  ; 0
.loop_38:
    nop
    mov r12, 2  ; 2
    mov r15, r11  ; root
    muldiv.Mul r12, r12, r15
    mov r15, 1  ; 1
    alu.Add r12, r12, r15
    mov r15, r12  ; left_child
    mov r14, r9  ; end
    bgt r15, r14, .set_42
    mov r15, 0
    b .cmp_end_43
.set_42:
    nop
    mov r15, 1
.cmp_end_43:
    nop
    beq r15, zero, .endif_41
    b .endloop_39
.endif_41:
    nop
    mov r13, r12  ; left_child
    mov r15, 1  ; 1
    alu.Add r13, r13, r15
    mov r14, r11  ; root
    mov r14, r0  ; ptr
    mov r15, r12  ; left_child
    alui.Shl r15, r15, 3
    alu.Add r14, r14, r15
    load.Double r14, [r14]
    mov r15, r0  ; ptr
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    load.Double r15, [r15]
    blt r15, r14, .set_46
    mov r15, 0
    b .cmp_end_47
.set_46:
    nop
    mov r15, 1
.cmp_end_47:
    nop
    beq r15, zero, .endif_45
    mov r14, r12  ; left_child
.endif_45:
    nop
    mov r15, r0  ; ptr
    mov r14, r13  ; right_child
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    load.Double r15, [r15]
    mov r14, r0  ; ptr
    mov r15, r14  ; swap
    alui.Shl r15, r15, 3
    alu.Add r14, r14, r15
    load.Double r14, [r14]
    blt r14, r15, .set_50
    mov r14, 0
    b .cmp_end_51
.set_50:
    nop
    mov r14, 1
.cmp_end_51:
    nop
    mov r15, r13  ; right_child
    mov r14, r9  ; end
    ble r15, r14, .set_52
    mov r15, 0
    b .cmp_end_53
.set_52:
    nop
    mov r15, 1
.cmp_end_53:
    nop
    bne r15, zero, .set_54
    mov r15, 0
    b .cmp_end_55
.set_54:
    nop
    mov r15, 1
.cmp_end_55:
    nop
    mov r13, r14
    bne r13, zero, .set_56
    mov r13, 0
    b .cmp_end_57
.set_56:
    nop
    mov r13, 1
.cmp_end_57:
    nop
    alu.And r15, r15, r13
    beq r15, zero, .endif_49
    mov r14, r13  ; right_child
.endif_49:
    nop
    mov r15, r14  ; swap
    mov r14, r11  ; root
    beq r15, r14, .set_60
    mov r15, 0
    b .cmp_end_61
.set_60:
    nop
    mov r15, 1
.cmp_end_61:
    nop
    beq r15, zero, .endif_59
    b .endloop_39
.endif_59:
    nop
    mov r15, r0  ; ptr
    mov r14, r11  ; root
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    load.Double r15, [r15]
    mov r15, r0  ; ptr
    mov r14, r11  ; root
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, r0  ; ptr
    mov r15, r14  ; swap
    alui.Shl r15, r15, 3
    alu.Add r14, r14, r15
    load.Double r14, [r14]
    store.Double r14, [r15]
    mov r15, r0  ; ptr
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, r15  ; tmp2
    store.Double r14, [r15]
    mov r11, r14  ; swap
    b .loop_38
.endloop_39:
    nop
    b .while_34
.endwhile_35:
    nop
    halt
