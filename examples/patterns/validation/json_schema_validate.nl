; @name: JSON Schema Validate
; @description: Validate JSON object against expected schema
; @category: patterns/validation
; @difficulty: 3
;
; @prompt: validate json against schema
; @prompt: check json has required fields
; @prompt: validate json object structure
; @prompt: verify json schema compliance
; @prompt: check json field types
; @prompt: validate required json properties
; @prompt: json schema validation
; @prompt: check json object format
; @prompt: validate json request body
; @prompt: verify json has correct fields
;
; @param: has_id=r0 "Has id field (0/1)"
; @param: has_name=r1 "Has name field (0/1)"
; @param: has_email=r2 "Has email field (0/1)"
;
; @test: r0=1, r1=1, r2=1 -> r0=1
; @test: r0=0, r1=1, r2=1 -> r0=0
; @test: r0=1, r1=0, r2=1 -> r0=0
; @test: r0=1, r1=1, r2=0 -> r0=0
;
; @note: Returns 1 if valid, 0 if missing required fields
; @note: Required fields: id, name, email
;
; JSON Schema Validation Pattern
; ==============================
; Check that JSON object contains all required fields.

.entry main

.section .data

; Schema: required = ["id", "name", "email"]
required_count:     .word 3

.section .text

main:
    ; r0 = has_id, r1 = has_name, r2 = has_email
    ; (In real impl, would use ext.call to check JSON)

    ; Check id field
    beq r0, zero, missing_field

    ; Check name field
    beq r1, zero, missing_field

    ; Check email field
    beq r2, zero, missing_field

    ; All required fields present
    mov r0, 1
    halt

missing_field:
    mov r0, 0
    halt

; Full JSON schema validation using extensions
validate_json_schema:
    ; r0 = json handle, r1 = schema definition ptr
    mov r5, r0                      ; Save json handle
    mov r6, r1                      ; Save schema ptr

    ; Get required fields from schema
    ; (Simplified - assume schema is list of field names)
    mov r7, 0                       ; Field index

validate_field_loop:
    ; Get next required field name from schema
    ; (In real impl, would iterate schema array)
    mov r0, r6
    mov r1, r7
    call get_schema_field
    beq r0, zero, all_fields_ok     ; No more fields

    ; Check if JSON has this field
    mov r1, r0                      ; Field name
    mov r0, r5                      ; JSON handle
    ; ext.call json_has_key would go here
    beq r0, zero, field_missing

    ; Field exists, check next
    addi r7, r7, 1
    b validate_field_loop

field_missing:
    mov r0, 0                       ; Validation failed
    ret

all_fields_ok:
    mov r0, 1                       ; Validation passed
    ret

get_schema_field:
    ; Stub - would return field name at index
    mov r0, 0
    ret
