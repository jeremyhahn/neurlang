; @name: Next Power Of 2
; @description: Round up to next power of 2.
; @category: bitwise
; @difficulty: 2
;
; @prompt: round {n} up to the next power of 2
; @prompt: get the next power of 2 greater than or equal to {n}
; @prompt: find smallest power of 2 >= {n}
; @prompt: round {n} to next higher power of two
; @prompt: ceiling power of 2 for {n}
; @prompt: next power of 2 for {n}
; @prompt: round up {n} to power of 2
; @prompt: find the smallest power of 2 not less than {n}
; @prompt: get next highest power of 2 from {n}
; @prompt: compute ceiling power of two for {n}
; @prompt: round {n} up to nearest power of 2
; @prompt: next greater or equal power of 2 for {n}
; @prompt: find power of 2 ceiling of {n}
;
; @param: n=r0 "The value to round up"
;
; @test: r0=5 -> r0=8
; @test: r0=8 -> r0=8
; @test: r0=1 -> r0=1
;
; @export: next_power_of_2
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
    mov r0, 1  ; 1
    halt
.endif_1:
    nop
    mov r1, r0  ; n
    mov r15, 1  ; 1
    alu.Sub r1, r1, r15
    mov r15, r1  ; v
    mov r14, 1  ; 1
    alu.Shr r15, r15, r14
    alu.Or r1, r1, r15
    mov r15, r1  ; v
    mov r14, 2  ; 2
    alu.Shr r15, r15, r14
    alu.Or r1, r1, r15
    mov r15, r1  ; v
    mov r14, 4  ; 4
    alu.Shr r15, r15, r14
    alu.Or r1, r1, r15
    mov r15, r1  ; v
    mov r14, 8  ; 8
    alu.Shr r15, r15, r14
    alu.Or r1, r1, r15
    mov r15, r1  ; v
    mov r14, 16  ; 16
    alu.Shr r15, r15, r14
    alu.Or r1, r1, r15
    mov r15, r1  ; v
    mov r14, 32  ; 32
    alu.Shr r15, r15, r14
    alu.Or r1, r1, r15
    mov r0, r1  ; v
    mov r15, 1  ; 1
    alu.Add r0, r0, r15
    halt
