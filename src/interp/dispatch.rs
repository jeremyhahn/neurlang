//! Interpreter for small programs
//!
//! Uses computed goto (via match) for fast dispatch.
//! Recommended for programs with <10 instructions where compile overhead > execution time.

use super::coverage::CoverageTracker;
use crate::ir::{
    AluOp, AtomicOp, BitsOp, BranchCond, FileOp, FpuOp, Instruction, IoOp, MemWidth, MulDivOp,
    NetOp, NetOption, Opcode, Program, RandOp, Register, TimeOp, TrapType, DATA_BASE,
};
use crate::runtime::extensions::ExtensionRegistry;
use crate::stencil::io::{IOPermissions, IORuntime};
use crate::stencil::security::{SecurityContext, TaintLevel};
use std::sync::atomic::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};

/// Interpreter execution result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterpResult {
    /// Normal completion
    Ok(u64),
    /// Halted by HALT instruction
    Halted,
    /// Trapped (syscall, breakpoint, etc.)
    Trapped(TrapType),
    /// Division by zero
    DivByZero,
    /// Invalid instruction
    InvalidInstruction,
    /// Out of bounds memory access
    OutOfBounds,
    /// Capability violation
    CapabilityViolation,
    /// Maximum instructions exceeded
    MaxInstructionsExceeded,
}

/// Interpreter state
pub struct Interpreter {
    /// Register file (64-bit values)
    pub registers: [u64; 32],
    /// Memory (simulated)
    memory: Vec<u8>,
    /// Program counter
    pc: usize,
    /// Call stack for nested function calls
    call_stack: Vec<usize>,
    /// Security context
    security: SecurityContext,
    /// Instruction count
    instruction_count: u64,
    /// Maximum instructions to execute
    max_instructions: u64,
    /// I/O Runtime for file, network, console operations
    io_runtime: IORuntime,
    /// Whether data section has been loaded (for test memory setup)
    data_section_loaded: bool,
    /// Extension registry for ext.call opcode
    extensions: ExtensionRegistry,
    /// Coverage tracker (optional)
    coverage: Option<CoverageTracker>,
}

impl Interpreter {
    /// Create a new interpreter with given memory size
    /// Uses fully permissive I/O by default for convenience
    pub fn new(memory_size: usize) -> Self {
        Self::with_permissions(memory_size, IOPermissions::allow_all())
    }

    /// Create a new interpreter with custom I/O permissions
    pub fn with_permissions(memory_size: usize, permissions: IOPermissions) -> Self {
        Self {
            registers: [0; 32],
            memory: vec![0; memory_size],
            pc: 0,
            call_stack: Vec::with_capacity(256), // Support up to 256 nested calls
            security: SecurityContext::new(),
            instruction_count: 0,
            max_instructions: 1_000_000, // 1M default limit
            io_runtime: IORuntime::new(permissions),
            extensions: ExtensionRegistry::new(),
            coverage: None,
            data_section_loaded: false,
        }
    }

    /// Set maximum instructions to execute
    pub fn with_max_instructions(mut self, max: u64) -> Self {
        self.max_instructions = max;
        self
    }

    /// Set a custom extension registry
    pub fn with_extensions(mut self, extensions: ExtensionRegistry) -> Self {
        self.extensions = extensions;
        self
    }

    /// Enable coverage tracking
    pub fn with_coverage(mut self, total_instructions: usize) -> Self {
        self.coverage = Some(CoverageTracker::new(total_instructions));
        self
    }

    /// Get a mutable reference to the extension registry for setting mocks
    pub fn extensions_mut(&mut self) -> &mut ExtensionRegistry {
        &mut self.extensions
    }

    /// Get an immutable reference to the extension registry
    pub fn extensions(&self) -> &ExtensionRegistry {
        &self.extensions
    }

    /// Get a register value
    #[inline]
    fn get_reg(&self, reg: Register) -> u64 {
        if matches!(reg, Register::Zero) {
            0
        } else {
            self.registers[reg as usize]
        }
    }

    /// Set a register value
    #[inline]
    fn set_reg(&mut self, reg: Register, value: u64) {
        if reg.is_writable() {
            self.registers[reg as usize] = value;
        }
    }

    /// Read from memory
    fn read_mem(&self, addr: u64, width: MemWidth) -> Result<u64, InterpResult> {
        let addr = addr as usize;
        let size = width.byte_size();

        if addr + size > self.memory.len() {
            return Err(InterpResult::OutOfBounds);
        }

        let value = match width {
            MemWidth::Byte => self.memory[addr] as u64,
            MemWidth::Half => {
                let bytes = [self.memory[addr], self.memory[addr + 1]];
                u16::from_le_bytes(bytes) as u64
            }
            MemWidth::Word => {
                let bytes = [
                    self.memory[addr],
                    self.memory[addr + 1],
                    self.memory[addr + 2],
                    self.memory[addr + 3],
                ];
                u32::from_le_bytes(bytes) as u64
            }
            MemWidth::Double => {
                let bytes = [
                    self.memory[addr],
                    self.memory[addr + 1],
                    self.memory[addr + 2],
                    self.memory[addr + 3],
                    self.memory[addr + 4],
                    self.memory[addr + 5],
                    self.memory[addr + 6],
                    self.memory[addr + 7],
                ];
                u64::from_le_bytes(bytes)
            }
        };

        Ok(value)
    }

    /// Write to memory
    fn write_mem(&mut self, addr: u64, value: u64, width: MemWidth) -> Result<(), InterpResult> {
        let addr = addr as usize;
        let size = width.byte_size();

        if addr + size > self.memory.len() {
            return Err(InterpResult::OutOfBounds);
        }

        match width {
            MemWidth::Byte => {
                self.memory[addr] = value as u8;
            }
            MemWidth::Half => {
                let bytes = (value as u16).to_le_bytes();
                self.memory[addr..addr + 2].copy_from_slice(&bytes);
            }
            MemWidth::Word => {
                let bytes = (value as u32).to_le_bytes();
                self.memory[addr..addr + 4].copy_from_slice(&bytes);
            }
            MemWidth::Double => {
                let bytes = value.to_le_bytes();
                self.memory[addr..addr + 8].copy_from_slice(&bytes);
            }
        }

        Ok(())
    }

    /// Pre-load the data section into memory (call before setting up test memory)
    pub fn load_data_section(&mut self, program: &Program) {
        if self.data_section_loaded {
            return; // Already loaded, don't overwrite test memory
        }
        if !program.data_section.is_empty() {
            let data_start = DATA_BASE as usize;
            let data_end = data_start + program.data_section.len();

            // Ensure memory is large enough
            if data_end > self.memory.len() {
                self.memory.resize(data_end, 0);
            }

            // Copy data section to memory
            self.memory[data_start..data_end].copy_from_slice(&program.data_section);
        }
        self.data_section_loaded = true;
    }

    /// Execute a program
    pub fn execute(&mut self, program: &Program) -> InterpResult {
        self.pc = program.entry_point;
        self.instruction_count = 0;

        // Load data section into memory at DATA_BASE (skipped if already loaded)
        self.load_data_section(program);

        loop {
            if self.instruction_count >= self.max_instructions {
                return InterpResult::MaxInstructionsExceeded;
            }

            if self.pc >= program.instructions.len() {
                return InterpResult::Ok(self.get_reg(Register::R0));
            }

            // Track coverage if enabled
            if let Some(ref mut cov) = self.coverage {
                cov.mark_executed(self.pc);
            }

            let instr = &program.instructions[self.pc];
            self.instruction_count += 1;

            match self.execute_instruction(instr) {
                Ok(ControlFlow::Continue) => {
                    self.pc += 1;
                }
                Ok(ControlFlow::Jump(target)) => {
                    // Jump target is a relative instruction index offset
                    // Add to current PC to get new instruction index
                    let new_pc = (self.pc as i32 + target) as usize;
                    self.pc = new_pc;
                }
                Ok(ControlFlow::AbsoluteJump(target)) => {
                    // Jump to absolute instruction index (for ret, indirect jumps)
                    self.pc = target;
                }
                Ok(ControlFlow::Halt) => {
                    return InterpResult::Halted;
                }
                Err(result) => {
                    return result;
                }
            }
        }
    }

    /// Execute a single instruction
    fn execute_instruction(&mut self, instr: &Instruction) -> Result<ControlFlow, InterpResult> {
        match instr.opcode {
            // ALU operations
            Opcode::Alu => {
                let src1 = self.get_reg(instr.rs1);
                let src2 = self.get_reg(instr.rs2);
                let result =
                    self.alu_op(AluOp::from_u8(instr.mode).unwrap_or(AluOp::Add), src1, src2);
                self.set_reg(instr.rd, result);
            }

            Opcode::AluI => {
                let src1 = self.get_reg(instr.rs1);
                let imm = instr.imm.unwrap_or(0) as i64 as u64;
                let result =
                    self.alu_op(AluOp::from_u8(instr.mode).unwrap_or(AluOp::Add), src1, imm);
                self.set_reg(instr.rd, result);
            }

            Opcode::MulDiv => {
                let src1 = self.get_reg(instr.rs1);
                let src2 = self.get_reg(instr.rs2);

                if src2 == 0
                    && matches!(
                        MulDivOp::from_u8(instr.mode),
                        Some(MulDivOp::Div) | Some(MulDivOp::Mod)
                    )
                {
                    return Err(InterpResult::DivByZero);
                }

                let result = match MulDivOp::from_u8(instr.mode) {
                    Some(MulDivOp::Mul) => src1.wrapping_mul(src2),
                    Some(MulDivOp::MulH) => {
                        let full = (src1 as u128) * (src2 as u128);
                        (full >> 64) as u64
                    }
                    Some(MulDivOp::Div) => src1 / src2,
                    Some(MulDivOp::Mod) => src1 % src2,
                    None => 0,
                };
                self.set_reg(instr.rd, result);
            }

            // Memory operations
            Opcode::Load => {
                let base = self.get_reg(instr.rs1);
                let offset = instr.imm.unwrap_or(0) as i64;
                let addr = (base as i64 + offset) as u64;
                let width = MemWidth::from_u8(instr.mode).unwrap_or(MemWidth::Double);
                let value = self.read_mem(addr, width)?;
                self.set_reg(instr.rd, value);
            }

            Opcode::Store => {
                let base = self.get_reg(instr.rs1);
                let offset = instr.imm.unwrap_or(0) as i64;
                let addr = (base as i64 + offset) as u64;
                let value = self.get_reg(instr.rd);
                let width = MemWidth::from_u8(instr.mode).unwrap_or(MemWidth::Double);
                self.write_mem(addr, value, width)?;
            }

            Opcode::Atomic => {
                // Atomic operations (simplified - single-threaded)
                let addr = self.get_reg(instr.rs1);
                let value = self.get_reg(instr.rs2);
                let current = self.read_mem(addr, MemWidth::Double)?;

                let (result, store) = match AtomicOp::from_u8(instr.mode) {
                    Some(AtomicOp::Cas) => {
                        let expected = self.get_reg(instr.rd);
                        if current == expected {
                            (current, value)
                        } else {
                            (current, current)
                        }
                    }
                    Some(AtomicOp::Xchg) => (current, value),
                    Some(AtomicOp::Add) => (current, current.wrapping_add(value)),
                    Some(AtomicOp::And) => (current, current & value),
                    Some(AtomicOp::Or) => (current, current | value),
                    Some(AtomicOp::Xor) => (current, current ^ value),
                    Some(AtomicOp::Min) => (current, current.min(value)),
                    Some(AtomicOp::Max) => (current, current.max(value)),
                    None => (current, current),
                };

                self.write_mem(addr, store, MemWidth::Double)?;
                self.set_reg(instr.rd, result);
            }

            // Control flow
            Opcode::Branch => {
                let cond = BranchCond::from_u8(instr.mode).unwrap_or(BranchCond::Always);
                let src1 = self.get_reg(instr.rs1) as i64;
                let src2 = self.get_reg(instr.rs2) as i64;

                let take_branch = match cond {
                    BranchCond::Always => true,
                    BranchCond::Eq => src1 == src2,
                    BranchCond::Ne => src1 != src2,
                    BranchCond::Lt => src1 < src2,
                    BranchCond::Le => src1 <= src2,
                    BranchCond::Gt => src1 > src2,
                    BranchCond::Ge => src1 >= src2,
                    BranchCond::Ltu => (src1 as u64) < (src2 as u64),
                };

                // Track branch outcome for coverage (skip unconditional branches)
                if !matches!(cond, BranchCond::Always) {
                    if let Some(ref mut cov) = self.coverage {
                        cov.mark_branch(self.pc, take_branch);
                    }
                }

                if take_branch {
                    return Ok(ControlFlow::Jump(instr.imm.unwrap_or(0)));
                }
            }

            Opcode::Call => {
                // Push return address onto call stack for nested calls
                let return_addr = self.pc + 1;
                self.call_stack.push(return_addr);
                // Also set Lr for compatibility with code that reads it
                self.set_reg(Register::Lr, return_addr as u64);
                return Ok(ControlFlow::Jump(instr.imm.unwrap_or(0)));
            }

            Opcode::Ret => {
                // Pop return address from call stack
                let return_addr = self.call_stack.pop().unwrap_or_else(|| {
                    // Fallback to Lr if stack is empty (for backward compatibility)
                    self.get_reg(Register::Lr) as usize
                });
                return Ok(ControlFlow::AbsoluteJump(return_addr));
            }

            Opcode::Jump => {
                if instr.mode == 1 {
                    // Indirect - register holds absolute instruction index
                    let target = self.get_reg(instr.rs1) as usize;
                    return Ok(ControlFlow::AbsoluteJump(target));
                } else {
                    // Direct - immediate is relative offset
                    return Ok(ControlFlow::Jump(instr.imm.unwrap_or(0)));
                }
            }

            // Capabilities (simplified)
            Opcode::CapNew | Opcode::CapRestrict | Opcode::CapQuery => {
                // In a full implementation, these would manipulate fat pointers
                // For now, just pass through the value
                let value = self.get_reg(instr.rs1);
                self.set_reg(instr.rd, value);
            }

            // Concurrency (simplified - no actual threading)
            Opcode::Spawn => {
                // In interpreter mode, spawn is a no-op
                self.set_reg(instr.rd, 0);
            }

            Opcode::Join => {
                // No-op in interpreter
                self.set_reg(instr.rd, 0);
            }

            Opcode::Chan => {
                // No-op in interpreter
                self.set_reg(instr.rd, 0);
            }

            Opcode::Fence => {
                // Memory fence - no-op in single-threaded interpreter
                std::sync::atomic::fence(Ordering::SeqCst);
            }

            Opcode::Yield => {
                // Yield - no-op in interpreter
                std::thread::yield_now();
            }

            // Taint tracking
            Opcode::Taint => {
                let reg = instr.rd as u8;
                self.security.taint.taint(reg, TaintLevel::UserInput);
            }

            Opcode::Sanitize => {
                let reg = instr.rd as u8;
                self.security.taint.sanitize(reg);
            }

            // I/O opcodes (sandboxed - using IORuntime)
            Opcode::File => {
                let result = match FileOp::from_u8(instr.mode) {
                    Some(FileOp::Open) => {
                        // open(path_addr, path_len, flags) -> fd
                        // rs1 = path address, rs2 = path length (or flags if using different convention)
                        // For now: rs1 = path address, imm = flags
                        let path_addr = self.get_reg(instr.rs1) as usize;
                        let path_len = self.get_reg(instr.rs2) as usize;
                        let flags = instr.imm.unwrap_or(0) as u32;

                        if path_addr + path_len <= self.memory.len() {
                            let path_bytes = &self.memory[path_addr..path_addr + path_len];
                            if let Ok(path) = std::str::from_utf8(path_bytes) {
                                self.io_runtime.file_open(path, flags).unwrap_or(u64::MAX)
                            } else {
                                u64::MAX
                            }
                        } else {
                            u64::MAX
                        }
                    }
                    Some(FileOp::Read) => {
                        // read(fd, buf_addr, len) -> bytes_read
                        // If imm is 0, use rd as max length register (dynamic)
                        let fd = self.get_reg(instr.rs1);
                        let buf_addr = self.get_reg(instr.rs2) as usize;
                        let imm_len = instr.imm.unwrap_or(0) as usize;
                        let len = if imm_len == 0 {
                            self.get_reg(instr.rd) as usize
                        } else {
                            imm_len
                        };

                        if buf_addr + len <= self.memory.len() {
                            let buf = &mut self.memory[buf_addr..buf_addr + len];
                            match self.io_runtime.file_read(fd, buf) {
                                Ok(n) => n as u64,
                                Err(_) => u64::MAX,
                            }
                        } else {
                            u64::MAX
                        }
                    }
                    Some(FileOp::Write) => {
                        // write(fd, buf_addr, len) -> bytes_written
                        // If imm is 0, use rd as length register (dynamic)
                        let fd = self.get_reg(instr.rs1);
                        let buf_addr = self.get_reg(instr.rs2) as usize;
                        let imm_len = instr.imm.unwrap_or(0) as usize;
                        let len = if imm_len == 0 {
                            self.get_reg(instr.rd) as usize
                        } else {
                            imm_len
                        };

                        if buf_addr + len <= self.memory.len() {
                            let buf = &self.memory[buf_addr..buf_addr + len];
                            match self.io_runtime.file_write(fd, buf) {
                                Ok(n) => n as u64,
                                Err(_) => u64::MAX,
                            }
                        } else {
                            u64::MAX
                        }
                    }
                    Some(FileOp::Close) => {
                        // close(fd)
                        let fd = self.get_reg(instr.rs1);
                        match self.io_runtime.file_close(fd) {
                            Ok(()) => 0,
                            Err(_) => u64::MAX,
                        }
                    }
                    Some(FileOp::Seek) => {
                        // seek(fd, offset, whence) -> new_position
                        let fd = self.get_reg(instr.rs1);
                        let offset = self.get_reg(instr.rs2) as i64;
                        let whence = instr.imm.unwrap_or(0) as u32;
                        self.io_runtime
                            .file_seek(fd, offset, whence)
                            .unwrap_or(u64::MAX)
                    }
                    Some(FileOp::Stat) => {
                        // stat(path_addr, path_len) -> size (mtime in r1)
                        let path_addr = self.get_reg(instr.rs1) as usize;
                        let path_len = self.get_reg(instr.rs2) as usize;

                        if path_addr + path_len <= self.memory.len() {
                            let path_bytes = &self.memory[path_addr..path_addr + path_len];
                            if let Ok(path) = std::str::from_utf8(path_bytes) {
                                match self.io_runtime.file_stat(path) {
                                    Ok((size, mtime)) => {
                                        self.set_reg(Register::R1, mtime);
                                        size
                                    }
                                    Err(_) => u64::MAX,
                                }
                            } else {
                                u64::MAX
                            }
                        } else {
                            u64::MAX
                        }
                    }
                    Some(FileOp::Mkdir) => {
                        // mkdir(path_addr, path_len)
                        let path_addr = self.get_reg(instr.rs1) as usize;
                        let path_len = self.get_reg(instr.rs2) as usize;

                        if path_addr + path_len <= self.memory.len() {
                            let path_bytes = &self.memory[path_addr..path_addr + path_len];
                            if let Ok(path) = std::str::from_utf8(path_bytes) {
                                match self.io_runtime.file_mkdir(path) {
                                    Ok(()) => 0,
                                    Err(_) => u64::MAX,
                                }
                            } else {
                                u64::MAX
                            }
                        } else {
                            u64::MAX
                        }
                    }
                    Some(FileOp::Delete) => {
                        // delete(path_addr, path_len)
                        let path_addr = self.get_reg(instr.rs1) as usize;
                        let path_len = self.get_reg(instr.rs2) as usize;

                        if path_addr + path_len <= self.memory.len() {
                            let path_bytes = &self.memory[path_addr..path_addr + path_len];
                            if let Ok(path) = std::str::from_utf8(path_bytes) {
                                match self.io_runtime.file_delete(path) {
                                    Ok(()) => 0,
                                    Err(_) => u64::MAX,
                                }
                            } else {
                                u64::MAX
                            }
                        } else {
                            u64::MAX
                        }
                    }
                    None => u64::MAX,
                };
                self.set_reg(instr.rd, result);
            }

            Opcode::Net => {
                let result = match NetOp::from_u8(instr.mode) {
                    Some(NetOp::Socket) => {
                        // socket(domain, type) -> fd
                        let domain = self.get_reg(instr.rs1) as u32;
                        let socket_type = self.get_reg(instr.rs2) as u32;
                        self.io_runtime
                            .net_socket(domain, socket_type)
                            .unwrap_or(u64::MAX)
                    }
                    Some(NetOp::Connect) => {
                        // connect(fd, addr_ptr, port) -> 0 or error
                        let fd = self.get_reg(instr.rs1);
                        let addr_ptr = self.get_reg(instr.rs2) as usize;
                        let port = instr.imm.unwrap_or(0) as u16;

                        // Read null-terminated address or use length convention
                        // For now, read until null or end of reasonable buffer
                        let addr_end = self.memory[addr_ptr..]
                            .iter()
                            .position(|&b| b == 0)
                            .map(|p| addr_ptr + p)
                            .unwrap_or((addr_ptr + 256).min(self.memory.len()));

                        if addr_ptr < self.memory.len() {
                            let addr_bytes = &self.memory[addr_ptr..addr_end];
                            if let Ok(addr) = std::str::from_utf8(addr_bytes) {
                                match self.io_runtime.net_connect(fd, addr, port) {
                                    Ok(()) => 0,
                                    Err(_) => u64::MAX,
                                }
                            } else {
                                u64::MAX
                            }
                        } else {
                            u64::MAX
                        }
                    }
                    Some(NetOp::Bind) => {
                        // bind(fd, addr_ptr, port) -> 0 or error
                        let fd = self.get_reg(instr.rs1);
                        let addr_ptr = self.get_reg(instr.rs2) as usize;
                        let port = instr.imm.unwrap_or(0) as u16;

                        let addr_end = self.memory[addr_ptr..]
                            .iter()
                            .position(|&b| b == 0)
                            .map(|p| addr_ptr + p)
                            .unwrap_or((addr_ptr + 256).min(self.memory.len()));

                        if addr_ptr < self.memory.len() {
                            let addr_bytes = &self.memory[addr_ptr..addr_end];
                            if let Ok(addr) = std::str::from_utf8(addr_bytes) {
                                match self.io_runtime.net_bind(fd, addr, port) {
                                    Ok(()) => 0,
                                    Err(_) => u64::MAX,
                                }
                            } else {
                                u64::MAX
                            }
                        } else {
                            u64::MAX
                        }
                    }
                    Some(NetOp::Listen) => {
                        // listen(fd, backlog) -> 0 or error
                        let fd = self.get_reg(instr.rs1);
                        let backlog = self.get_reg(instr.rs2) as u32;
                        match self.io_runtime.net_listen(fd, backlog) {
                            Ok(()) => 0,
                            Err(_) => u64::MAX,
                        }
                    }
                    Some(NetOp::Accept) => {
                        // accept(fd) -> client_fd or error
                        let fd = self.get_reg(instr.rs1);
                        match self.io_runtime.net_accept(fd) {
                            Ok(client_fd) => client_fd,
                            Err(_) => {
                                // In mock mode, accept failure means "test complete"
                                // Halt gracefully instead of returning error code
                                if self.io_runtime.is_network_mock_mode() {
                                    return Err(InterpResult::Halted);
                                }
                                u64::MAX
                            }
                        }
                    }
                    Some(NetOp::Send) => {
                        // send(fd, buf_addr, len) -> bytes_sent
                        // If imm is 0, use rd as length register (dynamic length)
                        let fd = self.get_reg(instr.rs1);
                        let buf_addr = self.get_reg(instr.rs2) as usize;
                        let imm_len = instr.imm.unwrap_or(0) as usize;
                        let len = if imm_len == 0 {
                            self.get_reg(instr.rd) as usize // Dynamic length from rd
                        } else {
                            imm_len // Static length from immediate
                        };

                        if buf_addr + len <= self.memory.len() {
                            let buf = &self.memory[buf_addr..buf_addr + len];
                            match self.io_runtime.net_send(fd, buf) {
                                Ok(n) => n as u64,
                                Err(_) => u64::MAX,
                            }
                        } else {
                            u64::MAX
                        }
                    }
                    Some(NetOp::Recv) => {
                        // recv(fd, buf_addr, max_len) -> bytes_received
                        // If imm is 0, use rd as max length register (dynamic length)
                        let fd = self.get_reg(instr.rs1);
                        let buf_addr = self.get_reg(instr.rs2) as usize;
                        let imm_len = instr.imm.unwrap_or(0) as usize;
                        let len = if imm_len == 0 {
                            self.get_reg(instr.rd) as usize // Dynamic length from rd
                        } else {
                            imm_len // Static length from immediate
                        };

                        if buf_addr + len <= self.memory.len() {
                            let buf = &mut self.memory[buf_addr..buf_addr + len];
                            match self.io_runtime.net_recv(fd, buf) {
                                Ok(n) => n as u64,
                                Err(_) => u64::MAX,
                            }
                        } else {
                            u64::MAX
                        }
                    }
                    Some(NetOp::Close) => {
                        // close(fd) -> 0 or error
                        let fd = self.get_reg(instr.rs1);
                        match self.io_runtime.net_close(fd) {
                            Ok(()) => 0,
                            Err(_) => u64::MAX,
                        }
                    }
                    None => u64::MAX,
                };
                self.set_reg(instr.rd, result);
            }

            Opcode::NetSetopt => {
                // setopt(fd, option, value) -> 0 or error
                let fd = self.get_reg(instr.rs1);
                let value = self.get_reg(instr.rs2);
                let result = match NetOption::from_u8(instr.mode) {
                    Some(option) => match self.io_runtime.net_setopt(fd, option, value) {
                        Ok(()) => 0,
                        Err(_) => u64::MAX,
                    },
                    None => u64::MAX,
                };
                self.set_reg(instr.rd, result);
            }

            Opcode::Io => {
                // Console I/O - handle print and basic operations
                match IoOp::from_u8(instr.mode) {
                    Some(IoOp::Print) => {
                        // Print: rs1 = buffer address, rs2 = length
                        let addr = self.get_reg(instr.rs1) as usize;
                        let len = self.get_reg(instr.rs2) as usize;
                        if addr + len <= self.memory.len() {
                            let bytes = &self.memory[addr..addr + len];
                            if let Ok(s) = std::str::from_utf8(bytes) {
                                print!("{}", s);
                            }
                            self.set_reg(instr.rd, len as u64);
                        } else {
                            self.set_reg(instr.rd, u64::MAX);
                        }
                    }
                    Some(IoOp::ReadLine) => {
                        // ReadLine: rd = buffer address from rs1, rs2 = max length
                        let addr = self.get_reg(instr.rs1) as usize;
                        let max_len = self.get_reg(instr.rs2) as usize;
                        if addr + max_len <= self.memory.len() {
                            let mut input = String::new();
                            if std::io::stdin().read_line(&mut input).is_ok() {
                                let bytes = input.as_bytes();
                                let copy_len = bytes.len().min(max_len);
                                self.memory[addr..addr + copy_len]
                                    .copy_from_slice(&bytes[..copy_len]);
                                self.set_reg(instr.rd, copy_len as u64);
                            } else {
                                self.set_reg(instr.rd, u64::MAX);
                            }
                        } else {
                            self.set_reg(instr.rd, u64::MAX);
                        }
                    }
                    Some(IoOp::GetArgs) => {
                        // Return argc in r0, pointer would need allocation
                        self.set_reg(instr.rd, 0);
                    }
                    Some(IoOp::GetEnv) => {
                        // Return null for environment variable lookup
                        self.set_reg(instr.rd, 0);
                    }
                    None => {
                        self.set_reg(instr.rd, u64::MAX);
                    }
                }
            }

            Opcode::Time => {
                match TimeOp::from_u8(instr.mode) {
                    Some(TimeOp::Now) => {
                        // Return Unix timestamp
                        let now = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        self.set_reg(instr.rd, now);
                    }
                    Some(TimeOp::Sleep) => {
                        // Sleep for rs1 milliseconds
                        let ms = self.get_reg(instr.rs1);
                        std::thread::sleep(std::time::Duration::from_millis(ms));
                        self.set_reg(instr.rd, 0);
                    }
                    Some(TimeOp::Monotonic) => {
                        // Return monotonic nanoseconds
                        let ns = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_nanos() as u64;
                        self.set_reg(instr.rd, ns);
                    }
                    Some(TimeOp::Reserved) | None => {
                        self.set_reg(instr.rd, 0);
                    }
                }
            }

            // Math extensions
            Opcode::Fpu => {
                let src1 = f64::from_bits(self.get_reg(instr.rs1));
                let src2 = f64::from_bits(self.get_reg(instr.rs2));
                match FpuOp::from_u8(instr.mode) {
                    // Arithmetic operations return f64 bit pattern
                    Some(FpuOp::Fadd) => self.set_reg(instr.rd, (src1 + src2).to_bits()),
                    Some(FpuOp::Fsub) => self.set_reg(instr.rd, (src1 - src2).to_bits()),
                    Some(FpuOp::Fmul) => self.set_reg(instr.rd, (src1 * src2).to_bits()),
                    Some(FpuOp::Fdiv) => self.set_reg(instr.rd, (src1 / src2).to_bits()),
                    Some(FpuOp::Fsqrt) => self.set_reg(instr.rd, src1.sqrt().to_bits()),
                    Some(FpuOp::Fabs) => self.set_reg(instr.rd, src1.abs().to_bits()),
                    Some(FpuOp::Ffloor) => self.set_reg(instr.rd, src1.floor().to_bits()),
                    Some(FpuOp::Fceil) => self.set_reg(instr.rd, src1.ceil().to_bits()),
                    // Comparison operations return integer 1 or 0
                    Some(FpuOp::Fcmpeq) => self.set_reg(instr.rd, if src1 == src2 { 1 } else { 0 }),
                    Some(FpuOp::Fcmpne) => self.set_reg(instr.rd, if src1 != src2 { 1 } else { 0 }),
                    Some(FpuOp::Fcmplt) => self.set_reg(instr.rd, if src1 < src2 { 1 } else { 0 }),
                    Some(FpuOp::Fcmple) => self.set_reg(instr.rd, if src1 <= src2 { 1 } else { 0 }),
                    Some(FpuOp::Fcmpgt) => self.set_reg(instr.rd, if src1 > src2 { 1 } else { 0 }),
                    Some(FpuOp::Fcmpge) => self.set_reg(instr.rd, if src1 >= src2 { 1 } else { 0 }),
                    None => self.set_reg(instr.rd, 0),
                }
            }

            Opcode::Rand => {
                match RandOp::from_u8(instr.mode) {
                    Some(RandOp::RandBytes) => {
                        // Fill buffer at rs1 with rs2 random bytes
                        let addr = self.get_reg(instr.rs1) as usize;
                        let len = self.get_reg(instr.rs2) as usize;
                        if addr + len <= self.memory.len() {
                            // Use simple PRNG for interpreter (not cryptographic)
                            let seed = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_nanos() as u64;
                            let mut state = seed;
                            for i in 0..len {
                                state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                                self.memory[addr + i] = (state >> 33) as u8;
                            }
                            self.set_reg(instr.rd, len as u64);
                        } else {
                            self.set_reg(instr.rd, u64::MAX);
                        }
                    }
                    Some(RandOp::RandU64) => {
                        // Generate random u64
                        let seed = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_nanos() as u64;
                        let rand = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                        self.set_reg(instr.rd, rand);
                    }
                    None => {
                        self.set_reg(instr.rd, 0);
                    }
                }
            }

            Opcode::Bits => {
                let src = self.get_reg(instr.rs1);
                let result = match BitsOp::from_u8(instr.mode) {
                    Some(BitsOp::Popcount) => src.count_ones() as u64,
                    Some(BitsOp::Clz) => src.leading_zeros() as u64,
                    Some(BitsOp::Ctz) => src.trailing_zeros() as u64,
                    Some(BitsOp::Bswap) => src.swap_bytes(),
                    None => 0,
                };
                self.set_reg(instr.rd, result);
            }

            // System
            Opcode::Mov => {
                // Check if rs1 is a real register (not Zero) for register-to-register move
                if instr.rs1 != Register::Zero {
                    // Register-to-register move: rd = rs1
                    let value = self.get_reg(instr.rs1);
                    self.set_reg(instr.rd, value);
                } else if let Some(imm) = instr.imm {
                    // Load immediate: rd = imm
                    // Sign-extend the 32-bit immediate to 64-bit
                    self.set_reg(instr.rd, imm as i64 as u64);
                }
            }

            Opcode::Trap => {
                let trap_type = TrapType::from_u8(instr.mode).unwrap_or(TrapType::Syscall);
                return Err(InterpResult::Trapped(trap_type));
            }

            Opcode::ExtCall => {
                // Extension ID comes from the immediate field
                let ext_id = instr.imm.unwrap_or(0) as u32;

                // Collect arguments from registers (rs1, rs2, and r3, r4 for additional args)
                let args = [
                    self.get_reg(instr.rs1),
                    self.get_reg(instr.rs2),
                    self.get_reg(Register::R3),
                    self.get_reg(Register::R4),
                ];
                let mut outputs = [0u64; 4];

                // Call the extension registry
                match self.extensions.call(ext_id, &args, &mut outputs) {
                    Ok(result) => {
                        self.set_reg(instr.rd, result as u64);
                    }
                    Err(e) => {
                        // Set error value and continue (or return error for critical failures)
                        self.set_reg(instr.rd, u64::MAX);
                        // Log the error for debugging
                        eprintln!("ExtCall {} failed: {}", ext_id, e);
                    }
                }
            }

            Opcode::Nop => {
                // Do nothing
            }

            Opcode::Halt => {
                return Ok(ControlFlow::Halt);
            }
        }

        Ok(ControlFlow::Continue)
    }

    /// Execute ALU operation
    #[inline]
    fn alu_op(&self, op: AluOp, src1: u64, src2: u64) -> u64 {
        match op {
            AluOp::Add => src1.wrapping_add(src2),
            AluOp::Sub => src1.wrapping_sub(src2),
            AluOp::And => src1 & src2,
            AluOp::Or => src1 | src2,
            AluOp::Xor => src1 ^ src2,
            AluOp::Shl => src1 << (src2 & 63),
            AluOp::Shr => src1 >> (src2 & 63),
            AluOp::Sar => (src1 as i64 >> (src2 & 63)) as u64,
        }
    }

    /// Get instruction count
    pub fn instruction_count(&self) -> u64 {
        self.instruction_count
    }

    /// Get memory slice
    pub fn memory(&self) -> &[u8] {
        &self.memory
    }

    /// Get mutable memory slice
    pub fn memory_mut(&mut self) -> &mut [u8] {
        &mut self.memory
    }

    /// Get coverage tracker (if enabled)
    pub fn coverage(&self) -> Option<&CoverageTracker> {
        self.coverage.as_ref()
    }

    /// Take ownership of coverage tracker
    pub fn take_coverage(&mut self) -> Option<CoverageTracker> {
        self.coverage.take()
    }
}

/// Control flow result from instruction execution
enum ControlFlow {
    /// Continue to next instruction
    Continue,
    /// Jump to relative offset (for branches/calls)
    Jump(i32),
    /// Jump to absolute instruction index (for ret, indirect jumps)
    AbsoluteJump(usize),
    /// Halt execution
    Halt,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new(65536) // 64KB default memory with full I/O permissions
    }
}

/// Get a mutable reference to the I/O runtime
impl Interpreter {
    pub fn io_runtime(&mut self) -> &mut IORuntime {
        &mut self.io_runtime
    }
}

/// Execute a program with minimal overhead (for small programs)
/// Uses full I/O permissions by default
pub fn execute_fast(program: &Program, registers: &mut [u64; 32]) -> InterpResult {
    execute_fast_with_permissions(program, registers, IOPermissions::allow_all())
}

/// Execute a program with specific I/O permissions
pub fn execute_fast_with_permissions(
    program: &Program,
    registers: &mut [u64; 32],
    permissions: IOPermissions,
) -> InterpResult {
    // Use enough memory for data section (DATA_BASE = 0x10000 = 65536)
    let mut interp = Interpreter::with_permissions(DATA_BASE as usize + 0x10000, permissions);
    interp.registers = *registers;
    let result = interp.execute(program);
    *registers = interp.registers;
    result
}

/// Execute a program with a custom extension registry (for mock testing)
pub fn execute_fast_with_extensions(
    program: &Program,
    registers: &mut [u64; 32],
    extensions: ExtensionRegistry,
) -> InterpResult {
    let mut interp =
        Interpreter::with_permissions(DATA_BASE as usize + 0x10000, IOPermissions::allow_all())
            .with_extensions(extensions);
    interp.registers = *registers;
    let result = interp.execute(program);
    *registers = interp.registers;
    result
}

/// Mock specification for extension testing
#[derive(Clone)]
pub struct ExtensionMock {
    /// Extension name (for lookup or documentation)
    pub name: String,
    /// Extension ID (if known, otherwise 0 to lookup by name)
    pub id: u32,
    /// Return value
    pub return_value: i64,
    /// Output values
    pub outputs: Vec<u64>,
}

impl ExtensionMock {
    /// Create a mock by name (ID will be looked up)
    pub fn by_name(name: &str, return_value: i64, outputs: Vec<u64>) -> Self {
        Self {
            name: name.to_string(),
            id: 0,
            return_value,
            outputs,
        }
    }

    /// Create a mock by ID directly
    pub fn by_id(id: u32, return_value: i64, outputs: Vec<u64>) -> Self {
        Self {
            name: String::new(),
            id,
            return_value,
            outputs,
        }
    }
}

/// Execute a program with specified extension mocks
pub fn execute_with_mocks(
    program: &Program,
    registers: &mut [u64; 32],
    mocks: &[ExtensionMock],
) -> InterpResult {
    let mut extensions = ExtensionRegistry::new_with_mocks();

    for mock in mocks {
        if mock.id != 0 {
            // Use ID directly
            extensions.set_mock(mock.id, mock.return_value, mock.outputs.clone());
        } else {
            // Look up by name
            extensions.set_mock_by_name(&mock.name, mock.return_value, mock.outputs.clone());
        }
    }

    execute_fast_with_extensions(program, registers, extensions)
}

/// Execute a program with coverage tracking
/// Returns both the result and the coverage tracker
pub fn execute_with_coverage(
    program: &Program,
    registers: &mut [u64; 32],
) -> (InterpResult, CoverageTracker) {
    let mut interp =
        Interpreter::with_permissions(DATA_BASE as usize + 0x10000, IOPermissions::allow_all())
            .with_coverage(program.instructions.len());
    interp.registers = *registers;
    let result = interp.execute(program);
    *registers = interp.registers;
    let coverage = interp
        .take_coverage()
        .unwrap_or_else(CoverageTracker::disabled);
    (result, coverage)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_program() {
        let mut program = Program::new();
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            0,
            42,
        ));
        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        let mut interp = Interpreter::new(1024);
        let result = interp.execute(&program);

        assert_eq!(result, InterpResult::Halted);
        assert_eq!(interp.registers[0], 42);
    }

    #[test]
    fn test_arithmetic() {
        let mut program = Program::new();

        // r0 = 10
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            0,
            10,
        ));
        // r1 = 5
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R1,
            Register::Zero,
            0,
            5,
        ));
        // r2 = r0 + r1
        program.instructions.push(Instruction::new(
            Opcode::Alu,
            Register::R2,
            Register::R0,
            Register::R1,
            AluOp::Add as u8,
        ));
        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        let mut interp = Interpreter::new(1024);
        interp.execute(&program);

        assert_eq!(interp.registers[2], 15);
    }

    #[test]
    fn test_fibonacci() {
        // Compute fib(10) = 55
        let mut program = Program::new();

        // r0 = n (10)
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            0,
            10,
        ));
        // r1 = 0 (fib_prev)
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R1,
            Register::Zero,
            0,
            0,
        ));
        // r2 = 1 (fib_curr)
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R2,
            Register::Zero,
            0,
            1,
        ));
        // r3 = 1 (constant 1)
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R3,
            Register::Zero,
            0,
            1,
        ));

        // loop:
        // r4 = r1 + r2 (next)
        program.instructions.push(Instruction::new(
            Opcode::Alu,
            Register::R4,
            Register::R1,
            Register::R2,
            AluOp::Add as u8,
        ));
        // r1 = r2 (prev = curr)
        program.instructions.push(Instruction::new(
            Opcode::Mov,
            Register::R1,
            Register::R2,
            Register::Zero,
            0,
        ));
        // r2 = r4 (curr = next)
        program.instructions.push(Instruction::new(
            Opcode::Mov,
            Register::R2,
            Register::R4,
            Register::Zero,
            0,
        ));
        // r0 = r0 - 1
        program.instructions.push(Instruction::new(
            Opcode::Alu,
            Register::R0,
            Register::R0,
            Register::R3,
            AluOp::Sub as u8,
        ));
        // if r0 != 0, jump to loop
        // Branch is at instruction 8, loop starts at instruction 4, so relative offset is -4
        program.instructions.push(Instruction::with_imm(
            Opcode::Branch,
            Register::Zero,
            Register::R0,
            BranchCond::Ne as u8,
            -4, // Jump back 4 instructions (relative instruction index)
        ));

        // Return r1 (the fibonacci number)
        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        let mut interp = Interpreter::new(1024).with_max_instructions(1000);
        let result = interp.execute(&program);

        assert_eq!(result, InterpResult::Halted);
        assert_eq!(interp.registers[1], 55); // fib(10) = 55
    }

    #[test]
    fn test_memory_operations() {
        let mut program = Program::new();

        // Store 0xDEADBEEF at address 0
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            0,
            0xDEADBEEFu32 as i32,
        ));
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R1,
            Register::Zero,
            0,
            0,
        ));
        program.instructions.push(Instruction::with_imm(
            Opcode::Store,
            Register::R0,
            Register::R1,
            MemWidth::Word as u8,
            0,
        ));

        // Load it back
        program.instructions.push(Instruction::with_imm(
            Opcode::Load,
            Register::R2,
            Register::R1,
            MemWidth::Word as u8,
            0,
        ));

        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        let mut interp = Interpreter::new(1024);
        interp.execute(&program);

        assert_eq!(interp.registers[2] as u32, 0xDEADBEEF);
    }
}
