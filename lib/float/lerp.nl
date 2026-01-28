; @name: Lerp
; @description: Linear interpolation between two values.
; @category: float
; @difficulty: 2
;
; @prompt: lerp from {a} to {b} at {t}
; @prompt: linear interpolation {a} {b} {t}
; @prompt: lerp({a}, {b}, {t})
; @prompt: interpolate between {a} and {b} by {t}
; @prompt: blend {a} and {b} with factor {t}
; @prompt: mix {a} {b} at position {t}
; @prompt: compute lerp {a} to {b} t={t}
; @prompt: linear blend {a} {b} {t}
; @prompt: interpolate {t} between {a} and {b}
; @prompt: weighted average {a} {b} weight {t}
;
; @param: a=r0 "Start value"
; @param: b=r1 "End value"
; @param: t=r2 "Interpolation factor (0.0 to 1.0)"
;
; @test: r0=0, r1=0x4024000000000000, r2=0 -> r0=0
; @test: r0=0, r1=0x4024000000000000, r2=0x3FE0000000000000 -> r0=0x4014000000000000
; @test: r0=0, r1=0x4024000000000000, r2=0x3FF0000000000000 -> r0=0x4024000000000000
; @note: f64 bits. lerp(0, 10, 0)=0; lerp(0, 10, 0.5)=5; lerp(0, 10, 1.0)=10
;
; @export: lerp
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r15, r1  ; b
    mov r14, r0  ; a
    fpu.Fsub r15, r15, r14
    mov r14, r2  ; t
    fpu.Fmul r15, r15, r14
    fpu.Fadd r0, r0, r15
    halt
