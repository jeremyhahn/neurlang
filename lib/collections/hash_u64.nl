; @name: Hash U64
; @description: Simple hash function for u64.
; @export: hash_u64
; @category: stdlib
; @difficulty: 2
;
; @prompt: hash 64-bit integer {key}
; @prompt: compute hash of {key}
; @prompt: hash value {key} with capacity {capacity}
; @prompt: get hash bucket for {key}
; @prompt: splitmix64 hash of {key}
; @prompt: hash {key} to bucket index
; @prompt: compute bucket index for key {key}
; @prompt: hash function for {key} modulo {capacity}
; @prompt: map {key} to hash bucket
; @prompt: fast hash of integer {key}
; @prompt: distribute {key} across {capacity} buckets
; @prompt: calculate hash index for {key}
;
; @param: key=r0 "The 64-bit integer key to hash"
; @param: capacity=r1 "The hash table capacity (bucket count)"
;
; @test: r0=0, r1=16 -> r0=0
; @test: r0=1, r1=16 -> r0=2
; @test: r0=12345, r1=100 -> r0!=0
; @test: r0=999999, r1=64 -> r0<64
;
; @note: Division by zero if capacity=0. Hash uses splitmix64 variant
;
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r2, r0  ; key
    mov r15, r2  ; h
    mov r14, 33  ; 33
    alu.Shr r15, r15, r14
    alu.Xor r2, r2, r15
    mov r15, -313160499  ; 18397679294719823053
    muldiv.Mul r2, r2, r15
    mov r15, r2  ; h
    mov r14, 33  ; 33
    alu.Shr r15, r15, r14
    alu.Xor r2, r2, r15
    mov r0, r2  ; h
    mov r15, r1  ; capacity
    muldiv.Mod r0, r0, r15
    halt
