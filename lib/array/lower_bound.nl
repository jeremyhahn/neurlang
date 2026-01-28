; @name: Lower Bound
; @description: Find index of first element not less than target.
; @category: array
; @difficulty: 2
;
; @prompt: find lower bound for {target} in sorted {arr} with {len} elements
; @prompt: get first index not less than {target} in {arr} of size {len}
; @prompt: lower bound of {target} in sorted array {arr} containing {len} items
; @prompt: find insertion point for {target} in sorted {arr} with {len} values
; @prompt: locate first element >= {target} in {arr} of length {len}
; @prompt: binary search lower bound for {target} in {arr} with {len} entries
; @prompt: index of first value >= {target} in sorted {arr} of {len} items
; @prompt: find leftmost position for {target} in sorted {arr} with {len} elements
; @prompt: lower bound search for {target} in {arr} array of {len} values
; @prompt: get insertion index for {target} in sorted {arr} with {len}
; @prompt: find first position >= {target} in sorted {arr} of {len} elements
; @prompt: locate lower bound of {target} in {len} element sorted array {arr}
;
; @param: arr=r0 "Pointer to sorted array of u64 elements"
; @param: len=r1 "Number of elements in the array"
; @param: target=r2 "Value to find lower bound for"
;
; @test: r0=0, r1=0, r2=5 -> r0=0
; @note: Returns index where target would be inserted (before equals)
;
; @export: lower_bound
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r3, 0  ; 0
    mov r4, r1  ; len
.while_0:
    nop
    mov r15, r3  ; left
    mov r14, r4  ; right
    blt r15, r14, .set_2
    mov r15, 0
    b .cmp_end_3
.set_2:
    nop
    mov r15, 1
.cmp_end_3:
    nop
    beq r15, zero, .endwhile_1
    mov r15, r4  ; right
    mov r14, r3  ; left
    alu.Sub r15, r15, r14
    mov r14, 2  ; 2
    muldiv.Div r15, r15, r14
    mov r5, r3  ; left
    alu.Add r5, r5, r15
    mov r15, r0  ; ptr
    mov r14, r5  ; mid
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    load.Double r15, [r15]
    mov r14, r2  ; target
    blt r15, r14, .set_6
    mov r15, 0
    b .cmp_end_7
.set_6:
    nop
    mov r15, 1
.cmp_end_7:
    nop
    beq r15, zero, .else_4
    mov r3, r5  ; mid
    mov r15, 1  ; 1
    alu.Add r3, r3, r15
    b .endif_5
.else_4:
    nop
    mov r4, r5  ; mid
.endif_5:
    nop
    b .while_0
.endwhile_1:
    nop
    mov r0, r3  ; left
    halt
