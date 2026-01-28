; @name: GCD
; @description: Calculate greatest common divisor using Euclidean algorithm
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
;
; @param: a=r0 "First number"
; @param: b=r1 "Second number"
;
; @test: r0=48 r1=18 -> r0=6
; @test: r0=100 r1=35 -> r0=5
; @test: r0=17 r1=13 -> r0=1

.entry main

main:
    ; Euclidean algorithm: while b != 0, (a, b) = (b, a % b)
.loop:
    ; if b == 0, return a
    beq r1, zero, .done

    ; temp = b
    mov r2, r1

    ; b = a % b
    muldiv.mod r1, r0, r1

    ; a = temp
    mov r0, r2

    b .loop

.done:
    ; result is in r0
    halt
