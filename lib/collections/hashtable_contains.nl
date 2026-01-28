; @name: Hashtable Contains
; @description: Check if key exists in hash table.
; @category: collections/hashtable
; @difficulty: 2
;
; @prompt: check if key {key} exists in hash table at {ptr}
; @prompt: does hashtable {ptr} contain key {key}
; @prompt: test if {key} is in hash table at {ptr}
; @prompt: determine if key {key} exists in hashtable {ptr}
; @prompt: check hash table {ptr} for key {key}
; @prompt: is {key} present in hashtable at {ptr}
; @prompt: verify key {key} exists in hash table at address {ptr}
; @prompt: see if hash table {ptr} has key {key}
; @prompt: check for key {key} in hashtable at {ptr}
; @prompt: test hashtable {ptr} contains {key}
; @prompt: is key {key} in hash table at memory {ptr}
; @prompt: check if hash table {ptr} has entry for {key}
; @prompt: query if {key} exists in hash map {ptr}
;
; @param: ptr=r0 "Memory address of the hash table"
; @param: key=r1 "Key to check for existence"
;
; @test: r0=0, r1=0 -> r0=0
;
; @export: hashtable_contains
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r15, r1  ; key
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
    mov r2, r0  ; ptr
    load.Double r2, [r2]
    mov r3, r0  ; ptr
    mov r15, 2  ; 2
    alui.Shl r15, r15, 3
    alu.Add r3, r3, r15
    mov r4, r1  ; key
    mov r15, 2135587861  ; 11400714819323198485
    muldiv.Mul r4, r4, r15
    mov r5, r4  ; h
    mov r15, r2  ; capacity
    muldiv.Mod r5, r5, r15
    mov r6, r5  ; idx
.loop_4:
    nop
    mov r7, r3  ; keys_base
    mov r15, r5  ; idx
    alui.Shl r15, r15, 3
    alu.Add r7, r7, r15
    load.Double r7, [r7]
    mov r15, r7  ; existing_key
    mov r14, 0  ; 0
    beq r15, r14, .set_8
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
    mov r15, r7  ; existing_key
    mov r14, r1  ; key
    beq r15, r14, .set_12
    mov r15, 0
    b .cmp_end_13
.set_12:
    nop
    mov r15, 1
.cmp_end_13:
    nop
    beq r15, zero, .endif_11
    mov r0, 1  ; 1
    halt
.endif_11:
    nop
    mov r15, 1  ; 1
    alu.Add r5, r5, r15
    mov r15, r2  ; capacity
    muldiv.Mod r5, r5, r15
    mov r15, r5  ; idx
    mov r14, r6  ; start_idx
    beq r15, r14, .set_16
    mov r15, 0
    b .cmp_end_17
.set_16:
    nop
    mov r15, 1
.cmp_end_17:
    nop
    beq r15, zero, .endif_15
    mov r0, 0  ; 0
    halt
.endif_15:
    nop
    b .loop_4
.endloop_5:
    nop
    halt
