; @name: Clz
; @description: Count leading zeros.
; @category: bitwise
;
; @prompt: count leading zeros in {n}
; @prompt: how many leading zero bits in {n}
; @prompt: clz of {n}
; @prompt: get the number of leading zeros in {n}
; @prompt: find leading zero count of {n}
; @prompt: count zeros from the most significant bit in {n}
; @prompt: leading zero count for {n}
; @prompt: number of zero bits at the start of {n}
; @prompt: count leading 0s in {n}
; @prompt: get clz for value {n}
; @prompt: find how many zeros lead {n}
; @prompt: compute leading zero count of {n}
; @prompt: count zeros before first set bit in {n}
;
; @param: n=r0 "The value to count leading zeros in"
;
; @test: r0=0 -> r0=64
; @test: r0=1 -> r0=63
; @test: r0=0x8000000000000000 -> r0=0
; @test: r0=0xFF -> r0=56
;
; @export: clz
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
    mov r15, r2  ; val
    mov r14, 32  ; 32
    alu.Shr r15, r15, r14
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
    alu.Shl r2, r2, r15
.endif_5:
    nop
    mov r15, r2  ; val
    mov r14, 48  ; 48
    alu.Shr r15, r15, r14
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
    alu.Shl r2, r2, r15
.endif_9:
    nop
    mov r15, r2  ; val
    mov r14, 56  ; 56
    alu.Shr r15, r15, r14
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
    alu.Shl r2, r2, r15
.endif_13:
    nop
    mov r15, r2  ; val
    mov r14, 60  ; 60
    alu.Shr r15, r15, r14
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
    alu.Shl r2, r2, r15
.endif_17:
    nop
    mov r15, r2  ; val
    mov r14, 62  ; 62
    alu.Shr r15, r15, r14
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
    alu.Shl r2, r2, r15
.endif_21:
    nop
    mov r15, r2  ; val
    mov r14, 63  ; 63
    alu.Shr r15, r15, r14
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
