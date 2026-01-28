; @name: Swap Nibbles
; @description: Swaps high and low nibbles (4-bit halves) of a byte
; @category: algorithm/bitwise
; @difficulty: 2
;
; @prompt: swap nibbles in byte
; @prompt: exchange high and low nibbles
; @prompt: swap 4-bit halves
; @prompt: nibble swap operation
; @prompt: rotate nibbles in byte
; @prompt: exchange upper and lower 4 bits
; @prompt: swap hex digits in byte
; @prompt: nibble exchange
; @prompt: swap high low nibble
; @prompt: byte nibble swap
;
; @param: value=r0 "8-bit value"
;
; @test: r0=18 -> r0=33
; @test: r0=171 -> r0=186
; @note: 0x12 (18) -> 0x21 (33)
; @note: 0xAB (171) -> 0xBA (186)

.entry main

main:
    ; Swap nibbles: (value >> 4) | ((value & 0xF) << 4)
    mov r1, r0

    ; High nibble to low position
    alui.shr r2, r1, 4           ; value >> 4

    ; Low nibble to high position
    alui.and r3, r1, 0xF         ; value & 0xF
    alui.shl r3, r3, 4           ; << 4

    ; Combine
    alu.or r0, r2, r3

    halt
