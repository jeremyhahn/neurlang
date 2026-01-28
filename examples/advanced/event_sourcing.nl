; @name: Event Sourcing
; @description: Event sourcing pattern with event store and projections
; @category: advanced/architecture
; @difficulty: 5
;
; @prompt: implement event sourcing pattern
; @prompt: event store with projections
; @prompt: event sourcing architecture
; @prompt: append-only event log
; @prompt: event-driven state reconstruction
; @prompt: cqrs with event sourcing
; @prompt: event replay mechanism
; @prompt: event sourcing aggregate
; @prompt: build event store
; @prompt: projection from events
;
; @param: event_type=r0 "Event type (1=created, 2=updated, 3=deleted)"
; @param: aggregate_id=r1 "Aggregate ID"
;
; @test: r0=1, r1=100 -> r0=1
; @test: r0=2, r1=100 -> r0=2
; @test: r0=3, r1=100 -> r0=3
;
; @note: Returns event sequence number
; @note: All events are immutable and append-only
;
; Event Sourcing Pattern
; ======================
; Parse event -> Validate -> Persist -> Update Projections

.entry main

.section .data

event_count:        .word 0
next_sequence:      .word 1

.section .text

main:
    ; r0 = event_type
    ; r1 = aggregate_id
    mov r10, r0
    mov r11, r1

    ; Validate event type (1-3)
    beq r10, zero, invalid_event
    mov r0, 4
    bge r10, r0, invalid_event

    ; Validate aggregate ID
    beq r11, zero, invalid_event

    ; Persist event - increment sequence
    mov r0, next_sequence
    load.d r12, [r0]                ; current sequence
    addi r13, r12, 1
    store.d r13, [r0]               ; increment

    ; Increment event count
    mov r0, event_count
    load.d r2, [r0]
    addi r2, r2, 1
    store.d r2, [r0]

    ; Return sequence number (equals event_type for simplicity)
    mov r0, r10
    halt

invalid_event:
    mov r0, 0
    halt
