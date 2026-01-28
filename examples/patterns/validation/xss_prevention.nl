; @name: XSS Prevention
; @description: Escape HTML entities to prevent XSS attacks
; @category: patterns/validation
; @difficulty: 3
;
; @prompt: escape html entities for xss prevention
; @prompt: sanitize html output
; @prompt: prevent xss with html escaping
; @prompt: escape angle brackets and quotes
; @prompt: html entity encoding
; @prompt: xss attack prevention filter
; @prompt: escape html special characters
; @prompt: sanitize output for html display
; @prompt: encode html entities safely
; @prompt: prevent cross-site scripting
;
; @param: char=r0 "Character to check"
;
; @test: r0=0x3C -> r0=4
; @test: r0=0x3E -> r0=4
; @test: r0=0x26 -> r0=5
; @test: r0=0x22 -> r0=6
; @test: r0=0x27 -> r0=5
; @test: r0=0x41 -> r0=1
;
; @note: Returns escape sequence length (1 if no escape needed)
; @note: < -> &lt; (4), > -> &gt; (4), & -> &amp; (5), " -> &quot; (6)

.entry main

.section .text

main:
    ; r0 = character to check
    mov r10, r0

    ; Check for < (0x3C)
    mov r1, 0x3C
    beq r10, r1, escape_lt

    ; Check for > (0x3E)
    mov r1, 0x3E
    beq r10, r1, escape_gt

    ; Check for & (0x26)
    mov r1, 0x26
    beq r10, r1, escape_amp

    ; Check for " (0x22)
    mov r1, 0x22
    beq r10, r1, escape_quot

    ; Check for ' (0x27)
    mov r1, 0x27
    beq r10, r1, escape_apos

    ; No escape needed
    mov r0, 1
    halt

escape_lt:
    mov r0, 4                       ; &lt; = 4 chars
    halt

escape_gt:
    mov r0, 4                       ; &gt; = 4 chars
    halt

escape_amp:
    mov r0, 5                       ; &amp; = 5 chars
    halt

escape_quot:
    mov r0, 6                       ; &quot; = 6 chars
    halt

escape_apos:
    mov r0, 5                       ; &#39; = 5 chars
    halt
