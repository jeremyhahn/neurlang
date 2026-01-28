; @name: Isolate Lowest Bit
; @description: Isolate the lowest set bit.
; @category: bitwise
; @difficulty: 1
;
; @prompt: isolate the lowest set bit in {n}
; @prompt: extract only the rightmost 1 bit from {n}
; @prompt: get the least significant set bit of {n}
; @prompt: return only the lowest 1 bit of {n}
; @prompt: mask all but the lowest set bit in {n}
; @prompt: keep only the first set bit from the right in {n}
; @prompt: isolate LSB of {n}
; @prompt: get the lowest set bit value from {n}
; @prompt: extract the bottommost set bit of {n}
; @prompt: find the value of the lowest 1 bit in {n}
; @prompt: get only the rightmost 1 in {n}
; @prompt: compute {n} AND (NOT {n} + 1)
; @prompt: isolate least significant 1 bit of {n}
;
; @param: n=r0 "The value to isolate lowest set bit from"
;
; @test: r0=0 -> r0=0
;
; @export: isolate_lowest_bit
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r15, r0  ; n
    alui.Xor r15, r15, -1
    mov r14, 1  ; 1
    alu.Add r15, r15, r14
    alu.And r0, r0, r15
    halt
