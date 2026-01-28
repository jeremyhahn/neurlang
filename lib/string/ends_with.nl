; @name: Ends With
; @description: Check if string ends with suffix.
; @category: string
; @difficulty: 2
;
; @prompt: does {str} end with {suffix}
; @prompt: check if {str} ends with {suffix}
; @prompt: test if {str} has suffix {suffix}
; @prompt: verify {str} ends with {suffix}
; @prompt: is {suffix} a suffix of {str}
; @prompt: check string {str} ends with {suffix}
; @prompt: does {str} finish with {suffix}
; @prompt: has suffix {suffix} in {str}
; @prompt: ends with check for {suffix} in {str}
; @prompt: determine if {str} ends with {suffix}
; @prompt: test suffix {suffix} against {str}
; @prompt: is {str} suffixed by {suffix}
; @prompt: check ending of {str} for {suffix}
;
; @param: str=r0 "Pointer to string to check"
; @param: suffix=r1 "Pointer to suffix string"
;
; @test: r0=0x1000 r1=0x1100 [0x1000]="hello" [0x1100]="llo" -> r0=1
; @test: r0=0x1000 r1=0x1100 [0x1000]="hello" [0x1100]="xyz" -> r0=0
; @test: r0=0x1000 r1=0x1100 [0x1000]="hello" [0x1100]="" -> r0=1
;
; @export: ends_with
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r2, 0  ; 0
    mov r3, r0  ; str_ptr
.while_0:
    nop
    mov r15, r3  ; p
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
    alu.Add r2, r2, r15
    mov r15, 1  ; 1
    alu.Add r3, r3, r15
    b .while_0
.endwhile_1:
    nop
    mov r4, 0  ; 0
    mov r3, r1  ; suffix_ptr
.while_4:
    nop
    mov r15, r3  ; p
    load.Byte r15, [r15]
    mov r14, 0  ; 0
    bne r15, r14, .set_6
    mov r15, 0
    b .cmp_end_7
.set_6:
    nop
    mov r15, 1
.cmp_end_7:
    nop
    beq r15, zero, .endwhile_5
    mov r15, 1  ; 1
    alu.Add r4, r4, r15
    mov r15, 1  ; 1
    alu.Add r3, r3, r15
    b .while_4
.endwhile_5:
    nop
    mov r15, r4  ; suffix_len
    mov r14, r2  ; str_len
    bgt r15, r14, .set_10
    mov r15, 0
    b .cmp_end_11
.set_10:
    nop
    mov r15, 1
.cmp_end_11:
    nop
    beq r15, zero, .endif_9
    mov r0, 0  ; 0
    halt
.endif_9:
    nop
    mov r5, r2  ; str_len
    mov r15, r4  ; suffix_len
    alu.Sub r5, r5, r15
    mov r6, 0  ; 0
.while_12:
    nop
    mov r15, r6  ; i
    mov r14, r4  ; suffix_len
    blt r15, r14, .set_14
    mov r15, 0
    b .cmp_end_15
.set_14:
    nop
    mov r15, 1
.cmp_end_15:
    nop
    beq r15, zero, .endwhile_13
    ; Load suffix[i]
    mov r14, r1  ; suffix_ptr
    alu.Add r14, r14, r6  ; suffix_ptr + i
    load.Byte r14, [r14]  ; r14 = suffix[i]
    ; Load str[start + i]
    mov r15, r0  ; str_ptr
    mov r13, r5  ; start
    alu.Add r13, r13, r6  ; start + i
    alu.Add r15, r15, r13  ; str_ptr + start + i
    load.Byte r15, [r15]  ; r15 = str[start + i]
    ; Compare
    bne r15, r14, .set_18
    mov r15, 0
    b .cmp_end_19
.set_18:
    nop
    mov r15, 1
.cmp_end_19:
    nop
    beq r15, zero, .endif_17
    mov r0, 0  ; 0
    halt
.endif_17:
    nop
    mov r15, 1  ; 1
    alu.Add r6, r6, r15
    b .while_12
.endwhile_13:
    nop
    mov r0, 1  ; 1
    halt
