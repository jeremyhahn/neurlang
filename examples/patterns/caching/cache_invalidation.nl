; @name: Cache Invalidation
; @description: Invalidate cache on write operations
; @category: patterns/caching
; @difficulty: 3
;
; @prompt: invalidate cache on write
; @prompt: cache invalidation pattern
; @prompt: clear cache when data changes
; @prompt: write-through cache invalidation
; @prompt: invalidate on update
; @prompt: cache invalidation on mutation
; @prompt: delete cache entry on write
; @prompt: keep cache consistent with writes
; @prompt: invalidate cache after update
; @prompt: cache coherence pattern
;
; @param: operation=r0 "0=read, 1=write"
; @param: cached=r1 "Is value cached"
;
; @test: r0=0, r1=1 -> r0=100
; @test: r0=1, r1=1 -> r0=1
; @test: r0=0, r1=0 -> r0=50
;
; @note: Read returns cached (100) or origin (50)
; @note: Write invalidates cache and returns 1
;
; Cache Invalidation Pattern
; ==========================
; On write, invalidate cache to ensure fresh data on next read.

.entry main

.section .text

main:
    ; r0 = operation (0=read, 1=write)
    ; r1 = is_cached flag
    mov r10, r0
    mov r11, r1

    ; Dispatch operation
    beq r10, zero, do_read
    b do_write

do_read:
    ; Check if cached
    beq r11, zero, read_from_origin

    ; Return cached value (100)
    mov r0, 100
    halt

read_from_origin:
    ; Fetch from origin (50)
    mov r0, 50
    halt

do_write:
    ; Write operation - invalidate cache and return success
    mov r0, 1
    halt
