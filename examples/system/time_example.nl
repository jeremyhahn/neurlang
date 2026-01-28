; @name: Time Operations Example
; @description: Demonstrates TIME opcode for getting timestamps and sleeping
; @category: system/time
; @difficulty: 1
;
; @prompt: get the current unix timestamp
; @prompt: demonstrate time operations in neurlang
; @prompt: write a program that gets the current time and sleeps
; @prompt: use time.now to get unix timestamp
; @prompt: get monotonic clock time in nanoseconds
; @prompt: sleep for {milliseconds} milliseconds then get time
; @prompt: demonstrate time.now and time.monotonic opcodes
; @prompt: write code to measure elapsed time using monotonic clock
;
; @server: true
; @nondeterministic: true
; @note: time.now returns Unix timestamp in seconds
; @note: time.monotonic returns nanoseconds since system start
; @note: Output varies based on current time (non-deterministic)
;
; Time operations example
; Demonstrates TIME opcode

.entry main

main:
    ; Get current Unix timestamp
    time.now r0                    ; r0 = Unix timestamp in seconds

    ; Store timestamp for later
    mov r1, r0

    ; Sleep for 100 milliseconds
    mov r2, 100
    time.sleep r2

    ; Get monotonic time (nanoseconds)
    time.monotonic r3              ; r3 = monotonic nanoseconds

    ; Result is the original timestamp
    mov r0, r1

    halt

; Output:
; r0 = Unix timestamp at program start
; r3 = Monotonic clock in nanoseconds
