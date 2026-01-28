; @name: Bswap
; @description: Byte swap (reverse byte order for endian conversion).
; @category: bitwise
;
; @prompt: swap bytes in {n}
; @prompt: reverse byte order of {n}
; @prompt: convert endianness of {n}
; @prompt: byte swap {n}
; @prompt: bswap {n}
; @prompt: flip byte order in {n}
; @prompt: convert {n} from big endian to little endian
; @prompt: convert {n} from little endian to big endian
; @prompt: reverse the bytes of {n}
; @prompt: swap byte order for {n}
; @prompt: change endianness of {n}
; @prompt: perform byte reversal on {n}
; @prompt: endian swap {n}
;
; @param: n=r0 "The value to byte swap"
;
; @test:  -> r0=0
;
; @export: bswap
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r1, r0  ; n
    mov r15, 255  ; 255
    alu.And r1, r1, r15
    mov r15, 56  ; 56
    alu.Shl r1, r1, r15
    mov r2, r0  ; n
    mov r15, 8  ; 8
    alu.Shr r2, r2, r15
    mov r15, 255  ; 255
    alu.And r2, r2, r15
    mov r15, 48  ; 48
    alu.Shl r2, r2, r15
    mov r3, r0  ; n
    mov r15, 16  ; 16
    alu.Shr r3, r3, r15
    mov r15, 255  ; 255
    alu.And r3, r3, r15
    mov r15, 40  ; 40
    alu.Shl r3, r3, r15
    mov r4, r0  ; n
    mov r15, 24  ; 24
    alu.Shr r4, r4, r15
    mov r15, 255  ; 255
    alu.And r4, r4, r15
    mov r15, 32  ; 32
    alu.Shl r4, r4, r15
    mov r5, r0  ; n
    mov r15, 32  ; 32
    alu.Shr r5, r5, r15
    mov r15, 255  ; 255
    alu.And r5, r5, r15
    mov r15, 24  ; 24
    alu.Shl r5, r5, r15
    mov r6, r0  ; n
    mov r15, 40  ; 40
    alu.Shr r6, r6, r15
    mov r15, 255  ; 255
    alu.And r6, r6, r15
    mov r15, 16  ; 16
    alu.Shl r6, r6, r15
    mov r7, r0  ; n
    mov r15, 48  ; 48
    alu.Shr r7, r7, r15
    mov r15, 255  ; 255
    alu.And r7, r7, r15
    mov r15, 8  ; 8
    alu.Shl r7, r7, r15
    mov r8, r0  ; n
    mov r15, 56  ; 56
    alu.Shr r8, r8, r15
    mov r15, 255  ; 255
    alu.And r8, r8, r15
    mov r0, r1  ; b0
    mov r15, r2  ; b1
    alu.Or r0, r0, r15
    mov r15, r3  ; b2
    alu.Or r0, r0, r15
    mov r15, r4  ; b3
    alu.Or r0, r0, r15
    mov r15, r5  ; b4
    alu.Or r0, r0, r15
    mov r15, r6  ; b5
    alu.Or r0, r0, r15
    mov r15, r7  ; b6
    alu.Or r0, r0, r15
    mov r15, r8  ; b7
    alu.Or r0, r0, r15
    halt
