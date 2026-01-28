; @name: Float Circle Area
; @description: Computes circle area using pi * r^2
; @category: algorithm/math
; @difficulty: 1
;
; @prompt: compute circle area
; @prompt: calculate pi times r squared
; @prompt: area of circle with radius
; @prompt: circle area formula
; @prompt: pi * r * r
; @prompt: floating point circle calculation
;
; @param: none "Uses hardcoded radius 2.0"
;
; @test: -> r0=4623263855806786840
;
; @note: Uses hardcoded radius 2.0, result is pi * 4 = 12.566...
; @note: Float test - result compared with tolerance

.entry main

main:
    ; Build pi = 3.14159265358979323846 (f64 bits = 0x400921FB54442D18)
    ; High 32 bits = 0x400921FB, Low 32 bits = 0x54442D18
    mov r2, 0x400921FB          ; pi high 32 bits
    alui.shl r2, r2, 32         ; shift to upper position
    mov r15, 0x54442D18         ; pi low 32 bits
    alu.or r2, r2, r15          ; r2 = pi

    ; Build 2.0 (f64 bits = 0x4000000000000000)
    ; High 32 bits = 0x40000000, Low 32 bits = 0
    mov r3, 0x40000000          ; 2.0 high 32 bits
    alui.shl r3, r3, 32         ; r3 = 2.0

    ; Compute r^2 = 2.0 * 2.0 = 4.0
    fpu.fmul r4, r3, r3

    ; Compute pi * r^2
    fpu.fmul r0, r2, r4

    ; Result in r0 as f64 bits (pi * 4 ≈ 12.566...)
    halt
