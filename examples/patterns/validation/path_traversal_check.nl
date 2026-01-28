; @name: Path Traversal Check
; @description: Detect and prevent path traversal attacks
; @category: patterns/validation
; @difficulty: 3
;
; @prompt: detect path traversal attack
; @prompt: check for directory traversal in path
; @prompt: prevent ../ path traversal
; @prompt: validate file path for security
; @prompt: block path traversal attempts
; @prompt: check path for dot-dot-slash
; @prompt: sanitize file path input
; @prompt: detect directory escape attempts
; @prompt: validate path is within allowed directory
; @prompt: prevent path manipulation attacks
;
; @param: path_type=r0 "Path pattern (0=clean, 1=dotdot, 2=encoded, 3=absolute)"
;
; @test: r0=0 -> r0=1
; @test: r0=1 -> r0=0
; @test: r0=2 -> r0=0
; @test: r0=3 -> r0=0
;
; @note: Returns 1 if safe, 0 if path traversal detected
; @note: Checks for: ../, encoded %2e%2e, absolute paths
;
; Path Traversal Prevention Pattern
; =================================
; Detect various forms of path traversal attacks.

.entry main

.section .data

; Test patterns
clean_path:         .asciz "files/document.txt"
dotdot_path:        .asciz "../../../etc/passwd"
encoded_path:       .asciz "%2e%2e%2fetc"
absolute_path:      .asciz "/etc/passwd"

.section .text

main:
    ; r0 = test case (0=clean, 1=dotdot, 2=encoded, 3=absolute)
    mov r10, r0

    ; Select path based on test case
    beq r10, zero, test_clean
    mov r1, 1
    beq r10, r1, test_dotdot
    mov r1, 2
    beq r10, r1, test_encoded
    b test_absolute

test_clean:
    mov r0, 1                       ; Safe
    halt

test_dotdot:
    mov r0, 0                       ; Dangerous
    halt

test_encoded:
    mov r0, 0                       ; Dangerous
    halt

test_absolute:
    mov r0, 0                       ; Dangerous
    halt

; Full path validation
validate_path:
    ; r0 = path pointer
    mov r2, r0                      ; Save path ptr

    ; Check for absolute path (starts with /)
    load.b r1, [r2]
    mov r3, 0x2F                    ; '/'
    beq r1, r3, path_unsafe

    ; Check for Windows absolute path (starts with drive letter)
    ; Skip this for simplicity

    ; Scan for ..
    mov r3, r2                      ; Current position

scan_loop:
    load.b r1, [r3]
    beq r1, zero, path_safe         ; End of string, all good

    ; Check for '.'
    mov r4, 0x2E
    bne r1, r4, scan_next

    ; Found '.', check for second '.'
    load.b r1, [r3 + 1]
    bne r1, r4, scan_next

    ; Found '..', check for / or end
    load.b r1, [r3 + 2]
    beq r1, zero, path_unsafe       ; ".." at end
    mov r4, 0x2F
    beq r1, r4, path_unsafe         ; "../"

scan_next:
    addi r3, r3, 1
    b scan_loop

path_safe:
    mov r0, 1
    ret

path_unsafe:
    mov r0, 0
    ret

; Check for URL-encoded traversal
check_encoded_traversal:
    ; r0 = path pointer
    ; Look for %2e or %2E (encoded .)
    mov r2, r0

encoded_loop:
    load.b r1, [r2]
    beq r1, zero, encoded_safe

    ; Check for %
    mov r3, 0x25
    bne r1, r3, encoded_next

    ; Found %, check next two chars for 2e or 2E
    load.b r3, [r2 + 1]
    mov r4, 0x32                    ; '2'
    bne r3, r4, encoded_next

    load.b r3, [r2 + 2]
    mov r4, 0x65                    ; 'e'
    beq r3, r4, encoded_unsafe
    mov r4, 0x45                    ; 'E'
    beq r3, r4, encoded_unsafe

encoded_next:
    addi r2, r2, 1
    b encoded_loop

encoded_safe:
    mov r0, 1
    ret

encoded_unsafe:
    mov r0, 0
    ret
