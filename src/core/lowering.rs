#![cfg_attr(test, allow(dead_code, unused_variables))]
//! Lowering: AST -> IR (desugaring + deterministic ordering)
use crate::core::ir::*;
use crate::core::TokenKind; // Import TokenKind from the core module

// Real AST -> IR lowering

pub fn lower_ast_to_ir(program: &crate::core::ast::ASTNode, name: &str) -> Result<Module, String> {
    use crate::core::ast::ASTNode;

    // Expect a Program at the top; if not, wrap as single-item program.
    let items: Vec<ASTNode> = match program {
        ASTNode::Program(v) => v.clone(),
        other => vec![other.clone()],
    };

    // No explicit import nodes yet; keep empty and deterministic.
    let imports: Vec<Import> = Vec::new();

    let mut decls: Vec<Decl> = Vec::new();
    let mut main_stmts: Vec<Stmt> = Vec::new();
    
    for item in items {
        match item {
            ASTNode::Function { name: fn_name, params, body, .. } => {
                let param_names: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                let mut stmts: Vec<Stmt> = Vec::new();
                for stmt_node in body {
                    stmts.push(lower_stmt_ast(&stmt_node)?);
                }
                decls.push(Decl::Fn(FnDecl {
                    name: fn_name,
                    params: param_names,
                    body: Block { stmts },
                }));
            }
            // All other top-level items go into main function
            other => {
                let stmt = lower_stmt_ast(&other)?;
                main_stmts.push(stmt);
            }
        }
    }

    // Create main function if there are any statements
    if !main_stmts.is_empty() {
        decls.push(Decl::Fn(FnDecl {
            name: "main".to_string(),
            params: vec![],
            body: Block { stmts: main_stmts },
        }));
    }

    let mut m = Module { name: name.to_string(), imports, decls };
    m.imports.sort_by(|a, b| {
        (a.path.as_str(), a.alias.as_deref()).cmp(&(b.path.as_str(), b.alias.as_deref()))
    });
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
    A::Assignment { name, value, .. } => Stmt::Assign {
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
            let init_stmt = init.as_ref().map(|b| lower_stmt_init_ast(b)).transpose()?;
            Stmt::For {
                init: init_stmt.map(Box::new),
                cond: condition.as_ref().map(|b| lower_expr_ast(b)).transpose()?,
                step: increment.as_ref().map(|b| lower_expr_ast(b)).transpose()?,
                body: lower_block_ast(body)?,
            }
        }

        // Decls at statement position
    A::VariableDecl { name, value, .. } => Stmt::Let {
            name: name.clone(),
            value: Some(lower_expr_ast(value)?),
        },

        // Function within a statement position: ignore/emit no-op (top-level handled elsewhere).
        A::Function { .. } => Stmt::Expr(Expr::Object(vec![])),

        A::QuantumOp { op, qubits } => {
            let (fname, args) = map_quantum_op(op, qubits)?;
            Stmt::Expr(Expr::Call {
                callee: Box::new(Expr::Ident(fname)),
                args,
            })
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
    A::IdentifierSpanned { name, .. } => Stmt::Expr(Expr::Ident(name.clone())),
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
    A::VariableDecl { name, value, .. } => Stmt::Let {
            name: name.clone(),
            value: Some(lower_expr_ast(value)?),
        },
    A::Assignment { name, value, .. } => Stmt::Assign {
            target: Expr::Ident(name.clone()),
            value: lower_expr_ast(value)?,
        },
        A::Return(expr) => Stmt::Return(Some(lower_expr_ast(expr)?)),
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

        A::UnaryExpr { op, expr } => Expr::Unary {
            op: map_unop_token(op),
            expr: Box::new(lower_expr_ast(expr)?),
        },

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

    A::IdentifierSpanned { name, .. } => Expr::Ident(name.clone()),

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
        "+" => Add, "-" => Sub, "*" => Mul, "/" => Div, "%" => Mod,
        "==" => Eq, "!=" => Ne, "<" => Lt, "<=" => Le, ">" => Gt, ">=" => Ge,
        "&&" => And, "||" => Or,
        _ => { eprintln!("[lowering] unknown binop `{}` -> Eq", op); Eq }
    }
}

fn map_unop(op: &str) -> UnOp {
    match op {
        "-" => UnOp::Neg,
        "!" => UnOp::Not,
        _ => { eprintln!("[lowering] unknown unop `{}` -> Not", op); UnOp::Not }
    }
}

#[allow(dead_code)]
fn map_binop_unused(op: &str) -> BinOp { map_binop(op) }

#[allow(dead_code)]
fn map_unop_unused(op: &str) -> UnOp { map_unop(op) }

// Updated to use TokenKind directly instead of the full path
fn map_binop_token(tok: &TokenKind) -> BinOp {
    use BinOp::*;
    match tok {
        TokenKind::Plus => Add,
        TokenKind::Minus => Sub,
        TokenKind::Star => Mul,
        TokenKind::Slash => Div,
        TokenKind::DoubleEquals => Eq,
        TokenKind::NotEquals => Ne,
        TokenKind::LessThan => Lt,
        TokenKind::LessEqual => Le,
        TokenKind::GreaterThan => Gt,
        TokenKind::GreaterEqual => Ge,
        _ => { eprintln!("[lowering] unmapped token binop `{:?}` -> Eq", tok); Eq }
    }
}

// Updated to use TokenKind directly instead of the full path
fn map_unop_token(tok: &TokenKind) -> UnOp {
    match tok {
        TokenKind::Minus => UnOp::Neg,
        _ => UnOp::Not,
    }
}

// Quantum op lowering helper - updated to use TokenKind directly
fn map_quantum_op(
    tok: &TokenKind,
    args: &Vec<crate::core::ast::ASTNode>,
) -> Result<(String, Vec<Expr>), String> {
    let fname = match tok {
        TokenKind::Superpose => "superpose",
        TokenKind::Entangle => "entangle",
        TokenKind::Measure => "measure",
        TokenKind::Dod => "dod",
        _ => "qop",
    }
    .to_string();

    let mut lowered = Vec::with_capacity(args.len());
    for a in args {
        lowered.push(lower_expr_ast(a)?);
    }
    Ok((fname, lowered))
}