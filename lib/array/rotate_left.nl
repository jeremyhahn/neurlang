; @name: Rotate Left
; @description: Rotate array left by k positions.
; @category: array
; @difficulty: 2
;
; @prompt: rotate array {arr} left by {k} positions with {len} elements
; @prompt: left rotate {arr} of size {len} by {k} places
; @prompt: shift {arr} left cyclically by {k} over {len} items
; @prompt: circular left shift {arr} by {k} positions for {len} values
; @prompt: rotate {len} element array {arr} leftward by {k}
; @prompt: left rotation of {arr} with {len} entries by {k} positions
; @prompt: cycle {arr} left by {k} across {len} elements
; @prompt: move first {k} elements of {arr} to end for {len} total
; @prompt: rotate {arr} array of {len} items left by {k}
; @prompt: left circular rotate {arr} with {len} values by {k}
; @prompt: shift {arr} leftward {k} times over {len} elements
; @prompt: perform left rotation on {arr} of {len} by {k} positions
;
; @param: arr=r0 "Pointer to array of u64 elements (mutable)"
; @param: len=r1 "Number of elements in the array"
; @param: k=r2 "Number of positions to rotate left"
;
; @test: r0=0, r1=0, r2=0 -> r0=0
; @note: Modifies array in-place
;
; @export: rotate_left
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r14, r2  ; k
    mov r15, 0  ; 0
    beq r14, r15, .set_2
    mov r14, 0
    b .cmp_end_3
.set_2:
    nop
    mov r14, 1
.cmp_end_3:
    nop
    mov r15, r1  ; len
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
    mov r3, r2  ; k
    mov r15, r1  ; len
    muldiv.Mod r3, r3, r15
    mov r15, r3  ; rot_amount
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
    halt
.endif_9:
    nop
    mov r1, r3  ; rot_amount
    nop  ; TODO: function call not supported  ; call reverse
    mov r15, r3  ; rot_amount
    alui.Shl r15, r15, 3
    alu.Add r0, r0, r15
    mov r15, r3  ; rot_amount
    alu.Sub r1, r1, r15
    nop  ; TODO: function call not supported  ; call reverse
    nop  ; TODO: function call not supported  ; call reverse
    halt
