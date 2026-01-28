; @name: Sum
; @description: Sum all elements in an array.
; @category: array
; @difficulty: 1
;
; @prompt: sum all elements in {arr} with length {len}
; @prompt: calculate the total of {arr} array containing {len} elements
; @prompt: add up all {len} values in array {arr}
; @prompt: compute array sum for {arr} of size {len}
; @prompt: get the sum of {len} integers stored at {arr}
; @prompt: accumulate all elements from {arr} with {len} items
; @prompt: find total value of array {arr} having {len} elements
; @prompt: reduce {arr} by addition over {len} elements
; @prompt: sum {len} u64 values starting at {arr}
; @prompt: calculate cumulative sum of {arr} array with {len} entries
; @prompt: aggregate {len} numbers in {arr} into single sum
; @prompt: return sum of all {len} elements in {arr}
;
; @param: arr=r0 "Pointer to array of u64 elements"
; @param: len=r1 "Number of elements in the array"
;
; @test: r0=0, r1=0 -> r0=0
; @note: Sum elements from array at ptr. With len=0, returns 0
;
; @export: sum
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r2, 0  ; 0
    mov r3, 0  ; 0
.while_0:
    nop
    mov r15, r3  ; i
    mov r14, r1  ; len
    blt r15, r14, .set_2
    mov r15, 0
    b .cmp_end_3
.set_2:
    nop
    mov r15, 1
.cmp_end_3:
    nop
    beq r15, zero, .endwhile_1
    mov r15, r0  ; ptr
    mov r14, r3  ; i
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    load.Double r15, [r15]
    alu.Add r2, r2, r15
    mov r15, 1  ; 1
    alu.Add r3, r3, r15
    b .while_0
.endwhile_1:
    nop
    mov r0, r2  ; total
    halt
