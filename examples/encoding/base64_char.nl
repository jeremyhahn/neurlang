; @name: Base64 Character Encode
; @description: Converts 6-bit value to base64 character
; @category: encoding/base64
; @difficulty: 2
;
; @prompt: encode 6-bit value to base64 char
; @prompt: base64 character encoding
; @prompt: convert index to base64 character
; @prompt: base64 alphabet lookup
; @prompt: get base64 char for value
; @prompt: encode byte as base64
; @prompt: base64 encoding single char
; @prompt: map 0-63 to base64 character
; @prompt: base64 character lookup
; @prompt: convert to base64 alphabet
;
; @param: value=r0 "6-bit value (0-63)"
;
; @test: r0=0 -> r0=65
; @test: r0=26 -> r0=97
; @test: r0=52 -> r0=48
; @test: r0=62 -> r0=43
; @test: r0=63 -> r0=47
; @note: 0-25='A'-'Z', 26-51='a'-'z', 52-61='0'-'9', 62='+', 63='/'

.entry main

main:
    ; r0 = 6-bit value (0-63)

    ; 0-25: 'A'-'Z' (65-90)
    mov r1, 26
    bge r0, r1, not_upper
    alui.add r0, r0, 65          ; 'A' = 65
    halt

not_upper:
    ; 26-51: 'a'-'z' (97-122)
    mov r1, 52
    bge r0, r1, not_lower
    alui.sub r0, r0, 26
    alui.add r0, r0, 97          ; 'a' = 97
    halt

not_lower:
    ; 52-61: '0'-'9' (48-57)
    mov r1, 62
    bge r0, r1, not_digit
    alui.sub r0, r0, 52
    alui.add r0, r0, 48          ; '0' = 48
    halt

not_digit:
    ; 62: '+' (43)
    mov r1, 62
    bne r0, r1, is_slash
    mov r0, 43
    halt

is_slash:
    ; 63: '/' (47)
    mov r0, 47
    halt
