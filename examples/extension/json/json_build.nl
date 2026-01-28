; @name: JSON Object Builder
; @description: Builds a JSON object with key-value pairs using extensions
; @category: data/json
; @difficulty: 2
;
; @prompt: build a JSON object with id and name fields
; @prompt: create JSON response with {key} field
; @prompt: construct JSON object using ext.call
; @prompt: build JSON {"id": 1, "name": "test"}
; @prompt: create a JSON object and stringify it
; @prompt: demonstrate JSON object creation with extensions
; @prompt: build JSON response object
; @prompt: use json_new_object and json_set to build JSON
;
; @mock: json_new_object=1         ; json_new_object returns handle 1
; @mock: json_set=0                ; json_set returns 0 (success)
; @mock: json_stringify=12345      ; json_stringify returns mock string pointer
; @mock: json_free=0               ; json_free returns 0 (success)
;
; @test: r0=0 -> r0=1              ; verify we get success after building JSON
;
; @note: Returns 1 on success (JSON handle created)
; @note: Uses extensions: json_new_object(210), json_set(203), json_stringify(201)
;
; JSON Object Builder
; ===================
; Demonstrates building JSON objects using the extension system.
; This pattern is used for REST API responses.

.entry main

.section .data

key_id:     .asciz "id"
key_name:   .asciz "name"
value_name: .asciz "test"
result_buf: .space 256, 0

.section .text

main:
    ; Create new JSON object
    ; json_new_object() -> handle in r1
    ext.call r1, json_new_object

    ; Check if creation succeeded
    beq r1, zero, error

    ; Set "id" field to integer 1
    ; json_set(handle, key, value)
    mov r2, key_id
    mov r3, 1
    ext.call r4, json_set, r1, r2

    ; Set "name" field to "test"
    mov r2, key_name
    mov r3, value_name
    ext.call r4, json_set, r1, r2

    ; Stringify the object -> string in r5
    ext.call r5, json_stringify, r1

    ; Free the handle
    ext.call r4, json_free, r1

    ; Return success
    mov r0, 1
    halt

error:
    mov r0, 0
    halt
