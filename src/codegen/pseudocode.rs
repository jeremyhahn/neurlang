//! Pseudocode Generator for Neurlang IR (Human Lens)
//!
//! Translates Neurlang IR to human-readable pseudocode for debugging
//! and documentation purposes. This is the "human lens transpiler"
//! mentioned in the roadmap.
//!
//! # Output Style
//!
//! The output reads like English descriptions of what the code does:
//!
//! ```text
//! Program: 5 instructions
//!
//! [0] Set r0 to 42
//! [1] Add r1 and r2, store result in r0
//! [2] If r0 equals r1, jump to instruction 5
//! [3] Load 64-bit value from memory at address r2 into r0
//! [4] Stop execution
//! ```

use super::common::{
    analyze_branch_targets, register_name, CodeGenContext, CodeGenOptions, IndentWriter,
};
use super::{CodeGenError, CodeGenResult, CodeGenerator};
use crate::ir::{
    AluOp, AtomicOp, BitsOp, BranchCond, ChanOp, FenceMode, FileOp, FpuOp, Instruction, IoOp,
    MemWidth, MulDivOp, NetOp, Opcode, Program, RandOp, Register, TimeOp, TrapType,
};

/// Pseudocode generator for human-readable output
pub struct PseudocodeGenerator {
    writer: IndentWriter,
    context: CodeGenContext,
    options: CodeGenOptions,
    /// Whether to show instruction indices
    show_indices: bool,
    /// Whether to use verbose descriptions
    verbose: bool,
}

impl PseudocodeGenerator {
    /// Create a new pseudocode generator
    pub fn new() -> Self {
        Self {
            writer: IndentWriter::new(),
            context: CodeGenContext::new(0),
            options: CodeGenOptions::default(),
            show_indices: true,
            verbose: true,
        }
    }

    /// Create with custom options
    pub fn with_options(options: CodeGenOptions) -> Self {
        Self {
            writer: IndentWriter::with_options(options.clone()),
            context: CodeGenContext::new(0),
            options,
            show_indices: true,
            verbose: true,
        }
    }

    /// Set whether to show instruction indices
    pub fn show_indices(mut self, show: bool) -> Self {
        self.show_indices = show;
        self
    }

    /// Set verbosity level
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Format a register as a readable name
    fn fmt_reg(&self, r: Register) -> String {
        if r == Register::Zero {
            "zero".to_string()
        } else {
            register_name(r).to_string()
        }
    }

    /// Format an ALU operation
    fn fmt_alu_op(&self, op: AluOp) -> &'static str {
        match op {
            AluOp::Add => "add",
            AluOp::Sub => "subtract",
            AluOp::And => "bitwise AND",
            AluOp::Or => "bitwise OR",
            AluOp::Xor => "bitwise XOR",
            AluOp::Shl => "shift left",
            AluOp::Shr => "logical shift right",
            AluOp::Sar => "arithmetic shift right",
        }
    }

    /// Format a memory width
    fn fmt_width(&self, width: MemWidth) -> &'static str {
        match width {
            MemWidth::Byte => "8-bit",
            MemWidth::Half => "16-bit",
            MemWidth::Word => "32-bit",
            MemWidth::Double => "64-bit",
        }
    }

    /// Format a branch condition
    fn fmt_branch_cond(&self, cond: BranchCond) -> &'static str {
        match cond {
            BranchCond::Always => "unconditionally",
            BranchCond::Eq => "if equal",
            BranchCond::Ne => "if not equal",
            BranchCond::Lt => "if less than (signed)",
            BranchCond::Le => "if less or equal (signed)",
            BranchCond::Gt => "if greater than (signed)",
            BranchCond::Ge => "if greater or equal (signed)",
            BranchCond::Ltu => "if less than (unsigned)",
        }
    }

    /// Write a pseudocode line with optional index
    fn write_line(&mut self, index: usize, description: &str) {
        if self.show_indices {
            self.writer
                .writeln(&format!("[{:3}] {}", index, description));
        } else {
            self.writer.writeln(description);
        }
    }
}

impl Default for PseudocodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGenerator for PseudocodeGenerator {
    fn target_name(&self) -> &'static str {
        "Pseudocode"
    }

    fn file_extension(&self) -> &'static str {
        "txt"
    }

    fn generate(&mut self, program: &Program) -> CodeGenResult<String> {
        self.context = CodeGenContext::new(program.instructions.len());
        self.context.branch_targets = analyze_branch_targets(program);
        self.writer = IndentWriter::with_options(self.options.clone());

        self.emit_prologue()?;

        // Emit all instructions
        for (index, instr) in program.instructions.iter().enumerate() {
            self.context.current_index = index;
            self.emit_instruction(instr, index)?;
        }

        self.emit_epilogue()?;

        Ok(self.writer.output().to_string())
    }

    fn emit_prologue(&mut self) -> CodeGenResult<()> {
        self.writer
            .writeln("╔══════════════════════════════════════════════════════════════╗");
        self.writer
            .writeln("║           NEURLANG PROGRAM - HUMAN READABLE VIEW             ║");
        self.writer
            .writeln("╚══════════════════════════════════════════════════════════════╝");
        self.writer.newline();
        self.writer.writeln(&format!(
            "Program contains {} instruction(s)",
            self.context.instruction_count
        ));
        if !self.context.branch_targets.is_empty() {
            self.writer.writeln(&format!(
                "Branch targets: {:?}",
                self.context.branch_targets
            ));
        }
        self.writer.newline();
        self.writer
            .writeln("────────────────────────────────────────────────────────────────");
        self.writer.newline();

        Ok(())
    }

    fn emit_epilogue(&mut self) -> CodeGenResult<()> {
        self.writer.newline();
        self.writer
            .writeln("────────────────────────────────────────────────────────────────");
        self.writer.writeln("End of program");

        Ok(())
    }

    fn emit_instruction(&mut self, instr: &Instruction, index: usize) -> CodeGenResult<()> {
        // Mark branch targets
        if self.context.is_branch_target(index) {
            self.writer
                .writeln(&format!("  ▶ TARGET_{}: ──────────", index));
        }

        match instr.opcode {
            Opcode::Alu => {
                let op = AluOp::from_u8(instr.mode).unwrap_or(AluOp::Add);
                self.emit_alu(op, instr.rd, instr.rs1, instr.rs2)?;
            }
            Opcode::AluI => {
                let op = AluOp::from_u8(instr.mode).unwrap_or(AluOp::Add);
                let imm = instr.imm.ok_or(CodeGenError::MissingImmediate(index))?;
                self.emit_alu_imm(op, instr.rd, instr.rs1, imm)?;
            }
            Opcode::MulDiv => {
                let op = MulDivOp::from_u8(instr.mode).unwrap_or(MulDivOp::Mul);
                self.emit_muldiv(op, instr.rd, instr.rs1, instr.rs2)?;
            }
            Opcode::Load => {
                let width = MemWidth::from_u8(instr.mode).unwrap_or(MemWidth::Double);
                let offset = instr.imm.unwrap_or(0);
                self.emit_load(width, instr.rd, instr.rs1, offset)?;
            }
            Opcode::Store => {
                let width = MemWidth::from_u8(instr.mode).unwrap_or(MemWidth::Double);
                let offset = instr.imm.unwrap_or(0);
                self.emit_store(width, instr.rd, instr.rs1, offset)?;
            }
            Opcode::Atomic => {
                let op = AtomicOp::from_u8(instr.mode).unwrap_or(AtomicOp::Cas);
                self.emit_atomic(op, instr.rd, instr.rs1, instr.rs2)?;
            }
            Opcode::Branch => {
                let cond = BranchCond::from_u8(instr.mode).unwrap_or(BranchCond::Always);
                let target = instr.imm.unwrap_or(0);
                self.emit_branch(cond, instr.rs1, instr.rs2, target)?;
            }
            Opcode::Call => {
                let target = instr.imm.unwrap_or(0);
                self.emit_call(target)?;
            }
            Opcode::Ret => self.emit_ret()?,
            Opcode::Jump => {
                let target = instr.imm.unwrap_or(0);
                self.emit_jump(target, instr.mode == 1)?;
            }
            Opcode::CapNew => self.emit_cap_new(instr.rd, instr.rs1, instr.rs2)?,
            Opcode::CapRestrict => self.emit_cap_restrict(instr.rd, instr.rs1, instr.rs2)?,
            Opcode::CapQuery => {
                let query = instr.imm.unwrap_or(0);
                self.emit_cap_query(instr.rd, instr.rs1, query)?;
            }
            Opcode::Spawn => {
                let target = instr.imm.unwrap_or(0);
                self.emit_spawn(instr.rd, target, instr.rs1)?;
            }
            Opcode::Join => self.emit_join(instr.rs1)?,
            Opcode::Chan => {
                let op = ChanOp::from_u8(instr.mode).unwrap_or(ChanOp::Create);
                self.emit_chan(op, instr.rd, instr.rs1)?;
            }
            Opcode::Fence => {
                let mode = FenceMode::from_u8(instr.mode).unwrap_or(FenceMode::SeqCst);
                self.emit_fence(mode)?;
            }
            Opcode::Yield => self.emit_yield()?,
            Opcode::Taint => self.emit_taint(instr.rd, instr.rs1)?,
            Opcode::Sanitize => self.emit_sanitize(instr.rd, instr.rs1)?,
            Opcode::File => {
                let op = FileOp::from_u8(instr.mode).unwrap_or(FileOp::Open);
                self.emit_file(op, instr.rd, instr.rs1, instr.rs2, instr.imm)?;
            }
            Opcode::Net => {
                let op = NetOp::from_u8(instr.mode).unwrap_or(NetOp::Socket);
                self.emit_net(op, instr.rd, instr.rs1, instr.rs2, instr.imm)?;
            }
            Opcode::NetSetopt => {
                let op = crate::ir::NetOption::from_u8(instr.mode);
                self.write_line(
                    index,
                    &format!(
                        "Set socket option {:?} on {} to {}",
                        op,
                        self.fmt_reg(instr.rs1),
                        instr.imm.unwrap_or(0)
                    ),
                );
            }
            Opcode::Io => {
                let op = IoOp::from_u8(instr.mode).unwrap_or(IoOp::Print);
                self.emit_io(op, instr.rd, instr.rs1, instr.rs2)?;
            }
            Opcode::Time => {
                let op = TimeOp::from_u8(instr.mode).unwrap_or(TimeOp::Now);
                self.emit_time(op, instr.rd, instr.imm)?;
            }
            Opcode::Fpu => {
                let op = FpuOp::from_u8(instr.mode).unwrap_or(FpuOp::Fadd);
                self.emit_fpu(op, instr.rd, instr.rs1, instr.rs2)?;
            }
            Opcode::Rand => {
                let op = RandOp::from_u8(instr.mode).unwrap_or(RandOp::RandU64);
                self.emit_rand(op, instr.rd, instr.rs1)?;
            }
            Opcode::Bits => {
                let op = BitsOp::from_u8(instr.mode).unwrap_or(BitsOp::Popcount);
                self.emit_bits(op, instr.rd, instr.rs1)?;
            }
            Opcode::Mov => self.emit_mov(instr.rd, instr.rs1, instr.imm)?,
            Opcode::Trap => {
                let trap = TrapType::from_u8(instr.mode).unwrap_or(TrapType::User);
                self.emit_trap(trap, instr.imm)?;
            }
            Opcode::Nop => self.emit_nop()?,
            Opcode::Halt => self.emit_halt()?,
            Opcode::ExtCall => {
                let ext_id = instr.imm.unwrap_or(0);
                self.emit_ext_call(instr.rd, ext_id, instr.rs1, instr.rs2)?;
            }
        }

        Ok(())
    }

    fn emit_alu(
        &mut self,
        op: AluOp,
        rd: Register,
        rs1: Register,
        rs2: Register,
    ) -> CodeGenResult<()> {
        let desc = format!(
            "{} {} and {}, store result in {}",
            self.fmt_alu_op(op)
                .chars()
                .next()
                .unwrap()
                .to_uppercase()
                .collect::<String>()
                + &self.fmt_alu_op(op)[1..],
            self.fmt_reg(rs1),
            self.fmt_reg(rs2),
            self.fmt_reg(rd)
        );
        self.write_line(self.context.current_index, &desc);
        Ok(())
    }

    fn emit_alu_imm(
        &mut self,
        op: AluOp,
        rd: Register,
        rs1: Register,
        imm: i32,
    ) -> CodeGenResult<()> {
        let desc = format!(
            "{} {} and {}, store result in {}",
            self.fmt_alu_op(op)
                .chars()
                .next()
                .unwrap()
                .to_uppercase()
                .collect::<String>()
                + &self.fmt_alu_op(op)[1..],
            self.fmt_reg(rs1),
            imm,
            self.fmt_reg(rd)
        );
        self.write_line(self.context.current_index, &desc);
        Ok(())
    }

    fn emit_muldiv(
        &mut self,
        op: MulDivOp,
        rd: Register,
        rs1: Register,
        rs2: Register,
    ) -> CodeGenResult<()> {
        let op_name = match op {
            MulDivOp::Mul => "Multiply",
            MulDivOp::MulH => "Multiply (high bits)",
            MulDivOp::Div => "Divide",
            MulDivOp::Mod => "Modulo",
        };
        let desc = format!(
            "{} {} by {}, store result in {}",
            op_name,
            self.fmt_reg(rs1),
            self.fmt_reg(rs2),
            self.fmt_reg(rd)
        );
        self.write_line(self.context.current_index, &desc);
        Ok(())
    }

    fn emit_load(
        &mut self,
        width: MemWidth,
        rd: Register,
        base: Register,
        offset: i32,
    ) -> CodeGenResult<()> {
        let desc = if offset != 0 {
            format!(
                "Load {} value from memory[{} + {}] into {}",
                self.fmt_width(width),
                self.fmt_reg(base),
                offset,
                self.fmt_reg(rd)
            )
        } else {
            format!(
                "Load {} value from memory[{}] into {}",
                self.fmt_width(width),
                self.fmt_reg(base),
                self.fmt_reg(rd)
            )
        };
        self.write_line(self.context.current_index, &desc);
        Ok(())
    }

    fn emit_store(
        &mut self,
        width: MemWidth,
        src: Register,
        base: Register,
        offset: i32,
    ) -> CodeGenResult<()> {
        let desc = if offset != 0 {
            format!(
                "Store {} value from {} to memory[{} + {}]",
                self.fmt_width(width),
                self.fmt_reg(src),
                self.fmt_reg(base),
                offset
            )
        } else {
            format!(
                "Store {} value from {} to memory[{}]",
                self.fmt_width(width),
                self.fmt_reg(src),
                self.fmt_reg(base)
            )
        };
        self.write_line(self.context.current_index, &desc);
        Ok(())
    }

    fn emit_atomic(
        &mut self,
        op: AtomicOp,
        rd: Register,
        rs1: Register,
        rs2: Register,
    ) -> CodeGenResult<()> {
        let op_name = match op {
            AtomicOp::Cas => "Compare-and-swap",
            AtomicOp::Xchg => "Exchange",
            AtomicOp::Add => "Atomic add",
            AtomicOp::And => "Atomic AND",
            AtomicOp::Or => "Atomic OR",
            AtomicOp::Xor => "Atomic XOR",
            AtomicOp::Min => "Atomic min",
            AtomicOp::Max => "Atomic max",
        };
        let desc = format!(
            "{} at memory[{}] with {}, result in {}",
            op_name,
            self.fmt_reg(rs1),
            self.fmt_reg(rs2),
            self.fmt_reg(rd)
        );
        self.write_line(self.context.current_index, &desc);
        Ok(())
    }

    fn emit_branch(
        &mut self,
        cond: BranchCond,
        rs1: Register,
        rs2: Register,
        target: i32,
    ) -> CodeGenResult<()> {
        let target_index = (self.context.current_index as i32 + target) as usize;
        let desc = match cond {
            BranchCond::Always => {
                format!("Jump to instruction {}", target_index)
            }
            _ => {
                format!(
                    "Compare {} and {}, {} jump to instruction {}",
                    self.fmt_reg(rs1),
                    self.fmt_reg(rs2),
                    self.fmt_branch_cond(cond),
                    target_index
                )
            }
        };
        self.write_line(self.context.current_index, &desc);
        Ok(())
    }

    fn emit_call(&mut self, target: i32) -> CodeGenResult<()> {
        let target_index = (self.context.current_index as i32 + target) as usize;
        let desc = format!("Call function at instruction {}", target_index);
        self.write_line(self.context.current_index, &desc);
        Ok(())
    }

    fn emit_ret(&mut self) -> CodeGenResult<()> {
        self.write_line(self.context.current_index, "Return from function");
        Ok(())
    }

    fn emit_jump(&mut self, target: i32, indirect: bool) -> CodeGenResult<()> {
        let desc = if indirect {
            "Jump indirectly via register".to_string()
        } else {
            let target_index = (self.context.current_index as i32 + target) as usize;
            format!("Jump to instruction {}", target_index)
        };
        self.write_line(self.context.current_index, &desc);
        Ok(())
    }

    fn emit_cap_new(&mut self, rd: Register, base: Register, len: Register) -> CodeGenResult<()> {
        let desc = format!(
            "Create new capability in {} with base={}, length={}",
            self.fmt_reg(rd),
            self.fmt_reg(base),
            self.fmt_reg(len)
        );
        self.write_line(self.context.current_index, &desc);
        Ok(())
    }

    fn emit_cap_restrict(
        &mut self,
        rd: Register,
        src: Register,
        len: Register,
    ) -> CodeGenResult<()> {
        let desc = format!(
            "Restrict capability {} to length {}, store in {}",
            self.fmt_reg(src),
            self.fmt_reg(len),
            self.fmt_reg(rd)
        );
        self.write_line(self.context.current_index, &desc);
        Ok(())
    }

    fn emit_cap_query(
        &mut self,
        rd: Register,
        cap: Register,
        query_type: i32,
    ) -> CodeGenResult<()> {
        let query_name = match query_type {
            0 => "base address",
            1 => "length",
            2 => "permissions",
            _ => "property",
        };
        let desc = format!(
            "Query {} of capability {}, store in {}",
            query_name,
            self.fmt_reg(cap),
            self.fmt_reg(rd)
        );
        self.write_line(self.context.current_index, &desc);
        Ok(())
    }

    fn emit_spawn(&mut self, rd: Register, target: i32, arg: Register) -> CodeGenResult<()> {
        let target_index = (self.context.current_index as i32 + target) as usize;
        let desc = format!(
            "Spawn new task at instruction {} with argument {}, handle in {}",
            target_index,
            self.fmt_reg(arg),
            self.fmt_reg(rd)
        );
        self.write_line(self.context.current_index, &desc);
        Ok(())
    }

    fn emit_join(&mut self, task: Register) -> CodeGenResult<()> {
        let desc = format!("Wait for task {} to complete", self.fmt_reg(task));
        self.write_line(self.context.current_index, &desc);
        Ok(())
    }

    fn emit_chan(&mut self, op: ChanOp, rd: Register, rs1: Register) -> CodeGenResult<()> {
        let desc = match op {
            ChanOp::Create => format!("Create new channel in {}", self.fmt_reg(rd)),
            ChanOp::Send => format!("Send {} on channel {}", self.fmt_reg(rs1), self.fmt_reg(rd)),
            ChanOp::Recv => format!(
                "Receive from channel {} into {}",
                self.fmt_reg(rs1),
                self.fmt_reg(rd)
            ),
            ChanOp::Close => format!("Close channel {}", self.fmt_reg(rd)),
        };
        self.write_line(self.context.current_index, &desc);
        Ok(())
    }

    fn emit_fence(&mut self, mode: FenceMode) -> CodeGenResult<()> {
        let mode_name = match mode {
            FenceMode::Acquire => "acquire",
            FenceMode::Release => "release",
            FenceMode::AcqRel => "acquire-release",
            FenceMode::SeqCst => "sequentially consistent",
        };
        let desc = format!("Memory fence ({})", mode_name);
        self.write_line(self.context.current_index, &desc);
        Ok(())
    }

    fn emit_yield(&mut self) -> CodeGenResult<()> {
        self.write_line(self.context.current_index, "Yield to other tasks");
        Ok(())
    }

    fn emit_taint(&mut self, rd: Register, rs1: Register) -> CodeGenResult<()> {
        let desc = format!(
            "Mark {} as tainted (copy from {})",
            self.fmt_reg(rd),
            self.fmt_reg(rs1)
        );
        self.write_line(self.context.current_index, &desc);
        Ok(())
    }

    fn emit_sanitize(&mut self, rd: Register, rs1: Register) -> CodeGenResult<()> {
        let desc = format!(
            "Sanitize {} (copy from {})",
            self.fmt_reg(rd),
            self.fmt_reg(rs1)
        );
        self.write_line(self.context.current_index, &desc);
        Ok(())
    }

    fn emit_file(
        &mut self,
        op: FileOp,
        rd: Register,
        rs1: Register,
        rs2: Register,
        _imm: Option<i32>,
    ) -> CodeGenResult<()> {
        let desc = match op {
            FileOp::Open => format!(
                "Open file at path {}, result fd in {}",
                self.fmt_reg(rs1),
                self.fmt_reg(rd)
            ),
            FileOp::Read => format!(
                "Read from fd {} into buffer {}, bytes read in {}",
                self.fmt_reg(rs1),
                self.fmt_reg(rs2),
                self.fmt_reg(rd)
            ),
            FileOp::Write => format!(
                "Write to fd {} from buffer {}, bytes written in {}",
                self.fmt_reg(rs1),
                self.fmt_reg(rs2),
                self.fmt_reg(rd)
            ),
            FileOp::Close => format!("Close file descriptor {}", self.fmt_reg(rs1)),
            FileOp::Seek => format!("Seek fd {} to offset", self.fmt_reg(rs1)),
            FileOp::Stat => format!("Get file stats for path {}", self.fmt_reg(rs1)),
            FileOp::Mkdir => format!("Create directory at path {}", self.fmt_reg(rs1)),
            FileOp::Delete => format!("Delete file at path {}", self.fmt_reg(rs1)),
        };
        self.write_line(self.context.current_index, &desc);
        Ok(())
    }

    fn emit_net(
        &mut self,
        op: NetOp,
        rd: Register,
        rs1: Register,
        rs2: Register,
        _imm: Option<i32>,
    ) -> CodeGenResult<()> {
        let desc = match op {
            NetOp::Socket => format!("Create socket, fd in {}", self.fmt_reg(rd)),
            NetOp::Connect => format!("Connect socket {} to address", self.fmt_reg(rs1)),
            NetOp::Bind => format!("Bind socket {} to address", self.fmt_reg(rs1)),
            NetOp::Listen => format!("Listen on socket {}", self.fmt_reg(rs1)),
            NetOp::Accept => format!(
                "Accept connection on {}, new fd in {}",
                self.fmt_reg(rs1),
                self.fmt_reg(rd)
            ),
            NetOp::Send => format!(
                "Send data from {} on socket {}",
                self.fmt_reg(rs2),
                self.fmt_reg(rs1)
            ),
            NetOp::Recv => format!(
                "Receive data on socket {} into {}",
                self.fmt_reg(rs1),
                self.fmt_reg(rs2)
            ),
            NetOp::Close => format!("Close socket {}", self.fmt_reg(rs1)),
        };
        self.write_line(self.context.current_index, &desc);
        Ok(())
    }

    fn emit_io(
        &mut self,
        op: IoOp,
        rd: Register,
        rs1: Register,
        rs2: Register,
    ) -> CodeGenResult<()> {
        let desc = match op {
            IoOp::Print => format!(
                "Print {} bytes from buffer {}",
                self.fmt_reg(rs2),
                self.fmt_reg(rs1)
            ),
            IoOp::ReadLine => format!(
                "Read line into buffer {}, length in {}",
                self.fmt_reg(rs1),
                self.fmt_reg(rd)
            ),
            IoOp::GetArgs => "Get command line arguments".to_string(),
            IoOp::GetEnv => "Get environment variable".to_string(),
        };
        self.write_line(self.context.current_index, &desc);
        Ok(())
    }

    fn emit_time(&mut self, op: TimeOp, rd: Register, imm: Option<i32>) -> CodeGenResult<()> {
        let desc = match op {
            TimeOp::Now => format!("Get current Unix timestamp into {}", self.fmt_reg(rd)),
            TimeOp::Sleep => format!("Sleep for {} milliseconds", imm.unwrap_or(0)),
            TimeOp::Monotonic => {
                format!("Get monotonic time (nanoseconds) into {}", self.fmt_reg(rd))
            }
            TimeOp::Reserved => "Reserved time operation".to_string(),
        };
        self.write_line(self.context.current_index, &desc);
        Ok(())
    }

    fn emit_fpu(
        &mut self,
        op: FpuOp,
        rd: Register,
        rs1: Register,
        rs2: Register,
    ) -> CodeGenResult<()> {
        let desc = match op {
            FpuOp::Fadd => format!(
                "Float add {} + {}, result in {}",
                self.fmt_reg(rs1),
                self.fmt_reg(rs2),
                self.fmt_reg(rd)
            ),
            FpuOp::Fsub => format!(
                "Float subtract {} - {}, result in {}",
                self.fmt_reg(rs1),
                self.fmt_reg(rs2),
                self.fmt_reg(rd)
            ),
            FpuOp::Fmul => format!(
                "Float multiply {} * {}, result in {}",
                self.fmt_reg(rs1),
                self.fmt_reg(rs2),
                self.fmt_reg(rd)
            ),
            FpuOp::Fdiv => format!(
                "Float divide {} / {}, result in {}",
                self.fmt_reg(rs1),
                self.fmt_reg(rs2),
                self.fmt_reg(rd)
            ),
            FpuOp::Fsqrt => format!(
                "Square root of {}, result in {}",
                self.fmt_reg(rs1),
                self.fmt_reg(rd)
            ),
            FpuOp::Fabs => format!(
                "Absolute value of {}, result in {}",
                self.fmt_reg(rs1),
                self.fmt_reg(rd)
            ),
            FpuOp::Ffloor => format!(
                "Floor of {}, result in {}",
                self.fmt_reg(rs1),
                self.fmt_reg(rd)
            ),
            FpuOp::Fceil => format!(
                "Ceiling of {}, result in {}",
                self.fmt_reg(rs1),
                self.fmt_reg(rd)
            ),
            // Comparison operations
            FpuOp::Fcmpeq => format!(
                "Float compare {} == {}, result (0 or 1) in {}",
                self.fmt_reg(rs1),
                self.fmt_reg(rs2),
                self.fmt_reg(rd)
            ),
            FpuOp::Fcmpne => format!(
                "Float compare {} != {}, result (0 or 1) in {}",
                self.fmt_reg(rs1),
                self.fmt_reg(rs2),
                self.fmt_reg(rd)
            ),
            FpuOp::Fcmplt => format!(
                "Float compare {} < {}, result (0 or 1) in {}",
                self.fmt_reg(rs1),
                self.fmt_reg(rs2),
                self.fmt_reg(rd)
            ),
            FpuOp::Fcmple => format!(
                "Float compare {} <= {}, result (0 or 1) in {}",
                self.fmt_reg(rs1),
                self.fmt_reg(rs2),
                self.fmt_reg(rd)
            ),
            FpuOp::Fcmpgt => format!(
                "Float compare {} > {}, result (0 or 1) in {}",
                self.fmt_reg(rs1),
                self.fmt_reg(rs2),
                self.fmt_reg(rd)
            ),
            FpuOp::Fcmpge => format!(
                "Float compare {} >= {}, result (0 or 1) in {}",
                self.fmt_reg(rs1),
                self.fmt_reg(rs2),
                self.fmt_reg(rd)
            ),
        };
        self.write_line(self.context.current_index, &desc);
        Ok(())
    }

    fn emit_rand(&mut self, op: RandOp, rd: Register, _rs1: Register) -> CodeGenResult<()> {
        let desc = match op {
            RandOp::RandU64 => format!("Generate random 64-bit integer into {}", self.fmt_reg(rd)),
            RandOp::RandBytes => {
                format!("Generate random bytes into buffer at {}", self.fmt_reg(rd))
            }
        };
        self.write_line(self.context.current_index, &desc);
        Ok(())
    }

    fn emit_bits(&mut self, op: BitsOp, rd: Register, rs1: Register) -> CodeGenResult<()> {
        let desc = match op {
            BitsOp::Popcount => format!(
                "Count set bits in {}, result in {}",
                self.fmt_reg(rs1),
                self.fmt_reg(rd)
            ),
            BitsOp::Clz => format!(
                "Count leading zeros in {}, result in {}",
                self.fmt_reg(rs1),
                self.fmt_reg(rd)
            ),
            BitsOp::Ctz => format!(
                "Count trailing zeros in {}, result in {}",
                self.fmt_reg(rs1),
                self.fmt_reg(rd)
            ),
            BitsOp::Bswap => format!(
                "Byte swap {}, result in {}",
                self.fmt_reg(rs1),
                self.fmt_reg(rd)
            ),
        };
        self.write_line(self.context.current_index, &desc);
        Ok(())
    }

    fn emit_mov(&mut self, rd: Register, rs1: Register, imm: Option<i32>) -> CodeGenResult<()> {
        let desc = if rs1 != Register::Zero {
            format!("Copy {} to {}", self.fmt_reg(rs1), self.fmt_reg(rd))
        } else if let Some(i) = imm {
            format!("Set {} to {}", self.fmt_reg(rd), i)
        } else {
            format!("Set {} to 0", self.fmt_reg(rd))
        };
        self.write_line(self.context.current_index, &desc);
        Ok(())
    }

    fn emit_trap(&mut self, trap_type: TrapType, imm: Option<i32>) -> CodeGenResult<()> {
        let desc = match trap_type {
            TrapType::Syscall => format!("System call {}", imm.unwrap_or(0)),
            TrapType::Breakpoint => "Breakpoint".to_string(),
            TrapType::BoundsViolation => "Bounds violation trap".to_string(),
            TrapType::CapabilityViolation => "Capability violation trap".to_string(),
            TrapType::TaintViolation => "Taint violation trap".to_string(),
            TrapType::DivByZero => "Division by zero trap".to_string(),
            TrapType::InvalidOp => "Invalid operation trap".to_string(),
            TrapType::User => format!("User trap {}", imm.unwrap_or(0)),
        };
        self.write_line(self.context.current_index, &desc);
        Ok(())
    }

    fn emit_nop(&mut self) -> CodeGenResult<()> {
        self.write_line(self.context.current_index, "No operation");
        Ok(())
    }

    fn emit_halt(&mut self) -> CodeGenResult<()> {
        self.write_line(
            self.context.current_index,
            "★ HALT - Stop program execution",
        );
        Ok(())
    }

    fn emit_ext_call(
        &mut self,
        rd: Register,
        ext_id: i32,
        rs1: Register,
        rs2: Register,
    ) -> CodeGenResult<()> {
        let ext_name = match ext_id {
            1 => "SHA-256 hash",
            2 => "HMAC-SHA256",
            3 => "AES-256-GCM encrypt",
            4 => "AES-256-GCM decrypt",
            5 => "Constant-time comparison",
            6 => "Secure random bytes",
            7 => "PBKDF2-SHA256",
            8 => "Ed25519 sign",
            9 => "Ed25519 verify",
            10 => "X25519 key derive",
            _ => "Unknown extension",
        };
        let desc = format!(
            "Call extension {} ({}) with args {}, {}, result in {}",
            ext_id,
            ext_name,
            self.fmt_reg(rs1),
            self.fmt_reg(rs2),
            self.fmt_reg(rd)
        );
        self.write_line(self.context.current_index, &desc);
        Ok(())
    }

    fn emit_label(&mut self, index: usize) -> CodeGenResult<()> {
        self.writer.writeln(&format!("  ▶ TARGET_{}:", index));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::Assembler;

    #[test]
    fn test_pseudocode_generator_simple() {
        let mut asm = Assembler::new();
        let program = asm.assemble("mov r0, 42\nadd r1, r0, r2\nhalt").unwrap();

        let mut gen = PseudocodeGenerator::new();
        let code = gen.generate(&program).unwrap();

        assert!(code.contains("Set r0 to 42"));
        assert!(code.contains("Add r0 and r2"));
        assert!(code.contains("HALT"));
    }

    #[test]
    fn test_pseudocode_generator_branch() {
        let mut asm = Assembler::new();
        let program = asm
            .assemble("mov r0, 0\nloop:\nadd r0, r0, r1\nbne r0, r2, loop\nhalt")
            .unwrap();

        let mut gen = PseudocodeGenerator::new();
        let code = gen.generate(&program).unwrap();

        assert!(code.contains("Compare r0 and r2"));
        assert!(code.contains("if not equal"));
        assert!(code.contains("TARGET_1"));
    }
}
