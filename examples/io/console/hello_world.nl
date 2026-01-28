; @name: Hello World
; @description: Prints "Hello, Neurlang!" to stdout using I/O opcodes
; @category: io/basic
; @difficulty: 1
;
; @prompt: print hello world
; @prompt: hello world program
; @prompt: output "Hello, Neurlang!"
; @prompt: write hello to stdout
; @prompt: simple print example
; @prompt: io.print demonstration
;
; @test: -> r0=0
; @note: Returns 0 on success, prints "Hello, Neurlang!" to stdout

.entry main

.data
    hello: .ascii "Hello, Neurlang!\n"

main:
    ; Load address of string into r0
    mov r0, hello
    mov r1, 17                     ; length of string

    ; Print the string (io.print rd, buf_addr, length)
    io.print r2, r0, r1

    ; Return success (0)
    mov r0, 0
    halt
