//! Aeonmi code generation front-end.
//! - Default backend: **JS** (keeps legacy tests green)
//! - Optional backend: **AI** (canonical .ai via AiEmitter)
use crate::core::ai_emitter::AiEmitter;
use crate::core::ast::ASTNode;
use crate::core::token::TokenKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend {
    Js,
    Ai,
}

pub struct CodeGenerator {
    indent: usize,
    backend: Backend,
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeGenerator {
    pub fn new() -> Self {
        Self {
            indent: 0,
            backend: Backend::Js,
        }
    }
    pub fn new_ai() -> Self {
        Self {
            indent: 0,
            backend: Backend::Ai,
        }
    }
    pub fn generate(&mut self, ast: &ASTNode) -> Result<String, String> {
        self.generate_with_backend(ast, self.backend)
    }
    pub fn generate_with_backend(
        &mut self,
        ast: &ASTNode,
        backend: Backend,
    ) -> Result<String, String> {
        match backend {
            Backend::Js => Ok(self.emit_js(ast)),
            Backend::Ai => {
                let mut emitter = AiEmitter::new();
                emitter
                    .generate(ast)
                    .map_err(|e| format!("AiEmitter error: {e}"))
            }
        }
    }
    // JS BACKEND
    fn emit_js(&mut self, node: &ASTNode) -> String {
        match node {
            ASTNode::Program(items) => {
                let mut out = String::new();
                for item in items {
                    out.push_str(&self.emit_js(item));
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
                    s.push_str(&self.emit_js(it));
                    if !s.ends_with('\n') {
                        s.push('\n');
                    }
                }
                self.indent -= 1;
                s.push_str("}\n");
                s
            }
            ASTNode::VariableDecl { name, value, .. } => {
                format!("let {} = {};\n", name, self.emit_expr_js(value))
            }
            ASTNode::Function { name, params, body, .. } => {
                let mut s = String::new();
                s.push_str(&format!("function {}({}) ", name, params.iter().map(|p| p.name.clone()).collect::<Vec<_>>().join(", ")));
                let block = ASTNode::Block(body.clone());
                s.push_str(&self.emit_js(&block));
                s
            }
            ASTNode::Return(expr) => format!("return {};\n", self.emit_expr_js(expr)),
            ASTNode::Log(expr) => format!("console.log({});\n", self.emit_expr_js(expr)),
            ASTNode::Assignment { name, value, .. } => {
                format!("{} = {};\n", name, self.emit_expr_js(value))
            }
            ASTNode::Call { .. } => format!("{};\n", self.emit_expr_js(node)),
            ASTNode::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let mut s = String::new();
                // Tests expect exactly one extra pair around the emitted binary expression (which is already parenthesized)
                s.push_str(&format!("if ({}) ", self.emit_expr_js(condition)));
                s.push_str(&self.wrap_stmt_js(then_branch));
                if let Some(e) = else_branch {
                    s.push_str(" else ");
                    s.push_str(&self.wrap_stmt_js(e));
                }
                s.push('\n');
                s
            }
            ASTNode::While { condition, body } => {
                let mut s = String::new();
                s.push_str(&format!("while ({}) ", self.emit_expr_js(condition)));
                s.push_str(&self.wrap_stmt_js(body));
                s.push('\n');
                s
            }
            ASTNode::For {
                init,
                condition,
                increment,
                body,
            } => {
                let init_s = if let Some(i) = init.as_ref() {
                    Self::strip_trailing(self.emit_js(i))
                } else {
                    String::new()
                };
                let cond_s = if let Some(c) = condition.as_ref() {
                    self.emit_expr_js(c)
                } else {
                    String::new()
                };
                let incr_s = if let Some(inc) = increment.as_ref() {
                    Self::strip_trailing(self.emit_js(inc))
                } else {
                    String::new()
                };
                let mut s = String::new();
                s.push_str(&format!("for ({}; {}; {}) ", init_s, cond_s, incr_s));
                s.push_str(&self.wrap_stmt_js(body));
                s.push('\n');
                s
            }
            ASTNode::BinaryExpr { .. }
            | ASTNode::UnaryExpr { .. }
            | ASTNode::Identifier(_)
            | ASTNode::IdentifierSpanned { .. }
            | ASTNode::NumberLiteral(_)
            | ASTNode::StringLiteral(_)
            | ASTNode::BooleanLiteral(_) => format!("{};\n", self.emit_expr_js(node)),
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
                    .map(|q| self.emit_expr_js(q))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({});\n", opname, args)
            }
            ASTNode::HieroglyphicOp { symbol, args } => {
                let a = args
                    .iter()
                    .map(|e| self.emit_expr_js(e))
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
            ASTNode::IdentifierSpanned { name, .. } => name.clone(),
            ASTNode::NumberLiteral(n) => {
                if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            ASTNode::StringLiteral(s) => format!("\"{}\"", s),
            ASTNode::BooleanLiteral(b) => format!("{}", b),
            ASTNode::BinaryExpr { op, left, right } => {
                format!(
                    "({} {} {})",
                    self.emit_expr_js(left),
                    self.op_str(op),
                    self.emit_expr_js(right)
                )
            }
            ASTNode::UnaryExpr { op, expr } => {
                format!("{}{}", self.op_str(op), self.emit_expr_js(expr))
            }
            ASTNode::Call { callee, args } => {
                let c = self.emit_expr_js(callee);
                let a = args
                    .iter()
                    .map(|x| self.emit_expr_js(x))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", c, a)
            }
            ASTNode::Assignment { name, value, .. } => {
                format!("{} = {}", name, self.emit_expr_js(value))
            }
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
                    .map(|q| self.emit_expr_js(q))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", opname, args)
            }
            ASTNode::HieroglyphicOp { symbol, args } => {
                let a = args
                    .iter()
                    .map(|e| self.emit_expr_js(e))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("__glyph('{}', {})", symbol, a)
            }
            _ => "/*expr*/".into(),
        }
    }
    /// Returns a JS statement block string **without** a trailing newline.
    fn wrap_stmt_js(&mut self, n: &ASTNode) -> String {
        match n {
            ASTNode::Block(_) => {
                let mut b = self.emit_js(n);
                if b.ends_with('\n') {
                    b.pop();
                }
                b
            }
            _ => {
                let mut s = String::new();
                s.push_str("{\n");
                self.indent += 1;
                s.push_str(&self.indent_str());
                s.push_str(&self.emit_js(n));
                self.indent -= 1;
                s.push('}');
                s
            }
        }
    }
    fn op_str(&self, op: &TokenKind) -> &'static str {
        match op {
            TokenKind::Plus => "+",
            TokenKind::Minus => "-",
            TokenKind::Star => "*",
            TokenKind::Slash => "/",
            TokenKind::Equals => "=",
            TokenKind::DoubleEquals => "==",
            TokenKind::NotEquals => "!=",
            TokenKind::LessThan => "<",
            TokenKind::LessEqual => "<=",
            TokenKind::GreaterThan => ">",
            TokenKind::GreaterEqual => ">=",
            // Only match the variants that exist in TokenKind
            _ => "/*op*/",
        }
    }
    fn indent_str(&self) -> String {
        "  ".repeat(self.indent)
    }
    fn strip_trailing(mut s: String) -> String {
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
        // This test requires a mock or real AiEmitter implementation.
        // If not available, adjust accordingly.
        let ast = ASTNode::Program(vec![ASTNode::new_variable_decl("x", ASTNode::NumberLiteral(1.0))]);
        let mut g = CodeGenerator::new_ai();
        let out = g.generate(&ast).unwrap();
        assert!(out.contains("x") && (out.contains("1") || out.contains("1.0")));
    }
}
