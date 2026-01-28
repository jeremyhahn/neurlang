; @name: File I/O Test
; @description: Tests file and console I/O operations including create, write, read
; @category: io/file
; @difficulty: 2
;
; @prompt: write a test program for file I/O operations
; @prompt: create a file write to it and read it back
; @prompt: demonstrate file.open file.write file.read and file.close
; @prompt: test console and file I/O in neurlang
; @prompt: write data to a temp file and verify by reading it back
; @prompt: implement a file write and read test with io.print logging
; @prompt: create /tmp/{filename} write {content} and read it back
; @prompt: demonstrate complete file I/O workflow with error handling
;
; @test: -> r0=0
; @note: Returns 0 on success, 1 on I/O error
; @note: Creates /tmp/neurlang_io_test.txt with test content
;
; Neurlang I/O Test Program
; ======================
; Tests file and console I/O operations
;
; This program:
; 1. Prints a message to console
; 2. Creates a test file
; 3. Writes to it
; 4. Reads from it
; 5. Prints the result

.entry main

.section .data

; Messages
msg_start:      .asciz "Testing Neurlang I/O operations...\n"
msg_write:      .asciz "Writing to test file...\n"
msg_read:       .asciz "Reading from test file...\n"
msg_content:    .asciz "Read content: "
msg_success:    .asciz "\nI/O test completed successfully!\n"
msg_failed:     .asciz "\nI/O test FAILED!\n"
newline:        .asciz "\n"

; Test data
test_file:      .asciz "/tmp/neurlang_io_test.txt"
test_content:   .asciz "Hello from Neurlang!"

; Buffer for reading
read_buffer:    .space 128, 0

.section .text

main:
    ; Print start message (io.print rs1, rs2 -> print(buf@rs1, len@rs2))
    mov r0, msg_start
    mov r1, 33              ; length
    io.print r2, r0, r1     ; rd is unused for print

    ; =========================================
    ; Test 1: Create and write to file
    ; =========================================

    ; file.open rd, rs1, rs2, imm
    ; rd = resulting fd, rs1 = path_addr, rs2 = path_len, imm = flags
    ; Flags: 0x06 = 2 (write) | 4 (create) = 6
    mov r0, test_file       ; path addr
    mov r1, 22              ; path length ("/tmp/neurlang_io_test.txt" = 22 chars)
    file.open r10, r0, r1, 6 ; r10 = fd, flags = 6 (create+write)

    ; Check for error (fd = -1 = 0xFFFFFFFFFFFFFFFF)
    mov r3, -1
    beq r10, r3, io_error

    ; Print write message
    mov r0, msg_write
    mov r1, 24
    io.print r2, r0, r1

    ; file.write rd, rs1, rs2, imm
    ; rd = bytes written, rs1 = fd, rs2 = buf_addr, imm = len
    mov r0, test_content    ; buf addr
    file.write r2, r10, r0, 17  ; write 17 bytes ("Hello from Neurlang!")

    ; Check write result (should be 17)
    blt r2, zero, io_error

    ; Close file
    file.close r0, r10

    ; =========================================
    ; Test 2: Read from file
    ; =========================================

    ; Re-open for reading
    ; Flags: 0x01 = read
    mov r0, test_file
    mov r1, 22
    file.open r10, r0, r1, 1    ; r10 = fd, flags = 1 (read)

    beq r10, r3, io_error

    ; Print read message
    mov r0, msg_read
    mov r1, 25
    io.print r2, r0, r1

    ; file.read rd, rs1, rs2, imm
    ; rd = bytes read, rs1 = fd, rs2 = buf_addr, imm = max_len
    mov r1, read_buffer
    file.read r11, r10, r1, 128  ; r11 = bytes read

    blt r11, zero, io_error

    ; Close file
    file.close r0, r10

    ; Print "Read content: "
    mov r0, msg_content
    mov r1, 14
    io.print r2, r0, r1

    ; Print what we read
    mov r0, read_buffer
    mov r1, r11             ; bytes read
    io.print r2, r0, r1

    ; Print success message
    mov r0, msg_success
    mov r1, 34
    io.print r2, r0, r1

    ; Return success
    mov r0, 0
    halt

io_error:
    ; Print error message
    mov r0, msg_failed
    mov r1, 18
    io.print r2, r0, r1

    ; Return error
    mov r0, 1
    halt
