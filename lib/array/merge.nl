; @name: Merge
; @description: Merge two sorted halves of an array.
; @category: array
; @difficulty: 3
;
; @prompt: merge sorted halves of {arr} at {mid} with total length {len} using {temp}
; @prompt: combine two sorted subarrays in {arr} split at {mid} for {len} elements into {temp}
; @prompt: merge {arr} halves [0..{mid}) and [{mid}..{len}) using buffer {temp}
; @prompt: merge sort helper combining {arr} at midpoint {mid} with {len} total using {temp}
; @prompt: merge two sorted sections of {arr} at index {mid} with size {len} via {temp}
; @prompt: combine sorted partitions in {arr} divided at {mid} for {len} items using {temp}
; @prompt: merge sorted halves of array {arr} with split at {mid} length {len} buffer {temp}
; @prompt: join two sorted runs in {arr} at {mid} totaling {len} elements using {temp}
; @prompt: merge operation on {arr} with pivot {mid} and length {len} temp buffer {temp}
; @prompt: combine {arr} sorted halves at {mid} with {len} elements into {temp}
; @prompt: merge sorted left and right of {arr} at {mid} for {len} using {temp}
; @prompt: two-way merge of {arr} split at {mid} with {len} total via {temp} buffer
;
; @param: arr=r0 "Pointer to array with two sorted halves (mutable)"
; @param: temp=r1 "Pointer to temporary buffer of at least len elements"
; @param: mid=r2 "Index where second sorted half begins"
; @param: len=r3 "Total number of elements in the array"
;
; @test: r0=0, r1=0, r2=0, r3=0 -> r0=0
; @note: Modifies array in-place, requires temp buffer
;
; @export: merge
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r4, 0  ; 0
.while_0:
    nop
    mov r15, r4  ; i
    mov r14, r3  ; len
    blt r15, r14, .set_2
    mov r15, 0
    b .cmp_end_3
.set_2:
    nop
    mov r15, 1
.cmp_end_3:
    nop
    beq r15, zero, .endwhile_1
    mov r15, r1  ; temp
    mov r14, r4  ; i
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, r0  ; ptr
    mov r15, r4  ; i
    alui.Shl r15, r15, 3
    alu.Add r14, r14, r15
    load.Double r14, [r14]
    store.Double r14, [r15]
    mov r15, 1  ; 1
    alu.Add r4, r4, r15
    b .while_0
.endwhile_1:
    nop
    mov r5, 0  ; 0
    mov r6, r2  ; mid
    mov r7, 0  ; 0
.while_4:
    nop
    mov r14, r6  ; right
    mov r15, r3  ; len
    blt r14, r15, .set_6
    mov r14, 0
    b .cmp_end_7
.set_6:
    nop
    mov r14, 1
.cmp_end_7:
    nop
    mov r15, r5  ; left
    mov r14, r2  ; mid
    blt r15, r14, .set_8
    mov r15, 0
    b .cmp_end_9
.set_8:
    nop
    mov r15, 1
.cmp_end_9:
    nop
    bne r15, zero, .set_10
    mov r15, 0
    b .cmp_end_11
.set_10:
    nop
    mov r15, 1
.cmp_end_11:
    nop
    mov r13, r14
    bne r13, zero, .set_12
    mov r13, 0
    b .cmp_end_13
.set_12:
    nop
    mov r13, 1
.cmp_end_13:
    nop
    alu.And r15, r15, r13
    beq r15, zero, .endwhile_5
    mov r14, r1  ; temp
    mov r15, r6  ; right
    alui.Shl r15, r15, 3
    alu.Add r14, r14, r15
    load.Double r14, [r14]
    mov r15, r1  ; temp
    mov r14, r5  ; left
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    load.Double r15, [r15]
    ble r15, r14, .set_16
    mov r15, 0
    b .cmp_end_17
.set_16:
    nop
    mov r15, 1
.cmp_end_17:
    nop
    beq r15, zero, .else_14
    mov r15, r0  ; ptr
    mov r14, r7  ; dest
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, r1  ; temp
    mov r15, r5  ; left
    alui.Shl r15, r15, 3
    alu.Add r14, r14, r15
    load.Double r14, [r14]
    store.Double r14, [r15]
    mov r15, 1  ; 1
    alu.Add r5, r5, r15
    b .endif_15
.else_14:
    nop
    mov r15, r0  ; ptr
    mov r14, r7  ; dest
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, r1  ; temp
    mov r15, r6  ; right
    alui.Shl r15, r15, 3
    alu.Add r14, r14, r15
    load.Double r14, [r14]
    store.Double r14, [r15]
    mov r15, 1  ; 1
    alu.Add r6, r6, r15
.endif_15:
    nop
    mov r15, 1  ; 1
    alu.Add r7, r7, r15
    b .while_4
.endwhile_5:
    nop
.while_18:
    nop
    mov r15, r5  ; left
    mov r14, r2  ; mid
    blt r15, r14, .set_20
    mov r15, 0
    b .cmp_end_21
.set_20:
    nop
    mov r15, 1
.cmp_end_21:
    nop
    beq r15, zero, .endwhile_19
    mov r15, r0  ; ptr
    mov r14, r7  ; dest
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, r1  ; temp
    mov r15, r5  ; left
    alui.Shl r15, r15, 3
    alu.Add r14, r14, r15
    load.Double r14, [r14]
    store.Double r14, [r15]
    mov r15, 1  ; 1
    alu.Add r5, r5, r15
    mov r15, 1  ; 1
    alu.Add r7, r7, r15
    b .while_18
.endwhile_19:
    nop
.while_22:
    nop
    mov r15, r6  ; right
    mov r14, r3  ; len
    blt r15, r14, .set_24
    mov r15, 0
    b .cmp_end_25
.set_24:
    nop
    mov r15, 1
.cmp_end_25:
    nop
    beq r15, zero, .endwhile_23
    mov r15, r0  ; ptr
    mov r14, r7  ; dest
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, r1  ; temp
    mov r15, r6  ; right
    alui.Shl r15, r15, 3
    alu.Add r14, r14, r15
    load.Double r14, [r14]
    store.Double r14, [r15]
    mov r15, 1  ; 1
    alu.Add r6, r6, r15
    mov r15, 1  ; 1
    alu.Add r7, r7, r15
    b .while_22
.endwhile_23:
    nop
    halt
