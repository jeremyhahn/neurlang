; @name: Extract Bits
; @description: Extract a bit field.
; @category: bitwise
; @difficulty: 2
;
; @prompt: extract {len} bits from {n} starting at position {start}
; @prompt: get bit field from {n} at offset {start} with length {len}
; @prompt: extract bits {start} to {start}+{len} from {n}
; @prompt: read {len} bits from {n} at bit {start}
; @prompt: get bits from position {start} of width {len} in {n}
; @prompt: extract a {len}-bit field from {n} starting at {start}
; @prompt: pull {len} bits out of {n} at offset {start}
; @prompt: slice {len} bits from {n} beginning at {start}
; @prompt: get bit range from {n} starting at {start} for {len} bits
; @prompt: extract bit field at position {start} width {len} from {n}
; @prompt: read bit slice from {n} at {start} with size {len}
; @prompt: get {len} bits from {n} offset by {start}
;
; @param: n=r0 "Source value to extract bits from"
; @param: start=r1 "Starting bit position (0-indexed from LSB)"
; @param: len=r2 "Number of bits to extract"
;
; @test: r0=0xFF00, r1=8, r2=8 -> r0=0xFF
; @test: r0=0xABCD, r1=0, r2=4 -> r0=0xD
; @test: r0=0xABCD, r1=4, r2=4 -> r0=0xC
; @test: r0=0xFFFFFFFF, r1=0, r2=32 -> r0=0xFFFFFFFF
;
; @export: extract_bits
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r14, r1  ; start
    mov r15, 64  ; 64
    bge r14, r15, .set_2
    mov r14, 0
    b .cmp_end_3
.set_2:
    nop
    mov r14, 1
.cmp_end_3:
    nop
    mov r15, r2  ; len
    mov r14, 0  ; 0
    beq r15, r14, .set_4
    mov r15, 0
    b .cmp_end_5
.set_4:
    nop
    mov r15, 1
.cmp_end_5:
    nop
    alu.Or r15, r15, r14
    bne r15, zero, .set_6
    mov r15, 0
    b .cmp_end_7
.set_6:
    nop
    mov r15, 1
.cmp_end_7:
    nop
    beq r15, zero, .endif_1
    mov r0, 0  ; 0
    halt
.endif_1:
    nop
    mov r15, r2  ; len
    mov r14, 64  ; 64
    bge r15, r14, .set_10
    mov r15, 0
    b .cmp_end_11
.set_10:
    nop
    mov r15, 1
.cmp_end_11:
    nop
    beq r15, zero, .cond_else_8
    mov r3, -1  ; 18446744073709551615
    b .cond_end_9
.cond_else_8:
    nop
    mov r3, 1  ; 1
    mov r15, r2  ; len
    alu.Shl r3, r3, r15
    mov r15, 1  ; 1
    alu.Sub r3, r3, r15
.cond_end_9:
    nop
    mov r15, r1  ; start
    alu.Shr r0, r0, r15
    mov r15, r3  ; mask
    alu.And r0, r0, r15
    halt
