#![cfg_attr(test, allow(dead_code, unused_variables))]
//! Canonical .ai emitter for Aeonmi IR.
//! Produces deterministic output (sorted decls/imports, 2-space indent, LF, stable formatting)
//! Header includes a simple FNV-1a 64-bit of the body for reproducible integrity.

use crate::core::ir::*;
use std::fmt::Write as _;

pub fn emit_ai(module: &Module) -> String {
    // 1) Body (deterministic)
    let mut body = String::new();
    write_module(&mut body, module);

    // 2) Hash header (FNV-1a 64-bit, implemented here to avoid external deps)
    let hash = fnv1a64(body.as_bytes());

    // 3) Final output with header
    let mut out = String::new();
    writeln!(&mut out, "// aeonmi:1").unwrap();
    writeln!(&mut out, "// hash:{:016x}", hash).unwrap();
    writeln!(&mut out, "// tool:aeonmi unknown").unwrap();
    out.push('\n');
    out.push_str(&body);
    out
}

fn write_module(dst: &mut String, m: &Module) {
    // Imports first (sorted by path/alias externally; re-sort here to be safe)
    let mut imports = m.imports.clone();
    imports.sort_by(|a, b| (a.path.as_str(), a.alias.as_deref()).cmp(&(b.path.as_str(), b.alias.as_deref())));
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
                    if pi > 0 { dst.push_str(", "); }
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
            // Canonical for: lowered form: for (init; cond; step) { ... }
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
            write!(dst, "let {}", escape_sym(name)).unwrap();
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
    // Used in for(...) init; keep it single-line without trailing semicolon duplication
    match s {
        Stmt::Let { name, value } => {
            write!(dst, "let {}", escape_sym(name)).unwrap();
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
        _ => {
            // Fallback: emit nothing (unsupported inline)
        }
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
            write!(dst, " {} ", op).unwrap();
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
                write!(dst, "{}: ", escape_sym(k)).unwrap();
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
                write!(dst, "{}", *n as i64).unwrap();
            } else {
                write!(dst, "{}", n).unwrap();
            }
        }
        Lit::String(s) => {
            dst.push('"');
            for ch in s.chars() {
                match ch {
                    '\\' => dst.push_str("\\\\"),
                    '"' => dst.push_str("\\\""),
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

fn indent_spaces(dst: &mut String, n: usize) {
    for _ in 0..n { dst.push(' '); }
}

fn escape_sym(sym: &str) -> String {
    // Minimal: keep glyphs/Unicode; escape spaces and control chars with `_`.
    if sym.chars().all(|c| !c.is_whitespace() && !c.is_control()) {
        sym.to_string()
    } else {
        sym.chars().map(|c| if c.is_whitespace() || c.is_control() { '_' } else { c }).collect()
    }
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    let mut hash = FNV_OFFSET;
    for b in bytes {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}
