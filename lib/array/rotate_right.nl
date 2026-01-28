; @name: Rotate Right
; @description: Rotate array right by k positions.
; @category: array
; @difficulty: 2
;
; @prompt: rotate array {arr} right by {k} positions with {len} elements
; @prompt: right rotate {arr} of size {len} by {k} places
; @prompt: shift {arr} right cyclically by {k} over {len} items
; @prompt: circular right shift {arr} by {k} positions for {len} values
; @prompt: rotate {len} element array {arr} rightward by {k}
; @prompt: right rotation of {arr} with {len} entries by {k} positions
; @prompt: cycle {arr} right by {k} across {len} elements
; @prompt: move last {k} elements of {arr} to front for {len} total
; @prompt: rotate {arr} array of {len} items right by {k}
; @prompt: right circular rotate {arr} with {len} values by {k}
; @prompt: shift {arr} rightward {k} times over {len} elements
; @prompt: perform right rotation on {arr} of {len} by {k} positions
;
; @param: arr=r0 "Pointer to array of u64 elements (mutable)"
; @param: len=r1 "Number of elements in the array"
; @param: k=r2 "Number of positions to rotate right"
;
; @test: r0=0, r1=0, r2=0 -> r0=0
; @note: Modifies array in-place
;
; @export: rotate_right
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
    mov r2, r1  ; len
    mov r15, r3  ; rot_amount
    alu.Sub r2, r2, r15
    nop  ; TODO: function call not supported  ; call rotate_left
    halt
