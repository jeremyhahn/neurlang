; @name: Is Sorted
; @description: Check if array is sorted in ascending order.
; @category: array
; @difficulty: 1
;
; @prompt: check if {arr} is sorted ascending with {len} elements
; @prompt: verify {arr} of size {len} is in sorted order
; @prompt: is array {arr} sorted with {len} items
; @prompt: test if {len} elements in {arr} are in ascending order
; @prompt: check sorted property of {arr} with {len} values
; @prompt: determine if {arr} of length {len} is sorted
; @prompt: verify ascending order in {arr} for {len} entries
; @prompt: is {arr} array of {len} elements already sorted
; @prompt: check if {len} values in {arr} are non-decreasing
; @prompt: test sorted ascending for {arr} with {len} items
; @prompt: validate {arr} is sorted over {len} elements
; @prompt: return true if {arr} of {len} is in sorted order
;
; @param: arr=r0 "Pointer to array of u64 elements"
; @param: len=r1 "Number of elements in the array"
;
; @test: r0=0, r1=0 -> r0=1
; @test: r0=0, r1=1 -> r0=1
; @note: Empty or single-element array is considered sorted (returns 1)
;
; @export: is_sorted
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
    mov r0, 1  ; 1
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
    mov r14, r0  ; ptr
    mov r15, r2  ; i
    mov r14, 1  ; 1
    alu.Sub r15, r15, r14
    alui.Shl r15, r15, 3
    alu.Add r14, r14, r15
    load.Double r14, [r14]
    mov r15, r0  ; ptr
    mov r14, r2  ; i
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    load.Double r15, [r15]
    blt r15, r14, .set_10
    mov r15, 0
    b .cmp_end_11
.set_10:
    nop
    mov r15, 1
.cmp_end_11:
    nop
    beq r15, zero, .endif_9
    mov r0, 0  ; 0
    halt
.endif_9:
    nop
    mov r15, 1  ; 1
    alu.Add r2, r2, r15
    b .while_4
.endwhile_5:
    nop
    mov r0, 1  ; 1
    halt
