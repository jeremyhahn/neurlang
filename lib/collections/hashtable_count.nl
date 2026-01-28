; @name: Hashtable Count
; @description: Get the count of entries in the hash table.
; @category: collections/hashtable
; @difficulty: 1
;
; @prompt: get count of entries in hash table at {ptr}
; @prompt: how many key-value pairs in hashtable {ptr}
; @prompt: return hash table size for {ptr}
; @prompt: count entries in hash table at {ptr}
; @prompt: get number of items in hashtable {ptr}
; @prompt: hash table length at address {ptr}
; @prompt: find hash table entry count at {ptr}
; @prompt: get element count for hashtable {ptr}
; @prompt: how many entries in hash table at {ptr}
; @prompt: return number of keys in hash table {ptr}
; @prompt: query hashtable size at memory {ptr}
; @prompt: get current size of hash map {ptr}
; @prompt: count items in hash table at {ptr}
;
; @param: ptr=r0 "Memory address of the hash table"
;
; @test: r0=0 -> r0=0
;
; @export: hashtable_count
; Generated from Rust by nl stdlib build

.entry:
    nop
    mov r15, 1  ; 1
    alui.Shl r15, r15, 3
    alu.Add r0, r0, r15
    load.Double r0, [r0]
    halt
