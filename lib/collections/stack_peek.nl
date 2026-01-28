; @name: Stack Peek
; @description: Peek at the top of the stack without popping.
; @category: collections/stack
; @difficulty: 1
;
; @prompt: peek at top of stack {ptr}
; @prompt: get top value from stack {ptr} without removing
; @prompt: look at top element of stack at {ptr}
; @prompt: read top of stack {ptr}
; @prompt: peek stack at address {ptr}
; @prompt: view top item on stack {ptr}
; @prompt: get stack top from {ptr} without popping
; @prompt: examine top of stack at {ptr}
; @prompt: check what is on top of stack {ptr}
; @prompt: peek at stack {ptr} top element
; @prompt: read top value from LIFO stack {ptr}
; @prompt: inspect top of stack at memory {ptr}
; @prompt: see top element of stack {ptr} without removal
;
; @param: ptr=r0 "Memory address of the stack"
;
; @test: r0=0 -> r0=0
;
; @export: stack_peek
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r1, r0  ; ptr
    mov r15, 1  ; 1
    alui.Shl r15, r15, 3
    alu.Add r1, r1, r15
    load.Double r1, [r1]
    mov r15, r1  ; size
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
    mov r0, 0  ; 0
    halt
.endif_1:
    nop
    mov r14, r1  ; size
    mov r15, 1  ; 1
    alu.Sub r14, r14, r15
    mov r15, 2  ; 2
    alu.Add r15, r15, r14
    alui.Shl r15, r15, 3
    alu.Add r0, r0, r15
    load.Double r0, [r0]
    halt
