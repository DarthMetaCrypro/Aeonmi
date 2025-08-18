#![cfg_attr(test, allow(dead_code, unused_variables))]
//! Canonical .ai emitter for Aeonmi IR.
<<<<<<< HEAD
//! - Deterministic output: sorted imports/decls, 2-space indent, LF
//! - Header embeds FNV-1a 64-bit hash of body for reproducibility
//!
//! Exposes both the legacy free function `emit_ai(&Module)` and a thin
//! `AiEmitter` facade so other parts of the compiler can request canonical `.ai`.

use crate::core::ir::*;
use crate::core::lowering::lower_ast_to_ir;
use std::fmt::Write as _;

/// Thin wrapper to emit canonical `.ai` from IR or AST.
#[derive(Default)]
pub struct AiEmitter;

impl AiEmitter {
    pub fn new() -> Self {
        Self::default()
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

/// Legacy functional API (kept for existing tests)
=======
//! Produces deterministic output (sorted decls/imports, 2-space indent, LF, stable formatting)
//! Header includes a simple FNV-1a 64-bit of the body for reproducible integrity.

use crate::core::ir::*;
use std::fmt::Write as _;

>>>>>>> 9543281 (feat: TUI editor + neon shell + hardened lexer (NFC, AI blocks, comments, tests))
pub fn emit_ai(module: &Module) -> String {
    // 1) Body (deterministic)
    let mut body = String::new();
    write_module(&mut body, module);

<<<<<<< HEAD
    // 2) Hash header (FNV-1a 64-bit)
    let hash = fnv1a64(body.as_bytes());

    // 3) Final output with header (LF newlines only)
    let mut out = String::new();
    let _ = writeln!(&mut out, "// aeonmi:1");
    let _ = writeln!(&mut out, "// hash:{:016x}", hash);
    let _ = writeln!(&mut out, "// tool:aeonmi unknown");
=======
    // 2) Hash header (FNV-1a 64-bit, implemented here to avoid external deps)
    let hash = fnv1a64(body.as_bytes());

    // 3) Final output with header
    let mut out = String::new();
    writeln!(&mut out, "// aeonmi:1").unwrap();
    writeln!(&mut out, "// hash:{:016x}", hash).unwrap();
    writeln!(&mut out, "// tool:aeonmi unknown").unwrap();
>>>>>>> 9543281 (feat: TUI editor + neon shell + hardened lexer (NFC, AI blocks, comments, tests))
    out.push('\n');
    out.push_str(&body);
    out
}

fn write_module(dst: &mut String, m: &Module) {
<<<<<<< HEAD
    // Imports first (sorted)
    let mut imports = m.imports.clone();
    imports.sort_by(|a, b| (a.path.as_str(), a.alias.as_deref()).cmp(&(b.path.as_str(), b.alias.as_deref())));
    for im in imports {
        match im.alias {
            Some(alias) => {
                let _ = writeln!(dst, "import {} as {};", escape_sym(&im.path), escape_sym(&alias));
            }
            None => {
                let _ = writeln!(dst, "import {};", escape_sym(&im.path));
            }
=======
    // Imports first (sorted by path/alias externally; re-sort here to be safe)
    let mut imports = m.imports.clone();
    imports.sort_by(|a, b| (a.path.as_str(), a.alias.as_deref()).cmp(&(b.path.as_str(), b.alias.as_deref())));
    for im in imports {
        if let Some(alias) = im.alias {
            writeln!(dst, "import {} as {};", escape_sym(&im.path), escape_sym(&alias)).unwrap();
        } else {
            writeln!(dst, "import {};", escape_sym(&im.path)).unwrap();
>>>>>>> 9543281 (feat: TUI editor + neon shell + hardened lexer (NFC, AI blocks, comments, tests))
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
<<<<<<< HEAD
                let _ = write!(dst, "const {} = ", escape_sym(&c.name));
=======
                write!(dst, "const {} = ", escape_sym(&c.name)).unwrap();
>>>>>>> 9543281 (feat: TUI editor + neon shell + hardened lexer (NFC, AI blocks, comments, tests))
                write_expr(dst, &c.value, 0);
                dst.push_str(";\n");
            }
            Decl::Let(l) => {
<<<<<<< HEAD
                let _ = write!(dst, "let {}", escape_sym(&l.name));
=======
                write!(dst, "let {}", escape_sym(&l.name)).unwrap();
>>>>>>> 9543281 (feat: TUI editor + neon shell + hardened lexer (NFC, AI blocks, comments, tests))
                if let Some(v) = &l.value {
                    dst.push_str(" = ");
                    write_expr(dst, v, 0);
                }
                dst.push_str(";\n");
            }
            Decl::Fn(f) => {
<<<<<<< HEAD
                let _ = write!(dst, "fn {}(", escape_sym(&f.name));
                for (pi, p) in f.params.iter().enumerate() {
                    if pi > 0 { dst.push_str(", "); }
                    let _ = write!(dst, "{}", escape_sym(p));
=======
                write!(dst, "fn {}(", escape_sym(&f.name)).unwrap();
                for (pi, p) in f.params.iter().enumerate() {
                    if pi > 0 { dst.push_str(", "); }
                    write!(dst, "{}", escape_sym(p)).unwrap();
>>>>>>> 9543281 (feat: TUI editor + neon shell + hardened lexer (NFC, AI blocks, comments, tests))
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
        For { init, cond, step, body } => {
<<<<<<< HEAD
            // Canonical for: for (init; cond; step) { ... }
=======
            // Canonical for: lowered form: for (init; cond; step) { ... }
>>>>>>> 9543281 (feat: TUI editor + neon shell + hardened lexer (NFC, AI blocks, comments, tests))
            indent_spaces(dst, indent);
            dst.push_str("for (");
            if let Some(init_s) = init {
                write_stmt_inline(dst, init_s, indent);
            }
            dst.push_str("; ");
            if let Some(c) = cond {
                write_expr(dst, c, indent);
            }
            dst.push_str("; ");
            if let Some(st) = step {
                write_expr(dst, st, indent);
            }
            dst.push_str(") ");
            write_block(dst, body, indent);
            dst.push('\n');
        }
        Let { name, value } => {
            indent_spaces(dst, indent);
<<<<<<< HEAD
            let _ = write!(dst, "let {}", escape_sym(name));
=======
            write!(dst, "let {}", escape_sym(name)).unwrap();
>>>>>>> 9543281 (feat: TUI editor + neon shell + hardened lexer (NFC, AI blocks, comments, tests))
            if let Some(v) = value {
                dst.push_str(" = ");
                write_expr(dst, v, indent);
            }
            dst.push_str(";\n");
        }
        Assign { target, value } => {
            indent_spaces(dst, indent);
            write_expr(dst, target, indent);
            dst.push_str(" = ");
            write_expr(dst, value, indent);
            dst.push_str(";\n");
        }
    }
}

fn write_stmt_inline(dst: &mut String, s: &Stmt, indent: usize) {
<<<<<<< HEAD
    // Used in for(...) init; keep single-line; no trailing semicolon duplication
    match s {
        Stmt::Let { name, value } => {
            let _ = write!(dst, "let {}", escape_sym(name));
=======
    // Used in for(...) init; keep it single-line without trailing semicolon duplication
    match s {
        Stmt::Let { name, value } => {
            write!(dst, "let {}", escape_sym(name)).unwrap();
>>>>>>> 9543281 (feat: TUI editor + neon shell + hardened lexer (NFC, AI blocks, comments, tests))
            if let Some(v) = value {
                dst.push_str(" = ");
                write_expr(dst, v, indent);
            }
        }
        Stmt::Assign { target, value } => {
            write_expr(dst, target, indent);
            dst.push_str(" = ");
            write_expr(dst, value, indent);
        }
        Stmt::Expr(e) => {
            write_expr(dst, e, indent);
        }
<<<<<<< HEAD
        _ => { /* unsupported inline; emit nothing */ }
=======
        _ => {
            // Fallback: emit nothing (unsupported inline)
        }
>>>>>>> 9543281 (feat: TUI editor + neon shell + hardened lexer (NFC, AI blocks, comments, tests))
    }
}

fn write_expr(dst: &mut String, e: &Expr, indent: usize) {
    use Expr::*;
    match e {
        Lit(l) => write_lit(dst, l),
        Ident(s) => dst.push_str(&escape_sym(s)),
        Call { callee, args } => {
            write_expr(dst, callee, indent);
            dst.push('(');
            for (i, a) in args.iter().enumerate() {
                if i > 0 { dst.push_str(", "); }
                write_expr(dst, a, indent);
            }
            dst.push(')');
        }
        Binary { left, op, right } => {
            write_expr(dst, left, indent);
<<<<<<< HEAD
            let _ = write!(dst, " {} ", op);
=======
            write!(dst, " {} ", op).unwrap();
>>>>>>> 9543281 (feat: TUI editor + neon shell + hardened lexer (NFC, AI blocks, comments, tests))
            write_expr(dst, right, indent);
        }
        Unary { op, expr } => {
            match op {
                crate::core::ir::UnOp::Neg => dst.push('-'),
                crate::core::ir::UnOp::Not => dst.push('!'),
            }
            write_expr(dst, expr, indent);
        }
        Array(items) => {
            dst.push('[');
            for (i, it) in items.iter().enumerate() {
                if i > 0 { dst.push_str(", "); }
                write_expr(dst, it, indent);
            }
            dst.push(']');
        }
        Object(kvs) => {
            dst.push('{');
            for (i, (k, v)) in kvs.iter().enumerate() {
                if i > 0 { dst.push_str(", "); }
<<<<<<< HEAD
                let _ = write!(dst, "{}: ", escape_sym(k));
=======
                write!(dst, "{}: ", escape_sym(k)).unwrap();
>>>>>>> 9543281 (feat: TUI editor + neon shell + hardened lexer (NFC, AI blocks, comments, tests))
                write_expr(dst, v, indent);
            }
            dst.push('}');
        }
    }
}

fn write_lit(dst: &mut String, l: &Lit) {
    match l {
        Lit::Null => dst.push_str("null"),
        Lit::Bool(b) => dst.push_str(if *b { "true" } else { "false" }),
        Lit::Number(n) => {
            if n.fract() == 0.0 {
<<<<<<< HEAD
                let _ = write!(dst, "{}", *n as i64);
            } else {
                let _ = write!(dst, "{}", n);
=======
                write!(dst, "{}", *n as i64).unwrap();
            } else {
                write!(dst, "{}", n).unwrap();
>>>>>>> 9543281 (feat: TUI editor + neon shell + hardened lexer (NFC, AI blocks, comments, tests))
            }
        }
        Lit::String(s) => {
            dst.push('"');
            for ch in s.chars() {
                match ch {
                    '\\' => dst.push_str("\\\\"),
<<<<<<< HEAD
                    '"'  => dst.push_str("\\\""),
=======
                    '"' => dst.push_str("\\\""),
>>>>>>> 9543281 (feat: TUI editor + neon shell + hardened lexer (NFC, AI blocks, comments, tests))
                    '\n' => dst.push_str("\\n"),
                    '\r' => dst.push_str("\\r"),
                    '\t' => dst.push_str("\\t"),
                    _ => dst.push(ch),
                }
            }
            dst.push('"');
        }
    }
}

<<<<<<< HEAD
#[inline]
fn indent_spaces(dst: &mut String, n: usize) {
    // cheaper than " ".repeat(n) allocation
=======
fn indent_spaces(dst: &mut String, n: usize) {
>>>>>>> 9543281 (feat: TUI editor + neon shell + hardened lexer (NFC, AI blocks, comments, tests))
    for _ in 0..n { dst.push(' '); }
}

fn escape_sym(sym: &str) -> String {
<<<<<<< HEAD
    // Keep Unicode; replace whitespace/control with underscores.
=======
    // Minimal: keep glyphs/Unicode; escape spaces and control chars with `_`.
>>>>>>> 9543281 (feat: TUI editor + neon shell + hardened lexer (NFC, AI blocks, comments, tests))
    if sym.chars().all(|c| !c.is_whitespace() && !c.is_control()) {
        sym.to_string()
    } else {
        sym.chars().map(|c| if c.is_whitespace() || c.is_control() { '_' } else { c }).collect()
    }
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
<<<<<<< HEAD
    const FNV_PRIME:  u64 = 0x00000100000001B3;
=======
    const FNV_PRIME: u64 = 0x100000001b3;
>>>>>>> 9543281 (feat: TUI editor + neon shell + hardened lexer (NFC, AI blocks, comments, tests))
    let mut hash = FNV_OFFSET;
    for b in bytes {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}
<<<<<<< HEAD

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ast::ASTNode;

    #[test]
    fn emitter_roundtrip_minimal() {
        // Smoke test the AST → IR → .ai path remains wired.
        let ast = ASTNode::Program(vec![
            ASTNode::new_variable_decl("x", ASTNode::NumberLiteral(1.0)),
            ASTNode::new_variable_decl("y", ASTNode::NumberLiteral(2.0)),
        ]);
        let mut em = AiEmitter::new();
        let out = em.generate(&ast).expect("ai emit");
        assert!(out.contains("let x = 1;"));
        assert!(out.contains("let y = 2;"));
        assert!(out.lines().next().unwrap().starts_with("// aeonmi:1"));
    }
}
=======
>>>>>>> 9543281 (feat: TUI editor + neon shell + hardened lexer (NFC, AI blocks, comments, tests))
