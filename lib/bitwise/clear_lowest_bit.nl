; @name: Clear Lowest Bit
; @description: Clear the lowest set bit.
; @category: bitwise
; @difficulty: 1
;
; @prompt: clear the lowest set bit in {n}
; @prompt: turn off the rightmost 1 bit in {n}
; @prompt: unset the least significant set bit in {n}
; @prompt: remove the lowest 1 bit from {n}
; @prompt: clear the first set bit from the right in {n}
; @prompt: zero out the lowest set bit in {n}
; @prompt: reset the rightmost 1 in {n}
; @prompt: turn off LSB of {n}
; @prompt: clear lowest 1 bit in {n}
; @prompt: unset first set bit from right in {n}
; @prompt: remove the bottommost set bit from {n}
; @prompt: mask off the lowest set bit in {n}
; @prompt: compute {n} AND ({n} - 1)
;
; @param: n=r0 "The value to clear lowest set bit in"
;
; @test: r0=0 -> r0=0
;
; @export: clear_lowest_bit
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r15, r0  ; n
    mov r14, 1  ; 1
    alu.Sub r15, r15, r14
    alu.And r0, r0, r15
    halt
