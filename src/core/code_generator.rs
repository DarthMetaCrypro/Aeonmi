// src/core/code_generator.rs
//! Aeonmi code generation front-end.
//! - Default backend: **JS** (keeps legacy tests green)
//! - Optional backend: **AI** (canonical .ai via AiEmitter)
//!
//! Usage:
//!   let mut gen = CodeGenerator::new();            // JS by default
//!   let js = gen.generate(&ast)?;
//!
//!   let mut gen_ai = CodeGenerator::new_ai();      // AI backend
//!   let ai = gen_ai.generate(&ast)?;
//!
//!   // or:
//!   let ai2 = gen.generate_with_backend(&ast, Backend::Ai)?;

use crate::core::ast::ASTNode;
use crate::core::token::TokenKind;

<<<<<<< HEAD
// Pull in the pretty-printing .ai emitter.
// This file already exists in your tree: src/core/ai_emitter.rs
use crate::core::ai_emitter::AiEmitter;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend {
    Js,
    Ai,
}

pub struct CodeGenerator {
    indent: usize,
    backend: Backend,
}

impl CodeGenerator {
    /// New generator with **JS** backend (legacy default).
    pub fn new() -> Self {
        Self {
            indent: 0,
            backend: Backend::Js,
        }
=======
pub struct CodeGenerator {
    indent: usize,
}

impl CodeGenerator {
    pub fn new() -> Self {
        Self { indent: 0 }
    }
    pub fn generate(&mut self, ast: &ASTNode) -> Result<String, String> {
        Ok(self.emit(ast))
>>>>>>> 9543281 (feat: TUI editor + neon shell + hardened lexer (NFC, AI blocks, comments, tests))
    }

    /// New generator with **AI** backend (canonical .ai output).
    pub fn new_ai() -> Self {
        Self {
            indent: 0,
            backend: Backend::Ai,
        }
    }

    /// Emit using the generator's configured backend.
    pub fn generate(&mut self, ast: &ASTNode) -> Result<String, String> {
        self.generate_with_backend(ast, self.backend)
    }

    /// Emit using an explicit backend selection.
    pub fn generate_with_backend(
        &mut self,
        ast: &ASTNode,
        backend: Backend,
    ) -> Result<String, String> {
        match backend {
            Backend::Js => Ok(self.emit_js(ast)),
            Backend::Ai => {
                // Delegate to the canonical .ai pretty-printer.
                let mut emitter = AiEmitter::new();
                emitter
                    .generate(ast)
                    .map_err(|e| format!("AiEmitter error: {e}"))
            }
        }
    }

    // =========================
    // JS BACKEND (legacy path)
    // =========================
    fn emit_js(&mut self, node: &ASTNode) -> String {
        match node {
            ASTNode::Program(items) => {
                let mut out = String::new();
                for item in items {
<<<<<<< HEAD
                    out.push_str(&self.emit_js(item));
=======
                    out.push_str(&self.emit(item));
>>>>>>> 9543281 (feat: TUI editor + neon shell + hardened lexer (NFC, AI blocks, comments, tests))
                    if !out.ends_with('\n') {
                        out.push('\n');
                    }
                }
                out
            }
            ASTNode::Block(items) => {
                let mut s = String::new();
                s.push_str("{\n");
                self.indent += 1;
                for it in items {
                    s.push_str(&self.indent_str());
<<<<<<< HEAD
                    s.push_str(&self.emit_js(it));
=======
                    s.push_str(&self.emit(it));
>>>>>>> 9543281 (feat: TUI editor + neon shell + hardened lexer (NFC, AI blocks, comments, tests))
                    if !s.ends_with('\n') {
                        s.push('\n');
                    }
                }
                self.indent -= 1;
                s.push_str("}\n");
                s
            }

            // declarations / statements
            ASTNode::VariableDecl { name, value } => {
<<<<<<< HEAD
                format!("let {} = {};\n", name, self.emit_expr_js(value))
=======
                format!("let {} = {};\n", name, self.emit_expr(value))
>>>>>>> 9543281 (feat: TUI editor + neon shell + hardened lexer (NFC, AI blocks, comments, tests))
            }
            ASTNode::Function { name, params, body } => {
                let mut s = String::new();
                s.push_str(&format!("function {}({}) ", name, params.join(", ")));
                let block = ASTNode::Block(body.clone());
                s.push_str(&self.emit_js(&block));
                s
            }
            ASTNode::Return(expr) => format!("return {};\n", self.emit_expr_js(expr)),
            ASTNode::Log(expr) => format!("console.log({});\n", self.emit_expr_js(expr)),

<<<<<<< HEAD
            // assignments/calls as statements
            ASTNode::Assignment { name, value } => {
                format!("{} = {};\n", name, self.emit_expr_js(value))
            }
            ASTNode::Call { .. } => format!("{};\n", self.emit_expr_js(node)),
=======
            // NEW: emit assignments/calls as statements (no extra parens)
            ASTNode::Assignment { name, value } => {
                format!("{} = {};\n", name, self.emit_expr(value))
            }
            ASTNode::Call { .. } => format!("{};\n", self.emit_expr(node)),
>>>>>>> 9543281 (feat: TUI editor + neon shell + hardened lexer (NFC, AI blocks, comments, tests))

            ASTNode::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let mut s = String::new();
                s.push_str(&format!("if ({}) ", self.emit_expr_js(condition)));
                s.push_str(&self.wrap_stmt_js(then_branch)); // no trailing \n
                if let Some(e) = else_branch {
                    s.push_str(" else ");
                    s.push_str(&self.wrap_stmt_js(e)); // no trailing \n
                }
                s.push('\n');
                s
            }
            ASTNode::While { condition, body } => {
                let mut s = String::new();
                s.push_str(&format!("while ({}) ", self.emit_expr_js(condition)));
                s.push_str(&self.wrap_stmt_js(body)); // no trailing \n
                s.push('\n');
                s
            }
            ASTNode::For {
                init,
                condition,
                increment,
                body,
            } => {
                // avoid overlapping borrows: compute piecewise
                let init_s = if let Some(i) = init.as_ref() {
                    let tmp = self.emit_js(i);
                    self.strip_trailing(tmp)
                } else {
                    String::new()
                };
                let cond_s = condition
                    .as_ref()
<<<<<<< HEAD
                    .map(|c| self.emit_expr_js(c))
                    .unwrap_or_default();
                let inc_s = increment
                    .as_ref()
                    .map(|i| self.emit_expr_js(i))
=======
                    .map(|c| self.emit_expr(c))
                    .unwrap_or_default();
                let inc_s = increment
                    .as_ref()
                    .map(|i| self.emit_expr(i))
>>>>>>> 9543281 (feat: TUI editor + neon shell + hardened lexer (NFC, AI blocks, comments, tests))
                    .unwrap_or_default();

                let mut s = String::new();
                s.push_str(&format!("for ({}; {}; {}) ", init_s, cond_s, inc_s));
                s.push_str(&self.wrap_stmt_js(body)); // no trailing \n
                s.push('\n');
                s
            }

            // Expression statements fallback
            ASTNode::BinaryExpr { .. }
            | ASTNode::UnaryExpr { .. }
            | ASTNode::Identifier(_)
            | ASTNode::NumberLiteral(_)
            | ASTNode::StringLiteral(_)
            | ASTNode::BooleanLiteral(_) => format!("{};\n", self.emit_expr_js(node)),

            // Quantum & Hieroglyphic placeholders (JS shims)
            ASTNode::QuantumOp { op, qubits } => {
                let opname = match op {
                    TokenKind::Superpose => "superpose",
                    TokenKind::Entangle => "entangle",
                    TokenKind::Measure => "measure",
                    TokenKind::Dod => "dod",
                    _ => "qop",
                };
                let args = qubits
                    .iter()
<<<<<<< HEAD
                    .map(|q| self.emit_expr_js(q))
=======
                    .map(|q| self.emit_expr(q))
>>>>>>> 9543281 (feat: TUI editor + neon shell + hardened lexer (NFC, AI blocks, comments, tests))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({});\n", opname, args)
            }
            ASTNode::HieroglyphicOp { symbol, args } => {
                let a = args
                    .iter()
<<<<<<< HEAD
                    .map(|e| self.emit_expr_js(e))
=======
                    .map(|e| self.emit_expr(e))
>>>>>>> 9543281 (feat: TUI editor + neon shell + hardened lexer (NFC, AI blocks, comments, tests))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("__glyph('{}', {});\n", symbol, a)
            }

            ASTNode::Error(msg) => format!("/* ERROR NODE: {} */\n", msg),
        }
    }

    fn emit_expr_js(&mut self, node: &ASTNode) -> String {
        match node {
            ASTNode::Identifier(s) => s.clone(),
            ASTNode::NumberLiteral(n) => {
                if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            ASTNode::StringLiteral(s) => format!("{:?}", s),
            ASTNode::BooleanLiteral(b) => b.to_string(),

            ASTNode::UnaryExpr { op, expr } => {
<<<<<<< HEAD
                format!("({}{})", self.op_str(op), self.emit_expr_js(expr))
            }
            ASTNode::BinaryExpr { op, left, right } => format!(
                "({} {} {})",
                self.emit_expr_js(left),
                self.op_str(op),
                self.emit_expr_js(right)
=======
                format!("({}{})", self.op_str(op), self.emit_expr(expr))
            }
            ASTNode::BinaryExpr { op, left, right } => format!(
                "({} {} {})",
                self.emit_expr(left),
                self.op_str(op),
                self.emit_expr(right)
>>>>>>> 9543281 (feat: TUI editor + neon shell + hardened lexer (NFC, AI blocks, comments, tests))
            ),

            // keep parens when used inside other expressions
            ASTNode::Assignment { name, value } => {
<<<<<<< HEAD
                format!("({} = {})", name, self.emit_expr_js(value))
            }

            ASTNode::Call { callee, args } => {
                let c = self.emit_expr_js(callee);
                let a = args
                    .iter()
                    .map(|e| self.emit_expr_js(e))
=======
                format!("({} = {})", name, self.emit_expr(value))
            }

            ASTNode::Call { callee, args } => {
                let c = self.emit_expr(callee);
                let a = args
                    .iter()
                    .map(|e| self.emit_expr(e))
>>>>>>> 9543281 (feat: TUI editor + neon shell + hardened lexer (NFC, AI blocks, comments, tests))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", c, a)
            }

            _ => "/* unsupported-expr */".to_string(),
        }
    }

    /// Returns a JS statement block string **without** a trailing newline.
    fn wrap_stmt_js(&mut self, n: &ASTNode) -> String {
        match n {
            ASTNode::Block(_) => {
                // Use existing block emission but drop the trailing newline.
                let mut b = self.emit_js(n);
                if b.ends_with('\n') {
                    b.pop();
                }
                b
            }
            _ => {
                // Wrap a single statement in a block, no trailing newline.
                let mut s = String::new();
                s.push_str("{\n");
                self.indent += 1;
                s.push_str(&self.indent_str());
                s.push_str(&self.emit_js(n)); // inner includes its own newline
                self.indent -= 1;
                s.push('}');
                s
            }
        }
    }

    fn op_str(&self, op: &TokenKind) -> &'static str {
        use TokenKind::*;
        match op {
            Plus => "+",
            Minus => "-",
            Star => "*",
            Slash => "/",
            Equals => "=", // used in decls; assignments have their own node
            DoubleEquals => "==",
            NotEquals => "!=",
            LessThan => "<",
            LessEqual => "<=",
            GreaterThan => ">",
            GreaterEqual => ">=",
            _ => "/*op*/",
        }
    }

    fn indent_str(&self) -> String {
        "  ".repeat(self.indent)
    }
    fn strip_trailing(&self, mut s: String) -> String {
        if s.ends_with('\n') {
            s.pop();
        }
        if s.ends_with(';') {
            s.pop();
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ast::ASTNode;

    #[test]
    fn gen_let_and_log_js() {
        let ast = ASTNode::Program(vec![
            ASTNode::new_variable_decl("x", ASTNode::NumberLiteral(42.0)),
            ASTNode::new_log(ASTNode::Identifier("x".into())),
        ]);
        let mut g = CodeGenerator::new(); // JS default
        let js = g.generate(&ast).unwrap();
        assert!(js.contains("let x = 42;"));
        assert!(js.contains("console.log(x);"));
    }

    #[test]
    fn gen_assignment_and_call_js() {
        let call = ASTNode::new_call(
            ASTNode::Identifier("add".into()),
            vec![ASTNode::NumberLiteral(2.0), ASTNode::NumberLiteral(3.0)],
        );
        let prog = ASTNode::Program(vec![ASTNode::new_assignment("x", call)]);
        let mut g = CodeGenerator::new(); // JS default
        let js = g.generate(&prog).unwrap();
        assert!(js.contains("x = add(2, 3);"));
    }

    #[test]
    fn gen_minimal_ai_backend() {
        // Smoke-test that the AI backend path is wired (exact formatting is owned by AiEmitter tests)
        let ast = ASTNode::Program(vec![ASTNode::new_variable_decl(
            "x",
            ASTNode::NumberLiteral(1.0),
        )]);
        let mut g = CodeGenerator::new_ai();
        let out = g.generate(&ast).unwrap();
        // Very loose assertion to avoid coupling to formatting rules:
        assert!(
            out.contains("x") && (out.contains("1") || out.contains("1.0")),
            "ai output should reference the declared symbol and value"
        );
    }
}
