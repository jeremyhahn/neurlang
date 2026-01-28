; @name: Binary Search
; @description: Binary search in a sorted array.
; @category: array
; @difficulty: 2
;
; @prompt: binary search for {target} in sorted {arr} with {len} elements
; @prompt: find {target} using binary search in {arr} of size {len}
; @prompt: search sorted array {arr} for {target} with {len} items
; @prompt: locate {target} in sorted {arr} using divide and conquer over {len} elements
; @prompt: perform binary search on {arr} of length {len} for value {target}
; @prompt: efficient search for {target} in sorted {arr} containing {len} entries
; @prompt: bisect {arr} array of {len} elements to find {target}
; @prompt: binary search {len} sorted values in {arr} for {target}
; @prompt: find index of {target} in sorted array {arr} with {len} items
; @prompt: search {arr} with binary algorithm for {target} across {len} elements
; @prompt: logarithmic search for {target} in sorted {arr} of {len} values
; @prompt: divide and conquer search for {target} in {arr} with {len} entries
;
; @param: arr=r0 "Pointer to sorted array of u64 elements"
; @param: len=r1 "Number of elements in the array"
; @param: target=r2 "Value to search for"
;
; @test: r0=0, r1=0, r2=5 -> r0=0xFFFFFFFFFFFFFFFF
; @note: Returns -1 (0xFFFFFFFFFFFFFFFF) when not found or empty array
;
; @export: binary_search
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
    mov r3, 0  ; 0
    mov r4, r1  ; len
    mov r15, 1  ; 1
    alu.Sub r4, r4, r15
.while_4:
    nop
    mov r15, r3  ; left
    mov r14, r4  ; right
    ble r15, r14, .set_6
    mov r15, 0
    b .cmp_end_7
.set_6:
    nop
    mov r15, 1
.cmp_end_7:
    nop
    beq r15, zero, .endwhile_5
    mov r15, r4  ; right
    mov r14, r3  ; left
    alu.Sub r15, r15, r14
    mov r14, 2  ; 2
    muldiv.Div r15, r15, r14
    mov r5, r3  ; left
    alu.Add r5, r5, r15
    mov r6, r0  ; ptr
    mov r15, r5  ; mid
    alui.Shl r15, r15, 3
    alu.Add r6, r6, r15
    load.Double r6, [r6]
    mov r15, r6  ; val
    mov r14, r2  ; target
    beq r15, r14, .set_10
    mov r15, 0
    b .cmp_end_11
.set_10:
    nop
    mov r15, 1
.cmp_end_11:
    nop
    beq r15, zero, .else_8
    mov r0, r5  ; mid
    halt
    b .endif_9
.else_8:
    nop
    mov r15, r6  ; val
    mov r14, r2  ; target
    blt r15, r14, .set_14
    mov r15, 0
    b .cmp_end_15
.set_14:
    nop
    mov r15, 1
.cmp_end_15:
    nop
    beq r15, zero, .else_12
    mov r3, r5  ; mid
    mov r15, 1  ; 1
    alu.Add r3, r3, r15
    b .endif_13
.else_12:
    nop
    mov r15, r5  ; mid
    mov r14, 0  ; 0
    beq r15, r14, .set_18
    mov r15, 0
    b .cmp_end_19
.set_18:
    nop
    mov r15, 1
.cmp_end_19:
    nop
    beq r15, zero, .endif_17
    b .endwhile_5
.endif_17:
    nop
    mov r4, r5  ; mid
    mov r15, 1  ; 1
    alu.Sub r4, r4, r15
.endif_13:
    nop
.endif_9:
    nop
    b .while_4
.endwhile_5:
    nop
    mov r0, -1  ; 18446744073709551615
    halt
