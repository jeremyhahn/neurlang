; @name: Is Power Of 2
; @description: Check if exactly one bit is set (power of 2 check).
; @category: bitwise
; @difficulty: 1
;
; @prompt: check if {n} is a power of 2
; @prompt: is {n} a power of two
; @prompt: test if {n} has exactly one bit set
; @prompt: determine if {n} is a power of 2
; @prompt: check whether {n} is power of 2
; @prompt: is {n} an exact power of 2
; @prompt: verify {n} is a power of two
; @prompt: test power of 2 for {n}
; @prompt: check if only one bit is set in {n}
; @prompt: is {n} a binary power
; @prompt: determine whether {n} is 2^k for some k
; @prompt: check if {n} equals 2 to some power
; @prompt: validate {n} as power of 2
;
; @param: n=r0 "The value to check"
;
; @test: r0=0 -> r0=0
; @test: r0=1 -> r0=1
; @test: r0=16 -> r0=1
; @test: r0=17 -> r0=0
;
; @export: is_power_of_2
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r15, r0  ; n
    mov r14, 0  ; 0
    beq r15, r14, .set_2
    mov r15, 0
    b .cmp_end_3
.set_2:
    nop
    mov r15, 1
.cmp_end_3:
    nop
    beq r15, zero, .endif_1
    mov r0, 0  ; 0
    halt
.endif_1:
    nop
    mov r14, r0  ; n
    mov r15, 1  ; 1
    alu.Sub r14, r14, r15
    mov r15, r0  ; n
    alu.And r15, r15, r14
    mov r14, 0  ; 0
    beq r15, r14, .set_6
    mov r15, 0
    b .cmp_end_7
.set_6:
    nop
    mov r15, 1
.cmp_end_7:
    nop
    beq r15, zero, .else_4
    mov r0, 1  ; 1
    b .endif_5
.else_4:
    nop
    mov r0, 0  ; 0
.endif_5:
    nop
    halt
