; @name: Config File Parser
; @description: Parses key=value pairs from a config file into memory structures
; @category: parsing/config
; @difficulty: 3
;
; @prompt: parse a config file with key=value pairs
; @prompt: read and parse configuration from {filename}
; @prompt: implement a config file parser that stores key-value pairs
; @prompt: parse config format key1=value1 into memory arrays
; @prompt: write a configuration parser with file I/O and string operations
; @prompt: load config file and extract keys and values into separate buffers
; @prompt: demonstrate file reading and string parsing for config files
; @prompt: create a parser for simple key=value configuration format
;
; @test: -> r0=0
; @note: Config format is key=value with one pair per line
; @note: Returns number of config entries parsed in r0 (0 when no file)
; @note: Maximum 8 key-value pairs supported
;
; Simple Config File Parser
; =========================
; Parses key=value pairs from a config file.
; Demonstrates: file I/O, string parsing, memory operations
;
; Config format:
;   key1=value1
;   key2=value2
;
; Loads and parses config into memory structure.

.entry main

.section .data

config_file:    .asciz "config.txt"
log_parse:      .asciz "Parsing config...\n"
log_done:       .asciz "Config loaded\n"

; Buffers
file_buf:       .space 1024, 0    ; Raw file contents
key_buf:        .space 64, 0      ; Current key
value_buf:      .space 256, 0     ; Current value

; Parsed config storage (simplified: 8 key-value pairs max)
config_keys:    .space 512, 0     ; 8 x 64 bytes
config_values:  .space 2048, 0    ; 8 x 256 bytes
config_count:   .word 0           ; Number of pairs

.section .text

main:
    ; Print status
    mov r0, log_parse
    mov r1, 18
    io.print r2, r0, r1

    ; Open config file
    mov r0, config_file
    mov r1, 10                    ; "config.txt" length
    file.open r5, r0, r1, 1       ; flags=1 (read only)

    ; Check for error
    mov r3, -1
    beq r5, r3, no_file

    ; Read file contents
    mov r1, file_buf
    mov r2, 1024
    file.read r6, r5, r1, 0       ; r6 = bytes read
    file.close r0, r5

    ; Parse the config
    mov r7, 0                     ; current offset
    mov r8, 0                     ; config entry count

parse_loop:
    bge r7, r6, parse_done

    ; Parse key (until '=' or newline)
    mov r0, file_buf
    add r0, r0, r7
    mov r1, key_buf
    call parse_until_eq           ; Returns key length in r0

    beq r0, zero, skip_line       ; Empty key, skip

    ; Skip '='
    addi r7, r7, 1
    add r7, r7, r0

    ; Parse value (until newline)
    mov r0, file_buf
    add r0, r0, r7
    mov r1, value_buf
    call parse_until_newline      ; Returns value length in r0

    ; Store key and value in config arrays
    ; key_ptr = config_keys + (config_count * 64)
    mov r2, r8
    mov r3, 64
    mul r2, r2, r3
    mov r3, config_keys
    add r3, r3, r2                ; r3 = key dest

    mov r0, key_buf
    call copy_string              ; Copy key to config_keys[count]

    ; value_ptr = config_values + (config_count * 256)
    mov r2, r8
    mov r3, 256
    mul r2, r2, r3
    mov r3, config_values
    add r3, r3, r2                ; r3 = value dest

    mov r0, value_buf
    call copy_string              ; Copy value to config_values[count]

    addi r8, r8, 1                ; config_count++
    mov r0, 8
    beq r8, r0, parse_done        ; Max 8 entries

    b parse_loop

skip_line:
    ; Find next newline and continue
    mov r0, file_buf
    add r0, r0, r7

skip_loop:
    bge r7, r6, parse_done
    load.b r1, [r0]
    mov r2, 0x0A
    beq r1, r2, skip_done
    addi r0, r0, 1
    addi r7, r7, 1
    b skip_loop

skip_done:
    addi r7, r7, 1                ; Skip the newline
    b parse_loop

parse_done:
    ; Store config count
    mov r0, config_count
    store.d r8, [r0]

    ; Print done
    mov r0, log_done
    mov r1, 14
    io.print r2, r0, r1

    ; Return number of config entries
    mov r0, r8
    halt

no_file:
    mov r0, 0
    halt

; Parse until '=' character
; Input: r0 = source, r1 = dest
; Output: r0 = length, advances pointer
parse_until_eq:
    mov r9, 0                     ; length

parse_eq_loop:
    load.b r2, [r0]
    beq r2, zero, parse_eq_done
    mov r3, 0x3D                  ; '='
    beq r2, r3, parse_eq_done
    mov r3, 0x0A                  ; newline
    beq r2, r3, parse_eq_done
    store.b r2, [r1]
    addi r0, r0, 1
    addi r1, r1, 1
    addi r9, r9, 1
    b parse_eq_loop

parse_eq_done:
    store.b zero, [r1]            ; Null terminate
    mov r0, r9
    ret

; Parse until newline
; Input: r0 = source, r1 = dest
; Output: r0 = length
parse_until_newline:
    mov r9, 0

parse_nl_loop:
    load.b r2, [r0]
    beq r2, zero, parse_nl_done
    mov r3, 0x0A
    beq r2, r3, parse_nl_done
    mov r3, 0x0D                  ; CR
    beq r2, r3, parse_nl_done
    store.b r2, [r1]
    addi r0, r0, 1
    addi r1, r1, 1
    addi r9, r9, 1
    b parse_nl_loop

parse_nl_done:
    store.b zero, [r1]
    mov r0, r9
    ret

; Copy null-terminated string from r0 to r3
copy_string:
    mov r4, 0

copy_loop:
    load.b r5, [r0]
    store.b r5, [r3]
    beq r5, zero, copy_done
    addi r0, r0, 1
    addi r3, r3, 1
    addi r4, r4, 1
    b copy_loop

copy_done:
    ret
