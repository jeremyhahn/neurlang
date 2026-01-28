; @name: Stack Pop
; @description: Pop a value from the stack.
; @category: collections/stack
; @difficulty: 1
;
; @prompt: pop value from stack at {ptr}
; @prompt: remove top element from stack {ptr}
; @prompt: pop from stack at address {ptr}
; @prompt: get and remove top of stack {ptr}
; @prompt: stack pop from {ptr}
; @prompt: remove top item from LIFO stack {ptr}
; @prompt: pop element off stack at {ptr}
; @prompt: take value from top of stack {ptr}
; @prompt: extract top element from stack at {ptr}
; @prompt: pop the stack located at {ptr}
; @prompt: remove and return top of stack {ptr}
; @prompt: get value from top of stack {ptr} and remove it
; @prompt: pop item from stack structure at {ptr}
;
; @param: ptr=r0 "Memory address of the stack"
;
; @test: r0=0 -> r0=0
;
; @export: stack_pop
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
    mov r2, r1  ; size
    mov r15, 1  ; 1
    alu.Sub r2, r2, r15
    mov r3, r0  ; ptr
    mov r14, r2  ; new_size
    mov r15, 2  ; 2
    alu.Add r15, r15, r14
    alui.Shl r15, r15, 3
    alu.Add r3, r3, r15
    load.Double r3, [r3]
    mov r15, r0  ; ptr
    mov r14, 1  ; 1
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, r2  ; new_size
    store.Double r14, [r15]
    mov r0, r3  ; value
    halt
