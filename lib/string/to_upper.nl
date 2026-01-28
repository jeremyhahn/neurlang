; @name: To Upper
; @description: Convert lowercase letter to uppercase.
; @category: string
; @difficulty: 1
;
; @prompt: convert {c} to uppercase
; @prompt: make {c} uppercase
; @prompt: uppercase character {c}
; @prompt: transform {c} to upper case
; @prompt: capitalize character {c}
; @prompt: change {c} to uppercase
; @prompt: toupper for {c}
; @prompt: get uppercase of {c}
; @prompt: convert lowercase {c} to upper
; @prompt: make letter {c} uppercase
; @prompt: shift {c} to uppercase
; @prompt: upper case conversion of {c}
; @prompt: convert {c} from lower to upper
;
; @param: c=r0 "ASCII character to convert"
;
; @test: r0=97 -> r0=65
; @test: r0=122 -> r0=90
; @test: r0=65 -> r0=65
;
; @export: to_upper
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r15, r0  ; c
    mov r14, 97  ; 97
    blt r15, r14, .set_2
    mov r15, 0
    b .cmp_end_3
.set_2:
    nop
    mov r15, 1
.cmp_end_3:
    nop
    beq r15, zero, .endif_1
    halt
.endif_1:
    nop
    mov r15, r0  ; c
    mov r14, 122  ; 122
    bgt r15, r14, .set_6
    mov r15, 0
    b .cmp_end_7
.set_6:
    nop
    mov r15, 1
.cmp_end_7:
    nop
    beq r15, zero, .endif_5
    halt
.endif_5:
    nop
    mov r15, 32  ; 32
    alu.Sub r0, r0, r15
    halt
