; @name: Strcmp
; @description: Compare two strings for equality.
; @category: string
; @difficulty: 1
;
; @prompt: compare {str1} and {str2}
; @prompt: check if {str1} equals {str2}
; @prompt: are strings {str1} and {str2} the same
; @prompt: test string equality of {str1} and {str2}
; @prompt: do {str1} and {str2} match
; @prompt: is {str1} equal to {str2}
; @prompt: compare strings {str1} with {str2}
; @prompt: check string equality between {str1} and {str2}
; @prompt: determine if {str1} is identical to {str2}
; @prompt: verify {str1} matches {str2}
; @prompt: strcmp {str1} and {str2}
; @prompt: are {str1} and {str2} equal strings
; @prompt: test if {str1} == {str2}
;
; @param: str1=r0 "Pointer to first null-terminated string"
; @param: str2=r1 "Pointer to second null-terminated string"
;
; @test: r0=0x1000 r1=0x1100 [0x1000]="abc" [0x1100]="abc" -> r0=1
; @test: r0=0x1000 r1=0x1100 [0x1000]="abc" [0x1100]="abd" -> r0=0
; @test: r0=0x1000 r1=0x1100 [0x1000]="" [0x1100]="" -> r0=1
;
; @export: strcmp
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r2, r0  ; a
    mov r3, r1  ; b
.loop_0:
    nop
    mov r4, r2  ; pa
    load.Byte r4, [r4]
    mov r5, r3  ; pb
    load.Byte r5, [r5]
    mov r15, r4  ; ca
    mov r14, r5  ; cb
    bne r15, r14, .set_4
    mov r15, 0
    b .cmp_end_5
.set_4:
    nop
    mov r15, 1
.cmp_end_5:
    nop
    beq r15, zero, .endif_3
    mov r0, 0  ; 0
    halt
.endif_3:
    nop
    mov r15, r4  ; ca
    mov r14, 0  ; 0
    beq r15, r14, .set_8
    mov r15, 0
    b .cmp_end_9
.set_8:
    nop
    mov r15, 1
.cmp_end_9:
    nop
    beq r15, zero, .endif_7
    mov r0, 1  ; 1
    halt
.endif_7:
    nop
    mov r15, 1  ; 1
    alu.Add r2, r2, r15
    mov r15, 1  ; 1
    alu.Add r3, r3, r15
    b .loop_0
.endloop_1:
    nop
    halt
