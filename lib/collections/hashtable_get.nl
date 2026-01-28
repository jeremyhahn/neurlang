; @name: Hashtable Get
; @description: Get a value by key.
; @category: collections/hashtable
; @difficulty: 2
;
; @prompt: get value for key {key} from hash table at {ptr}
; @prompt: lookup {key} in hashtable {ptr}
; @prompt: retrieve value for {key} from hash table at {ptr}
; @prompt: find value of key {key} in hashtable at {ptr}
; @prompt: hash table get {key} from {ptr}
; @prompt: read value for key {key} in hash table {ptr}
; @prompt: fetch entry for {key} from hashtable at {ptr}
; @prompt: get {key} from hash table at address {ptr}
; @prompt: query hash table {ptr} for key {key}
; @prompt: obtain value mapped to {key} in hashtable {ptr}
; @prompt: hashtable lookup {key} at memory {ptr}
; @prompt: search for {key} in hash table {ptr}
; @prompt: access value of {key} in hash map {ptr}
;
; @param: ptr=r0 "Memory address of the hash table"
; @param: key=r1 "Key to look up"
;
; @test: r0=0, r1=0 -> r0=0
;
; @export: hashtable_get
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
    mov r4, r0  ; ptr
    mov r14, r2  ; capacity
    mov r15, 2  ; 2
    alu.Add r15, r15, r14
    alui.Shl r15, r15, 3
    alu.Add r4, r4, r15
    mov r5, r1  ; key
    mov r15, 2135587861  ; 11400714819323198485
    muldiv.Mul r5, r5, r15
    mov r6, r5  ; h
    mov r15, r2  ; capacity
    muldiv.Mod r6, r6, r15
    mov r7, r6  ; idx
.loop_4:
    nop
    mov r8, r3  ; keys_base
    mov r15, r6  ; idx
    alui.Shl r15, r15, 3
    alu.Add r8, r8, r15
    load.Double r8, [r8]
    mov r15, r8  ; existing_key
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
    mov r15, r8  ; existing_key
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
    mov r0, r4  ; values_base
    mov r15, r6  ; idx
    alui.Shl r15, r15, 3
    alu.Add r0, r0, r15
    load.Double r0, [r0]
    halt
.endif_11:
    nop
    mov r15, 1  ; 1
    alu.Add r6, r6, r15
    mov r15, r2  ; capacity
    muldiv.Mod r6, r6, r15
    mov r15, r6  ; idx
    mov r14, r7  ; start_idx
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
