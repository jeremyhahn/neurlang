//! Benchmarks for Neurlang compilation and execution

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use neurlang::compile::Compiler;
use neurlang::interp::Interpreter;
use neurlang::ir::{AluOp, Assembler, Instruction, Opcode, Program, Register};
use neurlang::runtime::BufferPool;

/// Benchmark compile time for varying program sizes
fn bench_compile_time(c: &mut Criterion) {
    let mut group = c.benchmark_group("compile_time");

    for &size in &[1, 8, 16, 32, 64, 128] {
        let program = create_program(size);
        group.throughput(Throughput::Elements(size as u64));

        let mut compiler = Compiler::new();

        // Warm up
        let _ = compiler.compile(&program);

        group.bench_function(format!("{}_instrs", size), |b| {
            b.iter(|| {
                let result = compiler.compile(&program);
                black_box(result)
            })
        });
    }

    group.finish();
}

/// Benchmark buffer pool acquisition
fn bench_buffer_pool(c: &mut Criterion) {
    let pool = BufferPool::new(64);

    c.bench_function("buffer_acquire_release", |b| {
        b.iter(|| {
            let buf = pool.acquire().unwrap();
            black_box(&buf);
            drop(buf);
        })
    });
}

/// Benchmark interpreter execution
fn bench_interpreter(c: &mut Criterion) {
    let mut group = c.benchmark_group("interpreter");

    // Fibonacci benchmark
    let fib_program = create_fib_program(20);
    group.bench_function("fib_20", |b| {
        b.iter(|| {
            let mut interp = Interpreter::new(1024);
            let result = interp.execute(&fib_program);
            black_box(result)
        })
    });

    // Simple loop benchmark
    let loop_program = create_loop_program(1000);
    group.bench_function("loop_1000", |b| {
        b.iter(|| {
            let mut interp = Interpreter::new(1024);
            let result = interp.execute(&loop_program);
            black_box(result)
        })
    });

    group.finish();
}

/// Benchmark assembler
fn bench_assembler(c: &mut Criterion) {
    let source = r#"
        mov r0, 10
        mov r1, 1
        loop:
            add r2, r0, r1
            sub r0, r0, r1
            bne r0, zero, loop
        halt
    "#;

    c.bench_function("assemble_simple", |b| {
        b.iter(|| {
            let mut asm = Assembler::new();
            let result = asm.assemble(black_box(source));
            black_box(result)
        })
    });
}

/// Create a program with N instructions
fn create_program(size: usize) -> Program {
    let mut program = Program::new();

    // Add some MOV instructions
    for _i in 0..size.saturating_sub(1) {
        program.instructions.push(Instruction::new(
            Opcode::Alu,
            Register::R0,
            Register::R1,
            Register::R2,
            AluOp::Add as u8,
        ));
    }

    // End with HALT
    program.instructions.push(Instruction::new(
        Opcode::Halt,
        Register::Zero,
        Register::Zero,
        Register::Zero,
        0,
    ));

    program
}

/// Create fibonacci program
fn create_fib_program(n: i32) -> Program {
    let mut asm = Assembler::new();
    let source = format!(
        r#"
        mov r0, {}
        mov r1, 0
        mov r2, 1
        loop:
            beq r0, zero, done
            add r3, r1, r2
            mov r1, r2
            mov r2, r3
            subi r0, r0, 1
            b loop
        done:
            mov r0, r1
            halt
        "#,
        n
    );
    asm.assemble(&source).unwrap()
}

/// Create simple loop program
fn create_loop_program(n: i32) -> Program {
    let mut asm = Assembler::new();
    let source = format!(
        r#"
        mov r0, {}
        mov r1, 0
        loop:
            beq r0, zero, done
            addi r1, r1, 1
            subi r0, r0, 1
            b loop
        done:
            halt
        "#,
        n
    );
    asm.assemble(&source).unwrap()
}

criterion_group!(
    benches,
    bench_compile_time,
    bench_buffer_pool,
    bench_interpreter,
    bench_assembler,
);
criterion_main!(benches);
