; @name: State Machine Example
; @description: Implements a state machine with states and transitions using event-driven dispatch
; @category: patterns/state-machine
; @difficulty: 3
;
; @prompt: implement a state machine with IDLE RUNNING PAUSED STOPPED states
; @prompt: create a state machine that responds to START PAUSE RESUME STOP events
; @prompt: write an event-driven state machine with transition handlers
; @prompt: demonstrate state machine pattern with lookup-based dispatch
; @prompt: build a finite state machine with {num_states} states
; @prompt: implement FSM transitions using branch-based dispatch
; @prompt: create a state machine that processes a sequence of events
; @prompt: write code for state transitions: IDLE->RUNNING->PAUSED->STOPPED
;
; @test: -> r0=3
;
; @note: Returns final state (3=STOPPED) after START, PAUSE, RESUME, STOP
; @note: States: 0=IDLE, 1=RUNNING, 2=PAUSED, 3=STOPPED
; @note: Events: 0=START, 1=PAUSE, 2=RESUME, 3=STOP
;
; State Machine Example
; =====================
; Implements a simple state machine with states and transitions.
; Demonstrates: control flow, lookup tables, function dispatch
;
; States:
;   0 = IDLE
;   1 = RUNNING
;   2 = PAUSED
;   3 = STOPPED
;
; Events:
;   0 = START
;   1 = PAUSE
;   2 = RESUME
;   3 = STOP

.entry main

.section .data

; State names for logging
state_idle:     .asciz "IDLE\n"
state_running:  .asciz "RUNNING\n"
state_paused:   .asciz "PAUSED\n"
state_stopped:  .asciz "STOPPED\n"

; Transition log
log_trans:      .asciz "Transition: "
log_arrow:      .asciz " -> "

; Current state
current_state:  .dword 0          ; Start in IDLE

; Test sequence of events
test_events:    .byte 0, 1, 2, 3  ; START, PAUSE, RESUME, STOP

.section .text

main:
    ; Run test sequence of events
    mov r10, test_events          ; r10 = event pointer
    mov r11, 4                    ; r11 = number of events

    ; Print initial state
    call print_state

event_loop:
    beq r11, zero, done

    ; Load next event
    load.b r0, [r10]

    ; Process event and transition
    call process_event

    ; Print new state
    call print_state

    addi r10, r10, 1              ; Next event
    subi r11, r11, 1
    b event_loop

done:
    ; Return final state
    mov r0, current_state
    load.d r0, [r0]
    halt

; Process event and update state
; Input: r0 = event
; Modifies current_state based on transition table
process_event:
    ; Load current state
    mov r1, current_state
    load.d r1, [r1]               ; r1 = current state

    ; State dispatch
    beq r1, zero, state_0_handler ; IDLE
    mov r2, 1
    beq r1, r2, state_1_handler   ; RUNNING
    mov r2, 2
    beq r1, r2, state_2_handler   ; PAUSED
    mov r2, 3
    beq r1, r2, state_3_handler   ; STOPPED
    ret                           ; Unknown state

; IDLE state handler
state_0_handler:
    ; Only START (event 0) transitions to RUNNING
    beq r0, zero, goto_running
    ret                           ; Ignore other events

; RUNNING state handler
state_1_handler:
    ; PAUSE (1) -> PAUSED, STOP (3) -> STOPPED
    mov r2, 1
    beq r0, r2, goto_paused
    mov r2, 3
    beq r0, r2, goto_stopped
    ret

; PAUSED state handler
state_2_handler:
    ; RESUME (2) -> RUNNING, STOP (3) -> STOPPED
    mov r2, 2
    beq r0, r2, goto_running
    mov r2, 3
    beq r0, r2, goto_stopped
    ret

; STOPPED state handler (terminal state)
state_3_handler:
    ; No transitions from STOPPED
    ret

; State transition helpers
goto_running:
    mov r3, current_state
    mov r4, 1
    store.d r4, [r3]
    ret

goto_paused:
    mov r3, current_state
    mov r4, 2
    store.d r4, [r3]
    ret

goto_stopped:
    mov r3, current_state
    mov r4, 3
    store.d r4, [r3]
    ret

; Print current state name
print_state:
    mov r0, current_state
    load.d r0, [r0]

    beq r0, zero, print_idle
    mov r1, 1
    beq r0, r1, print_running
    mov r1, 2
    beq r0, r1, print_paused
    mov r1, 3
    beq r0, r1, print_stopped
    ret

print_idle:
    mov r0, state_idle
    mov r1, 5
    io.print r2, r0, r1
    ret

print_running:
    mov r0, state_running
    mov r1, 8
    io.print r2, r0, r1
    ret

print_paused:
    mov r0, state_paused
    mov r1, 7
    io.print r2, r0, r1
    ret

print_stopped:
    mov r0, state_stopped
    mov r1, 8
    io.print r2, r0, r1
    ret
