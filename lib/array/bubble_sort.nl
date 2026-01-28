; @name: Bubble Sort
; @description: Bubble sort (simple in-place sort).
; @category: array
; @difficulty: 2
;
; @prompt: bubble sort {arr} with {len} elements
; @prompt: sort array {arr} of size {len} using bubble sort
; @prompt: perform bubble sort on {arr} containing {len} items
; @prompt: in-place bubble sort {len} elements in {arr}
; @prompt: sort {arr} of length {len} with bubble algorithm
; @prompt: apply bubble sort to {arr} with {len} values
; @prompt: bubble sort {len} element array {arr} ascending
; @prompt: sort {arr} array of {len} entries using bubble method
; @prompt: simple sort {arr} with {len} items via bubbling
; @prompt: use bubble sort on {arr} containing {len} u64 values
; @prompt: sort {len} numbers in {arr} using bubble sort
; @prompt: ascending bubble sort on {arr} with {len} elements
;
; @param: arr=r0 "Pointer to array of u64 elements (mutable)"
; @param: len=r1 "Number of elements to sort"
;
; @test: r0=0, r1=0 -> r0=0
;
; @export: bubble_sort
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
    mov r2, 0  ; 0
.while_4:
    nop
    mov r14, r1  ; len
    mov r15, 1  ; 1
    alu.Sub r14, r14, r15
    mov r15, r2  ; i
    blt r15, r14, .set_6
    mov r15, 0
    b .cmp_end_7
.set_6:
    nop
    mov r15, 1
.cmp_end_7:
    nop
    beq r15, zero, .endwhile_5
    mov r3, 0  ; 0
.while_8:
    nop
    mov r14, r1  ; len
    mov r15, 1  ; 1
    alu.Sub r14, r14, r15
    mov r15, r2  ; i
    alu.Sub r14, r14, r15
    mov r15, r3  ; j
    blt r15, r14, .set_10
    mov r15, 0
    b .cmp_end_11
.set_10:
    nop
    mov r15, 1
.cmp_end_11:
    nop
    beq r15, zero, .endwhile_9
    mov r4, r0  ; ptr
    mov r15, r3  ; j
    alui.Shl r15, r15, 3
    alu.Add r4, r4, r15
    load.Double r4, [r4]
    mov r5, r0  ; ptr
    mov r15, r3  ; j
    mov r14, 1  ; 1
    alu.Add r15, r15, r14
    alui.Shl r15, r15, 3
    alu.Add r5, r5, r15
    load.Double r5, [r5]
    mov r15, r4  ; a
    mov r14, r5  ; b
    bgt r15, r14, .set_14
    mov r15, 0
    b .cmp_end_15
.set_14:
    nop
    mov r15, 1
.cmp_end_15:
    nop
    beq r15, zero, .endif_13
    mov r15, r0  ; ptr
    mov r14, r3  ; j
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, r5  ; b
    store.Double r14, [r15]
    mov r15, r0  ; ptr
    mov r14, r3  ; j
    mov r15, 1  ; 1
    alu.Add r14, r14, r15
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, r4  ; a
    store.Double r14, [r15]
.endif_13:
    nop
    mov r15, 1  ; 1
    alu.Add r3, r3, r15
    b .while_8
.endwhile_9:
    nop
    mov r15, 1  ; 1
    alu.Add r2, r2, r15
    b .while_4
.endwhile_5:
    nop
    halt
