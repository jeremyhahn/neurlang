//! JIT Executor
//!
//! Executes Neurlang programs using native code handlers with full I/O support.
//!
//! ## Multi-Worker Mode
//!
//! The executor supports spawning multiple worker threads for server workloads.
//! Each worker runs a separate copy of the program with its own memory and I/O state.
//!
//! ### Strategy Selection (automatic)
//!
//! The runtime automatically selects the best strategy:
//!
//! 1. **SO_REUSEPORT** (Linux 3.9+, macOS, FreeBSD)
//!    - Each worker has its own socket bound to the same port
//!    - Kernel load-balances incoming connections
//!    - Best performance, no contention
//!
//! 2. **Shared Listener** (Windows, old Linux)
//!    - Single listener shared across workers via Arc
//!    - Workers compete for accept() calls
//!    - Good performance with some contention
//!
//! 3. **Single-threaded** (default, or when workers=1)
//!    - Simplest, no synchronization overhead
//!    - Best for low-concurrency or CPU-bound workloads

use crate::ir::Program;
use crate::jit::context::JitContext;
use crate::jit::handlers::{execute_instruction, ControlFlow};
use crate::stencil::io::IOPermissions;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

/// Multi-worker strategy
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WorkerStrategy {
    /// Single-threaded execution
    SingleThreaded,
    /// Each worker has its own socket (SO_REUSEPORT)
    ReusePort,
    /// Workers share a single listener
    SharedListener,
}

impl WorkerStrategy {
    /// Detect the best available strategy for this platform
    pub fn detect() -> Self {
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "freebsd"))]
        {
            // SO_REUSEPORT is available on these platforms
            WorkerStrategy::ReusePort
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "freebsd")))]
        {
            // Fall back to shared listener on Windows and other platforms
            WorkerStrategy::SharedListener
        }
    }

    /// Get a human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            WorkerStrategy::SingleThreaded => "single-threaded",
            WorkerStrategy::ReusePort => "SO_REUSEPORT (kernel load-balancing)",
            WorkerStrategy::SharedListener => "shared listener",
        }
    }
}

/// JIT execution result
#[derive(Debug)]
pub enum JitResult {
    /// Successful completion - returns R0 value
    Ok(u64),
    /// Program halted normally
    Halted(u64),
    /// Error occurred
    Error(String),
}

/// JIT Executor - runs programs at native speed
pub struct JitExecutor {
    /// Execution context (registers, memory, I/O)
    pub context: JitContext,
    /// Statistics
    pub stats: JitStats,
}

/// Execution statistics
#[derive(Debug, Default)]
pub struct JitStats {
    pub instructions_executed: u64,
    pub execution_time_us: u64,
}

impl JitExecutor {
    /// Create a new JIT executor with default settings (1MB memory, full I/O)
    pub fn new() -> Self {
        Self::with_memory_size(1024 * 1024)
    }

    /// Create a new JIT executor with specified memory size
    pub fn with_memory_size(memory_size: usize) -> Self {
        Self {
            context: JitContext::new(memory_size),
            stats: JitStats::default(),
        }
    }

    /// Create a new JIT executor with custom permissions
    pub fn with_permissions(memory_size: usize, permissions: IOPermissions) -> Self {
        Self {
            context: JitContext::with_permissions(memory_size, permissions),
            stats: JitStats::default(),
        }
    }

    /// Execute a program
    pub fn execute(&mut self, program: &Program) -> JitResult {
        // Reset context
        self.context.pc = program.entry_point;
        self.context.halted = false;
        self.context.error = None;
        self.context.instruction_count = 0;

        // Load data section
        self.context.load_data_section(program);

        let start = Instant::now();

        // Main execution loop
        loop {
            // Check if we've run past the end of the program
            if self.context.pc >= program.instructions.len() {
                self.stats.instructions_executed = self.context.instruction_count;
                self.stats.execution_time_us = start.elapsed().as_micros() as u64;
                return JitResult::Ok(self.context.registers[0]);
            }

            // Get current instruction
            let instr = &program.instructions[self.context.pc];
            self.context.instruction_count += 1;

            // Execute instruction
            match execute_instruction(&mut self.context, instr) {
                ControlFlow::Continue => {
                    self.context.pc += 1;
                }
                ControlFlow::Jump(offset) => {
                    // Relative jump
                    let new_pc = (self.context.pc as i32 + offset) as usize;
                    self.context.pc = new_pc;
                }
                ControlFlow::AbsoluteJump(target) => {
                    self.context.pc = target;
                }
                ControlFlow::Halt => {
                    self.context.halted = true;
                    self.stats.instructions_executed = self.context.instruction_count;
                    self.stats.execution_time_us = start.elapsed().as_micros() as u64;
                    return JitResult::Halted(self.context.registers[0]);
                }
                ControlFlow::Error => {
                    self.stats.instructions_executed = self.context.instruction_count;
                    self.stats.execution_time_us = start.elapsed().as_micros() as u64;
                    let error = self
                        .context
                        .error
                        .take()
                        .unwrap_or_else(|| "Unknown error".into());
                    return JitResult::Error(error);
                }
            }
        }
    }

    /// Get register value
    pub fn get_register(&self, reg: usize) -> u64 {
        self.context.get_reg(reg)
    }

    /// Set register value
    pub fn set_register(&mut self, reg: usize, value: u64) {
        self.context.set_reg(reg, value);
    }

    /// Get all registers
    pub fn registers(&self) -> &[u64; 32] {
        &self.context.registers
    }

    /// Get memory slice
    pub fn memory(&self) -> &[u8] {
        &self.context.memory
    }

    /// Get mutable memory slice
    pub fn memory_mut(&mut self) -> &mut [u8] {
        &mut self.context.memory
    }

    /// Pre-load the data section (call before setting up test memory)
    pub fn load_data_section(&mut self, program: &Program) {
        self.context.load_data_section(program);
    }

    /// Enable mock mode for testing extensions
    pub fn enable_mock_mode(&mut self) {
        self.context.extensions.set_mock_mode(true);
    }

    /// Set a mock response for an extension ID
    pub fn set_extension_mock(&mut self, ext_id: u32, return_value: i64, outputs: Vec<u64>) {
        self.context
            .extensions
            .set_mock(ext_id, return_value, outputs);
    }

    /// Set a stateful mock with a sequence of return values
    pub fn set_extension_mock_sequence(
        &mut self,
        ext_id: u32,
        return_values: Vec<i64>,
        outputs: Vec<u64>,
    ) {
        self.context
            .extensions
            .set_mock_sequence(ext_id, return_values, outputs);
    }

    /// Clear all mocks and disable mock mode
    pub fn clear_mocks(&mut self) {
        self.context.extensions.clear_all_mocks();
    }

    /// Execute a program with multiple workers
    ///
    /// Automatically detects the best strategy:
    /// - SO_REUSEPORT on Linux/macOS/FreeBSD (kernel load-balancing)
    /// - Shared listener on Windows (workers share one socket)
    /// - Single-threaded when workers <= 1
    ///
    /// Returns the result from the first worker to complete (usually never for servers).
    pub fn execute_multi_worker(
        program: Arc<Program>,
        num_workers: usize,
        memory_size: usize,
        permissions: IOPermissions,
    ) -> JitResult {
        Self::execute_with_strategy(
            program,
            num_workers,
            memory_size,
            permissions,
            None, // Auto-detect strategy
        )
    }

    /// Execute with a specific strategy (or auto-detect if None)
    pub fn execute_with_strategy(
        program: Arc<Program>,
        num_workers: usize,
        memory_size: usize,
        permissions: IOPermissions,
        strategy: Option<WorkerStrategy>,
    ) -> JitResult {
        // Determine actual worker count
        let num_cpus = std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(1);
        let actual_workers = if num_workers == 0 {
            1
        } else {
            num_workers.min(num_cpus * 2)
        };

        // Auto-detect strategy if not specified
        let strategy = strategy.unwrap_or_else(|| {
            if actual_workers <= 1 {
                WorkerStrategy::SingleThreaded
            } else {
                WorkerStrategy::detect()
            }
        });

        eprintln!(
            "Starting {} worker(s) using {} strategy",
            actual_workers,
            strategy.description()
        );

        match strategy {
            WorkerStrategy::SingleThreaded => {
                let mut executor = JitExecutor::with_permissions(memory_size, permissions);
                executor.execute(&program)
            }
            WorkerStrategy::ReusePort => {
                Self::execute_reuseport(program, actual_workers, memory_size, permissions)
            }
            WorkerStrategy::SharedListener => {
                Self::execute_shared_listener(program, actual_workers, memory_size, permissions)
            }
        }
    }

    /// Execute with SO_REUSEPORT - each worker has its own socket
    fn execute_reuseport(
        program: Arc<Program>,
        num_workers: usize,
        memory_size: usize,
        permissions: IOPermissions,
    ) -> JitResult {
        let mut handles = Vec::with_capacity(num_workers);

        for _worker_id in 0..num_workers {
            let prog = Arc::clone(&program);
            let perms = permissions.clone();

            let handle = thread::spawn(move || {
                let mut executor = JitExecutor::with_permissions(memory_size, perms);
                executor.execute(&prog)
            });

            handles.push(handle);
        }

        // Wait for all workers
        for handle in handles {
            match handle.join() {
                Ok(result) => return result,
                Err(e) => {
                    eprintln!("Worker thread panicked: {:?}", e);
                }
            }
        }

        JitResult::Error("All workers exited".to_string())
    }

    /// Execute with shared listener - one worker creates listener, others attach
    ///
    /// This strategy works as follows:
    /// 1. All workers race to call net.bind
    /// 2. First worker succeeds and creates the listener
    /// 3. Other workers' net.bind calls attach to the shared listener
    /// 4. All workers compete on accept() - OS handles synchronization
    ///
    /// This is simpler than channel-based distribution and works on all platforms.
    fn execute_shared_listener(
        program: Arc<Program>,
        num_workers: usize,
        memory_size: usize,
        permissions: IOPermissions,
    ) -> JitResult {
        use std::net::TcpListener;
        use std::sync::atomic::AtomicBool;
        use std::sync::Mutex;

        // Shared state for listener coordination
        let shared_listener: Arc<Mutex<Option<Arc<TcpListener>>>> = Arc::new(Mutex::new(None));
        let listener_ready = Arc::new(AtomicBool::new(false));

        let mut handles = Vec::with_capacity(num_workers);

        for worker_id in 0..num_workers {
            let prog = Arc::clone(&program);
            let perms = permissions.clone();
            let shared = Arc::clone(&shared_listener);
            let ready = Arc::clone(&listener_ready);

            let handle = thread::spawn(move || {
                let mut executor = JitExecutor::with_permissions(memory_size, perms);

                // Set shared listener state in IO runtime
                executor
                    .context
                    .io_runtime
                    .set_shared_listener_mode(shared, ready, worker_id);

                // Execute the program
                executor.execute(&prog)
            });

            handles.push(handle);
        }

        // Wait for all workers
        for handle in handles {
            match handle.join() {
                Ok(result) => return result,
                Err(e) => {
                    eprintln!("Worker thread panicked: {:?}", e);
                }
            }
        }

        JitResult::Error("All workers exited".to_string())
    }
}

/// Execute a program with multiple workers (convenience function)
pub fn execute_multi_worker(program: &Program, num_workers: usize) -> JitResult {
    JitExecutor::execute_multi_worker(
        Arc::new(program.clone()),
        num_workers,
        1024 * 1024,
        IOPermissions::allow_all(),
    )
}

/// Execute with explicit strategy selection
pub fn execute_with_strategy(
    program: &Program,
    num_workers: usize,
    strategy: WorkerStrategy,
) -> JitResult {
    JitExecutor::execute_with_strategy(
        Arc::new(program.clone()),
        num_workers,
        1024 * 1024,
        IOPermissions::allow_all(),
        Some(strategy),
    )
}

impl Default for JitExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{AluOp, Instruction, Opcode, Register};

    /// Execute a program using the JIT executor
    fn execute(program: &Program) -> JitResult {
        let mut executor = JitExecutor::new();
        executor.execute(program)
    }

    #[test]
    fn test_simple_program() {
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

        let result = execute(&program);
        match result {
            JitResult::Halted(val) => assert_eq!(val, 42),
            other => panic!("Expected Halted, got {:?}", other),
        }
    }

    #[test]
    fn test_arithmetic() {
        let mut program = Program::new();
        // r0 = 10
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R0,
            Register::Zero,
            1,
            10,
        ));
        // r1 = 5
        program.instructions.push(Instruction::with_imm(
            Opcode::Mov,
            Register::R1,
            Register::Zero,
            1,
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
        // r0 = r2
        program.instructions.push(Instruction::new(
            Opcode::Mov,
            Register::R0,
            Register::R2,
            Register::Zero,
            0,
        ));
        program.instructions.push(Instruction::new(
            Opcode::Halt,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        ));

        let result = execute(&program);
        match result {
            JitResult::Halted(val) => assert_eq!(val, 15),
            other => panic!("Expected Halted(15), got {:?}", other),
        }
    }
}
