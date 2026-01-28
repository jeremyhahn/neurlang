//! Common utilities for code generation
//!
//! Provides shared functionality used by all code generators:
//! - Indentation management
//! - Variable naming conventions
//! - Context tracking during generation

use crate::ir::Register;
use std::collections::HashMap;

/// Options for code generation
#[derive(Debug, Clone)]
pub struct CodeGenOptions {
    /// Indent size (number of spaces)
    pub indent_size: usize,
    /// Use tabs instead of spaces
    pub use_tabs: bool,
    /// Generate comments
    pub emit_comments: bool,
    /// Generate line number references to original IR
    pub emit_line_refs: bool,
    /// Maximum line width before wrapping
    pub max_line_width: usize,
    /// Include safety checks (bounds, null, etc.)
    pub include_safety_checks: bool,
    /// Generate debug assertions
    pub debug_assertions: bool,
}

impl Default for CodeGenOptions {
    fn default() -> Self {
        Self {
            indent_size: 4,
            use_tabs: false,
            emit_comments: true,
            emit_line_refs: false,
            max_line_width: 100,
            include_safety_checks: true,
            debug_assertions: false,
        }
    }
}

/// Context for code generation
#[derive(Debug)]
pub struct CodeGenContext {
    /// Current function name being generated
    pub current_function: Option<String>,
    /// Label to instruction index mapping
    pub labels: HashMap<usize, String>,
    /// Instructions that are branch targets
    pub branch_targets: Vec<usize>,
    /// Current instruction index
    pub current_index: usize,
    /// Total instruction count
    pub instruction_count: usize,
    /// Whether we're inside a loop
    pub in_loop: bool,
    /// Stack of loop labels for break/continue
    pub loop_stack: Vec<String>,
}

impl CodeGenContext {
    pub fn new(instruction_count: usize) -> Self {
        Self {
            current_function: None,
            labels: HashMap::new(),
            branch_targets: Vec::new(),
            current_index: 0,
            instruction_count,
            in_loop: false,
            loop_stack: Vec::new(),
        }
    }

    /// Generate a label name for an instruction index
    pub fn label_for(&mut self, index: usize) -> String {
        if let Some(label) = self.labels.get(&index) {
            label.clone()
        } else {
            let label = format!("L{}", index);
            self.labels.insert(index, label.clone());
            label
        }
    }

    /// Check if an instruction index is a branch target
    pub fn is_branch_target(&self, index: usize) -> bool {
        self.branch_targets.contains(&index)
    }

    /// Mark an instruction as a branch target
    pub fn mark_branch_target(&mut self, index: usize) {
        if !self.branch_targets.contains(&index) {
            self.branch_targets.push(index);
        }
    }
}

/// Helper for managing indentation in generated code
#[derive(Debug)]
pub struct IndentWriter {
    output: String,
    indent_level: usize,
    options: CodeGenOptions,
    at_line_start: bool,
}

impl IndentWriter {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
            options: CodeGenOptions::default(),
            at_line_start: true,
        }
    }

    pub fn with_options(options: CodeGenOptions) -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
            options,
            at_line_start: true,
        }
    }

    /// Get the current output
    pub fn output(&self) -> &str {
        &self.output
    }

    /// Take ownership of the output
    pub fn into_output(self) -> String {
        self.output
    }

    /// Increase indentation level
    pub fn indent(&mut self) {
        self.indent_level += 1;
    }

    /// Decrease indentation level
    pub fn dedent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    /// Write raw string without indentation
    pub fn write_raw(&mut self, s: &str) {
        self.output.push_str(s);
        self.at_line_start = s.ends_with('\n');
    }

    /// Write a string with current indentation
    pub fn write(&mut self, s: &str) {
        if self.at_line_start && !s.is_empty() && !s.starts_with('\n') {
            self.write_indent();
            self.at_line_start = false;
        }
        self.output.push_str(s);
        if s.ends_with('\n') {
            self.at_line_start = true;
        }
    }

    /// Write a line with current indentation
    pub fn writeln(&mut self, s: &str) {
        self.write(s);
        self.output.push('\n');
        self.at_line_start = true;
    }

    /// Write an empty line
    pub fn newline(&mut self) {
        self.output.push('\n');
        self.at_line_start = true;
    }

    /// Write the current indentation
    fn write_indent(&mut self) {
        let indent = if self.options.use_tabs {
            "\t".repeat(self.indent_level)
        } else {
            " ".repeat(self.indent_level * self.options.indent_size)
        };
        self.output.push_str(&indent);
    }

    /// Write a comment (language-specific prefix should be included)
    pub fn write_comment(&mut self, comment: &str) {
        if self.options.emit_comments {
            self.writeln(comment);
        }
    }

    /// Write a block with braces
    pub fn write_block<F>(&mut self, header: &str, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.writeln(header);
        self.indent();
        f(self);
        self.dedent();
    }
}

impl Default for IndentWriter {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a register to a variable name
pub fn register_name(reg: Register) -> &'static str {
    match reg {
        Register::R0 => "r0",
        Register::R1 => "r1",
        Register::R2 => "r2",
        Register::R3 => "r3",
        Register::R4 => "r4",
        Register::R5 => "r5",
        Register::R6 => "r6",
        Register::R7 => "r7",
        Register::R8 => "r8",
        Register::R9 => "r9",
        Register::R10 => "r10",
        Register::R11 => "r11",
        Register::R12 => "r12",
        Register::R13 => "r13",
        Register::R14 => "r14",
        Register::R15 => "r15",
        Register::Sp => "sp",
        Register::Fp => "fp",
        Register::Lr => "lr",
        Register::Pc => "pc",
        Register::Csp => "csp",
        Register::Cfp => "cfp",
        Register::Zero => "zero",
        _ => "reserved",
    }
}

/// Convert a register to array index access (for C-style)
pub fn register_index(reg: Register) -> usize {
    reg as usize
}

/// Get the C type for a memory width
pub fn c_type_for_width(width: MemWidth) -> &'static str {
    use crate::ir::MemWidth;
    match width {
        MemWidth::Byte => "uint8_t",
        MemWidth::Half => "uint16_t",
        MemWidth::Word => "uint32_t",
        MemWidth::Double => "uint64_t",
    }
}

/// Get the Go type for a memory width
pub fn go_type_for_width(width: MemWidth) -> &'static str {
    use crate::ir::MemWidth;
    match width {
        MemWidth::Byte => "uint8",
        MemWidth::Half => "uint16",
        MemWidth::Word => "uint32",
        MemWidth::Double => "uint64",
    }
}

/// Get the Rust type for a memory width
pub fn rust_type_for_width(width: MemWidth) -> &'static str {
    use crate::ir::MemWidth;
    match width {
        MemWidth::Byte => "u8",
        MemWidth::Half => "u16",
        MemWidth::Word => "u32",
        MemWidth::Double => "u64",
    }
}

use crate::ir::MemWidth;

/// Format a number as hex if it's large
pub fn format_number(n: i32) -> String {
    if n.abs() > 255 {
        format!("0x{:x}", n)
    } else {
        n.to_string()
    }
}

/// Analyze a program to find branch targets
pub fn analyze_branch_targets(program: &crate::ir::Program) -> Vec<usize> {
    use crate::ir::Opcode;

    let mut targets = Vec::new();

    for (index, instr) in program.instructions.iter().enumerate() {
        match instr.opcode {
            Opcode::Branch | Opcode::Call | Opcode::Jump => {
                if let Some(offset) = instr.imm {
                    let target = (index as i32 + offset) as usize;
                    if target < program.instructions.len() && !targets.contains(&target) {
                        targets.push(target);
                    }
                }
            }
            Opcode::Spawn => {
                if let Some(offset) = instr.imm {
                    let target = (index as i32 + offset) as usize;
                    if target < program.instructions.len() && !targets.contains(&target) {
                        targets.push(target);
                    }
                }
            }
            _ => {}
        }
    }

    targets.sort();
    targets
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indent_writer() {
        let mut w = IndentWriter::new();
        w.writeln("function() {");
        w.indent();
        w.writeln("statement;");
        w.dedent();
        w.writeln("}");

        let output = w.into_output();
        assert!(output.contains("    statement;"));
    }

    #[test]
    fn test_register_name() {
        assert_eq!(register_name(Register::R0), "r0");
        assert_eq!(register_name(Register::Sp), "sp");
        assert_eq!(register_name(Register::Zero), "zero");
    }
}
