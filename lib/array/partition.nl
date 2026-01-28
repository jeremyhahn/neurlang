; @name: Partition
; @description: Partition for quicksort (Lomuto scheme).
; @category: array
; @difficulty: 3
;
; @prompt: partition {arr} from {low} to {high} for quicksort
; @prompt: lomuto partition on {arr} between indices {low} and {high}
; @prompt: partition array {arr} from {low} to {high} around pivot
; @prompt: quicksort partition {arr} with range {low} to {high}
; @prompt: perform partition step on {arr} from {low} to {high}
; @prompt: divide {arr} for quicksort between {low} and {high}
; @prompt: partition {arr} array using lomuto scheme from {low} to {high}
; @prompt: rearrange {arr} around pivot from index {low} to {high}
; @prompt: partition elements in {arr} between {low} and {high}
; @prompt: quicksort helper partition {arr} range {low} to {high}
; @prompt: lomuto partition step on {arr} from {low} through {high}
; @prompt: split {arr} around pivot element from {low} to {high}
;
; @param: arr=r0 "Pointer to array of u64 elements (mutable)"
; @param: low=r1 "Starting index of partition range"
; @param: high=r2 "Ending index of partition range (pivot location)"
;
; @test: r0=0, r1=0, r2=0 -> r0=0
; @note: Returns pivot index after partitioning
;
; @export: partition
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r3, r0  ; ptr
    mov r15, r2  ; high
    alui.Shl r15, r15, 3
    alu.Add r3, r3, r15
    load.Double r3, [r3]
    mov r4, r1  ; low
    mov r5, r1  ; low
.while_0:
    nop
    mov r15, r5  ; j
    mov r14, r2  ; high
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
    mov r14, r5  ; j
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    load.Double r15, [r15]
    mov r14, r3  ; pivot
    blt r15, r14, .set_6
    mov r15, 0
    b .cmp_end_7
.set_6:
    nop
    mov r15, 1
.cmp_end_7:
    nop
    beq r15, zero, .endif_5
    mov r6, r0  ; ptr
    mov r15, r4  ; i
    alui.Shl r15, r15, 3
    alu.Add r6, r6, r15
    load.Double r6, [r6]
    mov r15, r0  ; ptr
    mov r14, r4  ; i
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, r0  ; ptr
    mov r15, r5  ; j
    alui.Shl r15, r15, 3
    alu.Add r14, r14, r15
    load.Double r14, [r14]
    store.Double r14, [r15]
    mov r15, r0  ; ptr
    mov r14, r5  ; j
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, r6  ; temp
    store.Double r14, [r15]
    mov r15, 1  ; 1
    alu.Add r4, r4, r15
.endif_5:
    nop
    mov r15, 1  ; 1
    alu.Add r5, r5, r15
    b .while_0
.endwhile_1:
    nop
    mov r7, r0  ; ptr
    mov r15, r4  ; i
    alui.Shl r15, r15, 3
    alu.Add r7, r7, r15
    load.Double r7, [r7]
    mov r15, r0  ; ptr
    mov r14, r4  ; i
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, r0  ; ptr
    mov r15, r2  ; high
    alui.Shl r15, r15, 3
    alu.Add r14, r14, r15
    load.Double r14, [r14]
    store.Double r14, [r15]
    mov r15, r0  ; ptr
    mov r14, r2  ; high
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, r7  ; temp
    store.Double r14, [r15]
    mov r0, r4  ; i
    halt
