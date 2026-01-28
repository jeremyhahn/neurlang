; @name: JSON Parse Field
; @description: Parses JSON string and extracts a specific field value
; @category: extension/json
; @difficulty: 1
;
; @prompt: parse json and get field
; @prompt: extract value from json string
; @prompt: get json property value
; @prompt: read field from json
; @prompt: json field extraction
; @prompt: parse json object and get key
; @prompt: access json field by name
; @prompt: json get property
; @prompt: extract json key value
; @prompt: parse json get field value
;
; Mock extensions for testing:
; @mock: 170=1                     ; json_parse returns handle 1
; @mock: 172=42                    ; json_get returns mock value 42
; @mock: 179=0                     ; json_free returns success
;
; @test: r0=0 -> r0=42             ; verify we get the extracted field value
;
; @note: Extension IDs: json_parse=170, json_get=172, json_free=179
; @note: Returns extracted value in r0, or 0 on error

.entry main

.section .data
    json_input: .asciz "{\"name\":\"Alice\",\"age\":30}"
    field_key:  .asciz "name"

.section .text

main:
    ; Parse JSON string into handle
    mov r0, json_input
    ext.call r1, json_parse, r0      ; r1 = json handle

    ; Check parse success
    beq r1, zero, error

    ; Get "name" field value
    mov r2, field_key
    ext.call r3, json_get, r1, r2    ; r3 = field value

    ; Free JSON handle
    ext.call r4, json_free, r1

    ; Return the extracted value
    mov r0, r3
    halt

error:
    mov r0, 0
    halt
