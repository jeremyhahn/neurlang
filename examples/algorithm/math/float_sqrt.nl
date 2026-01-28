; @name: Float Square Root
; @description: Computes square root of a floating-point number
; @category: algorithm/math
; @difficulty: 1
;
; @prompt: compute square root of float
; @prompt: sqrt of floating point
; @prompt: float sqrt calculation
; @prompt: square root using FPU
; @prompt: compute sqrt(2.0)
; @prompt: floating point square root
;
; @param: none "Uses hardcoded input 2.0"
;
; @test: -> r0=4609047870845172685
;
; @note: Uses hardcoded input 2.0, result is sqrt(2) = 1.414...
; @note: Float test - result compared with tolerance

.entry main

main:
    ; 2.0 as f64 bits = 0x4000_0000_0000_0000
    ; High 32 bits = 0x40000000 (1073741824)
    ; Low 32 bits = 0x00000000

    ; Build 64-bit value manually
    mov r0, 0x40000000          ; high 32 bits
    alui.shl r0, r0, 32         ; shift left by 32

    ; r0 now contains 2.0 as f64 bits

    ; Compute sqrt(2.0)
    fpu.fsqrt r0, r0

    ; Result in r0 as f64 bits (sqrt(2) ≈ 1.414...)
    halt
