; @name: Strrchr
; @description: Find last occurrence of character in string.
; @category: string
; @difficulty: 2
;
; @prompt: find last {c} in {str}
; @prompt: locate last occurrence of {c} in {str}
; @prompt: search for last {c} in {str}
; @prompt: get index of last {c} in {str}
; @prompt: position of final {c} in {str}
; @prompt: where is last {c} in {str}
; @prompt: find rightmost {c} in {str}
; @prompt: strrchr for {c} in {str}
; @prompt: index of last character {c} in {str}
; @prompt: locate final {c} in {str}
; @prompt: search string {str} for last {c}
; @prompt: find position of last {c} in {str}
; @prompt: get offset of rightmost {c} in {str}
;
; @param: str=r0 "Pointer to null-terminated string to search"
; @param: c=r1 "Character to find"
;
; @test: r0=0x1000 r1=108 [0x1000]="hello" -> r0=3
; @test: r0=0x1000 r1=122 [0x1000]="hello" -> r0=-1
; @test: r0=0x1000 r1=104 [0x1000]="hello" -> r0=0
;
; @export: strrchr
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r2, r0  ; ptr
    mov r3, 0  ; 0
    mov r4, -1  ; 18446744073709551615
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
    mov r4, r3  ; idx
.endif_5:
    nop
    mov r15, 1  ; 1
    alu.Add r2, r2, r15
    mov r15, 1  ; 1
    alu.Add r3, r3, r15
    b .while_0
.endwhile_1:
    nop
    mov r0, r4  ; last
    halt
