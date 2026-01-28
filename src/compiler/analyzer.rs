//! Semantic Analyzer
//!
//! Performs type checking, scope analysis, and register allocation
//! for parsed Rust functions.

use super::parser::{
    BinaryOp, NeurlangMetadata, ParsedExpr, ParsedFunction, ParsedStmt, TypeInfo, UnaryOp,
};
use std::collections::HashMap;
use thiserror::Error;

/// Analysis errors.
#[derive(Debug, Error)]
pub enum AnalyzeError {
    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),

    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },

    #[error("Cannot assign to immutable variable: {0}")]
    ImmutableAssignment(String),

    #[error("Variable already defined: {0}")]
    DuplicateDefinition(String),

    #[error("Too many variables (register overflow)")]
    RegisterOverflow,

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
}

/// Analyzed function ready for code generation.
#[derive(Debug)]
pub struct AnalyzedFunction {
    pub name: String,
    pub params: Vec<AnalyzedParam>,
    pub return_type: TypeInfo,
    pub body: Vec<AnalyzedStmt>,
    pub variables: HashMap<String, Variable>,
    pub doc_comment: Option<String>,
    pub is_unsafe: bool,
    /// Maximum register used (for spilling if needed)
    pub max_register: u8,
    /// Neurlang-specific metadata (prompts, category, difficulty, etc.)
    pub neurlang_meta: NeurlangMetadata,
}

/// Analyzed parameter with register assignment.
#[derive(Debug)]
pub struct AnalyzedParam {
    pub name: String,
    pub ty: TypeInfo,
    pub register: u8,
}

/// Variable in scope.
#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub ty: TypeInfo,
    pub register: u8,
    pub is_mutable: bool,
    pub is_param: bool,
    /// Whether the variable has been initialized (for deferred initialization)
    pub is_initialized: bool,
}

/// Analyzed statement.
#[derive(Debug)]
pub enum AnalyzedStmt {
    /// Variable declaration
    Let {
        var: Variable,
        init: Option<AnalyzedExpr>,
    },
    /// Assignment
    Assign {
        target: AnalyzedExpr,
        value: AnalyzedExpr,
    },
    /// Expression statement
    Expr(AnalyzedExpr),
    /// If statement
    If {
        condition: AnalyzedExpr,
        then_branch: Vec<AnalyzedStmt>,
        else_branch: Option<Vec<AnalyzedStmt>>,
    },
    /// While loop
    While {
        condition: AnalyzedExpr,
        body: Vec<AnalyzedStmt>,
    },
    /// Infinite loop
    Loop { body: Vec<AnalyzedStmt> },
    /// For loop: for var in start..end { body }
    For {
        var: Variable,
        start: AnalyzedExpr,
        end: AnalyzedExpr,
        inclusive: bool,
        body: Vec<AnalyzedStmt>,
    },
    /// Return
    Return(Option<AnalyzedExpr>),
    /// Break
    Break,
    /// Continue
    Continue,
}

/// Analyzed expression with type information.
#[derive(Debug)]
pub struct AnalyzedExpr {
    pub kind: AnalyzedExprKind,
    pub ty: TypeInfo,
}

/// Expression kinds.
#[derive(Debug)]
pub enum AnalyzedExprKind {
    /// Integer literal
    IntLit(u64),
    /// Float literal
    FloatLit(f64),
    /// Boolean literal (0 or 1)
    BoolLit(bool),
    /// Variable reference
    Var { name: String, register: u8 },
    /// Binary operation
    Binary {
        op: BinaryOp,
        left: Box<AnalyzedExpr>,
        right: Box<AnalyzedExpr>,
    },
    /// Unary operation
    Unary {
        op: UnaryOp,
        expr: Box<AnalyzedExpr>,
    },
    /// Function call
    Call {
        func: String,
        args: Vec<AnalyzedExpr>,
    },
    /// Method call
    MethodCall {
        receiver: Box<AnalyzedExpr>,
        method: String,
        args: Vec<AnalyzedExpr>,
    },
    /// Pointer dereference
    Deref(Box<AnalyzedExpr>),
    /// Memory load: ptr[index]
    Load {
        base: Box<AnalyzedExpr>,
        offset: Box<AnalyzedExpr>,
    },
    /// Memory store: ptr[index] = value
    Store {
        base: Box<AnalyzedExpr>,
        offset: Box<AnalyzedExpr>,
        value: Box<AnalyzedExpr>,
    },
    /// Cast
    Cast {
        expr: Box<AnalyzedExpr>,
        target_ty: TypeInfo,
    },
    /// Block expression
    Block {
        stmts: Vec<AnalyzedStmt>,
        expr: Option<Box<AnalyzedExpr>>,
    },
    /// Conditional expression
    Conditional {
        condition: Box<AnalyzedExpr>,
        then_expr: Box<AnalyzedExpr>,
        else_expr: Box<AnalyzedExpr>,
    },
}

/// The semantic analyzer.
pub struct Analyzer {
    scopes: Vec<Scope>,
    next_register: u8,
    max_register: u8,
}

/// A scope containing variable bindings.
struct Scope {
    variables: HashMap<String, Variable>,
}

impl Analyzer {
    /// Create a new analyzer.
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope {
                variables: HashMap::new(),
            }],
            next_register: 0,
            max_register: 0,
        }
    }

    /// Analyze a parsed function.
    pub fn analyze(&mut self, func: &ParsedFunction) -> Result<AnalyzedFunction, AnalyzeError> {
        // Reset state
        self.scopes = vec![Scope {
            variables: HashMap::new(),
        }];
        self.next_register = 0;
        self.max_register = 0;

        // Register parameters (r0-r3 for first 4 params)
        let mut analyzed_params = Vec::new();
        for param in func.params.iter() {
            let register = self.allocate_register()?;
            let var = Variable {
                name: param.name.clone(),
                ty: param.ty.clone(),
                register,
                is_mutable: param.is_mutable,
                is_param: true,
                is_initialized: true, // Parameters are always initialized
            };
            self.define_variable(var.clone())?;
            analyzed_params.push(AnalyzedParam {
                name: param.name.clone(),
                ty: param.ty.clone(),
                register,
            });
        }

        // Analyze body
        let body = self.analyze_stmts(&func.body)?;

        // Determine return type
        let return_type = func.return_type.clone().unwrap_or(TypeInfo::Unit);

        Ok(AnalyzedFunction {
            name: func.name.clone(),
            params: analyzed_params,
            return_type,
            body,
            variables: self.scopes[0].variables.clone(),
            doc_comment: func.doc_comment.clone(),
            is_unsafe: func.is_unsafe,
            max_register: self.max_register,
            neurlang_meta: func.neurlang_meta.clone(),
        })
    }

    /// Allocate a register for a new variable.
    fn allocate_register(&mut self) -> Result<u8, AnalyzeError> {
        if self.next_register >= 16 {
            // Use r0-r15 for locals, higher registers for temporaries
            // In practice, we'll spill to memory if needed
            return Err(AnalyzeError::RegisterOverflow);
        }
        let reg = self.next_register;
        self.next_register += 1;
        if reg > self.max_register {
            self.max_register = reg;
        }
        Ok(reg)
    }

    /// Enter a new scope.
    fn push_scope(&mut self) {
        self.scopes.push(Scope {
            variables: HashMap::new(),
        });
    }

    /// Exit the current scope.
    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    /// Define a variable in the current scope.
    fn define_variable(&mut self, var: Variable) -> Result<(), AnalyzeError> {
        let scope = self.scopes.last_mut().unwrap();
        if scope.variables.contains_key(&var.name) {
            return Err(AnalyzeError::DuplicateDefinition(var.name.clone()));
        }
        scope.variables.insert(var.name.clone(), var);
        Ok(())
    }

    /// Look up a variable in all scopes.
    fn lookup_variable(&self, name: &str) -> Option<&Variable> {
        for scope in self.scopes.iter().rev() {
            if let Some(var) = scope.variables.get(name) {
                return Some(var);
            }
        }
        None
    }

    /// Mark a variable as initialized (for deferred initialization).
    fn mark_initialized(&mut self, name: &str) {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(var) = scope.variables.get_mut(name) {
                var.is_initialized = true;
                return;
            }
        }
    }

    /// Analyze a list of statements.
    fn analyze_stmts(&mut self, stmts: &[ParsedStmt]) -> Result<Vec<AnalyzedStmt>, AnalyzeError> {
        stmts.iter().map(|s| self.analyze_stmt(s)).collect()
    }

    /// Analyze a single statement.
    fn analyze_stmt(&mut self, stmt: &ParsedStmt) -> Result<AnalyzedStmt, AnalyzeError> {
        match stmt {
            ParsedStmt::Let {
                name,
                ty,
                init,
                is_mutable,
            } => {
                let init_expr = match init {
                    Some(e) => Some(self.analyze_expr(e)?),
                    None => None,
                };

                // Infer type from init if not specified
                let var_ty = match (ty, &init_expr) {
                    (Some(t), _) => t.clone(),
                    (None, Some(e)) => e.ty.clone(),
                    (None, None) => {
                        return Err(AnalyzeError::UnsupportedOperation(
                            "Uninitialized variable without type".to_string(),
                        ))
                    }
                };

                let register = self.allocate_register()?;
                let is_initialized = init_expr.is_some();
                let var = Variable {
                    name: name.clone(),
                    ty: var_ty,
                    register,
                    is_mutable: *is_mutable,
                    is_param: false,
                    is_initialized,
                };
                self.define_variable(var.clone())?;

                Ok(AnalyzedStmt::Let {
                    var,
                    init: init_expr,
                })
            }

            ParsedStmt::Assign { target, value } => {
                let target_expr = self.analyze_expr(target)?;
                let value_expr = self.analyze_expr(value)?;

                // Check mutability for simple variable assignment
                if let AnalyzedExprKind::Var { name, .. } = &target_expr.kind {
                    if let Some(var) = self.lookup_variable(name) {
                        if var.is_initialized && !var.is_mutable {
                            // Already initialized and not mutable - reject assignment
                            return Err(AnalyzeError::ImmutableAssignment(name.clone()));
                        }
                        // If not initialized, allow first assignment (deferred initialization)
                        // Mark as initialized
                        if !var.is_initialized {
                            self.mark_initialized(name);
                        }
                    }
                }

                Ok(AnalyzedStmt::Assign {
                    target: target_expr,
                    value: value_expr,
                })
            }

            ParsedStmt::Expr(expr) => {
                let analyzed = self.analyze_expr(expr)?;
                Ok(AnalyzedStmt::Expr(analyzed))
            }

            ParsedStmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond = self.analyze_expr(condition)?;

                self.push_scope();
                let then_stmts = self.analyze_stmts(then_branch)?;
                self.pop_scope();

                let else_stmts = match else_branch {
                    Some(stmts) => {
                        self.push_scope();
                        let result = self.analyze_stmts(stmts)?;
                        self.pop_scope();
                        Some(result)
                    }
                    None => None,
                };

                Ok(AnalyzedStmt::If {
                    condition: cond,
                    then_branch: then_stmts,
                    else_branch: else_stmts,
                })
            }

            ParsedStmt::While { condition, body } => {
                let cond = self.analyze_expr(condition)?;

                self.push_scope();
                let body_stmts = self.analyze_stmts(body)?;
                self.pop_scope();

                Ok(AnalyzedStmt::While {
                    condition: cond,
                    body: body_stmts,
                })
            }

            ParsedStmt::Loop { body } => {
                self.push_scope();
                let body_stmts = self.analyze_stmts(body)?;
                self.pop_scope();

                Ok(AnalyzedStmt::Loop { body: body_stmts })
            }

            ParsedStmt::For {
                var,
                start,
                end,
                inclusive,
                body,
            } => {
                let start_expr = self.analyze_expr(start)?;
                let end_expr = self.analyze_expr(end)?;

                // Create loop variable in new scope
                self.push_scope();

                let register = self.allocate_register()?;
                let loop_var = Variable {
                    name: var.clone(),
                    ty: TypeInfo::U64, // For loop variables are always u64
                    register,
                    is_mutable: true, // Loop var is implicitly mutable
                    is_param: false,
                    is_initialized: true,
                };
                self.define_variable(loop_var.clone())?;

                let body_stmts = self.analyze_stmts(body)?;
                self.pop_scope();

                Ok(AnalyzedStmt::For {
                    var: loop_var,
                    start: start_expr,
                    end: end_expr,
                    inclusive: *inclusive,
                    body: body_stmts,
                })
            }

            ParsedStmt::Return(expr) => {
                let analyzed = match expr {
                    Some(e) => Some(self.analyze_expr(e)?),
                    None => None,
                };
                Ok(AnalyzedStmt::Return(analyzed))
            }

            ParsedStmt::Break => Ok(AnalyzedStmt::Break),
            ParsedStmt::Continue => Ok(AnalyzedStmt::Continue),
        }
    }

    /// Analyze an expression.
    fn analyze_expr(&mut self, expr: &ParsedExpr) -> Result<AnalyzedExpr, AnalyzeError> {
        match expr {
            ParsedExpr::IntLit(n) => Ok(AnalyzedExpr {
                kind: AnalyzedExprKind::IntLit(*n),
                ty: TypeInfo::U64,
            }),

            ParsedExpr::FloatLit(f) => Ok(AnalyzedExpr {
                kind: AnalyzedExprKind::FloatLit(*f),
                ty: TypeInfo::F64,
            }),

            ParsedExpr::BoolLit(b) => Ok(AnalyzedExpr {
                kind: AnalyzedExprKind::BoolLit(*b),
                ty: TypeInfo::Bool,
            }),

            ParsedExpr::Var(name) => {
                // Check for special constants
                if name == "u64 :: MAX" || name == "u64::MAX" {
                    return Ok(AnalyzedExpr {
                        kind: AnalyzedExprKind::IntLit(u64::MAX),
                        ty: TypeInfo::U64,
                    });
                }

                let var = self
                    .lookup_variable(name)
                    .ok_or_else(|| AnalyzeError::UndefinedVariable(name.clone()))?;

                Ok(AnalyzedExpr {
                    kind: AnalyzedExprKind::Var {
                        name: name.clone(),
                        register: var.register,
                    },
                    ty: var.ty.clone(),
                })
            }

            ParsedExpr::Binary { op, left, right } => {
                let left_expr = self.analyze_expr(left)?;
                let right_expr = self.analyze_expr(right)?;

                // Determine result type
                let result_ty = match op {
                    BinaryOp::Eq
                    | BinaryOp::Ne
                    | BinaryOp::Lt
                    | BinaryOp::Le
                    | BinaryOp::Gt
                    | BinaryOp::Ge
                    | BinaryOp::LogicalAnd
                    | BinaryOp::LogicalOr => TypeInfo::Bool,
                    _ => {
                        // For arithmetic, use the type of the left operand
                        left_expr.ty.clone()
                    }
                };

                Ok(AnalyzedExpr {
                    kind: AnalyzedExprKind::Binary {
                        op: *op,
                        left: Box::new(left_expr),
                        right: Box::new(right_expr),
                    },
                    ty: result_ty,
                })
            }

            ParsedExpr::Unary { op, expr } => {
                let inner = self.analyze_expr(expr)?;
                let ty = inner.ty.clone();

                Ok(AnalyzedExpr {
                    kind: AnalyzedExprKind::Unary {
                        op: *op,
                        expr: Box::new(inner),
                    },
                    ty,
                })
            }

            ParsedExpr::Call { func, args } => {
                let analyzed_args: Vec<AnalyzedExpr> = args
                    .iter()
                    .map(|a| self.analyze_expr(a))
                    .collect::<Result<_, _>>()?;

                // Infer return type based on known functions
                let return_ty = self.infer_call_type(func, &analyzed_args);

                Ok(AnalyzedExpr {
                    kind: AnalyzedExprKind::Call {
                        func: func.clone(),
                        args: analyzed_args,
                    },
                    ty: return_ty,
                })
            }

            ParsedExpr::MethodCall {
                receiver,
                method,
                args,
            } => {
                let recv = self.analyze_expr(receiver)?;
                let analyzed_args: Vec<AnalyzedExpr> = args
                    .iter()
                    .map(|a| self.analyze_expr(a))
                    .collect::<Result<_, _>>()?;

                let return_ty = self.infer_method_type(&recv.ty, method, &analyzed_args);

                Ok(AnalyzedExpr {
                    kind: AnalyzedExprKind::MethodCall {
                        receiver: Box::new(recv),
                        method: method.clone(),
                        args: analyzed_args,
                    },
                    ty: return_ty,
                })
            }

            ParsedExpr::Deref(inner) => {
                let inner_expr = self.analyze_expr(inner)?;

                // Get the pointee type
                let pointee_ty = match &inner_expr.ty {
                    TypeInfo::Ptr(t) | TypeInfo::MutPtr(t) => t.as_ref().clone(),
                    _ => TypeInfo::U64, // Assume u64 for raw pointer deref
                };

                Ok(AnalyzedExpr {
                    kind: AnalyzedExprKind::Deref(Box::new(inner_expr)),
                    ty: pointee_ty,
                })
            }

            ParsedExpr::Index { base, index } => {
                let base_expr = self.analyze_expr(base)?;
                let index_expr = self.analyze_expr(index)?;

                // Get element type
                let elem_ty = match &base_expr.ty {
                    TypeInfo::Ptr(t) | TypeInfo::MutPtr(t) => t.as_ref().clone(),
                    _ => TypeInfo::U64,
                };

                Ok(AnalyzedExpr {
                    kind: AnalyzedExprKind::Load {
                        base: Box::new(base_expr),
                        offset: Box::new(index_expr),
                    },
                    ty: elem_ty,
                })
            }

            ParsedExpr::Cast { expr, ty } => {
                let inner = self.analyze_expr(expr)?;

                Ok(AnalyzedExpr {
                    kind: AnalyzedExprKind::Cast {
                        expr: Box::new(inner),
                        target_ty: ty.clone(),
                    },
                    ty: ty.clone(),
                })
            }

            ParsedExpr::Block { stmts, expr } => {
                self.push_scope();
                let analyzed_stmts = self.analyze_stmts(stmts)?;
                let result_expr = match expr {
                    Some(e) => Some(Box::new(self.analyze_expr(e)?)),
                    None => None,
                };
                self.pop_scope();

                let ty = result_expr
                    .as_ref()
                    .map(|e| e.ty.clone())
                    .unwrap_or(TypeInfo::Unit);

                Ok(AnalyzedExpr {
                    kind: AnalyzedExprKind::Block {
                        stmts: analyzed_stmts,
                        expr: result_expr,
                    },
                    ty,
                })
            }

            ParsedExpr::If {
                condition,
                then_expr,
                else_expr,
            } => {
                let cond = self.analyze_expr(condition)?;
                let then_e = self.analyze_expr(then_expr)?;

                let else_e = match else_expr {
                    Some(e) => self.analyze_expr(e)?,
                    None => {
                        return Err(AnalyzeError::UnsupportedOperation(
                            "If expression requires else branch".to_string(),
                        ))
                    }
                };

                let ty = then_e.ty.clone();

                Ok(AnalyzedExpr {
                    kind: AnalyzedExprKind::Conditional {
                        condition: Box::new(cond),
                        then_expr: Box::new(then_e),
                        else_expr: Box::new(else_e),
                    },
                    ty,
                })
            }
        }
    }

    /// Infer the return type of a function call.
    fn infer_call_type(&self, func: &str, _args: &[AnalyzedExpr]) -> TypeInfo {
        // Known function return types
        match func {
            // Math functions
            "gcd" | "lcm" | "power" | "isqrt" | "factorial" | "fibonacci" => TypeInfo::U64,
            "is_prime" | "is_power_of_2" => TypeInfo::U64,
            // Float functions
            "sqrt" | "abs" | "floor" | "ceil" | "round" => TypeInfo::F64,
            "fadd" | "fsub" | "fmul" | "fdiv" | "fsqrt" | "fabs" => TypeInfo::F64,
            // Bit functions
            "popcount" | "clz" | "ctz" | "bswap" | "rotl" | "rotr" => TypeInfo::U64,
            // String functions
            "strlen" | "strcmp" | "atoi" | "strchr" | "strrchr" => TypeInfo::U64,
            "is_digit" | "is_alpha" | "is_space" => TypeInfo::U64,
            "to_upper" | "to_lower" => TypeInfo::U8,
            // Array functions
            "sum" | "array_min" | "array_max" | "linear_search" | "binary_search" => TypeInfo::U64,
            "count" | "is_sorted" => TypeInfo::U64,
            // Collection functions
            "stack_push" | "stack_pop" | "stack_size" | "stack_is_empty" => TypeInfo::U64,
            "queue_enqueue" | "queue_dequeue" | "queue_size" | "queue_is_empty" => TypeInfo::U64,
            "hashtable_get" | "hashtable_put" | "hashtable_contains" | "hashtable_count" => {
                TypeInfo::U64
            }
            _ => TypeInfo::U64, // Default to u64
        }
    }

    /// Infer the return type of a method call.
    fn infer_method_type(
        &self,
        receiver_ty: &TypeInfo,
        method: &str,
        _args: &[AnalyzedExpr],
    ) -> TypeInfo {
        match (receiver_ty, method) {
            // Pointer methods
            (TypeInfo::Ptr(_), "add") | (TypeInfo::MutPtr(_), "add") => receiver_ty.clone(),
            (TypeInfo::Ptr(_), "offset") | (TypeInfo::MutPtr(_), "offset") => receiver_ty.clone(),

            // Integer methods
            (TypeInfo::U64, "wrapping_add") => TypeInfo::U64,
            (TypeInfo::U64, "wrapping_sub") => TypeInfo::U64,
            (TypeInfo::U64, "wrapping_mul") => TypeInfo::U64,
            (TypeInfo::U64, "saturating_add") => TypeInfo::U64,
            (TypeInfo::U64, "saturating_sub") => TypeInfo::U64,
            (TypeInfo::U64, "count_ones") => TypeInfo::U64,
            (TypeInfo::U64, "leading_zeros") => TypeInfo::U64,
            (TypeInfo::U64, "trailing_zeros") => TypeInfo::U64,
            (TypeInfo::U64, "to_bits") => TypeInfo::U64,

            // Float methods
            (TypeInfo::F64, "sqrt") => TypeInfo::F64,
            (TypeInfo::F64, "abs") => TypeInfo::F64,
            (TypeInfo::F64, "floor") => TypeInfo::F64,
            (TypeInfo::F64, "ceil") => TypeInfo::F64,
            (TypeInfo::F64, "round") => TypeInfo::F64,
            (TypeInfo::F64, "is_nan") => TypeInfo::Bool,
            (TypeInfo::F64, "is_infinite") => TypeInfo::Bool,
            (TypeInfo::F64, "to_bits") => TypeInfo::U64,

            // Default
            _ => TypeInfo::U64,
        }
    }
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::parser::parse_module;

    #[test]
    fn test_analyze_simple() {
        let source = r#"
            pub fn add(a: u64, b: u64) -> u64 {
                a + b
            }
        "#;

        let module = parse_module(source).unwrap();
        let mut analyzer = Analyzer::new();
        let func = analyzer.analyze(&module.functions[0]).unwrap();

        assert_eq!(func.name, "add");
        assert_eq!(func.params.len(), 2);
        assert_eq!(func.params[0].register, 0);
        assert_eq!(func.params[1].register, 1);
    }

    #[test]
    fn test_analyze_with_locals() {
        let source = r#"
            pub fn test(n: u64) -> u64 {
                let mut result: u64 = 1;
                let mut i: u64 = n;
                result
            }
        "#;

        let module = parse_module(source).unwrap();
        let mut analyzer = Analyzer::new();
        let func = analyzer.analyze(&module.functions[0]).unwrap();

        assert_eq!(func.params.len(), 1);
        // n=r0, result=r1, i=r2
        assert_eq!(func.max_register, 2);
    }
}
