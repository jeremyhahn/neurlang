; @name: Array Min
; @description: Find the minimum value in an array.
; @category: array
; @difficulty: 1
;
; @prompt: find minimum value in {arr} with {len} elements
; @prompt: get the smallest element from array {arr} of size {len}
; @prompt: return min of {len} values in {arr}
; @prompt: locate the lowest value in {arr} containing {len} items
; @prompt: find smallest number in array {arr} with length {len}
; @prompt: compute minimum across {len} elements at {arr}
; @prompt: get min value from {len} integers stored in {arr}
; @prompt: find the least element in {arr} of {len} entries
; @prompt: search for minimum in array {arr} having {len} elements
; @prompt: return lowest u64 from {arr} with {len} values
; @prompt: determine smallest of {len} numbers in {arr}
; @prompt: extract minimum element from {arr} array of size {len}
;
; @param: arr=r0 "Pointer to array of u64 elements"
; @param: len=r1 "Number of elements in the array"
;
; @test: r0=0, r1=0 -> r0=0xFFFFFFFFFFFFFFFF
; @note: For empty array (len=0), returns max u64 as sentinel value
;
; @export: array_min
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r15, r1  ; len
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
    mov r0, -1  ; 18446744073709551615
    halt
.endif_1:
    nop
    mov r2, r0  ; ptr
    load.Double r2, [r2]
    mov r3, 1  ; 1
.while_4:
    nop
    mov r15, r3  ; i
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
    mov r4, r0  ; ptr
    mov r15, r3  ; i
    alui.Shl r15, r15, 3
    alu.Add r4, r4, r15
    load.Double r4, [r4]
    mov r15, r4  ; val
    mov r14, r2  ; min_val
    blt r15, r14, .set_10
    mov r15, 0
    b .cmp_end_11
.set_10:
    nop
    mov r15, 1
.cmp_end_11:
    nop
    beq r15, zero, .endif_9
    mov r2, r4  ; val
.endif_9:
    nop
    mov r15, 1  ; 1
    alu.Add r3, r3, r15
    b .while_4
.endwhile_5:
    nop
    mov r0, r2  ; min_val
    halt
