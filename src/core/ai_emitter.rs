#![cfg_attr(test, allow(dead_code, unused_variables))]
//! Canonical .ai emitter for Aeonmi IR.
//! - Deterministic output: sorted imports/decls, 2-space indent, LF
//! - Header embeds FNV-1a 64-bit hash of body for reproducibility

use crate::core::ir::*;
use crate::core::lowering::lower_ast_to_ir;
use std::fmt::Write as _;

/// Thin wrapper to emit canonical `.ai` from IR or AST.
#[derive(Default)]
pub struct AiEmitter;

impl AiEmitter {
    pub fn new() -> Self {
        Self
    }

    /// Preferred entrypoint once you have IR.
    pub fn generate_from_ir(&mut self, module: &Module) -> Result<String, String> {
        Ok(emit_ai(module))
    }

    /// Entry point: emit canonical `.ai` directly from AST.
    /// Uses `lower_ast_to_ir` to produce IR, then pretty-prints via `emit_ai`.
    pub fn generate(&mut self, ast: &crate::core::ast::ASTNode) -> Result<String, String> {
        let module = lower_ast_to_ir(ast, "main").map_err(|e| format!("lowering error: {e}"))?;
        self.generate_from_ir(&module)
    }
}

// ===== Legacy functional API =====

/// Produces deterministic output (sorted decls/imports, 2-space indent, LF)
/// Header includes a simple FNV-1a 64-bit hash of the body.
pub fn emit_ai(module: &Module) -> String {
    // 1) Body
    let mut body = String::new();
    write_module(&mut body, module);

    // 2) Hash header
    let hash = fnv1a64(body.as_bytes());

    // 3) Final output
    let mut out = String::new();
    writeln!(&mut out, "// aeonmi:1").unwrap();
    writeln!(&mut out, "// hash:{:016x}", hash).unwrap();
    writeln!(&mut out, "// tool:aeonmi unknown").unwrap();
    out.push('\n');
    out.push_str(&body);
    out
}

fn write_module(dst: &mut String, m: &Module) {
    // Imports first (re-sort defensively)
    let mut imports = m.imports.clone();
    imports.sort_by(|a, b| {
        (a.path.as_str(), a.alias.as_deref()).cmp(&(b.path.as_str(), b.alias.as_deref()))
    });
    for im in imports {
        if let Some(alias) = im.alias {
            writeln!(dst, "import {} as {};", escape_sym(&im.path), escape_sym(&alias)).unwrap();
        } else {
            writeln!(dst, "import {};", escape_sym(&im.path)).unwrap();
        }
    }
    if !m.imports.is_empty() {
        dst.push('\n');
    }

    // Decls next (sorted by name)
    let mut decls = m.decls.clone();
    decls.sort_by(|a, b| a.name().cmp(b.name()));
    for (i, d) in decls.iter().enumerate() {
        match d {
            Decl::Const(c) => {
                write!(dst, "const {} = ", escape_sym(&c.name)).unwrap();
                write_expr(dst, &c.value, 0);
                dst.push_str(";\n");
            }
            Decl::Let(l) => {
                write!(dst, "let {}", escape_sym(&l.name)).unwrap();
                if let Some(v) = &l.value {
                    dst.push_str(" = ");
                    write_expr(dst, v, 0);
                }
                dst.push_str(";\n");
            }
            Decl::Fn(f) => {
                write!(dst, "fn {}(", escape_sym(&f.name)).unwrap();
                for (pi, p) in f.params.iter().enumerate() {
                    if pi > 0 {
                        dst.push_str(", ");
                    }
                    write!(dst, "{}", escape_sym(p)).unwrap();
                }
                dst.push_str(") ");
                write_block(dst, &f.body, 0);
                dst.push('\n');
            }
        }
        if i + 1 != decls.len() {
            dst.push('\n');
        }
    }
}

fn write_block(dst: &mut String, b: &Block, indent: usize) {
    dst.push_str("{\n");
    for s in &b.stmts {
        write_stmt(dst, s, indent + 2);
    }
    indent_spaces(dst, indent);
    dst.push('}');
}

fn write_stmt(dst: &mut String, s: &Stmt, indent: usize) {
    use Stmt::*;
    match s {
        Expr(e) => {
            indent_spaces(dst, indent);
            write_expr(dst, e, indent);
            dst.push_str(";\n");
        }
        Stmt::Let { name, value } => {
            indent_spaces(dst, indent);
            write!(dst, "let {}", escape_sym(name)).unwrap();
            if let Some(v) = value {
                dst.push_str(" = ");
                write_expr(dst, v, indent);
            }
            dst.push_str(";\n");
        }
        Stmt::Assign { target, value } => {
            indent_spaces(dst, indent);
            write_expr(dst, target, indent);
            dst.push_str(" = ");
            write_expr(dst, value, indent);
            dst.push_str(";\n");
        }
        Return(None) => {
            indent_spaces(dst, indent);
            dst.push_str("return;\n");
        }
        Return(Some(e)) => {
            indent_spaces(dst, indent);
            dst.push_str("return ");
            write_expr(dst, e, indent);
            dst.push_str(";\n");
        }
        If { cond, then_block, else_block } => {
            indent_spaces(dst, indent);
            dst.push_str("if (");
            write_expr(dst, cond, indent);
            dst.push_str(") ");
            write_block(dst, then_block, indent);
            if let Some(e) = else_block {
                dst.push(' ');
                dst.push_str("else ");
                write_block(dst, e, indent);
            }
            dst.push('\n');
        }
        While { cond, body } => {
            indent_spaces(dst, indent);
            dst.push_str("while (");
            write_expr(dst, cond, indent);
            dst.push_str(") ");
            write_block(dst, body, indent);
            dst.push('\n');
        }
        _ => { /* extend as needed */ }
    }
}

fn indent_spaces(dst: &mut String, count: usize) {
    for _ in 0..count {
        dst.push(' ');
    }
}

// ---------- helpers ----------

fn fnv1a64(data: &[u8]) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    let mut hash = FNV_OFFSET;
    for b in data {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn escape_sym(s: &str) -> String {
    if s.chars().all(|c| c == '_' || c.is_ascii_alphanumeric()) {
        s.to_string()
    } else {
        let mut out = String::new();
        out.push('"');
        for ch in s.chars() {
            match ch {
                '\\' => out.push_str("\\\\"),
                '"' => out.push_str("\\\""),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                other => out.push(other),
            }
        }
        out.push('"');
        out
    }
}

// Fixed: Removed underscore from the indent parameter
fn write_expr(dst: &mut String, e: &Expr, indent: usize) {
    match e {
        Expr::Lit(l) => match l {
            crate::core::ir::Lit::Null => dst.push_str("null"),
            crate::core::ir::Lit::Bool(b) => dst.push_str(&format!("{}", b)),
            crate::core::ir::Lit::Number(n) => {
                if n.fract() == 0.0 {
                    dst.push_str(&format!("{}", *n as i64));
                } else {
                    dst.push_str(&format!("{}", n));
                }
            }
            crate::core::ir::Lit::String(s) => dst.push_str(&format!("\"{}\"", s)),
        },
        Expr::Ident(s) => dst.push_str(s),
        Expr::Call { callee, args } => {
            write_expr(dst, callee, indent);
            dst.push('(');
            for (i, a) in args.iter().enumerate() {
                if i > 0 {
                    dst.push_str(", ");
                }
                write_expr(dst, a, indent);
            }
            dst.push(')');
        }
        Expr::Binary { left, op, right } => {
            write_expr(dst, left, indent);
            dst.push(' ');
            dst.push_str(&format!("{}", op));
            dst.push(' ');
            write_expr(dst, right, indent);
        }
        Expr::Unary { op, expr } => {
            use crate::core::ir::UnOp;
            let s = match op {
                UnOp::Neg => "-",
                UnOp::Not => "!",
            };
            dst.push_str(s);
            write_expr(dst, expr, indent);
        }
        Expr::Array(items) => {
            dst.push('[');
            for (i, it) in items.iter().enumerate() {
                if i > 0 {
                    dst.push_str(", ");
                }
                write_expr(dst, it, indent);
            }
            dst.push(']');
        }
        Expr::Object(fields) => {
            dst.push('{');
            for (i, (k, v)) in fields.iter().enumerate() {
                if i > 0 {
                    dst.push_str(", ");
                }
                dst.push_str(&escape_sym(k));
                dst.push_str(": ");
                write_expr(dst, v, indent);
            }
            dst.push('}');
        }
    }
}