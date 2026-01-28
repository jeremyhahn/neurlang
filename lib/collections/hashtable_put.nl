; @name: Hashtable Put
; @description: Insert or update a key-value pair.
; @category: collections/hashtable
; @difficulty: 3
;
; @prompt: put {key} with value {value} in hash table at {ptr}
; @prompt: insert key {key} value {value} into hashtable {ptr}
; @prompt: add entry {key}={value} to hash table at {ptr}
; @prompt: store {value} under key {key} in hashtable at {ptr}
; @prompt: set {key} to {value} in hash table {ptr}
; @prompt: hash table put {key} {value} at {ptr}
; @prompt: insert or update {key} with {value} in hashtable {ptr}
; @prompt: add key-value pair {key}:{value} to hash table at {ptr}
; @prompt: map {key} to {value} in hash table at address {ptr}
; @prompt: associate {key} with {value} in hashtable {ptr}
; @prompt: upsert {key}={value} into hash table at {ptr}
; @prompt: write {value} for key {key} in hashtable {ptr}
; @prompt: hashtable insert {key} {value} at memory {ptr}
; @prompt: put entry key={key} value={value} in hash map {ptr}
;
; @param: ptr=r0 "Memory address of the hash table"
; @param: key=r1 "Key to insert or update (must not be 0)"
; @param: value=r2 "Value to associate with the key"
;
; @test: r0=0, r1=0, r2=0 -> r0=0
; @note: Returns 0 if key is 0 (invalid), otherwise updates hash table
;
; @export: hashtable_put
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
    mov r3, r0  ; ptr
    load.Double r3, [r3]
    mov r4, r0  ; ptr
    mov r15, 1  ; 1
    alui.Shl r15, r15, 3
    alu.Add r4, r4, r15
    load.Double r4, [r4]
    mov r14, r3  ; capacity
    mov r15, 3  ; 3
    muldiv.Mul r14, r14, r15
    mov r15, 4  ; 4
    muldiv.Div r14, r14, r15
    mov r15, r4  ; count
    bge r15, r14, .set_6
    mov r15, 0
    b .cmp_end_7
.set_6:
    nop
    mov r15, 1
.cmp_end_7:
    nop
    beq r15, zero, .endif_5
    mov r0, 0  ; 0
    halt
.endif_5:
    nop
    mov r5, r0  ; ptr
    mov r15, 2  ; 2
    alui.Shl r15, r15, 3
    alu.Add r5, r5, r15
    mov r6, r0  ; ptr
    mov r14, r3  ; capacity
    mov r15, 2  ; 2
    alu.Add r15, r15, r14
    alui.Shl r15, r15, 3
    alu.Add r6, r6, r15
    mov r7, r1  ; key
    mov r15, 2135587861  ; 11400714819323198485
    muldiv.Mul r7, r7, r15
    mov r8, r7  ; h
    mov r15, r3  ; capacity
    muldiv.Mod r8, r8, r15
    mov r9, r8  ; idx
.loop_8:
    nop
    mov r10, r5  ; keys_base
    mov r15, r8  ; idx
    alui.Shl r15, r15, 3
    alu.Add r10, r10, r15
    load.Double r10, [r10]
    mov r15, r10  ; existing_key
    mov r14, 0  ; 0
    beq r15, r14, .set_12
    mov r15, 0
    b .cmp_end_13
.set_12:
    nop
    mov r15, 1
.cmp_end_13:
    nop
    beq r15, zero, .endif_11
    mov r15, r5  ; keys_base
    mov r14, r8  ; idx
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, r1  ; key
    store.Double r14, [r15]
    mov r15, r6  ; values_base
    mov r14, r8  ; idx
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, r2  ; value
    store.Double r14, [r15]
    mov r15, r0  ; ptr
    mov r14, 1  ; 1
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, r4  ; count
    mov r15, 1  ; 1
    alu.Add r14, r14, r15
    store.Double r14, [r15]
    mov r0, 1  ; 1
    halt
.endif_11:
    nop
    mov r15, r10  ; existing_key
    mov r14, r1  ; key
    beq r15, r14, .set_16
    mov r15, 0
    b .cmp_end_17
.set_16:
    nop
    mov r15, 1
.cmp_end_17:
    nop
    beq r15, zero, .endif_15
    mov r15, r6  ; values_base
    mov r14, r8  ; idx
    alui.Shl r14, r14, 3
    alu.Add r15, r15, r14
    mov r14, r2  ; value
    store.Double r14, [r15]
    mov r0, 1  ; 1
    halt
.endif_15:
    nop
    mov r15, 1  ; 1
    alu.Add r8, r8, r15
    mov r15, r3  ; capacity
    muldiv.Mod r8, r8, r15
    mov r15, r8  ; idx
    mov r14, r9  ; start_idx
    beq r15, r14, .set_20
    mov r15, 0
    b .cmp_end_21
.set_20:
    nop
    mov r15, 1
.cmp_end_21:
    nop
    beq r15, zero, .endif_19
    mov r0, 0  ; 0
    halt
.endif_19:
    nop
    b .loop_8
.endloop_9:
    nop
    halt
