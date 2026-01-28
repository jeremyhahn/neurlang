; @name: Capability-Based Security Demo
; @description: Shows how memory safety works with capabilities for bounds-checked access
; @category: security/capabilities
; @difficulty: 3
;
; @prompt: demonstrate capability-based memory safety
; @prompt: create a capability for a buffer with read write permissions
; @prompt: show how to restrict capabilities to read-only access
; @prompt: implement bounds-checked memory access using capabilities
; @prompt: demonstrate cap.new cap.restrict and cap.query operations
; @prompt: write code that creates and restricts memory capabilities
; @prompt: show capability-based security with permission enforcement
; @prompt: create a {size} byte buffer capability and restrict to first {restricted_size} bytes
;
; @param: buffer="256 bytes" "Memory buffer protected by capability"
;
; @test: -> r0=42
; @note: Demonstrates creating, restricting, and querying capabilities
; @note: Write through restricted read-only capability would trap
;
; Capability-based security demonstration
; Shows how memory safety works with capabilities

.entry main

.data
    buffer: .space 256             ; 256-byte buffer

main:
    ; Create a capability for the buffer
    ; CAP_NEW creates a capability with full permissions
    mov r0, buffer                 ; base address
    mov r1, 256                    ; length
    mov r2, 0b00000111             ; permissions: READ | WRITE | EXEC
    cap.new r3, r0, r1             ; r3 = capability for buffer

    ; Store a value through the capability
    mov r4, 42
    store.d r4, [r3]               ; Store at base (auto bounds-checked)

    ; Restrict the capability to read-only, first 64 bytes
    mov r5, 64
    mov r6, 0b00000001             ; READ only
    cap.restrict r7, r3, r5, r6   ; r7 = restricted capability

    ; Load value through restricted capability (allowed)
    load.d r8, [r7]                ; r8 = 42

    ; Query capability bounds
    cap.query r9, r7, 0            ; r9 = base address
    cap.query r10, r7, 1           ; r10 = length (64)
    cap.query r11, r7, 2           ; r11 = permissions (READ)

    ; Attempting to write through r7 would trap (CapabilityViolation)
    ; Attempting to access beyond 64 bytes would trap (BoundsViolation)

    ; Return the loaded value
    mov r0, r8

    halt

; Output: r0 = 42
; Demonstrates:
; - Creating capabilities with bounds
; - Restricting capabilities (can only shrink)
; - Automatic bounds checking on memory access
