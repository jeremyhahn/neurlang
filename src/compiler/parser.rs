//! Rust Source Parser
//!
//! Uses `syn` to parse Rust source code into an intermediate representation
//! that can be analyzed and compiled to Neurlang IR.

use syn::{
    parse_file, BinOp, Expr, ExprBinary, ExprBlock, ExprCall, ExprForLoop, ExprIf, ExprIndex,
    ExprLit, ExprLoop, ExprMethodCall, ExprPath, ExprRange, ExprUnary, ExprWhile, File, FnArg,
    Item, ItemFn, Lit, Local, Pat, PatIdent, PatType, ReturnType, Stmt, Type, UnOp,
};
use thiserror::Error;

/// Errors that can occur during parsing.
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Syn parse error: {0}")]
    Syn(#[from] syn::Error),

    #[error("Unsupported syntax: {0}")]
    Unsupported(String),
}

/// Parsed Rust module containing functions.
#[derive(Debug)]
pub struct ParsedModule {
    pub functions: Vec<ParsedFunction>,
}

/// Parameter documentation for Neurlang export.
#[derive(Debug, Clone)]
pub struct ParamDoc {
    pub name: String,
    pub register: String,
    pub description: String,
}

/// Neurlang-specific metadata extracted from doc comments.
#[derive(Debug, Clone, Default)]
pub struct NeurlangMetadata {
    pub prompts: Vec<String>,
    pub param_docs: Vec<ParamDoc>,
    pub category: Option<String>,
    pub difficulty: Option<u8>,
}

/// A parsed Rust function.
#[derive(Debug, Clone)]
pub struct ParsedFunction {
    pub name: String,
    pub params: Vec<FunctionParam>,
    pub return_type: Option<TypeInfo>,
    pub body: Vec<ParsedStmt>,
    pub doc_comment: Option<String>,
    pub is_unsafe: bool,
    pub neurlang_meta: NeurlangMetadata,
}

/// Function parameter.
#[derive(Debug, Clone)]
pub struct FunctionParam {
    pub name: String,
    pub ty: TypeInfo,
    pub is_mutable: bool,
}

/// Type information.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeInfo {
    U64,
    I64,
    U8,
    F64,
    Bool,
    Ptr(Box<TypeInfo>),    // *const T or *mut T
    MutPtr(Box<TypeInfo>), // *mut T
    Unit,
    Unknown(String),
}

impl TypeInfo {
    /// Parse a syn Type into TypeInfo.
    pub fn from_syn(ty: &Type) -> Self {
        match ty {
            Type::Path(tp) => {
                if let Some(ident) = tp.path.get_ident() {
                    match ident.to_string().as_str() {
                        "u64" => TypeInfo::U64,
                        "i64" => TypeInfo::I64,
                        "u8" => TypeInfo::U8,
                        "f64" => TypeInfo::F64,
                        "bool" => TypeInfo::Bool,
                        other => TypeInfo::Unknown(other.to_string()),
                    }
                } else {
                    TypeInfo::Unknown(quote::quote!(#tp).to_string())
                }
            }
            Type::Ptr(ptr) => {
                let inner = TypeInfo::from_syn(&ptr.elem);
                if ptr.mutability.is_some() {
                    TypeInfo::MutPtr(Box::new(inner))
                } else {
                    TypeInfo::Ptr(Box::new(inner))
                }
            }
            Type::Tuple(t) if t.elems.is_empty() => TypeInfo::Unit,
            _ => TypeInfo::Unknown(quote::quote!(#ty).to_string()),
        }
    }

    /// Check if this is a numeric type that fits in a register.
    pub fn is_register_type(&self) -> bool {
        matches!(
            self,
            TypeInfo::U64 | TypeInfo::I64 | TypeInfo::U8 | TypeInfo::F64 | TypeInfo::Bool
        )
    }

    /// Check if this is a pointer type.
    pub fn is_pointer(&self) -> bool {
        matches!(self, TypeInfo::Ptr(_) | TypeInfo::MutPtr(_))
    }

    /// Check if this is a floating-point type.
    pub fn is_float(&self) -> bool {
        matches!(self, TypeInfo::F64)
    }
}

/// Parsed statement.
#[derive(Debug, Clone)]
pub enum ParsedStmt {
    /// Let binding: let [mut] name [: type] = expr;
    Let {
        name: String,
        ty: Option<TypeInfo>,
        init: Option<ParsedExpr>,
        is_mutable: bool,
    },
    /// Expression statement: expr;
    Expr(ParsedExpr),
    /// If statement
    If {
        condition: ParsedExpr,
        then_branch: Vec<ParsedStmt>,
        else_branch: Option<Vec<ParsedStmt>>,
    },
    /// While loop
    While {
        condition: ParsedExpr,
        body: Vec<ParsedStmt>,
    },
    /// Infinite loop
    Loop { body: Vec<ParsedStmt> },
    /// For loop: for i in start..end { body }
    For {
        var: String,
        start: ParsedExpr,
        end: ParsedExpr,
        inclusive: bool,
        body: Vec<ParsedStmt>,
    },
    /// Return statement
    Return(Option<ParsedExpr>),
    /// Break statement
    Break,
    /// Continue statement
    Continue,
    /// Assignment: lhs = rhs;
    Assign {
        target: ParsedExpr,
        value: ParsedExpr,
    },
}

/// Parsed expression.
#[derive(Debug, Clone)]
pub enum ParsedExpr {
    /// Integer literal
    IntLit(u64),
    /// Float literal
    FloatLit(f64),
    /// Boolean literal
    BoolLit(bool),
    /// Variable reference
    Var(String),
    /// Binary operation
    Binary {
        op: BinaryOp,
        left: Box<ParsedExpr>,
        right: Box<ParsedExpr>,
    },
    /// Unary operation
    Unary { op: UnaryOp, expr: Box<ParsedExpr> },
    /// Function call
    Call { func: String, args: Vec<ParsedExpr> },
    /// Method call: expr.method(args)
    MethodCall {
        receiver: Box<ParsedExpr>,
        method: String,
        args: Vec<ParsedExpr>,
    },
    /// Field/pointer dereference: *ptr or ptr.add(n)
    Deref(Box<ParsedExpr>),
    /// Index expression: arr[i]
    Index {
        base: Box<ParsedExpr>,
        index: Box<ParsedExpr>,
    },
    /// Cast expression: x as T
    Cast { expr: Box<ParsedExpr>, ty: TypeInfo },
    /// Block expression: { stmts; expr }
    Block {
        stmts: Vec<ParsedStmt>,
        expr: Option<Box<ParsedExpr>>,
    },
    /// If expression (for ternary-like use)
    If {
        condition: Box<ParsedExpr>,
        then_expr: Box<ParsedExpr>,
        else_expr: Option<Box<ParsedExpr>>,
    },
}

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    // Bitwise
    And,
    Or,
    Xor,
    Shl,
    Shr,
    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    // Logical
    LogicalAnd,
    LogicalOr,
}

impl BinaryOp {
    fn from_syn(op: &BinOp) -> Option<Self> {
        Some(match op {
            BinOp::Add(_) => BinaryOp::Add,
            BinOp::Sub(_) => BinaryOp::Sub,
            BinOp::Mul(_) => BinaryOp::Mul,
            BinOp::Div(_) => BinaryOp::Div,
            BinOp::Rem(_) => BinaryOp::Rem,
            BinOp::BitAnd(_) => BinaryOp::And,
            BinOp::BitOr(_) => BinaryOp::Or,
            BinOp::BitXor(_) => BinaryOp::Xor,
            BinOp::Shl(_) => BinaryOp::Shl,
            BinOp::Shr(_) => BinaryOp::Shr,
            BinOp::Eq(_) => BinaryOp::Eq,
            BinOp::Ne(_) => BinaryOp::Ne,
            BinOp::Lt(_) => BinaryOp::Lt,
            BinOp::Le(_) => BinaryOp::Le,
            BinOp::Gt(_) => BinaryOp::Gt,
            BinOp::Ge(_) => BinaryOp::Ge,
            BinOp::And(_) => BinaryOp::LogicalAnd,
            BinOp::Or(_) => BinaryOp::LogicalOr,
            _ => return None,
        })
    }
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Neg,   // -x
    Not,   // !x
    Deref, // *x
}

impl UnaryOp {
    fn from_syn(op: &UnOp) -> Option<Self> {
        match op {
            UnOp::Neg(_) => Some(UnaryOp::Neg),
            UnOp::Not(_) => Some(UnaryOp::Not),
            UnOp::Deref(_) => Some(UnaryOp::Deref),
            _ => None, // Handle future variants
        }
    }
}

/// Parse a Rust source file into a module.
pub fn parse_module(source: &str) -> Result<ParsedModule, ParseError> {
    let file: File = parse_file(source)?;
    let mut functions = Vec::new();

    for item in file.items {
        match item {
            Item::Fn(func) => {
                if let Some(parsed) = parse_function(&func)? {
                    functions.push(parsed);
                }
            }
            Item::Mod(m) => {
                // Parse inline module content
                if let Some((_, items)) = m.content {
                    for item in items {
                        if let Item::Fn(func) = item {
                            if let Some(parsed) = parse_function(&func)? {
                                functions.push(parsed);
                            }
                        }
                    }
                }
            }
            _ => {} // Skip other items
        }
    }

    Ok(ParsedModule { functions })
}

/// Parse a single function.
fn parse_function(func: &ItemFn) -> Result<Option<ParsedFunction>, ParseError> {
    let name = func.sig.ident.to_string();

    // Skip test functions
    if name.starts_with("test_") {
        return Ok(None);
    }

    // Parse parameters
    let mut params = Vec::new();
    for arg in &func.sig.inputs {
        if let FnArg::Typed(PatType { pat, ty, .. }) = arg {
            if let Pat::Ident(PatIdent {
                ident, mutability, ..
            }) = pat.as_ref()
            {
                params.push(FunctionParam {
                    name: ident.to_string(),
                    ty: TypeInfo::from_syn(ty),
                    is_mutable: mutability.is_some(),
                });
            }
        }
    }

    // Parse return type
    let return_type = match &func.sig.output {
        ReturnType::Default => None,
        ReturnType::Type(_, ty) => Some(TypeInfo::from_syn(ty)),
    };

    // Parse body
    let body = parse_block(&func.block)?;

    // Extract doc comment
    let doc_comment = extract_doc_comment(func);

    // Parse Neurlang-specific metadata from doc comment
    let neurlang_meta = parse_neurlang_metadata(doc_comment.as_deref());

    Ok(Some(ParsedFunction {
        name,
        params,
        return_type,
        body,
        doc_comment,
        is_unsafe: func.sig.unsafety.is_some(),
        neurlang_meta,
    }))
}

/// Extract doc comment from function attributes.
fn extract_doc_comment(func: &ItemFn) -> Option<String> {
    let mut doc_lines = Vec::new();

    for attr in &func.attrs {
        if attr.path().is_ident("doc") {
            if let syn::Meta::NameValue(nv) = &attr.meta {
                if let syn::Expr::Lit(lit) = &nv.value {
                    if let syn::Lit::Str(s) = &lit.lit {
                        doc_lines.push(s.value().trim().to_string());
                    }
                }
            }
        }
    }

    if doc_lines.is_empty() {
        None
    } else {
        Some(doc_lines.join("\n"))
    }
}

/// Parse Neurlang-specific metadata from doc comments.
///
/// Extracts:
/// - `# Prompts` section: list of prompt templates
/// - `# Parameters` section: parameter documentation
/// - `- Category: xxx` line
/// - `- Difficulty: n` line
fn parse_neurlang_metadata(doc: Option<&str>) -> NeurlangMetadata {
    let mut meta = NeurlangMetadata::default();

    let doc = match doc {
        Some(d) => d,
        None => return meta,
    };

    let mut in_prompts_section = false;
    let mut in_params_section = false;

    for line in doc.lines() {
        let trimmed = line.trim();

        // Section headers
        if trimmed == "# Prompts" {
            in_prompts_section = true;
            in_params_section = false;
            continue;
        }
        if trimmed == "# Parameters" {
            in_prompts_section = false;
            in_params_section = true;
            continue;
        }
        // End section on other headers
        if trimmed.starts_with("# ") {
            in_prompts_section = false;
            in_params_section = false;
        }

        // Parse prompts
        if in_prompts_section {
            // Format: - prompt text here
            if let Some(prompt) = trimmed.strip_prefix("- ") {
                let prompt = prompt.trim();
                if !prompt.is_empty() {
                    meta.prompts.push(prompt.to_string());
                }
            }
        }

        // Parse parameter docs
        if in_params_section {
            // Format: - name=register "description"
            if let Some(param_line) = trimmed.strip_prefix("- ") {
                if let Some(param_doc) = parse_param_doc(param_line) {
                    meta.param_docs.push(param_doc);
                }
            }
        }

        // Parse inline metadata (outside sections)
        // Format: - Category: algorithm/math
        if let Some(cat) = trimmed.strip_prefix("- Category:") {
            meta.category = Some(cat.trim().to_string());
        }
        // Format: - Difficulty: 2
        if let Some(diff) = trimmed.strip_prefix("- Difficulty:") {
            if let Ok(n) = diff.trim().parse::<u8>() {
                meta.difficulty = Some(n);
            }
        }
    }

    meta
}

/// Parse a parameter documentation line.
/// Format: name=register "description"
fn parse_param_doc(line: &str) -> Option<ParamDoc> {
    // Find the = sign
    let eq_pos = line.find('=')?;
    let name = line[..eq_pos].trim().to_string();

    let rest = &line[eq_pos + 1..];

    // Find the space before description or quote
    let (register, description) = if let Some(quote_pos) = rest.find('"') {
        let reg = rest[..quote_pos].trim().to_string();
        // Extract text between quotes
        let desc_start = quote_pos + 1;
        let desc_end = rest[desc_start..]
            .find('"')
            .map(|p| desc_start + p)
            .unwrap_or(rest.len());
        let desc = rest[desc_start..desc_end].to_string();
        (reg, desc)
    } else {
        // No quotes, just register
        let parts: Vec<&str> = rest.trim().splitn(2, ' ').collect();
        let reg = parts.first().unwrap_or(&"r0").to_string();
        let desc = parts.get(1).unwrap_or(&"").to_string();
        (reg, desc)
    };

    Some(ParamDoc {
        name,
        register,
        description,
    })
}

/// Parse a block of statements.
fn parse_block(block: &syn::Block) -> Result<Vec<ParsedStmt>, ParseError> {
    let mut stmts = Vec::new();

    for stmt in &block.stmts {
        match stmt {
            Stmt::Local(local) => {
                stmts.push(parse_local(local)?);
            }
            Stmt::Expr(expr, _) => {
                // Check for early return, if/while, etc.
                stmts.push(parse_expr_stmt(expr)?);
            }
            Stmt::Item(_) => {
                // Skip inner items
            }
            Stmt::Macro(_) => {
                return Err(ParseError::Unsupported(
                    "Macros are not supported".to_string(),
                ));
            }
        }
    }

    Ok(stmts)
}

/// Parse a local variable declaration.
fn parse_local(local: &Local) -> Result<ParsedStmt, ParseError> {
    let (name, is_mutable) = match &local.pat {
        Pat::Ident(pi) => (pi.ident.to_string(), pi.mutability.is_some()),
        Pat::Type(pt) => {
            if let Pat::Ident(pi) = pt.pat.as_ref() {
                (pi.ident.to_string(), pi.mutability.is_some())
            } else {
                return Err(ParseError::Unsupported("Complex let patterns".to_string()));
            }
        }
        _ => return Err(ParseError::Unsupported("Complex let patterns".to_string())),
    };

    let ty = match &local.pat {
        Pat::Type(pt) => Some(TypeInfo::from_syn(&pt.ty)),
        _ => None,
    };

    let init = match &local.init {
        Some(init) => Some(parse_expr(&init.expr)?),
        None => None,
    };

    Ok(ParsedStmt::Let {
        name,
        ty,
        init,
        is_mutable,
    })
}

/// Parse an expression statement (may be control flow).
fn parse_expr_stmt(expr: &Expr) -> Result<ParsedStmt, ParseError> {
    match expr {
        Expr::If(expr_if) => parse_if(expr_if),
        Expr::While(expr_while) => parse_while(expr_while),
        Expr::Loop(expr_loop) => parse_loop(expr_loop),
        Expr::ForLoop(expr_for) => parse_for(expr_for),
        Expr::Return(expr_return) => {
            let value = match &expr_return.expr {
                Some(e) => Some(parse_expr(e)?),
                None => None,
            };
            Ok(ParsedStmt::Return(value))
        }
        Expr::Break(_) => Ok(ParsedStmt::Break),
        Expr::Continue(_) => Ok(ParsedStmt::Continue),
        Expr::Assign(assign) => {
            let target = parse_expr(&assign.left)?;
            let value = parse_expr(&assign.right)?;
            Ok(ParsedStmt::Assign { target, value })
        }
        _ => Ok(ParsedStmt::Expr(parse_expr(expr)?)),
    }
}

/// Parse an if expression/statement.
fn parse_if(expr: &ExprIf) -> Result<ParsedStmt, ParseError> {
    let condition = parse_expr(&expr.cond)?;
    let then_branch = parse_block(&expr.then_branch)?;

    let else_branch = match &expr.else_branch {
        Some((_, else_expr)) => {
            match else_expr.as_ref() {
                Expr::Block(block) => Some(parse_block(&block.block)?),
                Expr::If(nested_if) => {
                    // else if - wrap in a single-statement block
                    Some(vec![parse_if(nested_if)?])
                }
                _ => None,
            }
        }
        None => None,
    };

    Ok(ParsedStmt::If {
        condition,
        then_branch,
        else_branch,
    })
}

/// Parse a while loop.
fn parse_while(expr: &ExprWhile) -> Result<ParsedStmt, ParseError> {
    let condition = parse_expr(&expr.cond)?;
    let body = parse_block(&expr.body)?;

    Ok(ParsedStmt::While { condition, body })
}

/// Parse an infinite loop.
fn parse_loop(expr: &ExprLoop) -> Result<ParsedStmt, ParseError> {
    let body = parse_block(&expr.body)?;
    Ok(ParsedStmt::Loop { body })
}

/// Parse a for loop: for i in start..end { body }
fn parse_for(expr: &ExprForLoop) -> Result<ParsedStmt, ParseError> {
    // Extract loop variable name
    let var = match &*expr.pat {
        Pat::Ident(pi) => pi.ident.to_string(),
        _ => {
            return Err(ParseError::Unsupported(
                "Complex for loop pattern".to_string(),
            ))
        }
    };

    // Parse the range expression (must be start..end or start..=end)
    let (start, end, inclusive) = match &*expr.expr {
        Expr::Range(ExprRange {
            start, end, limits, ..
        }) => {
            let start_expr = match start {
                Some(s) => parse_expr(s)?,
                None => ParsedExpr::IntLit(0), // Default to 0
            };
            let end_expr = match end {
                Some(e) => parse_expr(e)?,
                None => {
                    return Err(ParseError::Unsupported(
                        "Unbounded range in for loop".to_string(),
                    ))
                }
            };
            let inclusive = matches!(limits, syn::RangeLimits::Closed(_));
            (start_expr, end_expr, inclusive)
        }
        _ => {
            return Err(ParseError::Unsupported(
                "For loop requires range expression (start..end or start..=end)".to_string(),
            ))
        }
    };

    let body = parse_block(&expr.body)?;

    Ok(ParsedStmt::For {
        var,
        start,
        end,
        inclusive,
        body,
    })
}

/// Parse an expression.
pub fn parse_expr(expr: &Expr) -> Result<ParsedExpr, ParseError> {
    match expr {
        Expr::Lit(ExprLit { lit, .. }) => parse_lit(lit),

        Expr::Path(ExprPath { path, .. }) => {
            if let Some(ident) = path.get_ident() {
                Ok(ParsedExpr::Var(ident.to_string()))
            } else {
                // Could be a path like std::u64::MAX
                let path_str = quote::quote!(#path).to_string();
                Ok(ParsedExpr::Var(path_str))
            }
        }

        Expr::Binary(ExprBinary {
            op, left, right, ..
        }) => {
            let parsed_op = BinaryOp::from_syn(op).ok_or_else(|| {
                ParseError::Unsupported("Unsupported binary operator".to_string())
            })?;
            let left = Box::new(parse_expr(left)?);
            let right = Box::new(parse_expr(right)?);
            Ok(ParsedExpr::Binary {
                op: parsed_op,
                left,
                right,
            })
        }

        Expr::Unary(ExprUnary { op, expr, .. }) => {
            let parsed_op = UnaryOp::from_syn(op)
                .ok_or_else(|| ParseError::Unsupported("Unsupported unary operator".to_string()))?;
            let inner_expr = Box::new(parse_expr(expr)?);

            if parsed_op == UnaryOp::Deref {
                Ok(ParsedExpr::Deref(inner_expr))
            } else {
                Ok(ParsedExpr::Unary {
                    op: parsed_op,
                    expr: inner_expr,
                })
            }
        }

        Expr::Call(ExprCall { func, args, .. }) => {
            let func_name = match func.as_ref() {
                Expr::Path(p) => quote::quote!(#p).to_string(),
                _ => return Err(ParseError::Unsupported("Complex function call".to_string())),
            };

            let args: Vec<ParsedExpr> = args.iter().map(parse_expr).collect::<Result<_, _>>()?;

            Ok(ParsedExpr::Call {
                func: func_name,
                args,
            })
        }

        Expr::MethodCall(ExprMethodCall {
            receiver,
            method,
            args,
            ..
        }) => {
            let receiver = Box::new(parse_expr(receiver)?);
            let method_name = method.to_string();
            let args: Vec<ParsedExpr> = args.iter().map(parse_expr).collect::<Result<_, _>>()?;

            Ok(ParsedExpr::MethodCall {
                receiver,
                method: method_name,
                args,
            })
        }

        Expr::Index(ExprIndex { expr, index, .. }) => {
            let base = Box::new(parse_expr(expr)?);
            let index = Box::new(parse_expr(index)?);
            Ok(ParsedExpr::Index { base, index })
        }

        Expr::Block(ExprBlock { block, .. }) => {
            let stmts = parse_block(block)?;
            // Check if last statement is an expression
            let (stmts, expr) = if !stmts.is_empty() {
                if let Some(ParsedStmt::Expr(e)) = stmts.last() {
                    let expr = Some(Box::new(e.clone()));
                    let stmts = stmts[..stmts.len() - 1].to_vec();
                    (stmts, expr)
                } else {
                    (stmts, None)
                }
            } else {
                (stmts, None)
            };
            Ok(ParsedExpr::Block { stmts, expr })
        }

        Expr::If(expr_if) => {
            let condition = Box::new(parse_expr(&expr_if.cond)?);

            // For if expressions, we need to extract the final expression
            let then_stmts = parse_block(&expr_if.then_branch)?;
            let then_expr = if let Some(ParsedStmt::Expr(e)) = then_stmts.last() {
                Box::new(e.clone())
            } else {
                return Err(ParseError::Unsupported(
                    "If expression must have value".to_string(),
                ));
            };

            let else_expr = match &expr_if.else_branch {
                Some((_, else_branch)) => {
                    let else_parsed = parse_expr(else_branch)?;
                    Some(Box::new(else_parsed))
                }
                None => None,
            };

            Ok(ParsedExpr::If {
                condition,
                then_expr,
                else_expr,
            })
        }

        Expr::Paren(p) => parse_expr(&p.expr),

        Expr::Cast(c) => {
            let expr = Box::new(parse_expr(&c.expr)?);
            let ty = TypeInfo::from_syn(&c.ty);
            Ok(ParsedExpr::Cast { expr, ty })
        }

        Expr::Return(ret) => match &ret.expr {
            Some(e) => parse_expr(e),
            None => Ok(ParsedExpr::Var("()".to_string())),
        },

        _ => Err(ParseError::Unsupported(format!(
            "Expression type: {}",
            quote::quote!(#expr)
        ))),
    }
}

/// Parse a literal.
fn parse_lit(lit: &Lit) -> Result<ParsedExpr, ParseError> {
    match lit {
        Lit::Int(i) => {
            let value = i
                .base10_parse::<u64>()
                .map_err(|e| ParseError::Unsupported(format!("Integer literal: {}", e)))?;
            Ok(ParsedExpr::IntLit(value))
        }
        Lit::Float(f) => {
            let value = f
                .base10_parse::<f64>()
                .map_err(|e| ParseError::Unsupported(format!("Float literal: {}", e)))?;
            Ok(ParsedExpr::FloatLit(value))
        }
        Lit::Bool(b) => Ok(ParsedExpr::BoolLit(b.value)),
        Lit::Byte(b) => Ok(ParsedExpr::IntLit(b.value() as u64)),
        Lit::Char(c) => Ok(ParsedExpr::IntLit(c.value() as u64)),
        _ => Err(ParseError::Unsupported(
            "Unsupported literal type".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_function() {
        let source = r#"
            pub fn add(a: u64, b: u64) -> u64 {
                a + b
            }
        "#;

        let module = parse_module(source).unwrap();
        assert_eq!(module.functions.len(), 1);
        assert_eq!(module.functions[0].name, "add");
        assert_eq!(module.functions[0].params.len(), 2);
    }

    #[test]
    fn test_parse_with_control_flow() {
        let source = r#"
            pub fn factorial(n: u64) -> u64 {
                let mut result: u64 = 1;
                let mut i: u64 = n;
                while i > 0 {
                    result = result * i;
                    i = i - 1;
                }
                result
            }
        "#;

        let module = parse_module(source).unwrap();
        assert_eq!(module.functions.len(), 1);
        assert_eq!(module.functions[0].name, "factorial");
    }

    #[test]
    fn test_parse_if_else() {
        let source = r#"
            pub fn max(a: u64, b: u64) -> u64 {
                if a > b { a } else { b }
            }
        "#;

        let module = parse_module(source).unwrap();
        assert_eq!(module.functions.len(), 1);
    }
}
