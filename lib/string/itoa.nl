; @name: Itoa
; @description: Convert unsigned integer to string (itoa).
; @category: string
; @difficulty: 2
;
; @prompt: convert integer {n} to string at {dst}
; @prompt: itoa of {n} into {dst}
; @prompt: write number {n} as string to {dst}
; @prompt: integer {n} to string at {dst}
; @prompt: format number {n} to buffer {dst}
; @prompt: convert {n} to decimal string in {dst}
; @prompt: stringify integer {n} at {dst}
; @prompt: render number {n} as text to {dst}
; @prompt: int to string for {n} into {dst}
; @prompt: write integer {n} to string buffer {dst}
; @prompt: transform {n} to string at {dst}
; @prompt: encode integer {n} as string in {dst}
; @prompt: number to text conversion of {n} to {dst}
;
; @param: n=r0 "Unsigned integer to convert"
; @param: dst=r1 "Pointer to destination buffer"
;
; @test: r0=0 r1=0x2000 -> r0=1
; @test: r0=123 r1=0x2000 -> r0=3
; @test: r0=9 r1=0x2000 -> r0=1
;
; @export: itoa
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
    mov r15, r1  ; dst
    mov r14, 48  ; 48
    store.Byte r14, [r15]
    mov r15, r1  ; dst
    mov r14, 1  ; 1
    alu.Add r15, r15, r14
    mov r14, 0  ; 0
    store.Byte r14, [r15]
    mov r0, 1  ; 1
    halt
.endif_1:
    nop
    mov r2, r0  ; n
    mov r3, 0  ; 0
.while_4:
    nop
    mov r15, r2  ; temp
    mov r14, 0  ; 0
    bgt r15, r14, .set_6
    mov r15, 0
    b .cmp_end_7
.set_6:
    nop
    mov r15, 1
.cmp_end_7:
    nop
    beq r15, zero, .endwhile_5
    mov r15, 1  ; 1
    alu.Add r3, r3, r15
    mov r15, 10  ; 10
    muldiv.Div r2, r2, r15
    b .while_4
.endwhile_5:
    nop
    mov r4, r3  ; len
.while_8:
    nop
    mov r15, r0  ; n
    mov r14, 0  ; 0
    bgt r15, r14, .set_10
    mov r15, 0
    b .cmp_end_11
.set_10:
    nop
    mov r15, 1
.cmp_end_11:
    nop
    beq r15, zero, .endwhile_9
    mov r15, 1  ; 1
    alu.Sub r4, r4, r15
    mov r15, r1  ; dst
    mov r14, r4  ; i
    alu.Add r15, r15, r14
    mov r14, r0  ; n
    mov r14, 10  ; 10
    muldiv.Mod r15, r15, r14
    mov r14, 48  ; 48
    alu.Add r14, r14, r15
    store.Byte r14, [r15]
    mov r15, 10  ; 10
    muldiv.Div r0, r0, r15
    b .while_8
.endwhile_9:
    nop
    mov r15, r1  ; dst
    mov r14, r3  ; len
    alu.Add r15, r15, r14
    mov r14, 0  ; 0
    store.Byte r14, [r15]
    mov r0, r3  ; len
    halt
