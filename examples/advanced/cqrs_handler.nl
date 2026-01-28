; @name: CQRS Handler
; @description: Command Query Responsibility Segregation pattern
; @category: advanced/architecture
; @difficulty: 5
;
; @prompt: implement cqrs pattern
; @prompt: command query separation
; @prompt: cqrs command handler
; @prompt: cqrs query handler
; @prompt: separate read and write models
; @prompt: cqrs with event sourcing
; @prompt: command query responsibility segregation
; @prompt: cqrs read model
; @prompt: cqrs write model
; @prompt: implement cqrs architecture
;
; @param: is_command=r0 "Is command (1) or query (0)"
; @param: operation_type=r1 "Operation type (1=create, 2=read, 3=update)"
;
; @test: r0=1, r1=1 -> r0=1
; @test: r0=1, r1=3 -> r0=1
; @test: r0=0, r1=2 -> r0=1
; @test: r0=0, r1=1 -> r0=0
;
; @note: Commands modify state, queries only read
; @note: Returns 1 for valid operation, 0 for invalid
;
; CQRS Pattern
; ============
; Commands -> Write Model -> Event -> Read Model
; Queries -> Read Model (optimized)

.entry main

.section .data

write_version:      .word 0
read_version:       .word 0

.section .text

main:
    ; r0 = is_command (1=command, 0=query)
    ; r1 = operation_type
    mov r10, r0
    mov r11, r1

    ; Route to command or query handler
    bne r10, zero, handle_command

    ; Query - only reads allowed (type 2)
    mov r0, 2
    bne r11, r0, invalid_query

    ; Valid query
    mov r0, 1
    halt

handle_command:
    ; Commands can be create(1) or update(3), not read(2)
    mov r0, 2
    beq r11, r0, invalid_command

    ; Valid command - update write model version
    mov r0, write_version
    load.d r2, [r0]
    addi r2, r2, 1
    store.d r2, [r0]

    ; Sync to read model
    mov r0, read_version
    store.d r2, [r0]

    mov r0, 1
    halt

invalid_query:
invalid_command:
    mov r0, 0
    halt
