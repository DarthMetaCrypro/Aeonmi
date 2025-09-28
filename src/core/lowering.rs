#![cfg_attr(test, allow(dead_code, unused_variables))]
//! Lowering: AST -> IR (desugaring + deterministic ordering)
<<<<<<< HEAD
<<<<<<< HEAD
//! Provides a stable IR surface regardless of AST evolution.
//!
//! Public entrypoints:
//!   - `lower_from_scratch(ScratchAst) -> Module`  (kept for tests)
//!   - `lower_ast_to_ir(&ast::ASTNode, name: &str) -> Result<Module, String>`
//!
//! Notes:
//!  * Unknown/unsupported constructs degrade gracefully.
//!  * QuantumOps & HieroglyphicOps lower to Calls (`superpose(...)`, `__glyph("…", ...)`).
//!  * `log(expr)` is the canonical side-effect call for AST `Log(expr)`.

use crate::core::ir::*;

// =======================
// Legacy "scratch" path
// =======================

=======
//! This file gives you a stable surface even if your external AST evolves.
=======
//! Provides a stable IR surface regardless of AST evolution.
>>>>>>> 42ca0eb (core: lexer/parser/token stabilized; tests passing; remove stray example)
//!
//! Public entrypoints:
//!   - `lower_from_scratch(ScratchAst) -> Module`  (kept for tests)
//!   - `lower_ast_to_ir(&ast::ASTNode, name: &str) -> Result<Module, String>`
//!
//! Notes:
//!  * Unknown/unsupported constructs degrade gracefully.
//!  * QuantumOps & HieroglyphicOps lower to Calls (`superpose(...)`, `__glyph("…", ...)`).
//!  * `log(expr)` is the canonical side-effect call for AST `Log(expr)`.

use crate::core::ir::*;

<<<<<<< HEAD
>>>>>>> 9543281 (feat: TUI editor + neon shell + hardened lexer (NFC, AI blocks, comments, tests))
=======
// =======================
// Legacy "scratch" path
// =======================

>>>>>>> 42ca0eb (core: lexer/parser/token stabilized; tests passing; remove stray example)
#[derive(Debug, Clone)]
pub struct ScratchAst {
    pub name: String,
    pub imports: Vec<(String, Option<String>)>,
    pub decls: Vec<ScratchDecl>,
}

#[derive(Debug, Clone)]
pub enum ScratchDecl {
    Const(String, ScratchExpr),
    Let(String, Option<ScratchExpr>),
    Fn {
        name: String,
        params: Vec<String>,
        body: Vec<ScratchStmt>,
    },
}

#[derive(Debug, Clone)]
pub enum ScratchStmt {
    Expr(ScratchExpr),
    Return(Option<ScratchExpr>),
    If { cond: ScratchExpr, then_body: Vec<ScratchStmt>, else_body: Vec<ScratchStmt> },
    While { cond: ScratchExpr, body: Vec<ScratchStmt> },
    Let(String, Option<ScratchExpr>),
    Assign(ScratchExpr, ScratchExpr),
}

#[derive(Debug, Clone)]
pub enum ScratchExpr {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Ident(String),
    Call(Box<ScratchExpr>, Vec<ScratchExpr>),
    Binary(Box<ScratchExpr>, &'static str, Box<ScratchExpr>),
    Unary(&'static str, Box<ScratchExpr>),
    Array(Vec<ScratchExpr>),
    Object(Vec<(String, ScratchExpr)>),
}

pub fn lower_from_scratch(ast: ScratchAst) -> Module {
    // Imports
<<<<<<< HEAD
<<<<<<< HEAD
=======
>>>>>>> 42ca0eb (core: lexer/parser/token stabilized; tests passing; remove stray example)
    let imports = ast
        .imports
        .into_iter()
        .map(|(p, a)| Import { path: p, alias: a })
        .collect();
<<<<<<< HEAD
=======
    let imports = ast.imports.into_iter().map(|(p, a)| Import { path: p, alias: a }).collect();
>>>>>>> 9543281 (feat: TUI editor + neon shell + hardened lexer (NFC, AI blocks, comments, tests))
=======
>>>>>>> 42ca0eb (core: lexer/parser/token stabilized; tests passing; remove stray example)

    // Decls
    let mut decls: Vec<Decl> = Vec::new();
    for d in ast.decls {
        match d {
            ScratchDecl::Const(name, e) => {
<<<<<<< HEAD
<<<<<<< HEAD
                decls.push(Decl::Const(ConstDecl { name, value: lower_expr_scratch(e) }));
            }
            ScratchDecl::Let(name, maybe_e) => {
                decls.push(Decl::Let(LetDecl { name, value: maybe_e.map(lower_expr_scratch) }));
=======
                decls.push(Decl::Const(ConstDecl { name, value: lower_expr(e) }));
            }
            ScratchDecl::Let(name, maybe_e) => {
                decls.push(Decl::Let(LetDecl { name, value: maybe_e.map(lower_expr) }));
>>>>>>> 9543281 (feat: TUI editor + neon shell + hardened lexer (NFC, AI blocks, comments, tests))
=======
                decls.push(Decl::Const(ConstDecl { name, value: lower_expr_scratch(e) }));
            }
            ScratchDecl::Let(name, maybe_e) => {
                decls.push(Decl::Let(LetDecl { name, value: maybe_e.map(lower_expr_scratch) }));
>>>>>>> 42ca0eb (core: lexer/parser/token stabilized; tests passing; remove stray example)
            }
            ScratchDecl::Fn { name, params, body } => {
                let mut stmts = Vec::new();
                for s in body {
<<<<<<< HEAD
<<<<<<< HEAD
                    stmts.push(lower_stmt_scratch(s));
=======
                    stmts.push(lower_stmt(s));
>>>>>>> 9543281 (feat: TUI editor + neon shell + hardened lexer (NFC, AI blocks, comments, tests))
=======
                    stmts.push(lower_stmt_scratch(s));
>>>>>>> 42ca0eb (core: lexer/parser/token stabilized; tests passing; remove stray example)
                }
                decls.push(Decl::Fn(FnDecl {
                    name,
                    params,
                    body: Block { stmts },
                }));
            }
        }
    }

    let mut m = Module { name: ast.name, imports, decls };
    // Deterministic sort
<<<<<<< HEAD
<<<<<<< HEAD
    m.imports
        .sort_by(|a, b| (a.path.as_str(), a.alias.as_deref()).cmp(&(b.path.as_str(), b.alias.as_deref())));
=======
    m.imports.sort_by(|a, b| (a.path.as_str(), a.alias.as_deref()).cmp(&(b.path.as_str(), b.alias.as_deref())));
>>>>>>> 9543281 (feat: TUI editor + neon shell + hardened lexer (NFC, AI blocks, comments, tests))
=======
    m.imports
        .sort_by(|a, b| (a.path.as_str(), a.alias.as_deref()).cmp(&(b.path.as_str(), b.alias.as_deref())));
>>>>>>> 42ca0eb (core: lexer/parser/token stabilized; tests passing; remove stray example)
    m.decls.sort_by(|a, b| a.name().cmp(b.name()));
    m
}

<<<<<<< HEAD
<<<<<<< HEAD
fn lower_stmt_scratch(s: ScratchStmt) -> Stmt {
    use ScratchStmt::*;
    match s {
        Expr(e) => Stmt::Expr(lower_expr_scratch(e)),
        Return(e) => Stmt::Return(e.map(lower_expr_scratch)),
        If { cond, then_body, else_body } => Stmt::If {
            cond: lower_expr_scratch(cond),
            then_block: Block { stmts: then_body.into_iter().map(lower_stmt_scratch).collect() },
            else_block: if else_body.is_empty() {
                None
            } else {
                Some(Block { stmts: else_body.into_iter().map(lower_stmt_scratch).collect() })
            },
        },
        While { cond, body } => Stmt::While {
            cond: lower_expr_scratch(cond),
            body: Block { stmts: body.into_iter().map(lower_stmt_scratch).collect() },
        },
        Let(name, v) => Stmt::Let { name, value: v.map(lower_expr_scratch) },
        Assign(a, b) => Stmt::Assign { target: lower_expr_scratch(a), value: lower_expr_scratch(b) },
    }
}

fn lower_expr_scratch(e: ScratchExpr) -> Expr {
=======
fn lower_stmt(s: ScratchStmt) -> Stmt {
=======
fn lower_stmt_scratch(s: ScratchStmt) -> Stmt {
>>>>>>> 42ca0eb (core: lexer/parser/token stabilized; tests passing; remove stray example)
    use ScratchStmt::*;
    match s {
        Expr(e) => Stmt::Expr(lower_expr_scratch(e)),
        Return(e) => Stmt::Return(e.map(lower_expr_scratch)),
        If { cond, then_body, else_body } => Stmt::If {
            cond: lower_expr_scratch(cond),
            then_block: Block { stmts: then_body.into_iter().map(lower_stmt_scratch).collect() },
            else_block: if else_body.is_empty() {
                None
            } else {
                Some(Block { stmts: else_body.into_iter().map(lower_stmt_scratch).collect() })
            },
        },
        While { cond, body } => Stmt::While {
            cond: lower_expr_scratch(cond),
            body: Block { stmts: body.into_iter().map(lower_stmt_scratch).collect() },
        },
        Let(name, v) => Stmt::Let { name, value: v.map(lower_expr_scratch) },
        Assign(a, b) => Stmt::Assign { target: lower_expr_scratch(a), value: lower_expr_scratch(b) },
    }
}

<<<<<<< HEAD
fn lower_expr(e: ScratchExpr) -> Expr {
>>>>>>> 9543281 (feat: TUI editor + neon shell + hardened lexer (NFC, AI blocks, comments, tests))
=======
fn lower_expr_scratch(e: ScratchExpr) -> Expr {
>>>>>>> 42ca0eb (core: lexer/parser/token stabilized; tests passing; remove stray example)
    use ScratchExpr::*;
    match e {
        Null => Expr::Lit(Lit::Null),
        Bool(b) => Expr::Lit(Lit::Bool(b)),
        Number(n) => Expr::Lit(Lit::Number(n)),
        String(s) => Expr::Lit(Lit::String(s)),
        Ident(s) => Expr::Ident(s),
<<<<<<< HEAD
<<<<<<< HEAD
=======
>>>>>>> 42ca0eb (core: lexer/parser/token stabilized; tests passing; remove stray example)
        Call(c, args) => Expr::Call {
            callee: Box::new(lower_expr_scratch(*c)),
            args: args.into_iter().map(lower_expr_scratch).collect(),
        },
        Binary(l, op, r) => Expr::Binary {
            left: Box::new(lower_expr_scratch(*l)),
            op: map_binop(op),
            right: Box::new(lower_expr_scratch(*r)),
        },
        Unary(op, x) => Expr::Unary { op: map_unop(op), expr: Box::new(lower_expr_scratch(*x)) },
        Array(xs) => Expr::Array(xs.into_iter().map(lower_expr_scratch).collect()),
        Object(kvs) => Expr::Object(kvs.into_iter().map(|(k, v)| (k, lower_expr_scratch(v))).collect()),
<<<<<<< HEAD
    }
}

// =======================
// Real AST -> IR lowering
// =======================

pub fn lower_ast_to_ir(
    program: &crate::core::ast::ASTNode,
    name: &str,
) -> Result<Module, String> {
    use crate::core::ast::ASTNode;

    // Expect a Program at the top; if not, wrap as single-item program.
    let items: Vec<ASTNode> = match program {
        ASTNode::Program(v) => v.clone(),
        other => vec![other.clone()],
    };

    // No explicit import nodes yet; keep empty and deterministic.
    let imports: Vec<Import> = Vec::new();

    let mut decls: Vec<Decl> = Vec::new();
    for item in items {
        match item {
            ASTNode::Function { name: fn_name, params, body } => {
                let mut stmts: Vec<Stmt> = Vec::new();
                for stmt_node in body {
                    stmts.push(lower_stmt_ast(&stmt_node)?);
                }
                decls.push(Decl::Fn(FnDecl {
                    name: fn_name,
                    params,
                    body: Block { stmts },
                }));
            }
            ASTNode::VariableDecl { name, value } => {
                decls.push(Decl::Let(LetDecl { name, value: Some(lower_expr_ast(&value)?)}));
            }
            // Top-level statements become a synthetic fn `main`.
            other => {
                let stmt = lower_stmt_ast(&other)?;
                if let Some(Decl::Fn(f)) =
                    decls.iter_mut().find(|d| matches!(d, Decl::Fn(FnDecl { name, .. }) if name == "main"))
                {
                    f.body.stmts.push(stmt);
                } else {
                    decls.push(Decl::Fn(FnDecl {
                        name: "main".to_string(),
                        params: vec![],
                        body: Block { stmts: vec![stmt] },
                    }));
                }
            }
        }
    }

    let mut m = Module { name: name.to_string(), imports, decls };
    m.imports
        .sort_by(|a, b| (a.path.as_str(), a.alias.as_deref()).cmp(&(b.path.as_str(), b.alias.as_deref())));
    m.decls.sort_by(|a, b| a.name().cmp(b.name()));
    Ok(m)
}

fn lower_stmt_ast(n: &crate::core::ast::ASTNode) -> Result<Stmt, String> {
    use crate::core::ast::ASTNode as A;

    Ok(match n {
        A::Block(items) => {
            // Blocks are handled by `lower_block_ast` at call sites; produce a no-op here.
            let _ = items;
            Stmt::Expr(Expr::Object(vec![]))
        }

        A::Return(expr) => Stmt::Return(Some(lower_expr_ast(expr)?)),
        A::Log(expr) => Stmt::Expr(Expr::Call {
            callee: Box::new(Expr::Ident("log".into())),
            args: vec![lower_expr_ast(expr)?],
        }),
        A::Assignment { name, value } => Stmt::Assign {
            target: Expr::Ident(name.clone()),
            value: lower_expr_ast(value)?,
        },
        A::Call { .. }
        | A::BinaryExpr { .. }
        | A::UnaryExpr { .. }
        | A::Identifier(_)
        | A::NumberLiteral(_)
        | A::StringLiteral(_)
        | A::BooleanLiteral(_) => Stmt::Expr(lower_expr_ast(n)?),

        A::If { condition, then_branch, else_branch } => Stmt::If {
            cond: lower_expr_ast(condition)?,
            then_block: lower_block_ast(then_branch)?,
            else_block: else_branch.as_ref().map(|e| lower_block_ast(e)).transpose()?,
        },

        A::While { condition, body } => Stmt::While {
            cond: lower_expr_ast(condition)?,
            body: lower_block_ast(body)?,
        },

        A::For { init, condition, increment, body } => {
            let init_stmt = init.as_ref().map(|b| lower_stmt_init_ast(&**b)).transpose()?;
            Stmt::For {
                init: init_stmt.map(Box::new),
                cond: condition.as_ref().map(|b| lower_expr_ast(b)).transpose()?,
                step: increment.as_ref().map(|b| lower_expr_ast(b)).transpose()?,
                body: lower_block_ast(body)?,
            }
        }

        // Decls at statement position
        A::VariableDecl { name, value } => Stmt::Let { name: name.clone(), value: Some(lower_expr_ast(value)?) },

        // Function within a statement position: ignore/emit no-op (top-level handled elsewhere).
        A::Function { .. } => Stmt::Expr(Expr::Object(vec![])),

        A::QuantumOp { op, qubits } => {
            let (fname, args) = map_quantum_op(op, qubits)?;
            Stmt::Expr(Expr::Call { callee: Box::new(Expr::Ident(fname)), args })
        }

        A::HieroglyphicOp { symbol, args } => Stmt::Expr(Expr::Call {
            callee: Box::new(Expr::Ident("__glyph".into())),
            args: {
                let mut v = Vec::with_capacity(args.len() + 1);
                v.push(Expr::Lit(Lit::String(symbol.clone())));
                for a in args {
                    v.push(lower_expr_ast(a)?);
                }
                v
            },
        }),

        A::Error(msg) => Stmt::Expr(Expr::Lit(Lit::String(format!("/* error: {msg} */")))),
        A::Program(_) => unreachable!("Program nodes are handled at the top level"),
    })
}

fn lower_block_ast(n: &crate::core::ast::ASTNode) -> Result<Block, String> {
    use crate::core::ast::ASTNode as A;
    match n {
        A::Block(items) => {
            let mut stmts = Vec::new();
            for it in items {
                stmts.push(lower_stmt_ast(it)?);
            }
            Ok(Block { stmts })
        }
        other => Ok(Block { stmts: vec![lower_stmt_ast(other)?] }),
    }
}

fn lower_stmt_init_ast(n: &crate::core::ast::ASTNode) -> Result<Stmt, String> {
    use crate::core::ast::ASTNode as A;
    Ok(match n {
        A::VariableDecl { name, value } => Stmt::Let { name: name.clone(), value: Some(lower_expr_ast(value)?) },
        A::Assignment { name, value } => Stmt::Assign { target: Expr::Ident(name.clone()), value: lower_expr_ast(value)? },
        A::Return(expr) => Stmt::Return(Some(lower_expr_ast(expr)?)),
        // Add more arms as needed for other variants, or fallback:
        _ => Stmt::Expr(lower_expr_ast(n)?),
    })
}

fn lower_expr_ast(n: &crate::core::ast::ASTNode) -> Result<Expr, String> {
    use crate::core::ast::ASTNode as A;

    Ok(match n {
        A::Identifier(s) => Expr::Ident(s.clone()),
        A::NumberLiteral(n) => Expr::Lit(Lit::Number(*n)),
        A::StringLiteral(s) => Expr::Lit(Lit::String(s.clone())),
        A::BooleanLiteral(b) => Expr::Lit(Lit::Bool(*b)),

        A::UnaryExpr { op, expr } => Expr::Unary { op: map_unop_token(op), expr: Box::new(lower_expr_ast(expr)?) },

        A::BinaryExpr { op, left, right } => Expr::Binary {
            left: Box::new(lower_expr_ast(left)?),
            op: map_binop_token(op),
            right: Box::new(lower_expr_ast(right)?),
        },

        // Assignment is not an expression in IR; degrade to a no-op value.
        A::Assignment { .. } => Expr::Object(vec![]),

        A::Call { callee, args } => Expr::Call {
            callee: Box::new(lower_expr_ast(callee)?),
            args: args.iter().map(|a| lower_expr_ast(a)).collect::<Result<Vec<_>, _>>()?,
        },

        A::Log(e) => Expr::Call {
            callee: Box::new(Expr::Ident("log".into())),
            args: vec![lower_expr_ast(e)?],
        },

        A::QuantumOp { op, qubits } => {
            let (fname, args) = map_quantum_op(op, qubits)?;
            Expr::Call { callee: Box::new(Expr::Ident(fname)), args }
        }

        A::HieroglyphicOp { symbol, args } => Expr::Call {
            callee: Box::new(Expr::Ident("__glyph".into())),
            args: {
                let mut v = Vec::with_capacity(args.len() + 1);
                v.push(Expr::Lit(Lit::String(symbol.clone())));
                for a in args {
                    v.push(lower_expr_ast(a)?);
                }
                v
            },
        },

    // Blocks/If/While/For/Function/VariableDecl/Return shouldn’t appear as pure Expr; surface as empty object.
    A::Block(_)
    | A::If { .. }
    | A::While { .. }
    | A::For { .. }
    | A::Function { .. }
    | A::VariableDecl { .. }
    | A::Return(_) 
    | A::Program(_) => Expr::Object(vec![]),

        A::Error(msg) => Expr::Lit(Lit::String(format!("/* error: {msg} */"))),
    })
}

// =======================
// Operator mapping
// =======================

fn map_binop(op: &str) -> BinOp {
    use BinOp::*;
    match op {
        "+" => Add,
        "-" => Sub,
        "*" => Mul,
        "/" => Div,
        "%" => Mod,
        "==" => Eq,
        "!=" => Ne,
        "<" => Lt,
        "<=" => Le,
        ">" => Gt,
        ">=" => Ge,
        "&&" => And,
        "||" => Or,
        // "=" (assignment) is not a BinOp in IR; handled as Stmt::Assign.
        _ => {
            eprintln!("[lowering] unknown binop `{}` -> Eq", op);
=======
        Call(c, args) => Expr::Call { callee: Box::new(lower_expr(*c)), args: args.into_iter().map(lower_expr).collect() },
        Binary(l, op, r) => Expr::Binary { left: Box::new(lower_expr(*l)), op: map_binop(op), right: Box::new(lower_expr(*r)) },
        Unary(op, x) => Expr::Unary { op: map_unop(op), expr: Box::new(lower_expr(*x)) },
        Array(xs) => Expr::Array(xs.into_iter().map(lower_expr).collect()),
        Object(kvs) => Expr::Object(kvs.into_iter().map(|(k, v)| (k, lower_expr(v))).collect()),
=======
>>>>>>> 42ca0eb (core: lexer/parser/token stabilized; tests passing; remove stray example)
    }
}

// =======================
// Real AST -> IR lowering
// =======================

pub fn lower_ast_to_ir(
    program: &crate::core::ast::ASTNode,
    name: &str,
) -> Result<Module, String> {
    use crate::core::ast::ASTNode;

    // Expect a Program at the top; if not, wrap as single-item program.
    let items: Vec<ASTNode> = match program {
        ASTNode::Program(v) => v.clone(),
        other => vec![other.clone()],
    };

    // No explicit import nodes yet; keep empty and deterministic.
    let imports: Vec<Import> = Vec::new();

    let mut decls: Vec<Decl> = Vec::new();
    for item in items {
        match item {
            ASTNode::Function { name: fn_name, params, body } => {
                let mut stmts: Vec<Stmt> = Vec::new();
                for stmt_node in body {
                    stmts.push(lower_stmt_ast(&stmt_node)?);
                }
                decls.push(Decl::Fn(FnDecl {
                    name: fn_name,
                    params,
                    body: Block { stmts },
                }));
            }
            ASTNode::VariableDecl { name, value } => {
                decls.push(Decl::Let(LetDecl { name, value: Some(lower_expr_ast(&value)?)}));
            }
            // Top-level statements become a synthetic fn `main`.
            other => {
                let stmt = lower_stmt_ast(&other)?;
                if let Some(Decl::Fn(f)) =
                    decls.iter_mut().find(|d| matches!(d, Decl::Fn(FnDecl { name, .. }) if name == "main"))
                {
                    f.body.stmts.push(stmt);
                } else {
                    decls.push(Decl::Fn(FnDecl {
                        name: "main".to_string(),
                        params: vec![],
                        body: Block { stmts: vec![stmt] },
                    }));
                }
            }
        }
    }

    let mut m = Module { name: name.to_string(), imports, decls };
    m.imports
        .sort_by(|a, b| (a.path.as_str(), a.alias.as_deref()).cmp(&(b.path.as_str(), b.alias.as_deref())));
    m.decls.sort_by(|a, b| a.name().cmp(b.name()));
    Ok(m)
}

fn lower_stmt_ast(n: &crate::core::ast::ASTNode) -> Result<Stmt, String> {
    use crate::core::ast::ASTNode as A;

    Ok(match n {
        A::Block(items) => {
            // Blocks are handled by `lower_block_ast` at call sites; produce a no-op here.
            let _ = items;
            Stmt::Expr(Expr::Object(vec![]))
        }

        A::Return(expr) => Stmt::Return(Some(lower_expr_ast(expr)?)),
        A::Log(expr) => Stmt::Expr(Expr::Call {
            callee: Box::new(Expr::Ident("log".into())),
            args: vec![lower_expr_ast(expr)?],
        }),
        A::Assignment { name, value } => Stmt::Assign {
            target: Expr::Ident(name.clone()),
            value: lower_expr_ast(value)?,
        },
        A::Call { .. }
        | A::BinaryExpr { .. }
        | A::UnaryExpr { .. }
        | A::Identifier(_)
        | A::NumberLiteral(_)
        | A::StringLiteral(_)
        | A::BooleanLiteral(_) => Stmt::Expr(lower_expr_ast(n)?),

        A::If { condition, then_branch, else_branch } => Stmt::If {
            cond: lower_expr_ast(condition)?,
            then_block: lower_block_ast(then_branch)?,
            else_block: else_branch.as_ref().map(|e| lower_block_ast(e)).transpose()?,
        },

        A::While { condition, body } => Stmt::While {
            cond: lower_expr_ast(condition)?,
            body: lower_block_ast(body)?,
        },

        A::For { init, condition, increment, body } => {
            let init_stmt = init.as_ref().map(|b| lower_stmt_init_ast(&**b)).transpose()?;
            Stmt::For {
                init: init_stmt.map(Box::new),
                cond: condition.as_ref().map(|b| lower_expr_ast(b)).transpose()?,
                step: increment.as_ref().map(|b| lower_expr_ast(b)).transpose()?,
                body: lower_block_ast(body)?,
            }
        }

        // Decls at statement position
        A::VariableDecl { name, value } => Stmt::Let { name: name.clone(), value: Some(lower_expr_ast(value)?) },

        // Function within a statement position: ignore/emit no-op (top-level handled elsewhere).
        A::Function { .. } => Stmt::Expr(Expr::Object(vec![])),

        A::QuantumOp { op, qubits } => {
            let (fname, args) = map_quantum_op(op, qubits)?;
            Stmt::Expr(Expr::Call { callee: Box::new(Expr::Ident(fname)), args })
        }

        A::HieroglyphicOp { symbol, args } => Stmt::Expr(Expr::Call {
            callee: Box::new(Expr::Ident("__glyph".into())),
            args: {
                let mut v = Vec::with_capacity(args.len() + 1);
                v.push(Expr::Lit(Lit::String(symbol.clone())));
                for a in args {
                    v.push(lower_expr_ast(a)?);
                }
                v
            },
        }),

        A::Error(msg) => Stmt::Expr(Expr::Lit(Lit::String(format!("/* error: {msg} */")))),
        A::Program(_) => unreachable!("Program nodes are handled at the top level"),
    })
}

fn lower_block_ast(n: &crate::core::ast::ASTNode) -> Result<Block, String> {
    use crate::core::ast::ASTNode as A;
    match n {
        A::Block(items) => {
            let mut stmts = Vec::new();
            for it in items {
                stmts.push(lower_stmt_ast(it)?);
            }
            Ok(Block { stmts })
        }
        other => Ok(Block { stmts: vec![lower_stmt_ast(other)?] }),
    }
}

fn lower_stmt_init_ast(n: &crate::core::ast::ASTNode) -> Result<Stmt, String> {
    use crate::core::ast::ASTNode as A;
    Ok(match n {
        A::VariableDecl { name, value } => Stmt::Let { name: name.clone(), value: Some(lower_expr_ast(value)?) },
        A::Assignment { name, value } => Stmt::Assign { target: Expr::Ident(name.clone()), value: lower_expr_ast(value)? },
        A::Return(expr) => Stmt::Return(Some(lower_expr_ast(expr)?)),
        // Add more arms as needed for other variants, or fallback:
        _ => Stmt::Expr(lower_expr_ast(n)?),
    })
}

fn lower_expr_ast(n: &crate::core::ast::ASTNode) -> Result<Expr, String> {
    use crate::core::ast::ASTNode as A;

    Ok(match n {
        A::Identifier(s) => Expr::Ident(s.clone()),
        A::NumberLiteral(n) => Expr::Lit(Lit::Number(*n)),
        A::StringLiteral(s) => Expr::Lit(Lit::String(s.clone())),
        A::BooleanLiteral(b) => Expr::Lit(Lit::Bool(*b)),

        A::UnaryExpr { op, expr } => Expr::Unary { op: map_unop_token(op), expr: Box::new(lower_expr_ast(expr)?) },

        A::BinaryExpr { op, left, right } => Expr::Binary {
            left: Box::new(lower_expr_ast(left)?),
            op: map_binop_token(op),
            right: Box::new(lower_expr_ast(right)?),
        },

        // Assignment is not an expression in IR; degrade to a no-op value.
        A::Assignment { .. } => Expr::Object(vec![]),

        A::Call { callee, args } => Expr::Call {
            callee: Box::new(lower_expr_ast(callee)?),
            args: args.iter().map(|a| lower_expr_ast(a)).collect::<Result<Vec<_>, _>>()?,
        },

        A::Log(e) => Expr::Call {
            callee: Box::new(Expr::Ident("log".into())),
            args: vec![lower_expr_ast(e)?],
        },

        A::QuantumOp { op, qubits } => {
            let (fname, args) = map_quantum_op(op, qubits)?;
            Expr::Call { callee: Box::new(Expr::Ident(fname)), args }
        }

        A::HieroglyphicOp { symbol, args } => Expr::Call {
            callee: Box::new(Expr::Ident("__glyph".into())),
            args: {
                let mut v = Vec::with_capacity(args.len() + 1);
                v.push(Expr::Lit(Lit::String(symbol.clone())));
                for a in args {
                    v.push(lower_expr_ast(a)?);
                }
                v
            },
        },

    // Blocks/If/While/For/Function/VariableDecl/Return shouldn’t appear as pure Expr; surface as empty object.
    A::Block(_)
    | A::If { .. }
    | A::While { .. }
    | A::For { .. }
    | A::Function { .. }
    | A::VariableDecl { .. }
    | A::Return(_) 
    | A::Program(_) => Expr::Object(vec![]),

        A::Error(msg) => Expr::Lit(Lit::String(format!("/* error: {msg} */"))),
    })
}

// =======================
// Operator mapping
// =======================

fn map_binop(op: &str) -> BinOp {
    use BinOp::*;
    match op {
<<<<<<< HEAD
        "+" => Add, "-" => Sub, "*" => Mul, "/" => Div, "%" => Mod,
        "==" => Eq, "!=" => Ne, "<" => Lt, "<=" => Le, ">" => Gt, ">=" => Ge,
        "&&" => And, "||" => Or,
        other => {
            // Unknown => default to Eq for now; adjust as grammar expands
            eprintln!("[lowering] unknown binop `{}` -> Eq", other);
>>>>>>> 9543281 (feat: TUI editor + neon shell + hardened lexer (NFC, AI blocks, comments, tests))
=======
        "+" => Add,
        "-" => Sub,
        "*" => Mul,
        "/" => Div,
        "%" => Mod,
        "==" => Eq,
        "!=" => Ne,
        "<" => Lt,
        "<=" => Le,
        ">" => Gt,
        ">=" => Ge,
        "&&" => And,
        "||" => Or,
        // "=" (assignment) is not a BinOp in IR; handled as Stmt::Assign.
        _ => {
            eprintln!("[lowering] unknown binop `{}` -> Eq", op);
>>>>>>> 42ca0eb (core: lexer/parser/token stabilized; tests passing; remove stray example)
            Eq
        }
    }
}

fn map_unop(op: &str) -> UnOp {
    match op {
        "-" => UnOp::Neg,
        "!" => UnOp::Not,
<<<<<<< HEAD
<<<<<<< HEAD
        _ => {
            eprintln!("[lowering] unknown unop `{}` -> Not", op);
=======
        other => {
            eprintln!("[lowering] unknown unop `{}` -> Not", other);
>>>>>>> 9543281 (feat: TUI editor + neon shell + hardened lexer (NFC, AI blocks, comments, tests))
=======
        _ => {
            eprintln!("[lowering] unknown unop `{}` -> Not", op);
>>>>>>> 42ca0eb (core: lexer/parser/token stabilized; tests passing; remove stray example)
            UnOp::Not
        }
    }
}
<<<<<<< HEAD
<<<<<<< HEAD
=======
>>>>>>> 42ca0eb (core: lexer/parser/token stabilized; tests passing; remove stray example)

// Map from token kinds in real AST to IR ops.
// Adjust to actual TokenKind variants present in your grammar.
fn map_binop_token(tok: &crate::core::token::TokenKind) -> BinOp {
    use crate::core::token::TokenKind as T;
    use BinOp::*;
    match tok {
        T::Plus => Add,
        T::Minus => Sub,
        T::Star => Mul,
        T::Slash => Div,
        T::DoubleEquals => Eq,
        T::NotEquals => Ne,
        T::LessThan => Lt,
        T::LessEqual => Le,
        T::GreaterThan => Gt,
        T::GreaterEqual => Ge,
        // Anything else (including assignment `Equals`) falls back to Eq safely.
        _ => {
            eprintln!("[lowering] unmapped token binop `{:?}` -> Eq", tok);
            Eq
        }
    }
}

fn map_unop_token(tok: &crate::core::token::TokenKind) -> UnOp {
    use crate::core::token::TokenKind as T;
    match tok {
        T::Minus => UnOp::Neg,
        // If a "not" token variant exists, map it here; otherwise fall back.
        _ => UnOp::Not,
    }
}

// Quantum op lowering helper
fn map_quantum_op(
    tok: &crate::core::token::TokenKind,
    args: &Vec<Box<crate::core::ast::ASTNode>>,
) -> Result<(String, Vec<Expr>), String> {
    use crate::core::token::TokenKind as T;
    let fname = match tok {
        T::Superpose => "superpose",
        T::Entangle => "entangle",
        T::Measure => "measure",
        T::Dod => "dod",
        _ => "qop",
    }
    .to_string();

    let mut lowered = Vec::with_capacity(args.len());
    for a in args {
        lowered.push(lower_expr_ast(a)?);
    }
    Ok((fname, lowered))
}
<<<<<<< HEAD
=======
>>>>>>> 9543281 (feat: TUI editor + neon shell + hardened lexer (NFC, AI blocks, comments, tests))
=======
>>>>>>> 42ca0eb (core: lexer/parser/token stabilized; tests passing; remove stray example)
