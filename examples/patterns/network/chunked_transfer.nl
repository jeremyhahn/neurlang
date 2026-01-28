; @name: Chunked Transfer
; @description: HTTP chunked transfer encoding for streaming
; @category: patterns/network
; @difficulty: 4
;
; @prompt: implement chunked transfer encoding
; @prompt: stream http response with chunks
; @prompt: http chunked encoding
; @prompt: send response in chunks
; @prompt: chunked response streaming
; @prompt: transfer-encoding chunked
; @prompt: stream large response in chunks
; @prompt: http chunked mode handler
; @prompt: parse chunked encoding
; @prompt: chunked http response
;
; @param: chunk_size=r0 "Size of chunk to send"
; @param: is_last=r1 "Is this the final chunk (0/1)"
;
; @test: r0=100, r1=0 -> r0=100
; @test: r0=0, r1=1 -> r0=0
; @test: r0=50, r1=1 -> r0=50
;
; @note: Returns chunk size sent
; @note: Final chunk is size 0
; @note: Testable chunked encoding logic
;
; Chunked Transfer Pattern
; ========================
; Stream response without knowing total size upfront.

.entry main

.section .data

chunk_header:       .space 16, 0    ; Hex size + CRLF

.section .text

main:
    ; r0 = chunk size
    ; r1 = is_last chunk
    mov r10, r0
    mov r11, r1

    ; Write chunk header (size in hex + CRLF)
    mov r0, r10
    call write_chunk_header

    ; If size > 0, write chunk data
    beq r10, zero, write_trailer

    ; Write chunk data
    ; (Would send actual data here)

    ; Write CRLF after chunk
    call write_crlf

    ; Check if last chunk
    bne r11, zero, write_final_chunk

    mov r0, r10
    halt

write_trailer:
write_final_chunk:
    ; Write final chunk (0-size)
    mov r0, 0
    call write_chunk_header
    call write_crlf                 ; Final CRLF after 0 chunk

    mov r0, r10
    halt

write_chunk_header:
    ; r0 = chunk size
    ; Write "{size_hex}\r\n"
    mov r1, chunk_header

    ; Convert size to hex (simplified)
    call int_to_hex
    ; r1 now has hex string

    ; Append CRLF
    call write_crlf

    ret

int_to_hex:
    ; Convert r0 to hex string at r1
    ; (Simplified - actual impl would do full conversion)
    ret

write_crlf:
    ; Write \r\n
    mov r0, 0x0D                    ; CR
    mov r1, 0x0A                    ; LF
    ret

; Parse incoming chunked data
parse_chunked_request:
    ; r0 = input buffer ptr

parse_chunk_loop:
    ; Read chunk size (hex)
    call read_chunk_size
    mov r10, r0                     ; chunk size

    ; Size 0 means end
    beq r10, zero, chunks_done

    ; Read chunk data
    mov r0, r10
    call read_chunk_data

    ; Read trailing CRLF
    call skip_crlf

    b parse_chunk_loop

chunks_done:
    ; Read trailing headers (if any)
    call read_trailers
    ret

read_chunk_size:
    ; Parse hex size from line
    mov r0, 0                       ; Stub
    ret

read_chunk_data:
    ; Read r0 bytes of chunk data
    ret

skip_crlf:
    ; Skip \r\n
    ret

read_trailers:
    ; Read optional trailer headers after final chunk
    ret
