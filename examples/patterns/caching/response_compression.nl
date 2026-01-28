; @name: Response Compression
; @description: Compress response if client accepts gzip
; @category: patterns/caching
; @difficulty: 3
;
; @prompt: compress response with gzip if accepted
; @prompt: conditional gzip compression
; @prompt: check accept-encoding for gzip
; @prompt: compress response based on accept header
; @prompt: gzip response if client supports
; @prompt: conditional response compression
; @prompt: compress large responses
; @prompt: gzip compression middleware
; @prompt: response compression pattern
; @prompt: compress if client accepts gzip
;
; @param: accepts_gzip=r0 "Client accepts gzip (0/1)"
; @param: content_size=r1 "Size of content"
;
; @test: r0=1, r1=1000 -> r0=1
; @test: r0=0, r1=1000 -> r0=0
; @test: r0=1, r1=100 -> r0=0
;
; @note: Returns 1 if compressed, 0 if not
; @note: Only compresses if client accepts AND size > threshold
;
; Response Compression Pattern
; ============================
; Check Accept-Encoding, compress if beneficial.

.entry main

.section .data

min_compress_size:  .word 512       ; Don't compress below this size

.section .text

main:
    ; r0 = accepts_gzip
    ; r1 = content_size
    mov r10, r0
    mov r11, r1

    ; Check if client accepts gzip
    beq r10, zero, no_compress

    ; Check if content is large enough to benefit
    mov r0, min_compress_size
    load.d r2, [r0]
    blt r11, r2, no_compress

    ; Compress the response
    ; In real impl: ext.call r0, gzip_compress, input, len, output

    mov r0, 1                       ; Compressed
    halt

no_compress:
    mov r0, 0                       ; Not compressed
    halt
