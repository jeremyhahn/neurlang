; @name: Array Eq
; @description: Check if two arrays are equal.
; @category: array
; @difficulty: 1
;
; @prompt: check if arrays {a} and {b} are equal with {len} elements
; @prompt: compare {a} and {b} arrays of size {len} for equality
; @prompt: test if {len} elements in {a} match {b}
; @prompt: are arrays {a} and {b} identical for {len} items
; @prompt: verify equality of {a} and {b} over {len} values
; @prompt: compare {len} element arrays {a} and {b}
; @prompt: check {a} equals {b} for {len} entries
; @prompt: test array equality between {a} and {b} of length {len}
; @prompt: do {len} elements of {a} equal those in {b}
; @prompt: memcmp {a} and {b} for {len} u64 values
; @prompt: compare {a} to {b} element by element for {len} items
; @prompt: return true if {a} and {b} match across {len} elements
;
; @param: a=r0 "Pointer to first array"
; @param: b=r1 "Pointer to second array"
; @param: len=r2 "Number of elements to compare"
;
; @test: r0=0, r1=0, r2=0 -> r0=1
; @note: Returns 1 if equal, 0 if not
;
; @export: array_eq
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r3, 0  ; 0
.while_0:
    nop
    mov r15, r3  ; i
    mov r14, r2  ; len
    blt r15, r14, .set_2
    mov r15, 0
    b .cmp_end_3
.set_2:
    nop
    mov r15, 1
.cmp_end_3:
    nop
    beq r15, zero, .endwhile_1
    mov r14, r1  ; b
    mov r15, r3  ; i
    alui.Shl r15, r15, 3
    alu.Add r14, r14, r15
    load.Double r14, [r14]
    mov r15, r0  ; a
    mov r14, r3  ; i
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    load.Double r15, [r15]
    bne r15, r14, .set_6
    mov r15, 0
    b .cmp_end_7
.set_6:
    nop
    mov r15, 1
.cmp_end_7:
    nop
    beq r15, zero, .endif_5
    mov r0, 0  ; 0
    halt
.endif_5:
    nop
    mov r15, 1  ; 1
    alu.Add r3, r3, r15
    b .while_0
.endwhile_1:
    nop
    mov r0, 1  ; 1
    halt
