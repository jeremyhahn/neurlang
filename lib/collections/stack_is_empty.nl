; @name: Stack Is Empty
; @description: Check if stack is empty.
; @category: collections/stack
; @difficulty: 1
;
; @prompt: check if stack at {ptr} is empty
; @prompt: is stack {ptr} empty
; @prompt: test if stack at {ptr} has no elements
; @prompt: determine if stack {ptr} is empty
; @prompt: check stack {ptr} for emptiness
; @prompt: is LIFO stack at {ptr} empty
; @prompt: verify stack at address {ptr} is empty
; @prompt: see if stack {ptr} contains no items
; @prompt: check for empty stack at {ptr}
; @prompt: test stack {ptr} emptiness
; @prompt: is stack at memory {ptr} empty
; @prompt: check if stack {ptr} has zero elements
; @prompt: query if stack at {ptr} is empty
;
; @param: ptr=r0 "Memory address of the stack"
;
; @test: r0=0 -> r0=1
; @note: Returns 1 if empty (size at ptr+8 is 0), 0 otherwise. With ptr=0, reads size from address 8
;
; @export: stack_is_empty
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r15, r0  ; ptr
    mov r14, 1  ; 1
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    load.Double r15, [r15]
    mov r14, 0  ; 0
    beq r15, r14, .set_2
    mov r15, 0
    b .cmp_end_3
.set_2:
    nop
    mov r15, 1
.cmp_end_3:
    nop
    beq r15, zero, .else_0
    mov r0, 1  ; 1
    b .endif_1
.else_0:
    nop
    mov r0, 0  ; 0
.endif_1:
    nop
    halt
