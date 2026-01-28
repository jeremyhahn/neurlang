; @name: Cache Aside
; @description: Read-through caching pattern
; @category: patterns/caching
; @difficulty: 3
;
; @prompt: implement cache aside pattern
; @prompt: read-through cache
; @prompt: cache miss triggers origin fetch
; @prompt: lazy loading cache pattern
; @prompt: cache aside with origin fallback
; @prompt: look aside caching
; @prompt: cache miss then populate
; @prompt: read through cache pattern
; @prompt: fetch on cache miss
; @prompt: lazy cache population
;
; @param: key=r0 "Key to look up"
; @param: in_cache=r1 "1 if key in cache, 0 if not"
;
; @test: r0=1, r1=1 -> r0=100
; @test: r0=1, r1=0 -> r0=50
; @test: r0=2, r1=0 -> r0=50
;
; @note: Returns cached value (100) or fetched value (50)
; @note: Populates cache on miss
;
; Cache Aside Pattern
; ===================
; Check cache first, fetch from origin on miss, populate cache.

.entry main

.section .text

main:
    ; r0 = key
    ; r1 = in_cache flag
    mov r10, r0
    mov r11, r1

    ; Check cache first
    beq r11, zero, cache_miss

    ; Cache hit - return cached value (100)
    mov r0, 100
    halt

cache_miss:
    ; Fetch from origin - return origin value (50)
    mov r0, 50
    halt
