; @name: Graceful Degradation
; @description: Fallback to cached or default value when primary fails
; @category: patterns/error-handling
; @difficulty: 3
;
; @prompt: implement graceful degradation pattern
; @prompt: fallback to cache when primary fails
; @prompt: handle service failure with fallback
; @prompt: graceful fallback on error
; @prompt: return cached value when primary unavailable
; @prompt: degrade gracefully to default value
; @prompt: implement fallback chain pattern
; @prompt: primary with cache fallback
; @prompt: service degradation with stale data
; @prompt: fail gracefully with cached response
;
; @param: primary_result=r0 "Primary operation result (0=success, 1=fail)"
; @param: cache_valid=r1 "Is cache valid (0=no, 1=yes)"
;
; @test: r0=0, r1=0 -> r0=100
; @test: r0=1, r1=1 -> r0=50
; @test: r0=1, r1=0 -> r0=0
;
; @note: Returns primary value (100), cache value (50), or default (0)
; @note: Tries in order: primary -> cache -> default
;
; Graceful Degradation Pattern
; ===========================
; When primary source fails, try cache, then fall back to default.

.entry main

.section .data

primary_value:      .dword 100      ; Value from primary source
cached_value:       .dword 50       ; Stale cached value
default_value:      .dword 0        ; Safe default

.section .text

main:
    ; r0 = primary_fails (0=success, 1=fail)
    ; r1 = cache_valid (0=invalid, 1=valid)
    mov r10, r0                     ; r10 = primary_fails
    mov r11, r1                     ; r11 = cache_valid

    ; Try primary source first
    call fetch_primary
    beq r0, zero, primary_success

    ; Primary failed - try cache
    call fetch_cached
    beq r0, zero, cache_success

    ; Cache also failed - use default
    call fetch_default
    mov r0, r1                      ; Return default value
    halt

primary_success:
    mov r0, primary_value
    load.d r0, [r0]
    halt

cache_success:
    mov r0, cached_value
    load.d r0, [r0]
    halt

fetch_primary:
    ; Simulate fetching from primary source
    beq r10, zero, fetch_ok
    mov r0, 1                       ; Failure
    ret
fetch_ok:
    mov r0, 0                       ; Success
    ret

fetch_cached:
    ; Simulate fetching from cache
    beq r11, zero, cache_miss
    mov r0, 0                       ; Cache hit
    ret
cache_miss:
    mov r0, 1                       ; Cache miss/invalid
    ret

fetch_default:
    ; Always succeeds - returns safe default
    mov r0, default_value
    load.d r1, [r0]
    mov r0, 0
    ret
