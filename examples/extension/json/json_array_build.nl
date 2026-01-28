; @name: JSON Array Builder
; @description: Builds a JSON array with multiple objects
; @category: extension/json
; @difficulty: 2
;
; @prompt: build json array
; @prompt: create json array of objects
; @prompt: json array construction
; @prompt: build list of json objects
; @prompt: json array builder
; @prompt: create json list
; @prompt: make json array
; @prompt: json collection builder
; @prompt: construct json array
; @prompt: build json object list
;
; @mock: json_new_array=100        ; json_new_array returns handle 100
; @mock: json_new_object=1         ; json_new_object returns handle 1 (reused)
; @mock: json_set=0                ; json_set returns 0 (success)
; @mock: json_array_push=0         ; json_array_push returns 0 (success)
; @mock: json_stringify=12345      ; json_stringify returns mock string pointer
; @mock: json_free=0               ; json_free returns 0 (success)
;
; @test: r0=0 -> r0=12345          ; verify we get stringified result pointer
;
; @note: Uses extensions: json_new_array(211), json_new_object(210), json_set(203), json_array_push(204), json_stringify(201)
; @note: Demonstrates building complex JSON structures

.entry main

.section .data
    key_id:    .asciz "id"
    key_name:  .asciz "name"
    name1:     .asciz "Alice"
    name2:     .asciz "Bob"
    name3:     .asciz "Carol"

.section .text

main:
    ; Create JSON array
    ext.call r10, json_new_array     ; r10 = array handle

    ; === Add first object ===
    ext.call r1, json_new_object
    mov r2, key_id
    mov r3, 1
    ext.call r0, json_set, r1, r2, r3
    mov r2, key_name
    mov r3, name1
    ext.call r0, json_set, r1, r2, r3
    ext.call r0, json_array_push, r10, r1
    ext.call r0, json_free, r1

    ; === Add second object ===
    ext.call r1, json_new_object
    mov r2, key_id
    mov r3, 2
    ext.call r0, json_set, r1, r2, r3
    mov r2, key_name
    mov r3, name2
    ext.call r0, json_set, r1, r2, r3
    ext.call r0, json_array_push, r10, r1
    ext.call r0, json_free, r1

    ; === Add third object ===
    ext.call r1, json_new_object
    mov r2, key_id
    mov r3, 3
    ext.call r0, json_set, r1, r2, r3
    mov r2, key_name
    mov r3, name3
    ext.call r0, json_set, r1, r2, r3
    ext.call r0, json_array_push, r10, r1
    ext.call r0, json_free, r1

    ; Stringify the array
    ext.call r0, json_stringify, r10
    ext.call r4, json_free, r10

    ; r0 = "[{\"id\":1,\"name\":\"Alice\"},{\"id\":2,\"name\":\"Bob\"},{\"id\":3,\"name\":\"Carol\"}]"
    halt
