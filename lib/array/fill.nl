; @name: Fill
; @description: Fill array with a value.
; @category: array
; @difficulty: 1
;
; @prompt: fill array {arr} with {value} for {len} elements
; @prompt: set all {len} elements in {arr} to {value}
; @prompt: initialize {arr} of size {len} with value {value}
; @prompt: fill {len} slots in {arr} with constant {value}
; @prompt: write {value} to all positions in {arr} with {len} items
; @prompt: memset {arr} to {value} for {len} u64 values
; @prompt: populate {arr} array of length {len} with {value}
; @prompt: fill {arr} buffer of {len} entries with {value}
; @prompt: set {len} element array {arr} to uniform value {value}
; @prompt: initialize all {len} elements of {arr} to {value}
; @prompt: fill entire {arr} of size {len} with {value}
; @prompt: assign {value} to each of {len} positions in {arr}
;
; @param: arr=r0 "Pointer to array of u64 elements (mutable)"
; @param: len=r1 "Number of elements to fill"
; @param: value=r2 "Value to fill with"
;
; @test: r0=0, r1=0, r2=42 -> r0=0
; @note: Fills array in-place with value
;
; @export: fill
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r3, 0  ; 0
.while_0:
    nop
    mov r15, r3  ; i
    mov r14, r1  ; len
    blt r15, r14, .set_2
    mov r15, 0
    b .cmp_end_3
.set_2:
    nop
    mov r15, 1
.cmp_end_3:
    nop
    beq r15, zero, .endwhile_1
    mov r15, r0  ; ptr
    mov r14, r3  ; i
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, r2  ; value
    store.Double r14, [r15]
    mov r15, 1  ; 1
    alu.Add r3, r3, r15
    b .while_0
.endwhile_1:
    nop
    halt
