; @name: Linear Search
; @description: Linear search for a value in an array.
; @category: array
; @difficulty: 1
;
; @prompt: linear search for {target} in {arr} with {len} elements
; @prompt: find {target} sequentially in array {arr} of size {len}
; @prompt: search {arr} linearly for value {target} across {len} items
; @prompt: locate {target} in unsorted array {arr} with {len} elements
; @prompt: scan {arr} of length {len} looking for {target}
; @prompt: sequential search for {target} in {len} element array {arr}
; @prompt: find index of {target} in {arr} by linear scan over {len} values
; @prompt: iterate through {arr} of {len} items to find {target}
; @prompt: perform linear search for {target} in {arr} containing {len} entries
; @prompt: search array {arr} element by element for {target} up to {len}
; @prompt: find first occurrence of {target} in {arr} with {len} elements
; @prompt: look for {target} in {arr} array of length {len} sequentially
;
; @param: arr=r0 "Pointer to array of u64 elements"
; @param: len=r1 "Number of elements in the array"
; @param: target=r2 "Value to search for"
;
; @test: r0=0, r1=0, r2=5 -> r0=0xFFFFFFFFFFFFFFFF
; @note: Returns -1 (0xFFFFFFFFFFFFFFFF) when not found or empty array
;
; @export: linear_search
; Generated from Rust by nl stdlib build

.entry:
    nop
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
    mov r0, r3  ; i
    halt
.endif_5:
    nop
    mov r15, 1  ; 1
    alu.Add r3, r3, r15
    b .while_0
.endwhile_1:
    nop
    mov r0, -1  ; 18446744073709551615
    halt
