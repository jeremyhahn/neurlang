; @name: Count Char
; @description: Count occurrences of character in string.
; @category: string
; @difficulty: 1
;
; @prompt: count {c} in {str}
; @prompt: how many {c} in {str}
; @prompt: count occurrences of {c} in {str}
; @prompt: number of {c} in string {str}
; @prompt: tally {c} in {str}
; @prompt: get count of {c} in {str}
; @prompt: count character {c} in {str}
; @prompt: occurrences of {c} in {str}
; @prompt: how many times does {c} appear in {str}
; @prompt: frequency of {c} in {str}
; @prompt: count instances of {c} in {str}
; @prompt: total {c} characters in {str}
; @prompt: count all {c} in string {str}
;
; @param: str=r0 "Pointer to null-terminated string"
; @param: c=r1 "Character to count"
;
; @test: r0=0x1000 r1=108 [0x1000]="hello" -> r0=2
;
; @export: count_char
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r2, r0  ; ptr
    mov r3, 0  ; 0
.while_0:
    nop
    mov r15, r2  ; p
    load.Byte r15, [r15]
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
    mov r15, r2  ; p
    load.Byte r15, [r15]
    mov r14, r1  ; c
    beq r15, r14, .set_6
    mov r15, 0
    b .cmp_end_7
.set_6:
    nop
    mov r15, 1
.cmp_end_7:
    nop
    beq r15, zero, .endif_5
    mov r15, 1  ; 1
    alu.Add r3, r3, r15
.endif_5:
    nop
    mov r15, 1  ; 1
    alu.Add r2, r2, r15
    b .while_0
.endwhile_1:
    nop
    mov r0, r3  ; count
    halt
