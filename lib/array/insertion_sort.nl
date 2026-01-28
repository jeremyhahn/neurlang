; @name: Insertion Sort
; @description: Insertion sort (stable, efficient for small arrays).
; @category: array
; @difficulty: 2
;
; @prompt: insertion sort {arr} with {len} elements
; @prompt: sort array {arr} of size {len} using insertion sort
; @prompt: perform insertion sort on {arr} containing {len} items
; @prompt: stable sort {len} elements in {arr} using insertion method
; @prompt: sort {arr} of length {len} with insertion algorithm
; @prompt: apply insertion sort to {arr} with {len} values
; @prompt: insertion sort {len} element array {arr} ascending
; @prompt: sort {arr} array of {len} entries using insertion method
; @prompt: sort small array {arr} with {len} items via insertion
; @prompt: use insertion sort on {arr} containing {len} u64 values
; @prompt: sort {len} numbers in {arr} using insertion sort
; @prompt: stable ascending sort on {arr} with {len} elements
;
; @param: arr=r0 "Pointer to array of u64 elements (mutable)"
; @param: len=r1 "Number of elements to sort"
;
; @test: r0=0, r1=0 -> r0=0
;
; @export: insertion_sort
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
    mov r2, 1  ; 1
.while_4:
    nop
    mov r15, r2  ; i
    mov r14, r1  ; len
    blt r15, r14, .set_6
    mov r15, 0
    b .cmp_end_7
.set_6:
    nop
    mov r15, 1
.cmp_end_7:
    nop
    beq r15, zero, .endwhile_5
    mov r3, r0  ; ptr
    mov r15, r2  ; i
    alui.Shl r15, r15, 3
    alu.Add r3, r3, r15
    load.Double r3, [r3]
    mov r4, r2  ; i
.while_8:
    nop
    mov r14, r0  ; ptr
    mov r15, r4  ; j
    mov r14, 1  ; 1
    alu.Sub r15, r15, r14
    alui.Shl r15, r15, 3
    alu.Add r14, r14, r15
    load.Double r14, [r14]
    mov r15, r3  ; key
    bgt r14, r15, .set_10
    mov r14, 0
    b .cmp_end_11
.set_10:
    nop
    mov r14, 1
.cmp_end_11:
    nop
    mov r15, r4  ; j
    mov r14, 0  ; 0
    bgt r15, r14, .set_12
    mov r15, 0
    b .cmp_end_13
.set_12:
    nop
    mov r15, 1
.cmp_end_13:
    nop
    bne r15, zero, .set_14
    mov r15, 0
    b .cmp_end_15
.set_14:
    nop
    mov r15, 1
.cmp_end_15:
    nop
    mov r13, r14
    bne r13, zero, .set_16
    mov r13, 0
    b .cmp_end_17
.set_16:
    nop
    mov r13, 1
.cmp_end_17:
    nop
    alu.And r15, r15, r13
    beq r15, zero, .endwhile_9
    mov r15, r0  ; ptr
    mov r14, r4  ; j
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, r0  ; ptr
    mov r15, r4  ; j
    mov r14, 1  ; 1
    alu.Sub r15, r15, r14
    alui.Shl r15, r15, 3
    alu.Add r14, r14, r15
    load.Double r14, [r14]
    store.Double r14, [r15]
    mov r15, 1  ; 1
    alu.Sub r4, r4, r15
    b .while_8
.endwhile_9:
    nop
    mov r15, r0  ; ptr
    mov r14, r4  ; j
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, r3  ; key
    store.Double r14, [r15]
    mov r15, 1  ; 1
    alu.Add r2, r2, r15
    b .while_4
.endwhile_5:
    nop
    halt
