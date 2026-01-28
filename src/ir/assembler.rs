//! Assembler and Disassembler for Neurlang IR
//!
//! Converts between text assembly and binary IR format.
//!
//! # Intrinsic Support
//!
//! The assembler supports intrinsic calls using the `@name args...` syntax.
//! Intrinsics are expanded at assembly time to optimized Neurlang IR sequences.
//!
//! Example:
//! ```text
//! @memcpy r0, r1, 256    ; Expands to optimized copy loop
//! @strlen r0             ; Expands to string length calculation
//! @gcd r0, r1            ; Expands to Euclidean GCD algorithm
//! ```

use crate::ir::format::{
    AluOp, AtomicOp, BitsOp, BranchCond, ChanOp, FenceMode, FileOp, FpuOp, Instruction, IoOp,
    MemWidth, MulDivOp, NetOp, NetOption, Opcode, Program, RandOp, Register, TimeOp, TrapType,
};
use crate::ir::intrinsics::{IntrinsicArg, IntrinsicCall, IntrinsicRegistry};
use crate::ir::rag_resolver::RagResolver;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AsmError {
    #[error("Invalid opcode: {0}")]
    InvalidOpcode(String),
    #[error("Invalid register: {0}")]
    InvalidRegister(String),
    #[error("Invalid immediate value: {0}")]
    InvalidImmediate(String),
    #[error("Missing operand at line {0}")]
    MissingOperand(usize),
    #[error("Undefined label: {0}")]
    UndefinedLabel(String),
    #[error("Duplicate label: {0}")]
    DuplicateLabel(String),
    #[error("Parse error at line {line}: {message}")]
    ParseError { line: usize, message: String },
    #[error("Intrinsic error at line {line}: {message}")]
    IntrinsicError { line: usize, message: String },
    #[error("Extension not found at line {line}: {intent}")]
    ExtensionNotFound { line: usize, intent: String },
}

/// Base address for data section in memory
pub const DATA_BASE: u64 = 0x10000;

/// Label type to distinguish code vs data labels
#[derive(Debug, Clone, Copy, PartialEq)]
enum LabelType {
    Code(usize), // Instruction index
    Data(usize), // Offset in data section
}

/// Assembler for Neurlang
pub struct Assembler {
    labels: HashMap<String, LabelType>,
    pending_labels: Vec<(usize, String, usize)>, // (instruction index, label, line)
    pending_data_refs: Vec<(usize, String, usize)>, // (instruction index, label, line) - for MOV of data labels
    in_data_section: bool,
    /// Intrinsic registry for expanding @intrinsic calls
    intrinsics: IntrinsicRegistry,
    /// RAG resolver for extension intent resolution (@"description" syntax)
    rag_resolver: RagResolver,
}

impl Assembler {
    pub fn new() -> Self {
        Self {
            labels: HashMap::new(),
            pending_labels: Vec::new(),
            pending_data_refs: Vec::new(),
            in_data_section: false,
            intrinsics: IntrinsicRegistry::new(),
            rag_resolver: RagResolver::new(),
        }
    }

    /// Create an assembler with a custom RAG resolver
    pub fn with_rag_resolver(rag_resolver: RagResolver) -> Self {
        Self {
            labels: HashMap::new(),
            pending_labels: Vec::new(),
            pending_data_refs: Vec::new(),
            in_data_section: false,
            intrinsics: IntrinsicRegistry::new(),
            rag_resolver,
        }
    }

    /// Get a reference to the RAG resolver
    pub fn rag_resolver(&self) -> &RagResolver {
        &self.rag_resolver
    }

    /// Get a mutable reference to the RAG resolver (for registering user extensions)
    pub fn rag_resolver_mut(&mut self) -> &mut RagResolver {
        &mut self.rag_resolver
    }

    /// Get a reference to the intrinsic registry
    pub fn intrinsics(&self) -> &IntrinsicRegistry {
        &self.intrinsics
    }

    /// Get code labels (label name -> instruction index)
    /// Returns only labels that point to code (not data labels)
    pub fn code_labels(&self) -> impl Iterator<Item = (&String, usize)> {
        self.labels
            .iter()
            .filter_map(|(name, label_type)| match label_type {
                LabelType::Code(idx) => Some((name, *idx)),
                LabelType::Data(_) => None,
            })
    }

    /// Assemble text to a Program
    pub fn assemble(&mut self, source: &str) -> Result<Program, AsmError> {
        self.labels.clear();
        self.pending_labels.clear();
        self.pending_data_refs.clear();
        self.in_data_section = false;

        let mut program = Program::new();
        let mut current_instr_idx = 0usize; // Track instruction index, not byte offset

        // First pass: collect labels and parse instructions
        for (line_num, line) in source.lines().enumerate() {
            let line = line.trim();

            // Strip inline comments
            let line = line.split(';').next().unwrap_or("");
            let line = line.split('#').next().unwrap_or("").trim();

            // Skip empty lines
            if line.is_empty() {
                continue;
            }

            // Handle directives
            if line.starts_with('.') {
                // Parse directive
                let parts: Vec<&str> = line.split_whitespace().collect();
                let directive = parts[0];
                match directive {
                    ".entry" => {
                        // .entry label - sets entry point
                        if let Some(label_name) = parts.get(1) {
                            // Store entry point label for later resolution
                            program.entry_label = Some(label_name.to_string());
                        }
                        continue;
                    }
                    ".section" => {
                        // .section name - check for data sections
                        if let Some(section_name) = parts.get(1) {
                            self.in_data_section =
                                *section_name == ".data" || *section_name == "data";
                        }
                        continue;
                    }
                    ".text" => {
                        self.in_data_section = false;
                        continue;
                    }
                    ".data" | ".bss" => {
                        self.in_data_section = true;
                        continue;
                    }
                    ".align" | ".p2align" => {
                        if self.in_data_section {
                            // Align data section to specified boundary
                            let align = parts
                                .get(1)
                                .and_then(|s| s.parse::<usize>().ok())
                                .unwrap_or(4);
                            let padding = (align - (program.data_section.len() % align)) % align;
                            program
                                .data_section
                                .extend(std::iter::repeat_n(0u8, padding));
                        }
                        continue;
                    }
                    ".global" | ".globl" => {
                        // Global symbol - ignored for now
                        continue;
                    }
                    ".byte" => {
                        if self.in_data_section {
                            // Parse byte values
                            for part in &parts[1..] {
                                let part = part.trim_end_matches(',');
                                if let Some(val) = self.try_parse_immediate(Some(part)) {
                                    program.data_section.push(val as u8);
                                }
                            }
                        }
                        continue;
                    }
                    ".word" => {
                        if self.in_data_section {
                            // Parse 32-bit word values
                            for part in &parts[1..] {
                                let part = part.trim_end_matches(',');
                                if let Some(val) = self.try_parse_immediate(Some(part)) {
                                    program
                                        .data_section
                                        .extend_from_slice(&(val as u32).to_le_bytes());
                                }
                            }
                        }
                        continue;
                    }
                    ".dword" | ".quad" => {
                        if self.in_data_section {
                            // Parse 64-bit double word values
                            for part in &parts[1..] {
                                let part = part.trim_end_matches(',');
                                if let Some(val) = self.try_parse_immediate(Some(part)) {
                                    program
                                        .data_section
                                        .extend_from_slice(&(val as u64).to_le_bytes());
                                }
                            }
                        }
                        continue;
                    }
                    ".space" | ".skip" | ".zero" => {
                        if self.in_data_section {
                            // Reserve n bytes (optionally filled with a value)
                            let size = parts
                                .get(1)
                                .and_then(|s| s.trim_end_matches(',').parse::<usize>().ok())
                                .unwrap_or(0);
                            let fill = parts
                                .get(2)
                                .and_then(|s| self.try_parse_immediate(Some(s)))
                                .unwrap_or(0) as u8;
                            program.data_section.extend(std::iter::repeat_n(fill, size));
                        }
                        continue;
                    }
                    ".ascii" | ".string" | ".asciz" => {
                        if self.in_data_section {
                            // Parse string literal
                            let rest = line[directive.len()..].trim();
                            if rest.starts_with('"') && rest.ends_with('"') {
                                let s = &rest[1..rest.len() - 1];
                                program.data_section.extend_from_slice(s.as_bytes());
                                if directive == ".asciz" || directive == ".string" {
                                    program.data_section.push(0); // Null terminate
                                }
                            }
                        }
                        continue;
                    }
                    _ => {
                        // Unknown directive - check if it's a local label like .loop
                        if directive.len() > 1
                            && directive.chars().nth(1).is_some_and(|c| c.is_alphabetic())
                        {
                            // It's a local label like .loop, .done - continue to label handling
                        } else {
                            return Err(AsmError::InvalidOpcode(directive.to_string()));
                        }
                    }
                }
            }

            // Check for label
            if let Some(label_end) = line.find(':') {
                let label = line[..label_end].trim().to_string();
                if self.labels.contains_key(&label) {
                    return Err(AsmError::DuplicateLabel(label));
                }

                // Check what follows the label to determine if it's code or data
                let rest = line[label_end + 1..].trim();
                let is_data_directive = !rest.is_empty()
                    && rest.starts_with('.')
                    && (rest.starts_with(".word")
                        || rest.starts_with(".space")
                        || rest.starts_with(".skip")
                        || rest.starts_with(".zero")
                        || rest.starts_with(".byte")
                        || rest.starts_with(".dword")
                        || rest.starts_with(".quad")
                        || rest.starts_with(".ascii")
                        || rest.starts_with(".asciz")
                        || rest.starts_with(".string"));

                // If we're in data section AND this has a data directive, treat as data label
                // Otherwise, treat as code label (even if we were in data section)
                if self.in_data_section && is_data_directive {
                    // Data label - offset into data section
                    self.labels
                        .insert(label.clone(), LabelType::Data(program.data_section.len()));

                    // Parse data directive after label
                    let parts: Vec<&str> = rest.split_whitespace().collect();
                    let directive = parts[0];
                    match directive {
                        ".word" => {
                            for part in &parts[1..] {
                                let part = part.trim_end_matches(',');
                                if let Some(val) = self.try_parse_immediate(Some(part)) {
                                    program
                                        .data_section
                                        .extend_from_slice(&(val as u32).to_le_bytes());
                                }
                            }
                        }
                        ".space" | ".skip" | ".zero" => {
                            let size = parts
                                .get(1)
                                .and_then(|s| s.trim_end_matches(',').parse::<usize>().ok())
                                .unwrap_or(0);
                            let fill = parts
                                .get(2)
                                .and_then(|s| self.try_parse_immediate(Some(s)))
                                .unwrap_or(0) as u8;
                            program.data_section.extend(std::iter::repeat_n(fill, size));
                        }
                        ".byte" => {
                            for part in &parts[1..] {
                                let part = part.trim_end_matches(',');
                                if let Some(val) = self.try_parse_immediate(Some(part)) {
                                    program.data_section.push(val as u8);
                                }
                            }
                        }
                        ".dword" | ".quad" => {
                            for part in &parts[1..] {
                                let part = part.trim_end_matches(',');
                                if let Some(val) = self.try_parse_immediate(Some(part)) {
                                    program
                                        .data_section
                                        .extend_from_slice(&(val as u64).to_le_bytes());
                                }
                            }
                        }
                        ".ascii" | ".string" | ".asciz" => {
                            // Parse string literal after label
                            // rest is the full string after the label, e.g. `.asciz "hello"`
                            let start = rest.find('"');
                            let end = rest.rfind('"');
                            if let (Some(s), Some(e)) = (start, end) {
                                if s < e {
                                    let string_content = &rest[s + 1..e];
                                    // Handle escape sequences
                                    let mut chars = string_content.chars().peekable();
                                    while let Some(c) = chars.next() {
                                        if c == '\\' {
                                            match chars.next() {
                                                Some('n') => program.data_section.push(b'\n'),
                                                Some('r') => program.data_section.push(b'\r'),
                                                Some('t') => program.data_section.push(b'\t'),
                                                Some('0') => program.data_section.push(0),
                                                Some('\\') => program.data_section.push(b'\\'),
                                                Some('"') => program.data_section.push(b'"'),
                                                Some(other) => {
                                                    program.data_section.push(b'\\');
                                                    program.data_section.push(other as u8);
                                                }
                                                None => program.data_section.push(b'\\'),
                                            }
                                        } else {
                                            program.data_section.push(c as u8);
                                        }
                                    }
                                    if directive == ".asciz" || directive == ".string" {
                                        program.data_section.push(0); // Null terminate
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                } else {
                    // Code label - instruction index
                    self.in_data_section = false; // Seeing a code label exits data section
                    self.labels
                        .insert(label, LabelType::Code(current_instr_idx));

                    // Continue parsing after label
                    if rest.is_empty() {
                        continue;
                    }
                    self.parse_instruction(rest, line_num, &mut program, &mut current_instr_idx)?;
                }
            } else if !self.in_data_section {
                // Check for intrinsic call (@name args...)
                if line.starts_with('@') {
                    self.parse_intrinsic(line, line_num, &mut program, &mut current_instr_idx)?;
                } else {
                    self.parse_instruction(line, line_num, &mut program, &mut current_instr_idx)?;
                }
            }
        }

        // Second pass: resolve code labels (for branches/jumps)
        for (instr_idx, label, _line) in &self.pending_labels {
            let label_type = self
                .labels
                .get(label)
                .ok_or_else(|| AsmError::UndefinedLabel(label.clone()))?;

            match label_type {
                LabelType::Code(target_instr_idx) => {
                    // Calculate relative instruction index offset
                    let relative = (*target_instr_idx as i32) - (*instr_idx as i32);
                    program.instructions[*instr_idx].imm = Some(relative);
                }
                LabelType::Data(_) => {
                    // Branch to data label doesn't make sense, but we'll allow it
                    return Err(AsmError::ParseError {
                        line: *_line,
                        message: format!("Cannot branch to data label: {}", label),
                    });
                }
            }
        }

        // Second pass: resolve data labels (for mov instructions)
        for (instr_idx, label, _line) in &self.pending_data_refs {
            let label_type = self
                .labels
                .get(label)
                .ok_or_else(|| AsmError::UndefinedLabel(label.clone()))?;

            match label_type {
                LabelType::Data(offset) => {
                    // Data label resolves to DATA_BASE + offset
                    let addr = DATA_BASE as i32 + *offset as i32;
                    program.instructions[*instr_idx].imm = Some(addr);
                }
                LabelType::Code(target_instr_idx) => {
                    // Code label in MOV - use instruction index as value
                    program.instructions[*instr_idx].imm = Some(*target_instr_idx as i32);
                }
            }
        }

        // Resolve entry label to entry point
        if let Some(ref entry_label) = program.entry_label {
            let label_type = self
                .labels
                .get(entry_label)
                .ok_or_else(|| AsmError::UndefinedLabel(entry_label.clone()))?;
            match label_type {
                LabelType::Code(entry_instr_idx) => {
                    program.entry_point = *entry_instr_idx;
                }
                LabelType::Data(_) => {
                    return Err(AsmError::ParseError {
                        line: 0,
                        message: format!("Entry point cannot be a data label: {}", entry_label),
                    });
                }
            }
        }

        Ok(program)
    }

    /// Calculate byte offset for an instruction (for label resolution)
    #[allow(dead_code)]
    fn instruction_offset(&self, program: &Program, idx: usize) -> usize {
        program.instructions[..idx].iter().map(|i| i.size()).sum()
    }

    fn parse_instruction(
        &mut self,
        line: &str,
        line_num: usize,
        program: &mut Program,
        offset: &mut usize,
    ) -> Result<(), AsmError> {
        // Strip inline comments (everything after ; or #)
        let line = line.split(';').next().unwrap_or("");
        let line = line.split('#').next().unwrap_or("").trim();

        let parts: Vec<&str> = line
            .split(|c: char| c.is_whitespace() || c == ',')
            .filter(|s| !s.is_empty())
            .collect();

        if parts.is_empty() {
            return Ok(());
        }

        let mnemonic = parts[0].to_lowercase();
        let (opcode, mode) = self.parse_mnemonic(&mnemonic)?;

        let instr = match opcode {
            Opcode::Nop => {
                Instruction::new(opcode, Register::Zero, Register::Zero, Register::Zero, 0)
            }
            Opcode::Halt => {
                Instruction::new(opcode, Register::Zero, Register::Zero, Register::Zero, 0)
            }
            Opcode::Yield => {
                Instruction::new(opcode, Register::Zero, Register::Zero, Register::Zero, 0)
            }
            Opcode::Ret => {
                Instruction::new(opcode, Register::Zero, Register::Zero, Register::Zero, 0)
            }

            Opcode::Mov => {
                let rd = self.parse_register(parts.get(1).copied(), line_num)?;
                let src = parts.get(2).ok_or(AsmError::MissingOperand(line_num))?;
                let src = src.trim().trim_end_matches(',');

                if let Some(imm) = self.try_parse_immediate(Some(src)) {
                    // Immediate value
                    Instruction::with_imm(opcode, rd, Register::Zero, 0, imm)
                } else if let Ok(rs1) = self.parse_register(Some(src), line_num) {
                    // Register source
                    Instruction::new(opcode, rd, rs1, Register::Zero, 0)
                } else {
                    // Must be a label reference (data or code)
                    let instr_idx = program.instructions.len();
                    self.pending_data_refs
                        .push((instr_idx, src.to_string(), line_num));
                    // Create instruction with placeholder immediate (will be resolved)
                    Instruction::with_imm(opcode, rd, Register::Zero, 0, 0)
                }
            }

            Opcode::Alu | Opcode::AluI => {
                let rd = self.parse_register(parts.get(1).copied(), line_num)?;
                let rs1 = self.parse_register(parts.get(2).copied(), line_num)?;
                if let Some(imm) = self.try_parse_immediate(parts.get(3).copied()) {
                    Instruction::with_imm(Opcode::AluI, rd, rs1, mode, imm)
                } else {
                    let rs2 = self.parse_register(parts.get(3).copied(), line_num)?;
                    Instruction::new(Opcode::Alu, rd, rs1, rs2, mode)
                }
            }

            Opcode::MulDiv => {
                let rd = self.parse_register(parts.get(1).copied(), line_num)?;
                let rs1 = self.parse_register(parts.get(2).copied(), line_num)?;
                let rs2 = self.parse_register(parts.get(3).copied(), line_num)?;
                Instruction::new(opcode, rd, rs1, rs2, mode)
            }

            Opcode::Load | Opcode::Store => {
                let rd = self.parse_register(parts.get(1).copied(), line_num)?;
                let (rs1, imm_offset) = self.parse_memory_operand(&parts[2..], line_num)?;
                Instruction::with_imm(opcode, rd, rs1, mode, imm_offset)
            }

            Opcode::Branch => {
                if mode == BranchCond::Always as u8 {
                    // Unconditional branch
                    let target = parts.get(1).ok_or(AsmError::MissingOperand(line_num))?;
                    let instr_idx = program.instructions.len();
                    self.pending_labels
                        .push((instr_idx, target.to_string(), line_num));
                    Instruction::with_imm(opcode, Register::Zero, Register::Zero, mode, 0)
                } else {
                    // Conditional branch: branch.cond rs1, rs2, label
                    let rs1 = self.parse_register(parts.get(1).copied(), line_num)?;
                    let rs2 = self.parse_register(parts.get(2).copied(), line_num)?;
                    let target = parts.get(3).ok_or(AsmError::MissingOperand(line_num))?;
                    let instr_idx = program.instructions.len();
                    self.pending_labels
                        .push((instr_idx, target.to_string(), line_num));
                    // Branch instruction with rs1 and rs2 for comparison
                    let mut instr = Instruction::with_imm(opcode, Register::Zero, rs1, mode, 0);
                    instr.rs2 = rs2;
                    instr
                }
            }

            Opcode::Call => {
                let target = parts.get(1).ok_or(AsmError::MissingOperand(line_num))?;
                if let Some(imm) = self.try_parse_immediate(Some(target)) {
                    Instruction::with_imm(opcode, Register::Zero, Register::Zero, 0, imm)
                } else {
                    let instr_idx = program.instructions.len();
                    self.pending_labels
                        .push((instr_idx, target.to_string(), line_num));
                    Instruction::with_imm(opcode, Register::Zero, Register::Zero, 0, 0)
                }
            }

            Opcode::Jump => {
                let target = parts.get(1).ok_or(AsmError::MissingOperand(line_num))?;
                if target.starts_with('r') || target.starts_with('R') {
                    // Indirect jump
                    let rs1 = self.parse_register(Some(target), line_num)?;
                    Instruction::new(opcode, Register::Zero, rs1, Register::Zero, 1)
                } else if let Some(imm) = self.try_parse_immediate(Some(target)) {
                    Instruction::with_imm(opcode, Register::Zero, Register::Zero, 0, imm)
                } else {
                    let instr_idx = program.instructions.len();
                    self.pending_labels
                        .push((instr_idx, target.to_string(), line_num));
                    Instruction::with_imm(opcode, Register::Zero, Register::Zero, 0, 0)
                }
            }

            Opcode::Spawn => {
                // spawn rd, target [, arg_reg]
                let rd = self.parse_register(parts.get(1).copied(), line_num)?;
                let target = parts.get(2).ok_or(AsmError::MissingOperand(line_num))?;
                let instr_idx = program.instructions.len();
                self.pending_labels
                    .push((instr_idx, target.to_string(), line_num));
                // Optional argument register
                let rs1 = if parts.len() > 3 {
                    self.parse_register(parts.get(3).copied(), line_num)?
                } else {
                    Register::Zero
                };
                Instruction::with_imm(opcode, rd, rs1, 0, 0)
            }

            Opcode::Join => {
                // join task_reg - wait for task to complete
                let rs1 = self.parse_register(parts.get(1).copied(), line_num)?;
                Instruction::new(opcode, Register::Zero, rs1, Register::Zero, 0)
            }

            Opcode::Chan => {
                let rd = self.parse_register(parts.get(1).copied(), line_num)?;
                let rs1 = if parts.len() > 2 {
                    self.parse_register(parts.get(2).copied(), line_num)?
                } else {
                    Register::Zero
                };
                Instruction::new(opcode, rd, rs1, Register::Zero, mode)
            }

            Opcode::Atomic => {
                let rd = self.parse_register(parts.get(1).copied(), line_num)?;
                let rs1 = self.parse_register(parts.get(2).copied(), line_num)?;
                let rs2 = if parts.len() > 3 {
                    self.parse_register(parts.get(3).copied(), line_num)?
                } else {
                    Register::Zero
                };
                Instruction::new(opcode, rd, rs1, rs2, mode)
            }

            Opcode::Fence => {
                Instruction::new(opcode, Register::Zero, Register::Zero, Register::Zero, mode)
            }

            Opcode::CapNew => {
                // cap.new rd, base_reg, len_reg [, perms_reg]
                let rd = self.parse_register(parts.get(1).copied(), line_num)?;
                let rs1 = self.parse_register(parts.get(2).copied(), line_num)?;
                let rs2 = if parts.len() > 3 {
                    self.parse_register(parts.get(3).copied(), line_num)?
                } else {
                    Register::Zero
                };
                Instruction::new(opcode, rd, rs1, rs2, mode)
            }

            Opcode::CapRestrict => {
                // cap.restrict rd, src_cap, new_len_reg, new_perms_reg
                let rd = self.parse_register(parts.get(1).copied(), line_num)?;
                let rs1 = self.parse_register(parts.get(2).copied(), line_num)?;
                let rs2 = if parts.len() > 3 {
                    self.parse_register(parts.get(3).copied(), line_num)?
                } else {
                    Register::Zero
                };
                // 4th arg (new_perms) goes into imm or we need another approach
                // For now, use mode bits or encode differently
                Instruction::new(opcode, rd, rs1, rs2, mode)
            }

            Opcode::CapQuery => {
                // cap.query rd, cap_reg, query_type_imm
                let rd = self.parse_register(parts.get(1).copied(), line_num)?;
                let rs1 = self.parse_register(parts.get(2).copied(), line_num)?;
                let query_type = self.try_parse_immediate(parts.get(3).copied()).unwrap_or(0);
                Instruction::with_imm(opcode, rd, rs1, mode, query_type)
            }

            Opcode::Taint | Opcode::Sanitize => {
                let rd = self.parse_register(parts.get(1).copied(), line_num)?;
                let rs1 = if parts.len() > 2 {
                    self.parse_register(parts.get(2).copied(), line_num)?
                } else {
                    rd
                };
                Instruction::new(opcode, rd, rs1, Register::Zero, mode)
            }

            Opcode::Trap => {
                if let Some(imm) = self.try_parse_immediate(parts.get(1).copied()) {
                    Instruction::with_imm(opcode, Register::Zero, Register::Zero, mode, imm)
                } else {
                    Instruction::new(opcode, Register::Zero, Register::Zero, Register::Zero, mode)
                }
            }

            // I/O opcodes - need rd, rs1, rs2, and optional imm
            // Syntax: file.op rd, rs1, rs2[, imm]
            // e.g., file.open r2, r0, r1, 6  -> open(path@r0, len@r1, flags=6) -> fd in r2
            //       file.read r2, r10, r1, 128 -> read(fd@r10, buf@r1, len=128) -> bytes in r2
            Opcode::File => {
                let rd = self.parse_register(parts.get(1).copied(), line_num)?;
                let rs1 = if parts.len() > 2 {
                    self.parse_register(parts.get(2).copied(), line_num)?
                } else {
                    Register::Zero
                };
                let rs2 = if parts.len() > 3 {
                    // Check if it's a register or immediate
                    if let Some(reg) = self.try_parse_register(parts.get(3).copied()) {
                        reg
                    } else {
                        Register::Zero
                    }
                } else {
                    Register::Zero
                };
                // Look for immediate in 4th or 5th position
                let imm = self.try_parse_immediate(parts.get(4).copied()).or_else(|| {
                    if rs2 == Register::Zero {
                        self.try_parse_immediate(parts.get(3).copied())
                    } else {
                        None
                    }
                });
                if let Some(imm) = imm {
                    let mut instr = Instruction::with_imm(opcode, rd, rs1, mode, imm);
                    instr.rs2 = rs2;
                    instr
                } else {
                    Instruction::new(opcode, rd, rs1, rs2, mode)
                }
            }

            // NET opcodes - same pattern as FILE
            // e.g., net.recv r2, r10, r1, 1024 -> recv(fd@r10, buf@r1, len=1024) -> bytes in r2
            Opcode::Net => {
                let rd = self.parse_register(parts.get(1).copied(), line_num)?;
                let rs1 = if parts.len() > 2 {
                    self.parse_register(parts.get(2).copied(), line_num)?
                } else {
                    Register::Zero
                };
                let rs2 = if parts.len() > 3 {
                    if let Some(reg) = self.try_parse_register(parts.get(3).copied()) {
                        reg
                    } else {
                        Register::Zero
                    }
                } else {
                    Register::Zero
                };
                let imm = self.try_parse_immediate(parts.get(4).copied()).or_else(|| {
                    if rs2 == Register::Zero {
                        self.try_parse_immediate(parts.get(3).copied())
                    } else {
                        None
                    }
                });
                if let Some(imm) = imm {
                    let mut instr = Instruction::with_imm(opcode, rd, rs1, mode, imm);
                    instr.rs2 = rs2;
                    instr
                } else {
                    Instruction::new(opcode, rd, rs1, rs2, mode)
                }
            }

            Opcode::NetSetopt => {
                let rs1 = self.parse_register(parts.get(1).copied(), line_num)?;
                let imm = self.try_parse_immediate(parts.get(2).copied()).unwrap_or(0);
                Instruction::with_imm(opcode, Register::Zero, rs1, mode, imm)
            }

            // IO opcodes - need rd, rs1, rs2
            // Syntax: io.print rd, rs1, rs2 -> print(buf@rs1, len@rs2)
            Opcode::Io => {
                let rd = self.parse_register(parts.get(1).copied(), line_num)?;
                let rs1 = if parts.len() > 2 {
                    self.parse_register(parts.get(2).copied(), line_num)?
                } else {
                    Register::Zero
                };
                let rs2 = if parts.len() > 3 {
                    if let Some(reg) = self.try_parse_register(parts.get(3).copied()) {
                        reg
                    } else {
                        Register::Zero
                    }
                } else {
                    Register::Zero
                };
                let imm = self.try_parse_immediate(parts.get(4).copied()).or_else(|| {
                    if rs2 == Register::Zero {
                        self.try_parse_immediate(parts.get(3).copied())
                    } else {
                        None
                    }
                });
                if let Some(imm) = imm {
                    let mut instr = Instruction::with_imm(opcode, rd, rs1, mode, imm);
                    instr.rs2 = rs2;
                    instr
                } else {
                    Instruction::new(opcode, rd, rs1, rs2, mode)
                }
            }

            Opcode::Time => {
                let rd = self.parse_register(parts.get(1).copied(), line_num)?;
                if let Some(imm) = self.try_parse_immediate(parts.get(2).copied()) {
                    Instruction::with_imm(opcode, rd, Register::Zero, mode, imm)
                } else {
                    Instruction::new(opcode, rd, Register::Zero, Register::Zero, mode)
                }
            }

            // Math extension opcodes
            Opcode::Fpu => {
                let rd = self.parse_register(parts.get(1).copied(), line_num)?;
                let rs1 = self.parse_register(parts.get(2).copied(), line_num)?;
                let rs2 = if parts.len() > 3 {
                    self.parse_register(parts.get(3).copied(), line_num)?
                } else {
                    Register::Zero
                };
                Instruction::new(opcode, rd, rs1, rs2, mode)
            }

            Opcode::Rand => {
                let rd = self.parse_register(parts.get(1).copied(), line_num)?;
                let rs1 = if parts.len() > 2 {
                    self.parse_register(parts.get(2).copied(), line_num)?
                } else {
                    Register::Zero
                };
                Instruction::new(opcode, rd, rs1, Register::Zero, mode)
            }

            Opcode::Bits => {
                let rd = self.parse_register(parts.get(1).copied(), line_num)?;
                let rs1 = self.parse_register(parts.get(2).copied(), line_num)?;
                Instruction::new(opcode, rd, rs1, Register::Zero, mode)
            }

            Opcode::ExtCall => {
                // ext.call rd, ext_id, rs1, rs2
                // or: ext.call rd, EXT_NAME, rs1, rs2 (symbolic name resolved to ID)
                // or: ext.call rd, @"intent description", rs1, rs2 (RAG-resolved)
                let rd = self.parse_register(parts.get(1).copied(), line_num)?;

                // Check for intent syntax: @"description"
                // This requires special handling because the description may contain spaces
                let (ext_id, remaining_parts_start) = if line.contains("@\"") {
                    // Find the intent string in the original line
                    let intent_start = line.find("@\"").unwrap();
                    let after_quote = &line[intent_start + 2..];
                    let intent_end = after_quote.find('"').ok_or(AsmError::ParseError {
                        line: line_num,
                        message: "Unterminated intent string (missing closing \")".to_string(),
                    })?;
                    let intent = &after_quote[..intent_end];

                    // Use RAG resolver to find the extension
                    let resolved =
                        self.rag_resolver
                            .resolve(intent)
                            .ok_or(AsmError::ExtensionNotFound {
                                line: line_num,
                                intent: intent.to_string(),
                            })?;

                    // Find remaining arguments after the closing quote
                    // The remaining line after the intent string
                    let remaining = &after_quote[intent_end + 1..];
                    let remaining_parts: Vec<&str> = remaining
                        .split(|c: char| c.is_whitespace() || c == ',')
                        .filter(|s| !s.is_empty())
                        .collect();

                    // remaining_parts now contains [rs1, rs2] if present
                    // We'll use indices 0 and 1 instead of 3 and 4
                    (resolved.id as i32, Some(remaining_parts))
                } else {
                    // Standard parsing: numeric ID or symbolic name
                    let ext_id_or_name = parts.get(2).ok_or(AsmError::MissingOperand(line_num))?;
                    let ext_id_or_name = ext_id_or_name.trim().trim_end_matches(',');

                    let id = if let Some(imm) = self.try_parse_immediate(Some(ext_id_or_name)) {
                        imm
                    } else {
                        // Try RAG resolver for symbolic names
                        if let Some(resolved) = self.rag_resolver.get_by_name(ext_id_or_name) {
                            resolved.id as i32
                        } else {
                            return Err(AsmError::ExtensionNotFound {
                                line: line_num,
                                intent: ext_id_or_name.to_string(),
                            });
                        }
                    };
                    (id, None)
                };

                // Parse rs1 and rs2
                let (rs1, rs2) = if let Some(remaining) = remaining_parts_start {
                    // Intent syntax: remaining parts are after the closing quote
                    let rs1 = if !remaining.is_empty() {
                        self.parse_register(remaining.first().copied(), line_num)?
                    } else {
                        Register::Zero
                    };
                    let rs2 = if remaining.len() > 1 {
                        self.parse_register(remaining.get(1).copied(), line_num)?
                    } else {
                        Register::Zero
                    };
                    (rs1, rs2)
                } else {
                    // Standard syntax: parts 3 and 4
                    let rs1 = if parts.len() > 3 {
                        self.parse_register(parts.get(3).copied(), line_num)?
                    } else {
                        Register::Zero
                    };
                    let rs2 = if parts.len() > 4 {
                        self.parse_register(parts.get(4).copied(), line_num)?
                    } else {
                        Register::Zero
                    };
                    (rs1, rs2)
                };

                // Create instruction with all fields for ext.call
                Instruction {
                    opcode,
                    rd,
                    rs1,
                    rs2,
                    mode: 0,
                    imm: Some(ext_id),
                }
            }
        };

        *offset += 1; // Increment instruction index (not byte offset)
        program.instructions.push(instr);
        Ok(())
    }

    /// Parse an intrinsic call and expand it to instructions
    fn parse_intrinsic(
        &self,
        line: &str,
        line_num: usize,
        program: &mut Program,
        offset: &mut usize,
    ) -> Result<(), AsmError> {
        // Strip @ prefix and comments
        let line = line[1..].trim();
        let line = line.split(';').next().unwrap_or("");
        let line = line.split('#').next().unwrap_or("").trim();

        // Parse: name arg1, arg2, ...
        let parts: Vec<&str> = line
            .split(|c: char| c.is_whitespace() || c == ',')
            .filter(|s| !s.is_empty())
            .collect();

        if parts.is_empty() {
            return Err(AsmError::ParseError {
                line: line_num,
                message: "Empty intrinsic call".to_string(),
            });
        }

        let name = parts[0].to_lowercase();

        // Parse arguments
        let mut args = Vec::new();
        for part in &parts[1..] {
            let part = part.trim().trim_end_matches(',');

            // Try as register first
            if let Some(reg) = self.try_parse_register(Some(part)) {
                args.push(IntrinsicArg::Register(reg));
            } else if let Some(imm) = self.try_parse_immediate(Some(part)) {
                args.push(IntrinsicArg::Immediate(imm));
            } else {
                return Err(AsmError::IntrinsicError {
                    line: line_num,
                    message: format!("Invalid argument: {}", part),
                });
            }
        }

        let call = IntrinsicCall {
            name: name.clone(),
            args,
        };

        // Expand the intrinsic
        match self.intrinsics.expand(&call) {
            Ok(instructions) => {
                for instr in instructions {
                    *offset += 1;
                    program.instructions.push(instr);
                }
                Ok(())
            }
            Err(e) => Err(AsmError::IntrinsicError {
                line: line_num,
                message: e.to_string(),
            }),
        }
    }

    fn parse_mnemonic(&self, mnemonic: &str) -> Result<(Opcode, u8), AsmError> {
        // Handle compound mnemonics like "add", "beq", "load.d"
        let (base, suffix) = if let Some(dot_pos) = mnemonic.find('.') {
            (&mnemonic[..dot_pos], Some(&mnemonic[dot_pos + 1..]))
        } else {
            (mnemonic, None)
        };

        match base {
            // ALU operations (map to ALU opcode + mode)
            "add" => Ok((Opcode::Alu, AluOp::Add as u8)),
            "sub" => Ok((Opcode::Alu, AluOp::Sub as u8)),
            "and" => Ok((Opcode::Alu, AluOp::And as u8)),
            "or" => Ok((Opcode::Alu, AluOp::Or as u8)),
            "xor" => Ok((Opcode::Alu, AluOp::Xor as u8)),
            "shl" | "sll" => Ok((Opcode::Alu, AluOp::Shl as u8)),
            "shr" | "srl" => Ok((Opcode::Alu, AluOp::Shr as u8)),
            "sar" | "sra" => Ok((Opcode::Alu, AluOp::Sar as u8)),

            // ALU with dot-notation: alu.add, alu.sub, etc.
            "alu" => {
                let op = match suffix {
                    Some("add") => AluOp::Add,
                    Some("sub") => AluOp::Sub,
                    Some("and") => AluOp::And,
                    Some("or") => AluOp::Or,
                    Some("xor") => AluOp::Xor,
                    Some("shl") | Some("sll") => AluOp::Shl,
                    Some("shr") | Some("srl") => AluOp::Shr,
                    Some("sar") | Some("sra") => AluOp::Sar,
                    _ => return Err(AsmError::InvalidOpcode(mnemonic.to_string())),
                };
                Ok((Opcode::Alu, op as u8))
            }

            "addi" => Ok((Opcode::AluI, AluOp::Add as u8)),
            "subi" => Ok((Opcode::AluI, AluOp::Sub as u8)),
            "andi" => Ok((Opcode::AluI, AluOp::And as u8)),
            "ori" => Ok((Opcode::AluI, AluOp::Or as u8)),
            "xori" => Ok((Opcode::AluI, AluOp::Xor as u8)),
            "shli" | "slli" => Ok((Opcode::AluI, AluOp::Shl as u8)),
            "shri" | "srli" => Ok((Opcode::AluI, AluOp::Shr as u8)),
            "sari" | "srai" => Ok((Opcode::AluI, AluOp::Sar as u8)),

            // ALUI with dot-notation: alui.add, alui.sub, etc.
            "alui" => {
                let op = match suffix {
                    Some("add") => AluOp::Add,
                    Some("sub") => AluOp::Sub,
                    Some("and") => AluOp::And,
                    Some("or") => AluOp::Or,
                    Some("xor") => AluOp::Xor,
                    Some("shl") | Some("sll") => AluOp::Shl,
                    Some("shr") | Some("srl") => AluOp::Shr,
                    Some("sar") | Some("sra") => AluOp::Sar,
                    _ => return Err(AsmError::InvalidOpcode(mnemonic.to_string())),
                };
                Ok((Opcode::AluI, op as u8))
            }

            // MulDiv
            "mul" => Ok((Opcode::MulDiv, MulDivOp::Mul as u8)),
            "mulh" => Ok((Opcode::MulDiv, MulDivOp::MulH as u8)),
            "div" => Ok((Opcode::MulDiv, MulDivOp::Div as u8)),
            "mod" | "rem" => Ok((Opcode::MulDiv, MulDivOp::Mod as u8)),

            // MulDiv with dot-notation: muldiv.mul, muldiv.div, etc.
            "muldiv" => {
                let op = match suffix {
                    Some("mul") => MulDivOp::Mul,
                    Some("mulh") => MulDivOp::MulH,
                    Some("div") => MulDivOp::Div,
                    Some("mod") | Some("rem") => MulDivOp::Mod,
                    _ => return Err(AsmError::InvalidOpcode(mnemonic.to_string())),
                };
                Ok((Opcode::MulDiv, op as u8))
            }

            // Memory (with width suffix)
            "load" | "ld" => {
                let width = self.parse_width_suffix(suffix);
                Ok((Opcode::Load, width as u8))
            }
            "store" | "st" => {
                let width = self.parse_width_suffix(suffix);
                Ok((Opcode::Store, width as u8))
            }
            "lb" => Ok((Opcode::Load, MemWidth::Byte as u8)),
            "lh" => Ok((Opcode::Load, MemWidth::Half as u8)),
            "lw" => Ok((Opcode::Load, MemWidth::Word as u8)),
            // Note: "ld" is handled above with "load" | "ld"
            "sb" => Ok((Opcode::Store, MemWidth::Byte as u8)),
            "sh" => Ok((Opcode::Store, MemWidth::Half as u8)),
            "sw" => Ok((Opcode::Store, MemWidth::Word as u8)),
            "sd" => Ok((Opcode::Store, MemWidth::Double as u8)),

            // Atomics
            "cas" | "cmpxchg" => Ok((Opcode::Atomic, AtomicOp::Cas as u8)),
            "xchg" => Ok((Opcode::Atomic, AtomicOp::Xchg as u8)),
            "atomic" => {
                let op = match suffix {
                    Some("add") => AtomicOp::Add,
                    Some("and") => AtomicOp::And,
                    Some("or") => AtomicOp::Or,
                    Some("xor") => AtomicOp::Xor,
                    Some("min") => AtomicOp::Min,
                    Some("max") => AtomicOp::Max,
                    Some("cas") => AtomicOp::Cas,
                    Some("xchg") => AtomicOp::Xchg,
                    _ => AtomicOp::Add,
                };
                Ok((Opcode::Atomic, op as u8))
            }

            // Branch
            "b" | "br" | "jmp" | "j" => Ok((Opcode::Branch, BranchCond::Always as u8)),
            "beq" => Ok((Opcode::Branch, BranchCond::Eq as u8)),
            "bne" => Ok((Opcode::Branch, BranchCond::Ne as u8)),
            "blt" => Ok((Opcode::Branch, BranchCond::Lt as u8)),
            "ble" => Ok((Opcode::Branch, BranchCond::Le as u8)),
            "bgt" => Ok((Opcode::Branch, BranchCond::Gt as u8)),
            "bge" => Ok((Opcode::Branch, BranchCond::Ge as u8)),
            "bltu" => Ok((Opcode::Branch, BranchCond::Ltu as u8)),

            // Branch with dot-notation: branch.eq, branch.lt, etc.
            "branch" => {
                let cond = match suffix {
                    Some("eq") => BranchCond::Eq,
                    Some("ne") => BranchCond::Ne,
                    Some("lt") => BranchCond::Lt,
                    Some("le") => BranchCond::Le,
                    Some("gt") => BranchCond::Gt,
                    Some("ge") => BranchCond::Ge,
                    Some("ltu") => BranchCond::Ltu,
                    None | Some("always") => BranchCond::Always,
                    _ => return Err(AsmError::InvalidOpcode(mnemonic.to_string())),
                };
                Ok((Opcode::Branch, cond as u8))
            }

            // Control flow
            "call" => Ok((Opcode::Call, 0)),
            "ret" | "return" => Ok((Opcode::Ret, 0)),
            "jump" => Ok((Opcode::Jump, 0)),

            // Capabilities
            "cap" => {
                let mode = match suffix {
                    Some("new") => (Opcode::CapNew, 0),
                    Some("restrict") => (Opcode::CapRestrict, 0),
                    Some("query") | Some("get") => (Opcode::CapQuery, 0),
                    _ => (Opcode::CapNew, 0),
                };
                Ok(mode)
            }

            // Concurrency
            "spawn" => Ok((Opcode::Spawn, 0)),
            "join" => Ok((Opcode::Join, 0)),
            "chan" => {
                let op = match suffix {
                    Some("create") | Some("new") => ChanOp::Create,
                    Some("send") => ChanOp::Send,
                    Some("recv") => ChanOp::Recv,
                    Some("close") => ChanOp::Close,
                    _ => ChanOp::Create,
                };
                Ok((Opcode::Chan, op as u8))
            }
            "send" => Ok((Opcode::Chan, ChanOp::Send as u8)),
            "recv" => Ok((Opcode::Chan, ChanOp::Recv as u8)),
            "fence" => {
                let mode = match suffix {
                    Some("acquire") | Some("acq") => FenceMode::Acquire,
                    Some("release") | Some("rel") => FenceMode::Release,
                    Some("acqrel") => FenceMode::AcqRel,
                    Some("seqcst") | Some("seq") => FenceMode::SeqCst,
                    _ => FenceMode::SeqCst,
                };
                Ok((Opcode::Fence, mode as u8))
            }
            "yield" => Ok((Opcode::Yield, 0)),

            // Taint
            "taint" => Ok((Opcode::Taint, 0)),
            "sanitize" | "untaint" => Ok((Opcode::Sanitize, 0)),

            // System
            "mov" | "move" => Ok((Opcode::Mov, 0)),
            "li" => Ok((Opcode::Mov, 1)), // Load immediate variant
            "trap" | "syscall" => Ok((Opcode::Trap, TrapType::Syscall as u8)),
            "break" | "brk" | "bkpt" => Ok((Opcode::Trap, TrapType::Breakpoint as u8)),
            "nop" => Ok((Opcode::Nop, 0)),
            "halt" | "hlt" => Ok((Opcode::Halt, 0)),

            // File I/O
            "file" => {
                let op = match suffix {
                    Some("open") => FileOp::Open,
                    Some("read") => FileOp::Read,
                    Some("write") => FileOp::Write,
                    Some("close") => FileOp::Close,
                    Some("seek") => FileOp::Seek,
                    Some("stat") => FileOp::Stat,
                    Some("mkdir") => FileOp::Mkdir,
                    Some("delete") | Some("rm") => FileOp::Delete,
                    _ => FileOp::Open,
                };
                Ok((Opcode::File, op as u8))
            }
            "fopen" => Ok((Opcode::File, FileOp::Open as u8)),
            "fread" => Ok((Opcode::File, FileOp::Read as u8)),
            "fwrite" => Ok((Opcode::File, FileOp::Write as u8)),
            "fclose" => Ok((Opcode::File, FileOp::Close as u8)),
            "fseek" => Ok((Opcode::File, FileOp::Seek as u8)),
            "fstat" => Ok((Opcode::File, FileOp::Stat as u8)),
            "mkdir" => Ok((Opcode::File, FileOp::Mkdir as u8)),
            "fdelete" | "frm" => Ok((Opcode::File, FileOp::Delete as u8)),

            // Network I/O
            "net" => {
                let op = match suffix {
                    Some("socket") => NetOp::Socket,
                    Some("connect") => NetOp::Connect,
                    Some("bind") => NetOp::Bind,
                    Some("listen") => NetOp::Listen,
                    Some("accept") => NetOp::Accept,
                    Some("send") => NetOp::Send,
                    Some("recv") => NetOp::Recv,
                    Some("close") => NetOp::Close,
                    _ => NetOp::Socket,
                };
                Ok((Opcode::Net, op as u8))
            }
            "socket" => Ok((Opcode::Net, NetOp::Socket as u8)),
            "connect" => Ok((Opcode::Net, NetOp::Connect as u8)),
            "bind" => Ok((Opcode::Net, NetOp::Bind as u8)),
            "listen" => Ok((Opcode::Net, NetOp::Listen as u8)),
            "accept" => Ok((Opcode::Net, NetOp::Accept as u8)),
            "nsend" => Ok((Opcode::Net, NetOp::Send as u8)),
            "nrecv" => Ok((Opcode::Net, NetOp::Recv as u8)),
            "nclose" => Ok((Opcode::Net, NetOp::Close as u8)),

            // Socket options
            "setopt" | "setsockopt" => {
                let opt = match suffix {
                    Some("nonblock") => NetOption::Nonblock,
                    Some("timeout") => NetOption::TimeoutMs,
                    Some("keepalive") => NetOption::Keepalive,
                    Some("reuseaddr") => NetOption::ReuseAddr,
                    Some("nodelay") => NetOption::NoDelay,
                    Some("rcvbuf") => NetOption::RecvBufSize,
                    Some("sndbuf") => NetOption::SendBufSize,
                    Some("linger") => NetOption::Linger,
                    _ => NetOption::Nonblock,
                };
                Ok((Opcode::NetSetopt, opt as u8))
            }

            // Console I/O
            "io" => {
                let op = match suffix {
                    Some("print") => IoOp::Print,
                    Some("readline") | Some("read") => IoOp::ReadLine,
                    Some("getargs") | Some("args") => IoOp::GetArgs,
                    Some("getenv") | Some("env") => IoOp::GetEnv,
                    _ => IoOp::Print,
                };
                Ok((Opcode::Io, op as u8))
            }
            "print" | "puts" => Ok((Opcode::Io, IoOp::Print as u8)),
            "readline" | "gets" => Ok((Opcode::Io, IoOp::ReadLine as u8)),
            "getargs" => Ok((Opcode::Io, IoOp::GetArgs as u8)),
            "getenv" => Ok((Opcode::Io, IoOp::GetEnv as u8)),

            // Time operations
            "time" => {
                let op = match suffix {
                    Some("now") => TimeOp::Now,
                    Some("sleep") => TimeOp::Sleep,
                    Some("mono") | Some("monotonic") => TimeOp::Monotonic,
                    _ => TimeOp::Now,
                };
                Ok((Opcode::Time, op as u8))
            }
            "now" => Ok((Opcode::Time, TimeOp::Now as u8)),
            "sleep" => Ok((Opcode::Time, TimeOp::Sleep as u8)),
            "monotonic" => Ok((Opcode::Time, TimeOp::Monotonic as u8)),

            // Floating point
            "fpu" => {
                let op = match suffix {
                    Some("add") | Some("fadd") => FpuOp::Fadd,
                    Some("sub") | Some("fsub") => FpuOp::Fsub,
                    Some("mul") | Some("fmul") => FpuOp::Fmul,
                    Some("div") | Some("fdiv") => FpuOp::Fdiv,
                    Some("sqrt") | Some("fsqrt") => FpuOp::Fsqrt,
                    Some("abs") | Some("fabs") => FpuOp::Fabs,
                    Some("floor") | Some("ffloor") => FpuOp::Ffloor,
                    Some("ceil") | Some("fceil") => FpuOp::Fceil,
                    // Comparison operations
                    Some("cmpeq") | Some("fcmpeq") => FpuOp::Fcmpeq,
                    Some("cmpne") | Some("fcmpne") => FpuOp::Fcmpne,
                    Some("cmplt") | Some("fcmplt") => FpuOp::Fcmplt,
                    Some("cmple") | Some("fcmple") => FpuOp::Fcmple,
                    Some("cmpgt") | Some("fcmpgt") => FpuOp::Fcmpgt,
                    Some("cmpge") | Some("fcmpge") => FpuOp::Fcmpge,
                    _ => FpuOp::Fadd,
                };
                Ok((Opcode::Fpu, op as u8))
            }
            "fadd" => Ok((Opcode::Fpu, FpuOp::Fadd as u8)),
            "fsub" => Ok((Opcode::Fpu, FpuOp::Fsub as u8)),
            "fmul" => Ok((Opcode::Fpu, FpuOp::Fmul as u8)),
            "fdiv" => Ok((Opcode::Fpu, FpuOp::Fdiv as u8)),
            "fsqrt" => Ok((Opcode::Fpu, FpuOp::Fsqrt as u8)),
            "fabs" => Ok((Opcode::Fpu, FpuOp::Fabs as u8)),
            "ffloor" => Ok((Opcode::Fpu, FpuOp::Ffloor as u8)),
            "fceil" => Ok((Opcode::Fpu, FpuOp::Fceil as u8)),
            // Float comparison shorthand
            "fcmpeq" => Ok((Opcode::Fpu, FpuOp::Fcmpeq as u8)),
            "fcmpne" => Ok((Opcode::Fpu, FpuOp::Fcmpne as u8)),
            "fcmplt" => Ok((Opcode::Fpu, FpuOp::Fcmplt as u8)),
            "fcmple" => Ok((Opcode::Fpu, FpuOp::Fcmple as u8)),
            "fcmpgt" => Ok((Opcode::Fpu, FpuOp::Fcmpgt as u8)),
            "fcmpge" => Ok((Opcode::Fpu, FpuOp::Fcmpge as u8)),

            // Random numbers
            "rand" => {
                let op = match suffix {
                    Some("bytes") => RandOp::RandBytes,
                    Some("u64") | Some("int") => RandOp::RandU64,
                    _ => RandOp::RandU64,
                };
                Ok((Opcode::Rand, op as u8))
            }
            "randbytes" => Ok((Opcode::Rand, RandOp::RandBytes as u8)),
            "randu64" | "randint" => Ok((Opcode::Rand, RandOp::RandU64 as u8)),

            // Bit manipulation
            "bits" => {
                let op = match suffix {
                    Some("popcount") | Some("popcnt") => BitsOp::Popcount,
                    Some("clz") => BitsOp::Clz,
                    Some("ctz") => BitsOp::Ctz,
                    Some("bswap") => BitsOp::Bswap,
                    _ => BitsOp::Popcount,
                };
                Ok((Opcode::Bits, op as u8))
            }
            "popcount" | "popcnt" => Ok((Opcode::Bits, BitsOp::Popcount as u8)),
            "clz" => Ok((Opcode::Bits, BitsOp::Clz as u8)),
            "ctz" => Ok((Opcode::Bits, BitsOp::Ctz as u8)),
            "bswap" => Ok((Opcode::Bits, BitsOp::Bswap as u8)),

            // Extension calls (Tier 2 FFI)
            "ext" => match suffix {
                Some("call") => Ok((Opcode::ExtCall, 0)),
                _ => Err(AsmError::InvalidOpcode(mnemonic.to_string())),
            },
            "extcall" => Ok((Opcode::ExtCall, 0)),

            _ => Err(AsmError::InvalidOpcode(mnemonic.to_string())),
        }
    }

    fn parse_width_suffix(&self, suffix: Option<&str>) -> MemWidth {
        match suffix {
            Some("b") | Some("byte") => MemWidth::Byte,
            Some("h") | Some("half") | Some("w16") => MemWidth::Half,
            Some("w") | Some("word") | Some("w32") => MemWidth::Word,
            Some("d") | Some("double") | Some("w64") | None => MemWidth::Double,
            _ => MemWidth::Double,
        }
    }

    fn parse_register(&self, s: Option<&str>, line: usize) -> Result<Register, AsmError> {
        let s = s.ok_or(AsmError::MissingOperand(line))?;
        let s = s.trim().to_lowercase();
        let s = s.trim_end_matches(',');

        match s {
            "r0" | "a0" | "ret" => Ok(Register::R0),
            "r1" | "a1" => Ok(Register::R1),
            "r2" | "a2" => Ok(Register::R2),
            "r3" | "a3" => Ok(Register::R3),
            "r4" | "a4" => Ok(Register::R4),
            "r5" | "a5" => Ok(Register::R5),
            "r6" => Ok(Register::R6),
            "r7" => Ok(Register::R7),
            "r8" => Ok(Register::R8),
            "r9" => Ok(Register::R9),
            "r10" => Ok(Register::R10),
            "r11" => Ok(Register::R11),
            "r12" => Ok(Register::R12),
            "r13" => Ok(Register::R13),
            "r14" => Ok(Register::R14),
            "r15" => Ok(Register::R15),
            "sp" => Ok(Register::Sp),
            "fp" => Ok(Register::Fp),
            "lr" => Ok(Register::Lr),
            "pc" => Ok(Register::Pc),
            "csp" => Ok(Register::Csp),
            "cfp" => Ok(Register::Cfp),
            "zero" | "x0" => Ok(Register::Zero),
            _ => Err(AsmError::InvalidRegister(s.to_string())),
        }
    }

    fn try_parse_register(&self, s: Option<&str>) -> Option<Register> {
        let s = s?.trim().to_lowercase();
        let s = s.trim_end_matches(',');

        match s {
            "r0" | "a0" | "ret" => Some(Register::R0),
            "r1" | "a1" => Some(Register::R1),
            "r2" | "a2" => Some(Register::R2),
            "r3" | "a3" => Some(Register::R3),
            "r4" | "a4" => Some(Register::R4),
            "r5" | "a5" => Some(Register::R5),
            "r6" => Some(Register::R6),
            "r7" => Some(Register::R7),
            "r8" => Some(Register::R8),
            "r9" => Some(Register::R9),
            "r10" => Some(Register::R10),
            "r11" => Some(Register::R11),
            "r12" => Some(Register::R12),
            "r13" => Some(Register::R13),
            "r14" => Some(Register::R14),
            "r15" => Some(Register::R15),
            "sp" => Some(Register::Sp),
            "fp" => Some(Register::Fp),
            "lr" => Some(Register::Lr),
            "pc" => Some(Register::Pc),
            "csp" => Some(Register::Csp),
            "cfp" => Some(Register::Cfp),
            "zero" | "x0" => Some(Register::Zero),
            _ => None,
        }
    }

    fn try_parse_immediate(&self, s: Option<&str>) -> Option<i32> {
        let s = s?.trim().trim_end_matches(',');

        // Check for label (no immediate)
        if s.chars().next()?.is_alphabetic() && !s.starts_with("0x") && !s.starts_with("0b") {
            return None;
        }

        if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
            // Parse as u32 first to support full 32-bit range, then bitcast to i32
            u32::from_str_radix(hex, 16).ok().map(|v| v as i32)
        } else if let Some(bin) = s.strip_prefix("0b").or_else(|| s.strip_prefix("0B")) {
            u32::from_str_radix(bin, 2).ok().map(|v| v as i32)
        } else if s.starts_with('-') {
            // Negative decimal - parse as i32 directly
            s.parse().ok()
        } else {
            // Positive decimal - parse as u32 for full range, then bitcast
            s.parse::<u32>()
                .ok()
                .map(|v| v as i32)
                .or_else(|| s.parse().ok())
        }
    }

    fn parse_memory_operand(
        &self,
        parts: &[&str],
        line: usize,
    ) -> Result<(Register, i32), AsmError> {
        // Formats: [reg], [reg + imm], [reg - imm], reg(imm), imm(reg)
        let combined: String = parts.join("");
        let combined = combined.trim();

        // [reg] or [reg + offset]
        if combined.starts_with('[') && combined.ends_with(']') {
            let inner = &combined[1..combined.len() - 1];

            if let Some(plus) = inner.find('+') {
                let reg_str = inner[..plus].trim();
                let imm_str = inner[plus + 1..].trim();
                let reg = self.parse_register(Some(reg_str), line)?;
                let imm = self.try_parse_immediate(Some(imm_str)).unwrap_or(0);
                return Ok((reg, imm));
            } else if let Some(minus) = inner.find('-') {
                let reg_str = inner[..minus].trim();
                let imm_str = inner[minus + 1..].trim();
                let reg = self.parse_register(Some(reg_str), line)?;
                let imm = -self.try_parse_immediate(Some(imm_str)).unwrap_or(0);
                return Ok((reg, imm));
            } else {
                let reg = self.parse_register(Some(inner.trim()), line)?;
                return Ok((reg, 0));
            }
        }

        // offset(reg) format
        if let Some(paren) = combined.find('(') {
            let offset_str = &combined[..paren];
            let reg_str = &combined[paren + 1..combined.len() - 1];
            let reg = self.parse_register(Some(reg_str), line)?;
            let offset = if offset_str.is_empty() {
                0
            } else {
                self.try_parse_immediate(Some(offset_str)).unwrap_or(0)
            };
            return Ok((reg, offset));
        }

        // Simple register
        let reg = self.parse_register(Some(combined), line)?;
        Ok((reg, 0))
    }
}

impl Default for Assembler {
    fn default() -> Self {
        Self::new()
    }
}

/// Disassembler for Neurlang
pub struct Disassembler {
    show_bytes: bool,
    show_offsets: bool,
}

impl Disassembler {
    pub fn new() -> Self {
        Self {
            show_bytes: false,
            show_offsets: true,
        }
    }

    pub fn with_bytes(mut self, show: bool) -> Self {
        self.show_bytes = show;
        self
    }

    pub fn with_offsets(mut self, show: bool) -> Self {
        self.show_offsets = show;
        self
    }

    /// Disassemble a program to text
    pub fn disassemble(&self, program: &Program) -> String {
        let mut output = String::new();
        let mut offset = 0;

        for instr in &program.instructions {
            if self.show_offsets {
                output.push_str(&format!("{:04x}:  ", offset));
            }

            if self.show_bytes {
                let bytes = instr.encode();
                for b in &bytes {
                    output.push_str(&format!("{:02x} ", b));
                }
                // Pad to fixed width
                for _ in bytes.len()..8 {
                    output.push_str("   ");
                }
            }

            output.push_str(&format!("{}\n", instr));
            offset += instr.size();
        }

        output
    }

    /// Disassemble raw bytes
    pub fn disassemble_bytes(&self, bytes: &[u8]) -> String {
        if let Some(program) = Program::decode(bytes) {
            self.disassemble(&program)
        } else {
            String::from("Invalid program format")
        }
    }
}

impl Default for Disassembler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::extensions::ext_ids;

    #[test]
    fn test_assemble_simple() {
        let mut asm = Assembler::new();
        let source = r#"
            mov r0, 42
            add r1, r0, r0
            halt
        "#;

        let program = asm.assemble(source).unwrap();
        assert_eq!(program.instructions.len(), 3);
        assert_eq!(program.instructions[0].opcode, Opcode::Mov);
        assert_eq!(program.instructions[0].imm, Some(42));
    }

    #[test]
    fn test_assemble_with_labels() {
        let mut asm = Assembler::new();
        let source = r#"
        start:
            mov r0, 10
        loop:
            sub r0, r0, r1
            bne r0, zero, loop
            halt
        "#;

        let program = asm.assemble(source).unwrap();
        assert_eq!(program.instructions.len(), 4);
    }

    #[test]
    fn test_assemble_memory_ops() {
        let mut asm = Assembler::new();
        let source = r#"
            load.d r0, [sp]
            load.w r1, [sp + 8]
            store.d r0, [fp - 16]
        "#;

        let program = asm.assemble(source).unwrap();
        assert_eq!(program.instructions.len(), 3);
        assert_eq!(program.instructions[0].opcode, Opcode::Load);
        assert_eq!(program.instructions[1].imm, Some(8));
        assert_eq!(program.instructions[2].imm, Some(-16));
    }

    #[test]
    fn test_roundtrip() {
        let mut asm = Assembler::new();
        let source = r#"
            mov r0, 42
            add r1, r0, r2
            halt
        "#;

        let program = asm.assemble(source).unwrap();
        let bytes = program.encode();
        let decoded = Program::decode(&bytes).unwrap();

        assert_eq!(program.instructions.len(), decoded.instructions.len());
    }

    #[test]
    fn test_disassemble() {
        let mut asm = Assembler::new();
        let source = "mov r0, 42\nhalt";
        let program = asm.assemble(source).unwrap();

        let disasm = Disassembler::new();
        let output = disasm.disassemble(&program);
        assert!(output.contains("mov"));
        assert!(output.contains("halt"));
    }

    #[test]
    fn test_ext_call_numeric_id() {
        let mut asm = Assembler::new();
        let source = r#"
            ext.call r0, 200, r1, r2
            halt
        "#;

        let program = asm.assemble(source).unwrap();
        assert_eq!(program.instructions.len(), 2);
        assert_eq!(program.instructions[0].opcode, Opcode::ExtCall);
        assert_eq!(program.instructions[0].imm, Some(200)); // json_parse
    }

    #[test]
    fn test_ext_call_symbolic_name() {
        let mut asm = Assembler::new();
        let source = r#"
            ext.call r0, json_parse, r1, r2
            ext.call r1, sha256, r2, r3
            halt
        "#;

        let program = asm.assemble(source).unwrap();
        assert_eq!(program.instructions.len(), 3);
        // Uses ext_ids as single source of truth
        assert_eq!(
            program.instructions[0].imm,
            Some(ext_ids::JSON_PARSE as i32)
        ); // json_parse
        assert_eq!(program.instructions[1].imm, Some(ext_ids::SHA256 as i32)); // sha256
    }

    #[test]
    fn test_ext_call_intent_syntax() {
        let mut asm = Assembler::new();
        // Test the new @"description" intent syntax
        let source = r#"
            ext.call r0, @"parse JSON string", r1, r2
            ext.call r1, @"calculate SHA256 hash", r2, r3
            halt
        "#;

        let program = asm.assemble(source).unwrap();
        assert_eq!(program.instructions.len(), 3);
        assert_eq!(program.instructions[0].opcode, Opcode::ExtCall);
        // Uses ext_ids as single source of truth
        assert_eq!(
            program.instructions[0].imm,
            Some(ext_ids::JSON_PARSE as i32)
        ); // json_parse resolved via RAG
        assert_eq!(program.instructions[1].imm, Some(ext_ids::SHA256 as i32)); // sha256 resolved via RAG
    }

    #[test]
    fn test_ext_call_intent_with_no_args() {
        let mut asm = Assembler::new();
        // Test intent syntax with no rs1/rs2 arguments
        let source = r#"
            ext.call r0, @"get current time"
            halt
        "#;

        let program = asm.assemble(source).unwrap();
        assert_eq!(program.instructions.len(), 2);
        assert_eq!(program.instructions[0].opcode, Opcode::ExtCall);
        // Uses ext_ids as single source of truth
        assert_eq!(
            program.instructions[0].imm,
            Some(ext_ids::DATETIME_NOW as i32)
        ); // datetime_now
        assert_eq!(program.instructions[0].rs1, Register::Zero);
        assert_eq!(program.instructions[0].rs2, Register::Zero);
    }

    #[test]
    fn test_ext_call_unknown_intent() {
        let mut asm = Assembler::new();
        // Test that unknown intent returns an error
        let source = r#"
            ext.call r0, @"frobnicator xyzzy magic", r1, r2
            halt
        "#;

        let result = asm.assemble(source);
        assert!(result.is_err());
        if let Err(AsmError::ExtensionNotFound { intent, .. }) = result {
            assert!(intent.contains("frobnicator"));
        } else {
            panic!("Expected ExtensionNotFound error");
        }
    }
}
