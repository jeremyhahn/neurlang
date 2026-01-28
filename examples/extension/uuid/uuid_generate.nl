; @name: UUID Generation
; @description: Generates UUID v4 and converts to string
; @category: extension/uuid
; @difficulty: 1
;
; @prompt: generate uuid
; @prompt: create unique id
; @prompt: uuid v4 generation
; @prompt: make random uuid
; @prompt: generate guid
; @prompt: create unique identifier
; @prompt: uuid random generation
; @prompt: new uuid
; @prompt: generate uuid string
; @prompt: create uuid v4
;
; Mock extensions for testing
; @mock: uuid_v4=12345
; @mock: uuid_to_string=67890
;
; @test: -> r0=67890
;
; @note: Uses extensions: uuid_v4(330), uuid_to_string(333)
; @note: Returns pointer to UUID string in r0

.entry main

.section .text

main:
    ; Generate UUID v4
    ext.call r1, uuid_v4             ; r1 = uuid bytes

    ; Convert to string format
    ext.call r0, uuid_to_string, r1  ; r0 = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"

    halt
