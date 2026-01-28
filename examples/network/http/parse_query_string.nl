; @name: Query String Parser
; @description: Counts parameters in URL query string like ?page=1&limit=10
; @category: network/http
; @difficulty: 3
;
; @prompt: parse URL query string parameters
; @prompt: count query string parameters
; @prompt: extract params from ?key=value&key2=value2
; @prompt: parse HTTP query string
; @prompt: count ampersands in query string
; @prompt: URL parameter counter
; @prompt: parse GET parameters from URL
; @prompt: query string parameter extraction
; @prompt: count URL params
; @prompt: HTTP GET query parser
;
; @test: r0=2 -> r0=2
; @note: Returns number of parameters (2 for "?page=1&limit=10")

.entry main

.section .data
    ; Test query: "?page=1&limit=10"
    query: .asciz "?page=1&limit=10"

.section .text

main:
    mov r1, query

    ; Skip '?' if present
    load.b r2, [r1]
    mov r3, 0x3F                 ; '?'
    bne r2, r3, start_count
    alui.add r1, r1, 1

start_count:
    ; Count parameters (1 + number of '&')
    mov r0, 0                    ; param count
    mov r4, 0                    ; saw any content?

count_loop:
    load.b r2, [r1]
    beq r2, zero, done

    ; Check for '&' (parameter separator)
    mov r3, 0x26                 ; '&'
    bne r2, r3, not_amp
    alui.add r0, r0, 1           ; count separator
    b next_char

not_amp:
    ; Check for '=' (means we have a param)
    mov r3, 0x3D                 ; '='
    bne r2, r3, next_char
    bne r4, zero, next_char      ; already counted this param
    mov r4, 1                    ; mark that we saw content

next_char:
    alui.add r1, r1, 1
    b count_loop

done:
    ; If we saw content, we have params = separators + 1
    beq r4, zero, no_params
    alui.add r0, r0, 1
    halt

no_params:
    mov r0, 0
    halt
