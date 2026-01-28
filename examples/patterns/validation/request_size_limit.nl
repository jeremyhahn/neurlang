; @name: Request Size Limit
; @description: Reject requests exceeding size limit
; @category: patterns/validation
; @difficulty: 2
;
; @prompt: validate request size limit
; @prompt: reject oversized requests
; @prompt: check content-length against limit
; @prompt: enforce maximum request size
; @prompt: validate request body size
; @prompt: limit request payload size
; @prompt: check request size before processing
; @prompt: reject large requests
; @prompt: content length validation
; @prompt: enforce size limit on request
;
; @param: request_size=r0 "Size of request in bytes"
; @param: max_size=r1 "Maximum allowed size"
;
; @test: r0=100, r1=1024 -> r0=1
; @test: r0=1024, r1=1024 -> r0=1
; @test: r0=1025, r1=1024 -> r0=0
; @test: r0=0, r1=1024 -> r0=1
;
; @note: Returns 1 if size OK, 0 if too large
;
; Request Size Limit Pattern
; ==========================
; Simple validation - reject if size exceeds limit.

.entry main

.section .data

default_max_size:   .word 10485760  ; 10MB default limit

.section .text

main:
    ; r0 = request_size
    ; r1 = max_size
    mov r10, r0                     ; r10 = request size
    mov r11, r1                     ; r11 = max size

    ; Zero size is allowed (empty body)
    beq r10, zero, size_ok

    ; Check against limit
    bgt r10, r11, size_exceeded

size_ok:
    mov r0, 1                       ; Accept
    halt

size_exceeded:
    mov r0, 0                       ; Reject
    halt

; Extended version with error details
validate_size_with_error:
    ; r0 = size, r1 = limit
    ; Returns: r0 = status (1=ok, 0=error), r1 = error code

    beq r0, zero, validate_ok

    bgt r0, r1, validate_too_large

validate_ok:
    mov r0, 1
    mov r1, 0                       ; No error
    ret

validate_too_large:
    mov r0, 0
    mov r1, 413                     ; HTTP 413 Payload Too Large
    ret
