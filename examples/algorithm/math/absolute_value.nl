; @name: Absolute Value
; @description: Computes absolute value of a signed integer
; @category: algorithm/math
; @difficulty: 1
;
; @prompt: absolute value of {n}
; @prompt: abs({n})
; @prompt: compute |{n}|
; @prompt: get absolute value of {n}
; @prompt: make {n} positive
; @prompt: magnitude of {n}
; @prompt: remove sign from {n}
; @prompt: convert {n} to positive
;
; @param: n=r0 "The signed integer value"
;
; @test: r0=42 -> r0=42
; @test: r0=0 -> r0=0
; @note: For positive inputs, returns unchanged

.entry main

main:
    ; Check sign bit (bit 63)
    ; If n >> 63 == 1, it's negative
    mov r1, r0
    alui.shr r1, r1, 63

    ; If not negative (sign bit == 0), return as-is
    beq r1, zero, .done

    ; Negate: -n = ~n + 1 (two's complement)
    alui.xor r0, r0, -1
    alui.add r0, r0, 1

.done:
    halt
