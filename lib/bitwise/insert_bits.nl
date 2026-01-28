; @name: Insert Bits
; @description: Insert bits into a value.
; @category: bitwise
; @difficulty: 2
;
; @prompt: insert {len} bits of {src} into {dest} at position {start}
; @prompt: set bit field in {dest} with {src} at offset {start} length {len}
; @prompt: write {len} bits from {src} to {dest} at bit {start}
; @prompt: place {src} bits into {dest} at position {start} for {len} bits
; @prompt: insert a {len}-bit field from {src} into {dest} at {start}
; @prompt: put {len} bits of {src} into {dest} starting at {start}
; @prompt: overwrite {len} bits in {dest} with {src} at offset {start}
; @prompt: set bits {start} to {start}+{len} of {dest} with {src}
; @prompt: insert bit field {src} into {dest} at {start} width {len}
; @prompt: write bit slice from {src} to {dest} at {start} with size {len}
; @prompt: embed {len} bits from {src} into {dest} at position {start}
; @prompt: modify {dest} by inserting {src} at bit {start} for {len} bits
;
; @param: dest=r0 "Destination value to insert bits into"
; @param: src=r1 "Source bits to insert"
; @param: start=r2 "Starting bit position (0-indexed from LSB)"
; @param: len=r3 "Number of bits to insert"
;
; @test: r0=0x0000, r1=0xFF, r2=8, r3=8 -> r0=0xFF00
; @test: r0=0xFFFF, r1=0x00, r2=4, r3=4 -> r0=0xFF0F
; @test: r0=0x0000, r1=0xAB, r2=0, r3=8 -> r0=0xAB
;
; @export: insert_bits
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r14, r2  ; start
    mov r15, 64  ; 64
    bge r14, r15, .set_2
    mov r14, 0
    b .cmp_end_3
.set_2:
    nop
    mov r14, 1
.cmp_end_3:
    nop
    mov r15, r3  ; len
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
    halt
.endif_1:
    nop
    mov r15, r3  ; len
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
    mov r4, -1  ; 18446744073709551615
    b .cond_end_9
.cond_else_8:
    nop
    mov r4, 1  ; 1
    mov r15, r3  ; len
    alu.Shl r4, r4, r15
    mov r15, 1  ; 1
    alu.Sub r4, r4, r15
.cond_end_9:
    nop
    mov r5, r4  ; mask
    mov r15, r2  ; start
    alu.Shl r5, r5, r15
    alui.Xor r5, r5, -1
    ; r5 = clear_mask = ~(mask << start)
    mov r6, r1  ; src
    mov r15, r4  ; mask
    alu.And r6, r6, r15
    mov r15, r2  ; start
    alu.Shl r6, r6, r15
    ; r6 = shifted_src = (src & mask) << start
    mov r15, r5  ; clear_mask
    alu.And r0, r0, r15
    ; r0 = dest & clear_mask
    mov r15, r6  ; shifted_src
    alu.Or r0, r0, r15
    ; r0 = (dest & clear_mask) | shifted_src
    halt
