#![cfg_attr(test, allow(dead_code, unused_variables))]
//! Lowering: AST -> IR (desugaring + deterministic ordering)
//! This file gives you a stable surface even if your external AST evolves.
//!
//! If you already have `crate::core::ast`, implement `From<ast::Program>`;
//! if not, use `lower_from_scratch` for quick tests.

use crate::core::ir::*;

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
    let imports = ast.imports.into_iter().map(|(p, a)| Import { path: p, alias: a }).collect();

    // Decls
    let mut decls: Vec<Decl> = Vec::new();
    for d in ast.decls {
        match d {
            ScratchDecl::Const(name, e) => {
                decls.push(Decl::Const(ConstDecl { name, value: lower_expr(e) }));
            }
            ScratchDecl::Let(name, maybe_e) => {
                decls.push(Decl::Let(LetDecl { name, value: maybe_e.map(lower_expr) }));
            }
            ScratchDecl::Fn { name, params, body } => {
                let mut stmts = Vec::new();
                for s in body {
                    stmts.push(lower_stmt(s));
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
    m.imports.sort_by(|a, b| (a.path.as_str(), a.alias.as_deref()).cmp(&(b.path.as_str(), b.alias.as_deref())));
    m.decls.sort_by(|a, b| a.name().cmp(b.name()));
    m
}

fn lower_stmt(s: ScratchStmt) -> Stmt {
    use ScratchStmt::*;
    match s {
        Expr(e) => Stmt::Expr(lower_expr(e)),
        Return(e) => Stmt::Return(e.map(lower_expr)),
        If { cond, then_body, else_body } => {
            Stmt::If {
                cond: lower_expr(cond),
                then_block: Block { stmts: then_body.into_iter().map(lower_stmt).collect() },
                else_block: if else_body.is_empty() { None } else { Some(Block { stmts: else_body.into_iter().map(lower_stmt).collect() }) },
            }
        }
        While { cond, body } => {
            Stmt::While { cond: lower_expr(cond), body: Block { stmts: body.into_iter().map(lower_stmt).collect() } }
        }
        Let(name, v) => Stmt::Let { name, value: v.map(lower_expr) },
        Assign(a, b) => Stmt::Assign { target: lower_expr(a), value: lower_expr(b) },
    }
}

fn lower_expr(e: ScratchExpr) -> Expr {
    use ScratchExpr::*;
    match e {
        Null => Expr::Lit(Lit::Null),
        Bool(b) => Expr::Lit(Lit::Bool(b)),
        Number(n) => Expr::Lit(Lit::Number(n)),
        String(s) => Expr::Lit(Lit::String(s)),
        Ident(s) => Expr::Ident(s),
        Call(c, args) => Expr::Call { callee: Box::new(lower_expr(*c)), args: args.into_iter().map(lower_expr).collect() },
        Binary(l, op, r) => Expr::Binary { left: Box::new(lower_expr(*l)), op: map_binop(op), right: Box::new(lower_expr(*r)) },
        Unary(op, x) => Expr::Unary { op: map_unop(op), expr: Box::new(lower_expr(*x)) },
        Array(xs) => Expr::Array(xs.into_iter().map(lower_expr).collect()),
        Object(kvs) => Expr::Object(kvs.into_iter().map(|(k, v)| (k, lower_expr(v))).collect()),
    }
}

fn map_binop(op: &str) -> BinOp {
    use BinOp::*;
    match op {
        "+" => Add, "-" => Sub, "*" => Mul, "/" => Div, "%" => Mod,
        "==" => Eq, "!=" => Ne, "<" => Lt, "<=" => Le, ">" => Gt, ">=" => Ge,
        "&&" => And, "||" => Or,
        other => {
            // Unknown => default to Eq for now; adjust as grammar expands
            eprintln!("[lowering] unknown binop `{}` -> Eq", other);
            Eq
        }
    }
}

fn map_unop(op: &str) -> UnOp {
    match op {
        "-" => UnOp::Neg,
        "!" => UnOp::Not,
        other => {
            eprintln!("[lowering] unknown unop `{}` -> Not", other);
            UnOp::Not
        }
    }
}
