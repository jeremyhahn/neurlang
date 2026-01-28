; @name: Gcd
; @description: Calculate greatest common divisor using Euclidean algorithm.
; @category: algorithm/math
; @difficulty: 2
;
; @prompt: find GCD of {a} and {b}
; @prompt: gcd({a}, {b})
; @prompt: greatest common divisor of {a} and {b}
; @prompt: euclidean algorithm for {a}, {b}
; @prompt: compute gcd of {a} {b}
; @prompt: what is the GCD of {a} and {b}
; @prompt: calculate greatest common divisor {a} {b}
; @prompt: find common divisor of {a} and {b}
; @prompt: highest common factor of {a} and {b}
; @prompt: HCF of {a} and {b}
; @prompt: largest divisor common to {a} and {b}
; @prompt: compute euclidean gcd({a}, {b})
;
; @param: a=r0 "First number"
; @param: b=r1 "Second number"
;
; @test: r0=48 r1=18 -> r0=6
; @test: r0=100 r1=35 -> r0=5
; @test: r0=17 r1=13 -> r0=1
; @test: r0=0 r1=5 -> r0=5
; @test: r0=12 r1=0 -> r0=12
;
; @export: gcd
; Generated from Rust by nl stdlib build

.entry:
    nop
.while_0:
    nop
    mov r15, r1  ; b
    mov r14, 0  ; 0
    bne r15, r14, .set_2
    mov r15, 0
    b .cmp_end_3
.set_2:
    nop
    mov r15, 1
.cmp_end_3:
    nop
    beq r15, zero, .endwhile_1
    mov r2, r1  ; b
    mov r15, r1  ; b
    mov r1, r0  ; a
    muldiv.Mod r1, r1, r15
    mov r0, r2  ; temp
    b .while_0
.endwhile_1:
    nop
    halt
