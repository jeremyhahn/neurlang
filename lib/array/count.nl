; @name: Count
; @description: Count elements equal to a value.
; @category: array
; @difficulty: 1
;
; @prompt: count occurrences of {target} in {arr} with {len} elements
; @prompt: how many times does {target} appear in array {arr} of size {len}
; @prompt: count {target} in {arr} containing {len} items
; @prompt: find frequency of {target} in {arr} with {len} values
; @prompt: tally occurrences of {target} in {len} element array {arr}
; @prompt: count how many elements equal {target} in {arr} of length {len}
; @prompt: number of {target} values in {arr} array with {len} entries
; @prompt: count matches for {target} in {arr} across {len} elements
; @prompt: get occurrence count of {target} in {arr} of {len} items
; @prompt: count elements matching {target} in {arr} with {len} values
; @prompt: frequency count of {target} in array {arr} having {len} elements
; @prompt: count all {target} in {len} element array {arr}
;
; @param: arr=r0 "Pointer to array of u64 elements"
; @param: len=r1 "Number of elements in the array"
; @param: target=r2 "Value to count"
;
; @test: r0=0, r1=0, r2=5 -> r0=0
; @note: Returns count of matching elements
;
; @export: count
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r3, 0  ; 0
    mov r4, 0  ; 0
.while_0:
    nop
    mov r15, r4  ; i
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
    mov r14, r4  ; i
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    load.Double r15, [r15]
    mov r14, r2  ; target
    beq r15, r14, .set_6
    mov r15, 0
    b .cmp_end_7
.set_6:
    nop
    mov r15, 1
.cmp_end_7:
    nop
    beq r15, zero, .endif_5
    mov r15, 1  ; 1
    alu.Add r3, r3, r15
.endif_5:
    nop
    mov r15, 1  ; 1
    alu.Add r4, r4, r15
    b .while_0
.endwhile_1:
    nop
    mov r0, r3  ; cnt
    halt
