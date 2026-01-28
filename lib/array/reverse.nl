; @name: Reverse
; @description: Reverse an array in place.
; @category: array
; @difficulty: 1
;
; @prompt: reverse array {arr} in place with {len} elements
; @prompt: flip the order of {len} elements in {arr}
; @prompt: reverse {arr} of size {len} in place
; @prompt: invert element order in array {arr} containing {len} items
; @prompt: swap elements to reverse {arr} with {len} values
; @prompt: reverse {len} element array {arr} without extra memory
; @prompt: flip {arr} array of length {len} end to end
; @prompt: in-place reversal of {arr} with {len} entries
; @prompt: mirror array {arr} containing {len} elements
; @prompt: reverse order of {len} u64 values in {arr}
; @prompt: turn {arr} array backwards across {len} items
; @prompt: swap first and last elements progressively in {arr} of {len}
;
; @param: arr=r0 "Pointer to array of u64 elements (mutable)"
; @param: len=r1 "Number of elements in the array"
;
; @test: r0=0, r1=0 -> r0=0
;
; @export: reverse
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
    mov r3, r1  ; len
    mov r15, 1  ; 1
    alu.Sub r3, r3, r15
.while_4:
    nop
    mov r15, r2  ; left
    mov r14, r3  ; right
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
    mov r15, r2  ; left
    alui.Shl r15, r15, 3
    alu.Add r4, r4, r15
    load.Double r4, [r4]
    mov r15, r0  ; ptr
    mov r14, r2  ; left
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, r0  ; ptr
    mov r15, r3  ; right
    alui.Shl r15, r15, 3
    alu.Add r14, r14, r15
    load.Double r14, [r14]
    store.Double r14, [r15]
    mov r15, r0  ; ptr
    mov r14, r3  ; right
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, r4  ; temp
    store.Double r14, [r15]
    mov r15, 1  ; 1
    alu.Add r2, r2, r15
    mov r15, 1  ; 1
    alu.Sub r3, r3, r15
    b .while_4
.endwhile_5:
    nop
    halt
