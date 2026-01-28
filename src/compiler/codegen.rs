//! Neurlang IR Code Generator
//!
//! Generates Neurlang assembly from analyzed Rust functions.

use super::analyzer::{AnalyzedExpr, AnalyzedExprKind, AnalyzedFunction, AnalyzedStmt};
use super::parser::NeurlangMetadata;
use super::parser::{BinaryOp, TypeInfo, UnaryOp};
use super::{CompilerConfig, CompilerError};
use crate::ir::{
    AluOp, BitsOp, BranchCond, FpuOp, Instruction, MemWidth, MulDivOp, Opcode, Register,
};

/// Generated function with IR instructions and metadata.
#[derive(Debug)]
pub struct GeneratedFunction {
    pub name: String,
    pub instructions: Vec<GeneratedInstr>,
    pub tests: Vec<super::test_gen::TestCase>,
    pub doc_comment: Option<String>,
    pub category: Option<String>,
    pub neurlang_meta: NeurlangMetadata,
}

/// A generated instruction with optional label and comment.
#[derive(Debug)]
pub struct GeneratedInstr {
    pub label: Option<String>,
    pub instr: Instruction,
    pub comment: Option<String>,
    /// For branch instructions, the target label name
    pub branch_target: Option<String>,
}

impl GeneratedInstr {
    fn new(instr: Instruction) -> Self {
        Self {
            label: None,
            instr,
            comment: None,
            branch_target: None,
        }
    }

    fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    fn with_comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    fn with_branch_target(mut self, target: impl Into<String>) -> Self {
        self.branch_target = Some(target.into());
        self
    }
}

impl GeneratedFunction {
    /// Convert to Neurlang assembly source code.
    pub fn to_nl_source(&self, config: &CompilerConfig) -> String {
        let mut output = String::new();

        // Header comment
        output.push_str(&format!("; @name: {}\n", to_title_case(&self.name)));

        if let Some(doc) = &self.doc_comment {
            // Extract first line as description
            if let Some(first_line) = doc.lines().next() {
                output.push_str(&format!("; @description: {}\n", first_line.trim()));
            }
        }

        // Category - prefer metadata, fall back to config
        let category = self
            .neurlang_meta
            .category
            .as_ref()
            .or(self.category.as_ref());
        if let Some(cat) = category {
            output.push_str(&format!("; @category: {}\n", cat));
        }

        // Difficulty
        if let Some(diff) = self.neurlang_meta.difficulty {
            output.push_str(&format!("; @difficulty: {}\n", diff));
        }

        output.push_str(";\n");

        // Prompt annotations
        if !self.neurlang_meta.prompts.is_empty() {
            for prompt in &self.neurlang_meta.prompts {
                output.push_str(&format!("; @prompt: {}\n", prompt));
            }
            output.push_str(";\n");
        }

        // Parameter documentation
        if !self.neurlang_meta.param_docs.is_empty() {
            for param in &self.neurlang_meta.param_docs {
                output.push_str(&format!(
                    "; @param: {}={} \"{}\"\n",
                    param.name, param.register, param.description
                ));
            }
            output.push_str(";\n");
        }

        // Test annotations
        for test in &self.tests {
            output.push_str(&format!(
                "; @test: {} -> {}\n",
                test.inputs_str(),
                test.outputs_str()
            ));
        }

        output.push_str(";\n");

        output.push_str(&format!("; @export: {}\n", self.name));

        if config.include_comments {
            output.push_str("; Generated from Rust by nl stdlib build\n");
        }

        output.push('\n');

        // Instructions
        for gen_instr in &self.instructions {
            if let Some(label) = &gen_instr.label {
                output.push_str(&format!(".{}:\n", label));
            }

            let instr_str =
                instr_to_string_with_target(&gen_instr.instr, gen_instr.branch_target.as_deref());
            output.push_str(&format!("    {}", instr_str));

            if config.include_comments {
                if let Some(comment) = &gen_instr.comment {
                    output.push_str(&format!("  ; {}", comment));
                }
            }

            output.push('\n');
        }

        output
    }
}

/// Convert instruction to assembly string, with optional branch target label.
fn instr_to_string_with_target(instr: &Instruction, branch_target: Option<&str>) -> String {
    // For branch instructions with a target label, use the label instead of numeric offset
    if let Some(target) = branch_target {
        match instr.opcode {
            Opcode::Branch => {
                let cond = BranchCond::from_u8(instr.mode).unwrap_or(BranchCond::Always);
                let rs1 = reg_to_string(instr.rs1); // First comparison register
                let rs2 = reg_to_string(instr.rs2); // Second comparison register
                match cond {
                    BranchCond::Always => format!("b .{}", target),
                    BranchCond::Eq => format!("beq {}, {}, .{}", rs1, rs2, target),
                    BranchCond::Ne => format!("bne {}, {}, .{}", rs1, rs2, target),
                    BranchCond::Lt => format!("blt {}, {}, .{}", rs1, rs2, target),
                    BranchCond::Le => format!("ble {}, {}, .{}", rs1, rs2, target),
                    BranchCond::Gt => format!("bgt {}, {}, .{}", rs1, rs2, target),
                    BranchCond::Ge => format!("bge {}, {}, .{}", rs1, rs2, target),
                    BranchCond::Ltu => format!("bltu {}, {}, .{}", rs1, rs2, target),
                }
            }
            _ => format!("{}", instr),
        }
    } else {
        format!("{}", instr)
    }
}

/// Convert a Register to its assembly string representation.
fn reg_to_string(reg: Register) -> &'static str {
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
        _ => "r0", // Reserved registers default to r0
    }
}

/// Convert snake_case to Title Case.
fn to_title_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The code generator.
pub struct CodeGenerator {
    /// Current label counter for generating unique labels
    label_counter: usize,
    /// Stack of loop labels for break/continue
    loop_labels: Vec<(String, String)>, // (start_label, end_label)
}

impl CodeGenerator {
    /// Create a new code generator.
    pub fn new() -> Self {
        Self {
            label_counter: 0,
            loop_labels: Vec::new(),
        }
    }

    /// Check if an expression references a specific register (through a variable).
    fn expr_references_reg(&self, expr: &AnalyzedExpr, reg: u8) -> bool {
        match &expr.kind {
            AnalyzedExprKind::Var { register, .. } => *register == reg,
            AnalyzedExprKind::Binary { left, right, .. } => {
                self.expr_references_reg(left, reg) || self.expr_references_reg(right, reg)
            }
            AnalyzedExprKind::Unary { expr: operand, .. } => self.expr_references_reg(operand, reg),
            AnalyzedExprKind::Call { args, .. } => {
                args.iter().any(|a| self.expr_references_reg(a, reg))
            }
            AnalyzedExprKind::MethodCall { receiver, args, .. } => {
                self.expr_references_reg(receiver, reg)
                    || args.iter().any(|a| self.expr_references_reg(a, reg))
            }
            AnalyzedExprKind::Cast { expr: inner, .. } => self.expr_references_reg(inner, reg),
            AnalyzedExprKind::Deref(ptr) => self.expr_references_reg(ptr, reg),
            AnalyzedExprKind::Load { base, offset, .. } => {
                self.expr_references_reg(base, reg) || self.expr_references_reg(offset, reg)
            }
            _ => false,
        }
    }

    /// Generate code for an analyzed function.
    pub fn generate(
        &mut self,
        func: &AnalyzedFunction,
        config: &CompilerConfig,
    ) -> Result<GeneratedFunction, CompilerError> {
        self.label_counter = 0;
        self.loop_labels.clear();

        let mut instructions = Vec::new();

        // Entry label
        let entry_instr = Instruction::new(
            Opcode::Nop,
            Register::Zero,
            Register::Zero,
            Register::Zero,
            0,
        );
        instructions.push(GeneratedInstr::new(entry_instr).with_label("entry"));

        // Generate code for body
        for stmt in &func.body {
            self.gen_stmt(stmt, &mut instructions)?;
        }

        // Ensure function ends with return
        if instructions
            .last()
            .map(|i| i.instr.opcode != Opcode::Halt)
            .unwrap_or(true)
        {
            instructions.push(GeneratedInstr::new(Instruction::new(
                Opcode::Halt,
                Register::Zero,
                Register::Zero,
                Register::Zero,
                0,
            )));
        }

        // Generate test cases from doc comments
        let tests = super::test_gen::TestGenerator::generate_from_doc(
            &func.name,
            func.doc_comment.as_deref(),
            &func.params,
        );

        Ok(GeneratedFunction {
            name: func.name.clone(),
            instructions,
            tests,
            doc_comment: func.doc_comment.clone(),
            category: config.category.clone(),
            neurlang_meta: func.neurlang_meta.clone(),
        })
    }

    /// Generate a unique label.
    fn gen_label(&mut self, prefix: &str) -> String {
        let label = format!("{}_{}", prefix, self.label_counter);
        self.label_counter += 1;
        label
    }

    /// Generate code for a statement.
    fn gen_stmt(
        &mut self,
        stmt: &AnalyzedStmt,
        instrs: &mut Vec<GeneratedInstr>,
    ) -> Result<(), CompilerError> {
        match stmt {
            AnalyzedStmt::Let { var, init } => {
                if let Some(init_expr) = init {
                    // Generate expression into the variable's register
                    self.gen_expr(init_expr, var.register, instrs)?;
                } else {
                    // Initialize to zero
                    instrs.push(
                        GeneratedInstr::new(Instruction::with_imm(
                            Opcode::Mov,
                            Register::from_u8(var.register).unwrap(),
                            Register::Zero,
                            0,
                            0,
                        ))
                        .with_comment(format!("{} = 0", var.name)),
                    );
                }
            }

            AnalyzedStmt::Assign { target, value } => {
                match &target.kind {
                    AnalyzedExprKind::Var { register, name: _ } => {
                        // Direct variable assignment
                        self.gen_expr(value, *register, instrs)?;
                    }
                    AnalyzedExprKind::Deref(ptr_expr) => {
                        // Store through pointer
                        // Generate pointer into temp register
                        let ptr_reg = 15u8; // Use r15 as temp
                        self.gen_expr(ptr_expr, ptr_reg, instrs)?;
                        // Generate value into another temp
                        let val_reg = 14u8;
                        self.gen_expr(value, val_reg, instrs)?;
                        // Store
                        let width = type_to_mem_width(&target.ty);
                        instrs.push(GeneratedInstr::new(Instruction::with_imm(
                            Opcode::Store,
                            Register::from_u8(val_reg).unwrap(),
                            Register::from_u8(ptr_reg).unwrap(),
                            width as u8,
                            0,
                        )));
                    }
                    AnalyzedExprKind::Load { base, offset } => {
                        // Store through indexed pointer
                        let base_reg = 15u8;
                        self.gen_expr(base, base_reg, instrs)?;
                        let offset_reg = 14u8;
                        self.gen_expr(offset, offset_reg, instrs)?;
                        let val_reg = 13u8;
                        self.gen_expr(value, val_reg, instrs)?;

                        // Calculate address: base + offset * element_size
                        let elem_size = type_size(&target.ty);
                        if elem_size > 1 {
                            // Multiply offset by element size
                            instrs.push(GeneratedInstr::new(Instruction::with_imm(
                                Opcode::AluI,
                                Register::from_u8(offset_reg).unwrap(),
                                Register::from_u8(offset_reg).unwrap(),
                                AluOp::Shl as u8,
                                elem_size.trailing_zeros() as i32,
                            )));
                        }
                        // Add to base
                        instrs.push(GeneratedInstr::new(Instruction::new(
                            Opcode::Alu,
                            Register::from_u8(base_reg).unwrap(),
                            Register::from_u8(base_reg).unwrap(),
                            Register::from_u8(offset_reg).unwrap(),
                            AluOp::Add as u8,
                        )));
                        // Store
                        let width = type_to_mem_width(&target.ty);
                        instrs.push(GeneratedInstr::new(Instruction::with_imm(
                            Opcode::Store,
                            Register::from_u8(val_reg).unwrap(),
                            Register::from_u8(base_reg).unwrap(),
                            width as u8,
                            0,
                        )));
                    }
                    _ => {
                        return Err(CompilerError::Unsupported(
                            "Complex assignment target".to_string(),
                        ));
                    }
                }
            }

            AnalyzedStmt::Expr(expr) => {
                // Evaluate expression for side effects
                self.gen_expr(expr, 0, instrs)?;
            }

            AnalyzedStmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let else_label = self.gen_label("else");
                let end_label = self.gen_label("endif");

                // Evaluate condition into temp register
                let cond_reg = 15u8;
                self.gen_expr(condition, cond_reg, instrs)?;

                // Branch to else if condition is false
                let branch_target = if else_branch.is_some() {
                    else_label.clone()
                } else {
                    end_label.clone()
                };
                instrs.push(
                    GeneratedInstr::new(Instruction::branch(
                        BranchCond::Eq,
                        Register::from_u8(cond_reg).unwrap(),
                        Register::Zero,
                        0,
                    ))
                    .with_branch_target(&branch_target),
                );

                // Then branch
                for s in then_branch {
                    self.gen_stmt(s, instrs)?;
                }

                if let Some(else_stmts) = else_branch {
                    // Jump over else
                    instrs.push(
                        GeneratedInstr::new(Instruction::branch(
                            BranchCond::Always,
                            Register::Zero,
                            Register::Zero,
                            0,
                        ))
                        .with_branch_target(&end_label),
                    );

                    // Else label
                    instrs.push(
                        GeneratedInstr::new(Instruction::new(
                            Opcode::Nop,
                            Register::Zero,
                            Register::Zero,
                            Register::Zero,
                            0,
                        ))
                        .with_label(&else_label),
                    );

                    for s in else_stmts {
                        self.gen_stmt(s, instrs)?;
                    }
                }

                // End label
                instrs.push(
                    GeneratedInstr::new(Instruction::new(
                        Opcode::Nop,
                        Register::Zero,
                        Register::Zero,
                        Register::Zero,
                        0,
                    ))
                    .with_label(&end_label),
                );
            }

            AnalyzedStmt::While { condition, body } => {
                let loop_start = self.gen_label("while");
                let loop_end = self.gen_label("endwhile");

                self.loop_labels
                    .push((loop_start.clone(), loop_end.clone()));

                // Loop start label
                instrs.push(
                    GeneratedInstr::new(Instruction::new(
                        Opcode::Nop,
                        Register::Zero,
                        Register::Zero,
                        Register::Zero,
                        0,
                    ))
                    .with_label(&loop_start),
                );

                // Evaluate condition
                let cond_reg = 15u8;
                self.gen_expr(condition, cond_reg, instrs)?;

                // Exit if false
                instrs.push(
                    GeneratedInstr::new(Instruction::branch(
                        BranchCond::Eq,
                        Register::from_u8(cond_reg).unwrap(),
                        Register::Zero,
                        0,
                    ))
                    .with_branch_target(&loop_end),
                );

                // Loop body
                for s in body {
                    self.gen_stmt(s, instrs)?;
                }

                // Jump back to start
                instrs.push(
                    GeneratedInstr::new(Instruction::branch(
                        BranchCond::Always,
                        Register::Zero,
                        Register::Zero,
                        0,
                    ))
                    .with_branch_target(&loop_start),
                );

                // End label
                instrs.push(
                    GeneratedInstr::new(Instruction::new(
                        Opcode::Nop,
                        Register::Zero,
                        Register::Zero,
                        Register::Zero,
                        0,
                    ))
                    .with_label(&loop_end),
                );

                self.loop_labels.pop();
            }

            AnalyzedStmt::Loop { body } => {
                let loop_start = self.gen_label("loop");
                let loop_end = self.gen_label("endloop");

                self.loop_labels
                    .push((loop_start.clone(), loop_end.clone()));

                // Loop start label
                instrs.push(
                    GeneratedInstr::new(Instruction::new(
                        Opcode::Nop,
                        Register::Zero,
                        Register::Zero,
                        Register::Zero,
                        0,
                    ))
                    .with_label(&loop_start),
                );

                // Loop body
                for s in body {
                    self.gen_stmt(s, instrs)?;
                }

                // Jump back to start
                instrs.push(
                    GeneratedInstr::new(Instruction::branch(
                        BranchCond::Always,
                        Register::Zero,
                        Register::Zero,
                        0,
                    ))
                    .with_branch_target(&loop_start),
                );

                // End label
                instrs.push(
                    GeneratedInstr::new(Instruction::new(
                        Opcode::Nop,
                        Register::Zero,
                        Register::Zero,
                        Register::Zero,
                        0,
                    ))
                    .with_label(&loop_end),
                );

                self.loop_labels.pop();
            }

            AnalyzedStmt::For {
                var,
                start,
                end,
                inclusive,
                body,
            } => {
                // For loop: for i in start..end { body }
                // Generates:
                //   i = start
                // loop_start:
                //   if i >= end (or > for inclusive) goto loop_end
                //   body
                //   i = i + 1
                //   goto loop_start
                // loop_end:

                let loop_start = self.gen_label("for");
                let loop_end = self.gen_label("endfor");
                let loop_var_reg = Register::from_u8(var.register).unwrap_or(Register::R4);

                // Initialize loop variable: i = start
                self.gen_expr(start, var.register, instrs)?;

                self.loop_labels
                    .push((loop_start.clone(), loop_end.clone()));

                // Loop start label
                instrs.push(
                    GeneratedInstr::new(Instruction::new(
                        Opcode::Nop,
                        Register::Zero,
                        Register::Zero,
                        Register::Zero,
                        0,
                    ))
                    .with_label(&loop_start),
                );

                // Generate end value into a temp register (use r14)
                let end_reg = 14u8;
                self.gen_expr(end, end_reg, instrs)?;
                let end_reg_r = Register::from_u8(end_reg).unwrap();

                // Branch condition: if i >= end goto loop_end (or > for inclusive)
                let cond = if *inclusive {
                    BranchCond::Gt // i > end for ..= (exit when past end)
                } else {
                    BranchCond::Ge // i >= end for .. (exit when at end)
                };
                instrs.push(
                    GeneratedInstr::new(Instruction::branch(cond, loop_var_reg, end_reg_r, 0))
                        .with_branch_target(&loop_end),
                );

                // Loop body
                for s in body {
                    self.gen_stmt(s, instrs)?;
                }

                // Increment: i = i + 1
                instrs.push(GeneratedInstr::new(Instruction::with_imm(
                    Opcode::AluI,
                    loop_var_reg,
                    loop_var_reg,
                    AluOp::Add as u8,
                    1,
                )));

                // Jump back to start
                instrs.push(
                    GeneratedInstr::new(Instruction::branch(
                        BranchCond::Always,
                        Register::Zero,
                        Register::Zero,
                        0,
                    ))
                    .with_branch_target(&loop_start),
                );

                // End label
                instrs.push(
                    GeneratedInstr::new(Instruction::new(
                        Opcode::Nop,
                        Register::Zero,
                        Register::Zero,
                        Register::Zero,
                        0,
                    ))
                    .with_label(&loop_end),
                );

                self.loop_labels.pop();
            }

            AnalyzedStmt::Return(expr) => {
                if let Some(e) = expr {
                    // Generate return value into r0
                    self.gen_expr(e, 0, instrs)?;
                }
                instrs.push(GeneratedInstr::new(Instruction::new(
                    Opcode::Halt,
                    Register::Zero,
                    Register::Zero,
                    Register::Zero,
                    0,
                )));
            }

            AnalyzedStmt::Break => {
                if let Some((_, end_label)) = self.loop_labels.last().cloned() {
                    instrs.push(
                        GeneratedInstr::new(Instruction::branch(
                            BranchCond::Always,
                            Register::Zero,
                            Register::Zero,
                            0,
                        ))
                        .with_branch_target(&end_label),
                    );
                }
            }

            AnalyzedStmt::Continue => {
                if let Some((start_label, _)) = self.loop_labels.last().cloned() {
                    instrs.push(
                        GeneratedInstr::new(Instruction::branch(
                            BranchCond::Always,
                            Register::Zero,
                            Register::Zero,
                            0,
                        ))
                        .with_branch_target(&start_label),
                    );
                }
            }
        }

        Ok(())
    }

    /// Generate code for an expression, storing result in dst_reg.
    fn gen_expr(
        &mut self,
        expr: &AnalyzedExpr,
        dst_reg: u8,
        instrs: &mut Vec<GeneratedInstr>,
    ) -> Result<(), CompilerError> {
        let dst = Register::from_u8(dst_reg).unwrap();

        match &expr.kind {
            AnalyzedExprKind::IntLit(n) => {
                instrs.push(
                    GeneratedInstr::new(Instruction::with_imm(
                        Opcode::Mov,
                        dst,
                        Register::Zero,
                        0,
                        *n as i32,
                    ))
                    .with_comment(format!("{}", n)),
                );
            }

            AnalyzedExprKind::FloatLit(f) => {
                // Store f64 bit pattern as immediate
                let bits = f.to_bits();
                // Note: This is a simplification - real implementation would
                // need to handle 64-bit immediates
                instrs.push(
                    GeneratedInstr::new(Instruction::with_imm(
                        Opcode::Mov,
                        dst,
                        Register::Zero,
                        0,
                        bits as i32,
                    ))
                    .with_comment(format!("{}", f)),
                );
            }

            AnalyzedExprKind::BoolLit(b) => {
                instrs.push(GeneratedInstr::new(Instruction::with_imm(
                    Opcode::Mov,
                    dst,
                    Register::Zero,
                    0,
                    if *b { 1 } else { 0 },
                )));
            }

            AnalyzedExprKind::Var { register, name } => {
                if *register != dst_reg {
                    instrs.push(
                        GeneratedInstr::new(Instruction::new(
                            Opcode::Mov,
                            dst,
                            Register::from_u8(*register).unwrap(),
                            Register::Zero,
                            0,
                        ))
                        .with_comment(name.clone()),
                    );
                }
                // If already in dst register, no code needed
            }

            AnalyzedExprKind::Binary { op, left, right } => {
                self.gen_binary_op(*op, left, right, dst_reg, &expr.ty, instrs)?;
            }

            AnalyzedExprKind::Unary { op, expr: inner } => {
                self.gen_expr(inner, dst_reg, instrs)?;

                match op {
                    UnaryOp::Neg => {
                        // dst = 0 - dst
                        instrs.push(GeneratedInstr::new(Instruction::new(
                            Opcode::Alu,
                            dst,
                            Register::Zero,
                            dst,
                            AluOp::Sub as u8,
                        )));
                    }
                    UnaryOp::Not => {
                        // Bitwise not: dst = dst xor -1
                        instrs.push(GeneratedInstr::new(Instruction::with_imm(
                            Opcode::AluI,
                            dst,
                            dst,
                            AluOp::Xor as u8,
                            -1,
                        )));
                    }
                    UnaryOp::Deref => {
                        // Load from pointer
                        let width = type_to_mem_width(&inner.ty);
                        instrs.push(GeneratedInstr::new(Instruction::with_imm(
                            Opcode::Load,
                            dst,
                            dst,
                            width as u8,
                            0,
                        )));
                    }
                }
            }

            AnalyzedExprKind::Call { func, args } => {
                self.gen_call(func, args, dst_reg, instrs)?;
            }

            AnalyzedExprKind::MethodCall {
                receiver,
                method,
                args,
            } => {
                self.gen_method_call(receiver, method, args, dst_reg, instrs)?;
            }

            AnalyzedExprKind::Deref(inner) => {
                // Generate pointer address
                self.gen_expr(inner, dst_reg, instrs)?;
                // Load from pointer
                let width = type_to_mem_width(&expr.ty);
                instrs.push(GeneratedInstr::new(Instruction::with_imm(
                    Opcode::Load,
                    dst,
                    dst,
                    width as u8,
                    0,
                )));
            }

            AnalyzedExprKind::Load { base, offset } => {
                // Generate base address
                self.gen_expr(base, dst_reg, instrs)?;
                // Generate offset
                let offset_reg = if dst_reg == 15 { 14 } else { 15 };
                self.gen_expr(offset, offset_reg, instrs)?;

                // Calculate address with element size
                let elem_size = type_size(&expr.ty);
                if elem_size > 1 {
                    instrs.push(GeneratedInstr::new(Instruction::with_imm(
                        Opcode::AluI,
                        Register::from_u8(offset_reg).unwrap(),
                        Register::from_u8(offset_reg).unwrap(),
                        AluOp::Shl as u8,
                        elem_size.trailing_zeros() as i32,
                    )));
                }

                // Add offset to base
                instrs.push(GeneratedInstr::new(Instruction::new(
                    Opcode::Alu,
                    dst,
                    dst,
                    Register::from_u8(offset_reg).unwrap(),
                    AluOp::Add as u8,
                )));

                // Load
                let width = type_to_mem_width(&expr.ty);
                instrs.push(GeneratedInstr::new(Instruction::with_imm(
                    Opcode::Load,
                    dst,
                    dst,
                    width as u8,
                    0,
                )));
            }

            AnalyzedExprKind::Store {
                base,
                offset,
                value,
            } => {
                // This is for assignment expressions
                self.gen_expr(base, 15, instrs)?;
                self.gen_expr(offset, 14, instrs)?;
                self.gen_expr(value, dst_reg, instrs)?;

                let elem_size = type_size(&expr.ty);
                if elem_size > 1 {
                    instrs.push(GeneratedInstr::new(Instruction::with_imm(
                        Opcode::AluI,
                        Register::R14,
                        Register::R14,
                        AluOp::Shl as u8,
                        elem_size.trailing_zeros() as i32,
                    )));
                }

                instrs.push(GeneratedInstr::new(Instruction::new(
                    Opcode::Alu,
                    Register::R15,
                    Register::R15,
                    Register::R14,
                    AluOp::Add as u8,
                )));

                let width = type_to_mem_width(&expr.ty);
                instrs.push(GeneratedInstr::new(Instruction::with_imm(
                    Opcode::Store,
                    dst,
                    Register::R15,
                    width as u8,
                    0,
                )));
            }

            AnalyzedExprKind::Cast {
                expr: inner,
                target_ty: _,
            } => {
                self.gen_expr(inner, dst_reg, instrs)?;
                // Most casts are no-ops at the IR level
                // Float<->int would need conversion instructions
            }

            AnalyzedExprKind::Block {
                stmts,
                expr: result_expr,
            } => {
                for s in stmts {
                    self.gen_stmt(s, instrs)?;
                }
                if let Some(e) = result_expr {
                    self.gen_expr(e, dst_reg, instrs)?;
                }
            }

            AnalyzedExprKind::Conditional {
                condition,
                then_expr,
                else_expr,
            } => {
                let else_label = self.gen_label("cond_else");
                let end_label = self.gen_label("cond_end");

                // Evaluate condition
                let cond_reg = if dst_reg == 15 { 14 } else { 15 };
                self.gen_expr(condition, cond_reg, instrs)?;

                // Branch to else if false
                instrs.push(
                    GeneratedInstr::new(Instruction::branch(
                        BranchCond::Eq,
                        Register::from_u8(cond_reg).unwrap(),
                        Register::Zero,
                        0,
                    ))
                    .with_branch_target(&else_label),
                );

                // Then value
                self.gen_expr(then_expr, dst_reg, instrs)?;

                // Jump to end
                instrs.push(
                    GeneratedInstr::new(Instruction::branch(
                        BranchCond::Always,
                        Register::Zero,
                        Register::Zero,
                        0,
                    ))
                    .with_branch_target(&end_label),
                );

                // Else label
                instrs.push(
                    GeneratedInstr::new(Instruction::new(
                        Opcode::Nop,
                        Register::Zero,
                        Register::Zero,
                        Register::Zero,
                        0,
                    ))
                    .with_label(&else_label),
                );

                // Else value
                self.gen_expr(else_expr, dst_reg, instrs)?;

                // End label
                instrs.push(
                    GeneratedInstr::new(Instruction::new(
                        Opcode::Nop,
                        Register::Zero,
                        Register::Zero,
                        Register::Zero,
                        0,
                    ))
                    .with_label(&end_label),
                );
            }
        }

        Ok(())
    }

    /// Generate binary operation.
    fn gen_binary_op(
        &mut self,
        op: BinaryOp,
        left: &AnalyzedExpr,
        right: &AnalyzedExpr,
        dst_reg: u8,
        _result_ty: &TypeInfo,
        instrs: &mut Vec<GeneratedInstr>,
    ) -> Result<(), CompilerError> {
        let dst = Register::from_u8(dst_reg).unwrap();

        // Check if this is a floating-point operation
        let is_float = left.ty.is_float() || right.ty.is_float();

        // Choose temp register that doesn't conflict with dst
        let right_reg = match dst_reg {
            15 => 14,
            14 => 15,
            _ => 15,
        };

        // Check if right is a complex expression that might clobber our dst
        let right_is_complex = !matches!(
            &right.kind,
            AnalyzedExprKind::IntLit(_)
                | AnalyzedExprKind::FloatLit(_)
                | AnalyzedExprKind::BoolLit(_)
                | AnalyzedExprKind::Var { .. }
        );

        // Check if right operand references a variable that we'd put in dst
        let right_uses_dst = self.expr_references_reg(right, dst_reg);

        if right_uses_dst || right_is_complex {
            // Evaluate right FIRST into right_reg to avoid clobbering
            // When right is complex, it might use dst_reg as a temp internally
            self.gen_expr(right, right_reg, instrs)?;
            self.gen_expr(left, dst_reg, instrs)?;
        } else {
            // Normal order: left into dst, right into temp
            self.gen_expr(left, dst_reg, instrs)?;
            self.gen_expr(right, right_reg, instrs)?;
        }

        let rs2 = Register::from_u8(right_reg).unwrap();

        if is_float {
            // FPU operations
            let fpu_op = match op {
                BinaryOp::Add => FpuOp::Fadd,
                BinaryOp::Sub => FpuOp::Fsub,
                BinaryOp::Mul => FpuOp::Fmul,
                BinaryOp::Div => FpuOp::Fdiv,
                // Float comparisons
                BinaryOp::Eq => FpuOp::Fcmpeq,
                BinaryOp::Ne => FpuOp::Fcmpne,
                BinaryOp::Lt => FpuOp::Fcmplt,
                BinaryOp::Le => FpuOp::Fcmple,
                BinaryOp::Gt => FpuOp::Fcmpgt,
                BinaryOp::Ge => FpuOp::Fcmpge,
                _ => {
                    return Err(CompilerError::Unsupported(format!(
                        "Float operation: {:?}",
                        op
                    )))
                }
            };
            instrs.push(GeneratedInstr::new(Instruction::new(
                Opcode::Fpu,
                dst,
                dst,
                rs2,
                fpu_op as u8,
            )));
        } else {
            // Integer operations
            match op {
                BinaryOp::Add => {
                    instrs.push(GeneratedInstr::new(Instruction::new(
                        Opcode::Alu,
                        dst,
                        dst,
                        rs2,
                        AluOp::Add as u8,
                    )));
                }
                BinaryOp::Sub => {
                    instrs.push(GeneratedInstr::new(Instruction::new(
                        Opcode::Alu,
                        dst,
                        dst,
                        rs2,
                        AluOp::Sub as u8,
                    )));
                }
                BinaryOp::Mul => {
                    instrs.push(GeneratedInstr::new(Instruction::new(
                        Opcode::MulDiv,
                        dst,
                        dst,
                        rs2,
                        MulDivOp::Mul as u8,
                    )));
                }
                BinaryOp::Div => {
                    instrs.push(GeneratedInstr::new(Instruction::new(
                        Opcode::MulDiv,
                        dst,
                        dst,
                        rs2,
                        MulDivOp::Div as u8,
                    )));
                }
                BinaryOp::Rem => {
                    instrs.push(GeneratedInstr::new(Instruction::new(
                        Opcode::MulDiv,
                        dst,
                        dst,
                        rs2,
                        MulDivOp::Mod as u8,
                    )));
                }
                BinaryOp::And => {
                    instrs.push(GeneratedInstr::new(Instruction::new(
                        Opcode::Alu,
                        dst,
                        dst,
                        rs2,
                        AluOp::And as u8,
                    )));
                }
                BinaryOp::Or => {
                    instrs.push(GeneratedInstr::new(Instruction::new(
                        Opcode::Alu,
                        dst,
                        dst,
                        rs2,
                        AluOp::Or as u8,
                    )));
                }
                BinaryOp::Xor => {
                    instrs.push(GeneratedInstr::new(Instruction::new(
                        Opcode::Alu,
                        dst,
                        dst,
                        rs2,
                        AluOp::Xor as u8,
                    )));
                }
                BinaryOp::Shl => {
                    instrs.push(GeneratedInstr::new(Instruction::new(
                        Opcode::Alu,
                        dst,
                        dst,
                        rs2,
                        AluOp::Shl as u8,
                    )));
                }
                BinaryOp::Shr => {
                    instrs.push(GeneratedInstr::new(Instruction::new(
                        Opcode::Alu,
                        dst,
                        dst,
                        rs2,
                        AluOp::Shr as u8,
                    )));
                }
                // Comparisons - result is 1 or 0
                BinaryOp::Eq
                | BinaryOp::Ne
                | BinaryOp::Lt
                | BinaryOp::Le
                | BinaryOp::Gt
                | BinaryOp::Ge => {
                    self.gen_comparison(op, dst, rs2, instrs)?;
                }
                BinaryOp::LogicalAnd => {
                    // Both must be non-zero
                    // Implement as: (left != 0) & (right != 0)
                    self.gen_comparison(BinaryOp::Ne, dst, Register::Zero, instrs)?;
                    let temp = Register::R13;
                    instrs.push(GeneratedInstr::new(Instruction::new(
                        Opcode::Mov,
                        temp,
                        rs2,
                        Register::Zero,
                        0,
                    )));
                    self.gen_comparison(BinaryOp::Ne, temp, Register::Zero, instrs)?;
                    instrs.push(GeneratedInstr::new(Instruction::new(
                        Opcode::Alu,
                        dst,
                        dst,
                        temp,
                        AluOp::And as u8,
                    )));
                }
                BinaryOp::LogicalOr => {
                    // Either must be non-zero
                    instrs.push(GeneratedInstr::new(Instruction::new(
                        Opcode::Alu,
                        dst,
                        dst,
                        rs2,
                        AluOp::Or as u8,
                    )));
                    self.gen_comparison(BinaryOp::Ne, dst, Register::Zero, instrs)?;
                }
            }
        }

        Ok(())
    }

    /// Generate comparison operation (result is 0 or 1).
    fn gen_comparison(
        &mut self,
        op: BinaryOp,
        dst: Register,
        rs2: Register,
        instrs: &mut Vec<GeneratedInstr>,
    ) -> Result<(), CompilerError> {
        // Comparisons are tricky - we need to convert branch conditions to values
        // Approach: subtract, then use sign/zero

        let set_label = self.gen_label("set");
        let end_label = self.gen_label("cmp_end");

        let cond = match op {
            BinaryOp::Eq => BranchCond::Eq,
            BinaryOp::Ne => BranchCond::Ne,
            BinaryOp::Lt => BranchCond::Lt,
            BinaryOp::Le => BranchCond::Le,
            BinaryOp::Gt => BranchCond::Gt,
            BinaryOp::Ge => BranchCond::Ge,
            _ => return Err(CompilerError::Unsupported(format!("Comparison: {:?}", op))),
        };

        // Branch if condition is true
        instrs.push(
            GeneratedInstr::new(Instruction::branch(cond, dst, rs2, 0))
                .with_branch_target(&set_label),
        );

        // Condition false: set dst = 0
        instrs.push(GeneratedInstr::new(Instruction::with_imm(
            Opcode::Mov,
            dst,
            Register::Zero,
            0,
            0,
        )));
        instrs.push(
            GeneratedInstr::new(Instruction::branch(
                BranchCond::Always,
                Register::Zero,
                Register::Zero,
                0,
            ))
            .with_branch_target(&end_label),
        );

        // Condition true: set dst = 1
        instrs.push(
            GeneratedInstr::new(Instruction::new(
                Opcode::Nop,
                Register::Zero,
                Register::Zero,
                Register::Zero,
                0,
            ))
            .with_label(&set_label),
        );
        instrs.push(GeneratedInstr::new(Instruction::with_imm(
            Opcode::Mov,
            dst,
            Register::Zero,
            0,
            1,
        )));

        // End
        instrs.push(
            GeneratedInstr::new(Instruction::new(
                Opcode::Nop,
                Register::Zero,
                Register::Zero,
                Register::Zero,
                0,
            ))
            .with_label(&end_label),
        );

        Ok(())
    }

    /// Generate function call.
    fn gen_call(
        &mut self,
        func: &str,
        args: &[AnalyzedExpr],
        dst_reg: u8,
        instrs: &mut Vec<GeneratedInstr>,
    ) -> Result<(), CompilerError> {
        let dst = Register::from_u8(dst_reg).unwrap();

        // Check for stdlib intrinsics that map directly to IR instructions
        match func {
            // Bitwise operations -> bits.X instructions
            "popcount" => {
                self.gen_expr(&args[0], dst_reg, instrs)?;
                instrs.push(
                    GeneratedInstr::new(Instruction::new(
                        Opcode::Bits,
                        dst,
                        dst,
                        Register::Zero,
                        BitsOp::Popcount as u8,
                    ))
                    .with_comment("popcount"),
                );
                return Ok(());
            }
            "clz" => {
                self.gen_expr(&args[0], dst_reg, instrs)?;
                instrs.push(
                    GeneratedInstr::new(Instruction::new(
                        Opcode::Bits,
                        dst,
                        dst,
                        Register::Zero,
                        BitsOp::Clz as u8,
                    ))
                    .with_comment("clz"),
                );
                return Ok(());
            }
            "ctz" => {
                self.gen_expr(&args[0], dst_reg, instrs)?;
                instrs.push(
                    GeneratedInstr::new(Instruction::new(
                        Opcode::Bits,
                        dst,
                        dst,
                        Register::Zero,
                        BitsOp::Ctz as u8,
                    ))
                    .with_comment("ctz"),
                );
                return Ok(());
            }
            "bswap" => {
                self.gen_expr(&args[0], dst_reg, instrs)?;
                instrs.push(
                    GeneratedInstr::new(Instruction::new(
                        Opcode::Bits,
                        dst,
                        dst,
                        Register::Zero,
                        BitsOp::Bswap as u8,
                    ))
                    .with_comment("bswap"),
                );
                return Ok(());
            }
            // f64 bit conversion - no-op since registers are 64-bit
            "f64 :: from_bits" | "from_bits" => {
                // Just evaluate the argument into dst - bits are already the value
                self.gen_expr(&args[0], dst_reg, instrs)?;
                return Ok(());
            }
            _ => {}
        }

        // Place arguments in r0-r3
        for (i, arg) in args.iter().take(4).enumerate() {
            self.gen_expr(arg, i as u8, instrs)?;
        }

        // Generate call instruction
        instrs.push(
            GeneratedInstr::new(Instruction::with_imm(
                Opcode::Call,
                Register::Zero,
                Register::Zero,
                0,
                0, // Address will be resolved by assembler/linker
            ))
            .with_comment(format!("call {}", func)),
        );

        // Move result to dst if needed
        if dst_reg != 0 {
            instrs.push(GeneratedInstr::new(Instruction::new(
                Opcode::Mov,
                Register::from_u8(dst_reg).unwrap(),
                Register::R0,
                Register::Zero,
                0,
            )));
        }

        Ok(())
    }

    /// Generate method call.
    fn gen_method_call(
        &mut self,
        receiver: &AnalyzedExpr,
        method: &str,
        args: &[AnalyzedExpr],
        dst_reg: u8,
        instrs: &mut Vec<GeneratedInstr>,
    ) -> Result<(), CompilerError> {
        let dst = Register::from_u8(dst_reg).unwrap();

        // Handle common methods inline
        match method {
            "add" | "offset" => {
                // Pointer arithmetic: ptr.add(n) or ptr.offset(n)
                self.gen_expr(receiver, dst_reg, instrs)?;
                if !args.is_empty() {
                    let offset_reg = if dst_reg == 15 { 14 } else { 15 };
                    self.gen_expr(&args[0], offset_reg, instrs)?;

                    // Get element size from pointer type
                    let elem_size = match &receiver.ty {
                        TypeInfo::Ptr(inner) | TypeInfo::MutPtr(inner) => type_size(inner),
                        _ => 8, // Default to u64 size
                    };

                    // Only multiply by element size if > 1 byte
                    if elem_size > 1 {
                        let shift = elem_size.trailing_zeros();
                        instrs.push(GeneratedInstr::new(Instruction::with_imm(
                            Opcode::AluI,
                            Register::from_u8(offset_reg).unwrap(),
                            Register::from_u8(offset_reg).unwrap(),
                            AluOp::Shl as u8,
                            shift as i32,
                        )));
                    }

                    instrs.push(GeneratedInstr::new(Instruction::new(
                        Opcode::Alu,
                        dst,
                        dst,
                        Register::from_u8(offset_reg).unwrap(),
                        AluOp::Add as u8,
                    )));
                }
            }
            "wrapping_add" | "saturating_add" => {
                self.gen_expr(receiver, dst_reg, instrs)?;
                if !args.is_empty() {
                    let rhs_reg = if dst_reg == 15 { 14 } else { 15 };
                    self.gen_expr(&args[0], rhs_reg, instrs)?;
                    instrs.push(GeneratedInstr::new(Instruction::new(
                        Opcode::Alu,
                        dst,
                        dst,
                        Register::from_u8(rhs_reg).unwrap(),
                        AluOp::Add as u8,
                    )));
                }
            }
            "wrapping_sub" | "saturating_sub" => {
                self.gen_expr(receiver, dst_reg, instrs)?;
                if !args.is_empty() {
                    let rhs_reg = if dst_reg == 15 { 14 } else { 15 };
                    self.gen_expr(&args[0], rhs_reg, instrs)?;
                    instrs.push(GeneratedInstr::new(Instruction::new(
                        Opcode::Alu,
                        dst,
                        dst,
                        Register::from_u8(rhs_reg).unwrap(),
                        AluOp::Sub as u8,
                    )));
                }
            }
            "wrapping_mul" => {
                self.gen_expr(receiver, dst_reg, instrs)?;
                if !args.is_empty() {
                    let rhs_reg = if dst_reg == 15 { 14 } else { 15 };
                    self.gen_expr(&args[0], rhs_reg, instrs)?;
                    instrs.push(GeneratedInstr::new(Instruction::new(
                        Opcode::MulDiv,
                        dst,
                        dst,
                        Register::from_u8(rhs_reg).unwrap(),
                        MulDivOp::Mul as u8,
                    )));
                }
            }
            "sqrt" => {
                self.gen_expr(receiver, dst_reg, instrs)?;
                instrs.push(GeneratedInstr::new(Instruction::new(
                    Opcode::Fpu,
                    dst,
                    dst,
                    Register::Zero,
                    FpuOp::Fsqrt as u8,
                )));
            }
            "abs" => {
                self.gen_expr(receiver, dst_reg, instrs)?;
                if receiver.ty.is_float() {
                    instrs.push(GeneratedInstr::new(Instruction::new(
                        Opcode::Fpu,
                        dst,
                        dst,
                        Register::Zero,
                        FpuOp::Fabs as u8,
                    )));
                }
            }
            "floor" => {
                self.gen_expr(receiver, dst_reg, instrs)?;
                instrs.push(GeneratedInstr::new(Instruction::new(
                    Opcode::Fpu,
                    dst,
                    dst,
                    Register::Zero,
                    FpuOp::Ffloor as u8,
                )));
            }
            "ceil" => {
                self.gen_expr(receiver, dst_reg, instrs)?;
                instrs.push(GeneratedInstr::new(Instruction::new(
                    Opcode::Fpu,
                    dst,
                    dst,
                    Register::Zero,
                    FpuOp::Fceil as u8,
                )));
            }
            "round" => {
                // round is floor(x + 0.5) for positive
                self.gen_expr(receiver, dst_reg, instrs)?;
                // TODO: proper rounding implementation
                instrs.push(
                    GeneratedInstr::new(Instruction::new(
                        Opcode::Fpu,
                        dst,
                        dst,
                        Register::Zero,
                        FpuOp::Ffloor as u8,
                    ))
                    .with_comment("approximate round"),
                );
            }
            "to_bits" => {
                // No-op for f64 -> u64 bit reinterpretation
                self.gen_expr(receiver, dst_reg, instrs)?;
            }
            "is_nan" | "is_infinite" => {
                // TODO: implement NaN/Inf checks
                self.gen_expr(receiver, dst_reg, instrs)?;
                instrs.push(
                    GeneratedInstr::new(Instruction::with_imm(
                        Opcode::Mov,
                        dst,
                        Register::Zero,
                        0,
                        0, // Placeholder
                    ))
                    .with_comment(format!("TODO: {}", method)),
                );
            }
            _ => {
                return Err(CompilerError::Unsupported(format!("Method: {}", method)));
            }
        }

        Ok(())
    }
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Get memory width for a type.
fn type_to_mem_width(ty: &TypeInfo) -> MemWidth {
    match ty {
        TypeInfo::U8 => MemWidth::Byte,
        TypeInfo::U64 | TypeInfo::I64 | TypeInfo::F64 | TypeInfo::Ptr(_) | TypeInfo::MutPtr(_) => {
            MemWidth::Double
        }
        TypeInfo::Bool => MemWidth::Byte,
        _ => MemWidth::Double,
    }
}

/// Get size in bytes for a type.
fn type_size(ty: &TypeInfo) -> u32 {
    match ty {
        TypeInfo::U8 | TypeInfo::Bool => 1,
        TypeInfo::U64 | TypeInfo::I64 | TypeInfo::F64 | TypeInfo::Ptr(_) | TypeInfo::MutPtr(_) => 8,
        _ => 8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::analyzer::Analyzer;
    use crate::compiler::parser::parse_module;

    #[test]
    fn test_codegen_simple() {
        let source = r#"
            pub fn add(a: u64, b: u64) -> u64 {
                a + b
            }
        "#;

        let module = parse_module(source).unwrap();
        let mut analyzer = Analyzer::new();
        let func = analyzer.analyze(&module.functions[0]).unwrap();

        let mut codegen = CodeGenerator::new();
        let config = CompilerConfig::default();
        let generated = codegen.generate(&func, &config).unwrap();

        assert!(!generated.instructions.is_empty());
        let nl_source = generated.to_nl_source(&config);
        assert!(nl_source.contains("add"));
    }

    #[test]
    fn test_byte_pointer_arithmetic() {
        // Test that *const u8 pointer arithmetic increments by 1, not 8
        let source = r#"
            pub unsafe fn strlen(ptr: *const u8) -> u64 {
                let mut len: u64 = 0;
                let mut p = ptr;
                while *p != 0 {
                    len = len + 1;
                    p = p.add(1);
                }
                len
            }
        "#;

        let module = parse_module(source).unwrap();
        let mut analyzer = Analyzer::new();
        let func = analyzer.analyze(&module.functions[0]).unwrap();

        let mut codegen = CodeGenerator::new();
        let config = CompilerConfig::default();
        let generated = codegen.generate(&func, &config).unwrap();

        let nl_source = generated.to_nl_source(&config);

        // Should NOT contain "alui.Shl r*, r*, 3" (which is * 8)
        // For byte pointers, we should just add the offset directly without shift
        assert!(
            !nl_source.contains("alui.Shl r15, r15, 3"),
            "Byte pointer arithmetic should not shift by 3 (multiply by 8)"
        );

        // Verify we have the add instruction for pointer arithmetic
        assert!(nl_source.contains("alu.Add"));
    }

    #[test]
    fn test_u64_pointer_arithmetic() {
        // Test that *const u64 pointer arithmetic increments by 8
        let source = r#"
            pub unsafe fn sum_array(ptr: *const u64, n: u64) -> u64 {
                let mut sum: u64 = 0;
                let mut p = ptr;
                let mut i: u64 = 0;
                while i < n {
                    sum = sum + *p;
                    p = p.add(1);
                    i = i + 1;
                }
                sum
            }
        "#;

        let module = parse_module(source).unwrap();
        let mut analyzer = Analyzer::new();
        let func = analyzer.analyze(&module.functions[0]).unwrap();

        let mut codegen = CodeGenerator::new();
        let config = CompilerConfig::default();
        let generated = codegen.generate(&func, &config).unwrap();

        let nl_source = generated.to_nl_source(&config);

        // Should contain shift by 3 (multiply by 8) for u64 pointers
        assert!(
            nl_source.contains("alui.Shl") && nl_source.contains(", 3"),
            "u64 pointer arithmetic should shift by 3 (multiply by 8)"
        );
    }

    #[test]
    fn test_float_function() {
        // Test that simple float functions compile correctly
        let source = r#"
            pub fn fadd(a: f64, b: f64) -> f64 {
                a + b
            }
        "#;

        let module = parse_module(source).unwrap();
        let mut analyzer = Analyzer::new();
        let func = analyzer.analyze(&module.functions[0]).unwrap();

        let mut codegen = CodeGenerator::new();
        let config = CompilerConfig::default();
        let generated = codegen.generate(&func, &config).unwrap();

        let nl_source = generated.to_nl_source(&config);
        // Should contain FPU add operation
        assert!(nl_source.contains("fpu.") || nl_source.contains("fadd"));
    }

    #[test]
    fn test_multiple_float_functions() {
        // Test compiling multiple float functions (like stdlib does)
        let source = r#"
            pub fn fadd(a: f64, b: f64) -> f64 { a + b }
            pub fn fsub(a: f64, b: f64) -> f64 { a - b }
            pub fn fmul(a: f64, b: f64) -> f64 { a * b }
            pub fn fdiv(a: f64, b: f64) -> f64 { a / b }
        "#;

        let module = parse_module(source).unwrap();
        assert_eq!(module.functions.len(), 4);

        for func in &module.functions {
            let mut analyzer = Analyzer::new();
            let analyzed = analyzer.analyze(func).unwrap();

            let mut codegen = CodeGenerator::new();
            let config = CompilerConfig::default();
            let generated = codegen.generate(&analyzed, &config).unwrap();

            assert!(!generated.instructions.is_empty());
        }
    }
}
