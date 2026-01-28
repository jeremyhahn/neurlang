; @name: ETL Pipeline
; @description: Extract-Transform-Load data pipeline pattern
; @category: advanced/data
; @difficulty: 4
;
; @prompt: implement etl pipeline
; @prompt: data extraction and loading
; @prompt: etl data transformation
; @prompt: build data pipeline
; @prompt: extract transform load pattern
; @prompt: data processing pipeline
; @prompt: etl batch processing
; @prompt: data pipeline with validation
; @prompt: etl workflow implementation
; @prompt: streaming etl pipeline
;
; @param: record_count=r0 "Number of records to process"
; @param: has_errors=r1 "Records contain errors (0/1)"
;
; @test: r0=100, r1=0 -> r0=100
; @test: r0=50, r1=1 -> r0=45
; @test: r0=0, r1=0 -> r0=0
;
; @note: Returns count of successfully processed records
; @note: Invalid records are filtered during transform
;
; ETL Pipeline Pattern
; ====================
; Extract from source -> Transform/validate -> Load to destination

.entry main

.section .data

records_loaded:     .word 0

.section .text

main:
    ; r0 = record_count
    ; r1 = has_errors (10% error rate if 1)
    mov r10, r0
    mov r11, r1

    ; If no records, return 0
    beq r10, zero, done_zero

    ; Calculate valid records
    ; If has_errors: valid = count - (count / 10)
    beq r11, zero, no_errors

    ; 10% error rate
    mov r0, 10
    muldiv.div r2, r10, r0          ; errors = count / 10
    sub r10, r10, r2                ; valid = count - errors
    b load_records

no_errors:
    ; All records valid
    b load_records

load_records:
    ; Store loaded count
    mov r0, records_loaded
    store.d r10, [r0]

    mov r0, r10
    halt

done_zero:
    mov r0, 0
    halt
