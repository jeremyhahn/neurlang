; @name: Concurrent Worker Example
; @description: Demonstrates SPAWN, JOIN, and CHAN opcodes for concurrent programming
; @category: concurrency/channels
; @difficulty: 3
;
; @prompt: spawn two worker threads that double values and sum results
; @prompt: demonstrate concurrent programming with channels
; @prompt: use spawn to create parallel workers that communicate via channels
; @prompt: write a program that spawns workers and collects results through a channel
; @prompt: demonstrate channel-based communication between spawned tasks
; @prompt: create {count} concurrent workers that process values in parallel
; @prompt: show how to use chan.create, chan.send, and chan.recv
; @prompt: implement a producer-consumer pattern with spawn and channels
;
; @server: true
; @runtime: concurrency
; @note: Workers double 10 and 20, then sum: (10*2) + (20*2) = 60
; @note: Requires concurrency runtime, not unit-testable with standard runner
;
; Concurrency example
; Demonstrates SPAWN, JOIN, and CHAN opcodes

.entry main

; Worker function that doubles its input and sends to channel
worker:
    ; r0 = input value from spawn
    ; r1 = channel id (passed as second arg)

    ; Double the value
    alu.add r2, r0, r0             ; r2 = r0 * 2

    ; Send result to channel
    chan.send r1, r2               ; send r2 to channel r1

    ret

main:
    ; Create a channel for results
    chan.create r5                 ; r5 = channel id

    ; Spawn worker 1 with value 10
    mov r0, 10
    mov r1, r5                     ; pass channel id
    spawn r6, worker, r0           ; r6 = task id for worker 1

    ; Spawn worker 2 with value 20
    mov r0, 20
    mov r1, r5
    spawn r7, worker, r0           ; r7 = task id for worker 2

    ; Receive first result from channel
    chan.recv r0, r5               ; r0 = first result (either 20 or 40)

    ; Receive second result
    chan.recv r1, r5               ; r1 = second result

    ; Add both results
    alu.add r0, r0, r1             ; r0 = 20 + 40 = 60

    ; Join workers (wait for completion)
    join r6
    join r7

    ; Close channel
    chan.close r5

    halt

; Expected output: r0 = 60 (10*2 + 20*2)
