//! Aeonmi IR (intermediate representation)
//! A small, desugared, deterministic representation for printing .ai and executing in the VM.

#![allow(dead_code)]

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub name: String,
    pub imports: Vec<Import>,
    pub decls: Vec<Decl>, // sorted deterministically by name
}

impl Default for Module {
    fn default() -> Self {
        Self {
            name: "aeonmi".to_string(),
            imports: Vec::new(),
            decls: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct Import {
    pub path: String,          // e.g., "std/io"
    pub alias: Option<String>, // e.g., "io"
}

#[derive(Debug, Clone, PartialEq)]
pub enum Decl {
    Const(ConstDecl),
    Let(LetDecl),
    Fn(FnDecl),
}

impl Decl {
    pub fn name(&self) -> &str {
        match self {
            Decl::Const(c) => &c.name,
            Decl::Let(l) => &l.name,
            Decl::Fn(f) => &f.name,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConstDecl {
    pub name: String,
    pub value: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LetDecl {
    pub name: String,
    pub value: Option<Expr>, // `let x;` or `let x = expr;`
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnDecl {
    pub name: String,
    pub params: Vec<String>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expr(Expr),
    Return(Option<Expr>),
    If {
        cond: Expr,
        then_block: Block,
        else_block: Option<Block>,
    },
    While {
        cond: Expr,
        body: Block,
    },
    For {
        // Desugared to while at lowering if needed; included for readability in IR.
        init: Option<Box<Stmt>>,
        cond: Option<Expr>,
        step: Option<Expr>,
        body: Block,
    },
    Let {
        name: String,
        value: Option<Expr>,
    },
    Assign {
        target: Expr, // Identifier or Index/Member in a future extension
        value: Expr,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Lit(Lit),
    Ident(String),
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
    Unary {
        op: UnOp,
        expr: Box<Expr>,
    },
    Array(Vec<Expr>),
    Object(Vec<(String, Expr)>), // simple map/object
}

#[derive(Debug, Clone, PartialEq)]
pub enum Lit {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg, Not,
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use BinOp::*;
        let s = match self {
            Add => "+", Sub => "-", Mul => "*", Div => "/", Mod => "%",
            Eq => "==", Ne => "!=", Lt => "<", Le => "<=", Gt => ">", Ge => ">=",
            And => "&&", Or => "||",
        };
        write!(f, "{}", s)
    }
}
