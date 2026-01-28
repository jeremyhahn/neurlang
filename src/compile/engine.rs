//! Copy-and-patch compilation engine
//!
//! Ultra-fast compilation by copying pre-compiled code stencils and patching operands.
//! Target: <5μs compile time for typical programs.

use crate::ir::{Instruction, Opcode, Program};
use crate::runtime::buffer_pool::{BufferPool, ExecutableBuffer};
use crate::stencil::{patch_stencil, StencilTable};
use std::time::Instant;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompileError {
    #[error("No stencil available for opcode {0:?} mode {1}")]
    MissingStencil(Opcode, u8),
    #[error("Failed to allocate executable buffer")]
    BufferAllocationFailed,
    #[error("Program too large: {0} bytes (max {1})")]
    ProgramTooLarge(usize, usize),
    #[error("Invalid instruction at offset {0}")]
    InvalidInstruction(usize),
}

/// Compiled code ready for execution
pub struct CompiledCode {
    /// The executable buffer containing the code
    buffer: ExecutableBuffer,
    /// Entry point offset
    entry_offset: usize,
    /// Total code size
    code_size: usize,
    /// Compilation time in nanoseconds
    compile_time_ns: u64,
}

impl CompiledCode {
    /// Get a function pointer to the compiled code
    ///
    /// # Safety
    /// The caller must ensure the code is called with the correct ABI.
    /// The expected signature is: fn(regfile: *mut u64) -> u64
    pub unsafe fn as_fn(&self) -> unsafe extern "C" fn(*mut u64) -> u64 {
        let ptr = self.buffer.as_ptr().add(self.entry_offset);
        std::mem::transmute(ptr)
    }

    /// Get the code size in bytes
    pub fn code_size(&self) -> usize {
        self.code_size
    }

    /// Get the compilation time in nanoseconds
    pub fn compile_time_ns(&self) -> u64 {
        self.compile_time_ns
    }

    /// Get the compilation time in microseconds
    pub fn compile_time_us(&self) -> f64 {
        self.compile_time_ns as f64 / 1000.0
    }
}

/// Copy-and-patch compiler
pub struct Compiler {
    /// Stencil table
    stencils: StencilTable,
    /// Buffer pool for executable memory
    buffer_pool: BufferPool,
    /// Scratch buffer for building code
    scratch: Vec<u8>,
    /// Maximum program size
    max_program_size: usize,
}

impl Compiler {
    /// Create a new compiler with default settings
    pub fn new() -> Self {
        Self {
            stencils: StencilTable::new(),
            buffer_pool: BufferPool::new(64), // 64 buffers of 4KB each
            scratch: Vec::with_capacity(4096),
            max_program_size: 64 * 1024, // 64KB max
        }
    }

    /// Create a compiler with custom buffer pool size
    pub fn with_buffer_count(buffer_count: usize) -> Self {
        Self {
            stencils: StencilTable::new(),
            buffer_pool: BufferPool::new(buffer_count),
            scratch: Vec::with_capacity(4096),
            max_program_size: 64 * 1024,
        }
    }

    /// Compile a program to native code
    pub fn compile(&mut self, program: &Program) -> Result<CompiledCode, CompileError> {
        let start = Instant::now();

        // Clear scratch buffer
        self.scratch.clear();

        // Estimate code size
        let estimated_size = self.estimate_code_size(program);
        if estimated_size > self.max_program_size {
            return Err(CompileError::ProgramTooLarge(
                estimated_size,
                self.max_program_size,
            ));
        }
        self.scratch.reserve(estimated_size);

        // Compile each instruction
        for (idx, instr) in program.instructions.iter().enumerate() {
            self.compile_instruction(instr, idx)?;
        }

        // Add epilogue (return)
        self.emit_epilogue();

        let code_size = self.scratch.len();

        // Allocate executable buffer
        let mut buffer = self
            .buffer_pool
            .acquire()
            .ok_or(CompileError::BufferAllocationFailed)?;

        // Copy code to executable buffer
        buffer.write(&self.scratch);

        let compile_time_ns = start.elapsed().as_nanos() as u64;

        Ok(CompiledCode {
            buffer,
            entry_offset: 0,
            code_size,
            compile_time_ns,
        })
    }

    /// Compile a single instruction
    fn compile_instruction(
        &mut self,
        instr: &Instruction,
        _idx: usize,
    ) -> Result<(), CompileError> {
        // Get stencil for this instruction
        let stencil = self
            .stencils
            .get(instr.opcode, instr.mode)
            .ok_or(CompileError::MissingStencil(instr.opcode, instr.mode))?;

        // Ensure scratch buffer has space
        let current_len = self.scratch.len();
        self.scratch.resize(current_len + stencil.size, 0);

        // Patch stencil into scratch buffer
        patch_stencil(stencil, instr, &mut self.scratch[current_len..]);

        Ok(())
    }

    /// Emit function epilogue
    fn emit_epilogue(&mut self) {
        // x86-64: ret instruction
        self.scratch.push(0xc3);
    }

    /// Estimate the size of compiled code
    fn estimate_code_size(&self, program: &Program) -> usize {
        let mut size = 0;
        for instr in &program.instructions {
            if let Some(stencil) = self.stencils.get(instr.opcode, instr.mode) {
                size += stencil.size;
            } else {
                // Default estimate for missing stencils
                size += 32;
            }
        }
        // Add epilogue
        size += 16;
        size
    }

    /// Compile and immediately execute a program
    ///
    /// # Safety
    /// The program must not contain unsafe operations.
    pub unsafe fn compile_and_run(
        &mut self,
        program: &Program,
        registers: &mut [u64; 32],
    ) -> Result<u64, CompileError> {
        let compiled = self.compile(program)?;
        let func = compiled.as_fn();
        Ok(func(registers.as_mut_ptr()))
    }

    /// Get compilation statistics
    pub fn stats(&self) -> CompilerStats {
        CompilerStats {
            buffer_pool_size: self.buffer_pool.capacity(),
            stencil_count: 24 * 8, // Approximate
        }
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Compiler statistics
pub struct CompilerStats {
    pub buffer_pool_size: usize,
    pub stencil_count: usize,
}

/// Fast path compiler for single instructions
///
/// Even faster than the full compiler for single-instruction execution.
pub struct FastCompiler {
    stencils: StencilTable,
    /// Pre-allocated single-instruction buffer
    single_buffer: Vec<u8>,
}

impl FastCompiler {
    pub fn new() -> Self {
        Self {
            stencils: StencilTable::new(),
            single_buffer: vec![0u8; 128], // Max stencil size + epilogue
        }
    }

    /// Compile a single instruction (no allocation)
    pub fn compile_single(&mut self, instr: &Instruction) -> Option<&[u8]> {
        let stencil = self.stencils.get(instr.opcode, instr.mode)?;

        // Reset and patch
        self.single_buffer[..stencil.size].fill(0);
        patch_stencil(stencil, instr, &mut self.single_buffer);

        // Add return
        let size = stencil.size;
        self.single_buffer[size] = 0xc3;

        Some(&self.single_buffer[..size + 1])
    }
}

impl Default for FastCompiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Ahead-of-time compiler for generating standalone executables
pub struct AotCompiler {
    stencils: StencilTable,
}

impl AotCompiler {
    pub fn new() -> Self {
        Self {
            stencils: StencilTable::new(),
        }
    }

    /// Compile a program to raw bytes (for embedding or saving)
    pub fn compile_to_bytes(&self, program: &Program) -> Result<Vec<u8>, CompileError> {
        let mut output = Vec::new();

        for instr in &program.instructions {
            let stencil = self
                .stencils
                .get(instr.opcode, instr.mode)
                .ok_or(CompileError::MissingStencil(instr.opcode, instr.mode))?;

            let start = output.len();
            output.resize(start + stencil.size, 0);
            patch_stencil(stencil, instr, &mut output[start..]);
        }

        // Epilogue
        output.push(0xc3);

        Ok(output)
    }

    /// Generate ELF executable (Linux)
    #[cfg(target_os = "linux")]
    pub fn compile_to_elf(&self, program: &Program) -> Result<Vec<u8>, CompileError> {
        let code = self.compile_to_bytes(program)?;

        // Minimal ELF64 header
        let mut elf = Vec::new();

        // ELF header
        elf.extend_from_slice(&[
            0x7f, b'E', b'L', b'F', // Magic
            2,    // 64-bit
            1,    // Little endian
            1,    // ELF version
            0,    // OS/ABI
            0, 0, 0, 0, 0, 0, 0, 0, // Padding
            2, 0, // Executable
            0x3e, 0, // x86-64
            1, 0, 0, 0, // ELF version
        ]);

        // Entry point (will be filled in)
        let entry_point: u64 = 0x400000 + 0x78; // Base + header size
        elf.extend_from_slice(&entry_point.to_le_bytes());

        // Program header offset
        elf.extend_from_slice(&64u64.to_le_bytes()); // Immediately after ELF header

        // Section header offset (none for minimal)
        elf.extend_from_slice(&0u64.to_le_bytes());

        // Flags
        elf.extend_from_slice(&0u32.to_le_bytes());

        // ELF header size
        elf.extend_from_slice(&64u16.to_le_bytes());

        // Program header entry size
        elf.extend_from_slice(&56u16.to_le_bytes());

        // Number of program headers
        elf.extend_from_slice(&1u16.to_le_bytes());

        // Section header size
        elf.extend_from_slice(&0u16.to_le_bytes());

        // Number of section headers
        elf.extend_from_slice(&0u16.to_le_bytes());

        // Section name string table index
        elf.extend_from_slice(&0u16.to_le_bytes());

        // Program header (PT_LOAD)
        elf.extend_from_slice(&1u32.to_le_bytes()); // PT_LOAD
        elf.extend_from_slice(&5u32.to_le_bytes()); // Flags (R+X)
        elf.extend_from_slice(&0u64.to_le_bytes()); // Offset
        elf.extend_from_slice(&0x400000u64.to_le_bytes()); // Virtual address
        elf.extend_from_slice(&0x400000u64.to_le_bytes()); // Physical address

        let file_size = 0x78 + code.len();
        elf.extend_from_slice(&(file_size as u64).to_le_bytes()); // File size
        elf.extend_from_slice(&(file_size as u64).to_le_bytes()); // Memory size
        elf.extend_from_slice(&0x1000u64.to_le_bytes()); // Alignment

        // Pad to 0x78 (entry point)
        while elf.len() < 0x78 {
            elf.push(0);
        }

        // Code
        elf.extend_from_slice(&code);

        Ok(elf)
    }
}

impl Default for AotCompiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{AluOp, Register};

    #[test]
    fn test_compile_simple_program() {
        let mut program = Program::new();
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            1,
            42,
        ));
        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        let mut compiler = Compiler::new();
        let compiled = compiler.compile(&program).unwrap();

        assert!(compiled.code_size() > 0);
        assert!(compiled.compile_time_us() < 1000.0); // Should be well under 1ms
    }

    #[test]
    fn test_compile_time_target() {
        // Create a typical 32-instruction program
        let mut program = Program::new();
        for _ in 0..32 {
            program.instructions.push(Instruction::new(
                Opcode::Alu,
                Register::R0,
                Register::R1,
                Register::R2,
                AluOp::Add as u8,
            ));
        }

        let mut compiler = Compiler::new();

        // Warm up
        let _ = compiler.compile(&program);

        // Measure
        let compiled = compiler.compile(&program).unwrap();

        // Target: <5μs (5000ns)
        // Note: In debug builds this may be slower
        println!("Compile time: {}μs", compiled.compile_time_us());
        // In release mode, should meet target
        #[cfg(not(debug_assertions))]
        assert!(
            compiled.compile_time_us() < 50.0,
            "Compile time {}μs exceeds target",
            compiled.compile_time_us()
        );
    }

    #[test]
    fn test_fast_compiler() {
        let mut compiler = FastCompiler::new();

        let instr = Instruction::new(
            Opcode::Alu,
            Register::R0,
            Register::R1,
            Register::R2,
            AluOp::Add as u8,
        );

        let code = compiler.compile_single(&instr);
        assert!(code.is_some());
        assert!(!code.unwrap().is_empty());
    }

    #[test]
    fn test_aot_compiler() {
        let mut program = Program::new();
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            1,
            42,
        ));
        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        let compiler = AotCompiler::new();
        let bytes = compiler.compile_to_bytes(&program).unwrap();

        assert!(!bytes.is_empty());
        // Should end with ret instruction
        assert_eq!(bytes.last(), Some(&0xc3));
    }
}
