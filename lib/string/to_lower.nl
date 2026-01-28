; @name: To Lower
; @description: Convert uppercase letter to lowercase.
; @category: string
; @difficulty: 1
;
; @prompt: convert {c} to lowercase
; @prompt: make {c} lowercase
; @prompt: lowercase character {c}
; @prompt: transform {c} to lower case
; @prompt: uncapitalize character {c}
; @prompt: change {c} to lowercase
; @prompt: tolower for {c}
; @prompt: get lowercase of {c}
; @prompt: convert uppercase {c} to lower
; @prompt: make letter {c} lowercase
; @prompt: shift {c} to lowercase
; @prompt: lower case conversion of {c}
; @prompt: convert {c} from upper to lower
;
; @param: c=r0 "ASCII character to convert"
;
; @test: r0=65 -> r0=97
; @test: r0=90 -> r0=122
; @test: r0=97 -> r0=97
;
; @export: to_lower
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r15, r0  ; c
    mov r14, 65  ; 65
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
    mov r14, 90  ; 90
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
    alu.Add r0, r0, r15
    halt
