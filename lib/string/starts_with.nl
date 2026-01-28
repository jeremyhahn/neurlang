; @name: Starts With
; @description: Check if string starts with prefix.
; @category: string
; @difficulty: 2
;
; @prompt: does {str} start with {prefix}
; @prompt: check if {str} begins with {prefix}
; @prompt: test if {str} has prefix {prefix}
; @prompt: verify {str} starts with {prefix}
; @prompt: is {prefix} a prefix of {str}
; @prompt: check string {str} starts with {prefix}
; @prompt: does {str} begin with {prefix}
; @prompt: has prefix {prefix} in {str}
; @prompt: starts with check for {prefix} in {str}
; @prompt: determine if {str} starts with {prefix}
; @prompt: test prefix {prefix} against {str}
; @prompt: is {str} prefixed by {prefix}
; @prompt: check beginning of {str} for {prefix}
;
; @param: str=r0 "Pointer to string to check"
; @param: prefix=r1 "Pointer to prefix string"
;
; @test: r0=0x1000 r1=0x1100 [0x1000]="hello" [0x1100]="hel" -> r0=1
; @test: r0=0x1000 r1=0x1100 [0x1000]="hello" [0x1100]="xyz" -> r0=0
; @test: r0=0x1000 r1=0x1100 [0x1000]="hello" [0x1100]="" -> r0=1
;
; @export: starts_with
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r2, r0  ; str_ptr
    mov r3, r1  ; prefix_ptr
.loop_0:
    nop
    mov r4, r3  ; pp
    load.Byte r4, [r4]
    mov r15, r4  ; cp
    mov r14, 0  ; 0
    beq r15, r14, .set_4
    mov r15, 0
    b .cmp_end_5
.set_4:
    nop
    mov r15, 1
.cmp_end_5:
    nop
    beq r15, zero, .endif_3
    mov r0, 1  ; 1
    halt
.endif_3:
    nop
    mov r5, r2  ; ps
    load.Byte r5, [r5]
    mov r15, r5  ; cs
    mov r14, r4  ; cp
    bne r15, r14, .set_8
    mov r15, 0
    b .cmp_end_9
.set_8:
    nop
    mov r15, 1
.cmp_end_9:
    nop
    beq r15, zero, .endif_7
    mov r0, 0  ; 0
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
