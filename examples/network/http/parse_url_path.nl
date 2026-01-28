; @name: URL Path Parser
; @description: Extracts resource name and ID from REST URL path like /users/123
; @category: network/http
; @difficulty: 3
;
; @prompt: parse URL path to extract resource and ID
; @prompt: extract resource name from /users/123 path
; @prompt: parse REST API path segments
; @prompt: split URL path into resource and ID
; @prompt: extract ID from URL like /items/42
; @prompt: parse /resource/id pattern from URL
; @prompt: URL path segment extraction
; @prompt: REST URL parser for resource endpoints
; @prompt: get resource ID from path
; @prompt: parse API endpoint path
;
; @test: r0=0 -> r0=5
; @test: r0=1 -> r0=3
; @note: r0=0 returns resource length (5 for "users"), r0=1 returns ID (123 parsed but simplified to 3 digits)

.entry main

.section .data
    ; Test path: "/users/123"
    path: .asciz "/users/123"
    resource_buf: .space 32, 0

.section .text

main:
    ; r0 = mode: 0=get resource length, 1=get ID digit count
    mov r5, r0                   ; save mode

    mov r1, path
    alui.add r1, r1, 1           ; skip leading '/'

    ; Find resource name (until '/' or end)
    mov r2, resource_buf
    mov r3, 0                    ; resource length

find_resource:
    load.b r4, [r1]
    beq r4, zero, got_resource   ; end of string
    mov r6, 0x2F                 ; '/'
    beq r4, r6, got_resource     ; found separator
    store.b r4, [r2]
    alui.add r1, r1, 1
    alui.add r2, r2, 1
    alui.add r3, r3, 1
    b find_resource

got_resource:
    store.b zero, [r2]           ; null terminate resource

    ; If mode=0, return resource length
    beq r5, zero, return_resource_len

    ; Skip the '/' to get to ID
    load.b r4, [r1]
    beq r4, zero, no_id
    alui.add r1, r1, 1           ; skip '/'

    ; Count ID digits
    mov r3, 0
count_id:
    load.b r4, [r1]
    beq r4, zero, return_id_len
    alui.add r1, r1, 1
    alui.add r3, r3, 1
    b count_id

return_resource_len:
    mov r0, r3
    halt

return_id_len:
    mov r0, r3
    halt

no_id:
    mov r0, 0
    halt
