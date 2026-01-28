; @name: Strlen
; @description: Calculate string length (null-terminated).
; @category: string
; @difficulty: 1
;
; @prompt: get the length of {str}
; @prompt: count characters in {str}
; @prompt: how many characters in {str}
; @prompt: find string length of {str}
; @prompt: calculate the size of {str}
; @prompt: measure the length of string {str}
; @prompt: get character count of {str}
; @prompt: return length of {str}
; @prompt: compute string size for {str}
; @prompt: determine how long {str} is
; @prompt: what is the length of {str}
; @prompt: count bytes in string {str}
; @prompt: strlen of {str}
;
; @param: str=r0 "Pointer to null-terminated string"
;
; @test: r0=0x1000 [0x1000]="hello" -> r0=5
; @test: r0=0x1000 [0x1000]="" -> r0=0
; @test: r0=0x1000 [0x1000]="a" -> r0=1
;
; @export: strlen
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r1, 0  ; 0
    mov r2, r0  ; ptr
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
    mov r15, 1  ; 1
    alu.Add r1, r1, r15  ; len += 1
    mov r15, 1  ; 1 (byte size)
    alu.Add r2, r2, r15  ; ptr += 1 (bytes, not words)
    b .while_0
.endwhile_1:
    nop
    mov r0, r1  ; len
    halt
