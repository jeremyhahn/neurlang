; @name: Ctz
; @description: Count trailing zeros.
; @category: bitwise
;
; @prompt: count trailing zeros in {n}
; @prompt: how many trailing zero bits in {n}
; @prompt: ctz of {n}
; @prompt: get the number of trailing zeros in {n}
; @prompt: find trailing zero count of {n}
; @prompt: count zeros from the least significant bit in {n}
; @prompt: trailing zero count for {n}
; @prompt: number of zero bits at the end of {n}
; @prompt: count trailing 0s in {n}
; @prompt: get ctz for value {n}
; @prompt: find how many zeros trail {n}
; @prompt: compute trailing zero count of {n}
; @prompt: count zeros after last set bit in {n}
;
; @param: n=r0 "The value to count trailing zeros in"
;
; @test: r0=0 -> r0=64
; @test: r0=1 -> r0=0
; @test: r0=8 -> r0=3
; @test: r0=0x8000000000000000 -> r0=63
;
; @export: ctz
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
    mov r0, 64  ; 64
    halt
.endif_1:
    nop
    mov r1, 0  ; 0
    mov r2, r0  ; n
    mov r3, 1  ; 1
    mov r15, 32  ; 32
    alu.Shl r3, r3, r15
    mov r15, 1  ; 1
    alu.Sub r3, r3, r15
    mov r4, r2  ; val
    mov r15, r3  ; mask32
    alu.And r4, r4, r15
    mov r15, r4  ; low32
    mov r14, 0  ; 0
    beq r15, r14, .set_6
    mov r15, 0
    b .cmp_end_7
.set_6:
    nop
    mov r15, 1
.cmp_end_7:
    nop
    beq r15, zero, .endif_5
    mov r15, 32  ; 32
    alu.Add r1, r1, r15
    mov r15, 32  ; 32
    alu.Shr r2, r2, r15
.endif_5:
    nop
    mov r5, 1  ; 1
    mov r15, 16  ; 16
    alu.Shl r5, r5, r15
    mov r15, 1  ; 1
    alu.Sub r5, r5, r15
    mov r6, r2  ; val
    mov r15, r5  ; mask16
    alu.And r6, r6, r15
    mov r15, r6  ; low16
    mov r14, 0  ; 0
    beq r15, r14, .set_10
    mov r15, 0
    b .cmp_end_11
.set_10:
    nop
    mov r15, 1
.cmp_end_11:
    nop
    beq r15, zero, .endif_9
    mov r15, 16  ; 16
    alu.Add r1, r1, r15
    mov r15, 16  ; 16
    alu.Shr r2, r2, r15
.endif_9:
    nop
    mov r7, 1  ; 1
    mov r15, 8  ; 8
    alu.Shl r7, r7, r15
    mov r15, 1  ; 1
    alu.Sub r7, r7, r15
    mov r8, r2  ; val
    mov r15, r7  ; mask8
    alu.And r8, r8, r15
    mov r15, r8  ; low8
    mov r14, 0  ; 0
    beq r15, r14, .set_14
    mov r15, 0
    b .cmp_end_15
.set_14:
    nop
    mov r15, 1
.cmp_end_15:
    nop
    beq r15, zero, .endif_13
    mov r15, 8  ; 8
    alu.Add r1, r1, r15
    mov r15, 8  ; 8
    alu.Shr r2, r2, r15
.endif_13:
    nop
    mov r9, 1  ; 1
    mov r15, 4  ; 4
    alu.Shl r9, r9, r15
    mov r15, 1  ; 1
    alu.Sub r9, r9, r15
    mov r10, r2  ; val
    mov r15, r9  ; mask4
    alu.And r10, r10, r15
    mov r15, r10  ; low4
    mov r14, 0  ; 0
    beq r15, r14, .set_18
    mov r15, 0
    b .cmp_end_19
.set_18:
    nop
    mov r15, 1
.cmp_end_19:
    nop
    beq r15, zero, .endif_17
    mov r15, 4  ; 4
    alu.Add r1, r1, r15
    mov r15, 4  ; 4
    alu.Shr r2, r2, r15
.endif_17:
    nop
    mov r11, 1  ; 1
    mov r15, 2  ; 2
    alu.Shl r11, r11, r15
    mov r15, 1  ; 1
    alu.Sub r11, r11, r15
    mov r12, r2  ; val
    mov r15, r11  ; mask2
    alu.And r12, r12, r15
    mov r15, r12  ; low2
    mov r14, 0  ; 0
    beq r15, r14, .set_22
    mov r15, 0
    b .cmp_end_23
.set_22:
    nop
    mov r15, 1
.cmp_end_23:
    nop
    beq r15, zero, .endif_21
    mov r15, 2  ; 2
    alu.Add r1, r1, r15
    mov r15, 2  ; 2
    alu.Shr r2, r2, r15
.endif_21:
    nop
    mov r13, r2  ; val
    mov r15, 1  ; 1
    alu.And r13, r13, r15
    mov r15, r13  ; low1
    mov r14, 0  ; 0
    beq r15, r14, .set_26
    mov r15, 0
    b .cmp_end_27
.set_26:
    nop
    mov r15, 1
.cmp_end_27:
    nop
    beq r15, zero, .endif_25
    mov r15, 1  ; 1
    alu.Add r1, r1, r15
.endif_25:
    nop
    mov r0, r1  ; count
    halt
