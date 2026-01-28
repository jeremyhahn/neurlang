; @name: Strchr
; @description: Find first occurrence of character in string.
; @category: string
; @difficulty: 2
;
; @prompt: find character {c} in {str}
; @prompt: locate {c} in string {str}
; @prompt: search for {c} in {str}
; @prompt: get index of {c} in {str}
; @prompt: position of {c} in {str}
; @prompt: where is {c} in {str}
; @prompt: find first occurrence of {c} in {str}
; @prompt: strchr for {c} in {str}
; @prompt: index of character {c} in {str}
; @prompt: locate first {c} in {str}
; @prompt: search string {str} for {c}
; @prompt: find position of {c} in {str}
; @prompt: get offset of {c} in {str}
;
; @param: str=r0 "Pointer to null-terminated string to search"
; @param: c=r1 "Character to find"
;
; @test: r0=0x1000 r1=108 [0x1000]="hello" -> r0=2
; @test: r0=0x1000 r1=122 [0x1000]="hello" -> r0=-1
; @test: r0=0x1000 r1=104 [0x1000]="hello" -> r0=0
;
; @export: strchr
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
    mov r0, r3  ; idx
    halt
.endif_5:
    nop
    mov r15, 1  ; 1
    alu.Add r2, r2, r15
    mov r15, 1  ; 1
    alu.Add r3, r3, r15
    b .while_0
.endwhile_1:
    nop
    mov r0, -1  ; 18446744073709551615
    halt
