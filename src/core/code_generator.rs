// src/core/code_generator.rs
//! Simple JS codegen backend for Aeonmi/QUBE/Titan.
//! - Supports assignments and function calls
//! - Preserves comparison ops and borrow-safe for-init

use crate::core::ast::ASTNode;
use crate::core::token::TokenKind;

pub struct CodeGenerator {
    indent: usize,
}

impl CodeGenerator {
    pub fn new() -> Self {
        Self { indent: 0 }
    }
    pub fn generate(&mut self, ast: &ASTNode) -> Result<String, String> {
        Ok(self.emit(ast))
    }

    fn emit(&mut self, node: &ASTNode) -> String {
        match node {
            ASTNode::Program(items) => {
                let mut out = String::new();
                for item in items {
                    out.push_str(&self.emit(item));
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
                    s.push_str(&self.emit(it));
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
                format!("let {} = {};\n", name, self.emit_expr(value))
            }
            ASTNode::Function { name, params, body } => {
                let mut s = String::new();
                s.push_str(&format!("function {}({}) ", name, params.join(", ")));
                let block = ASTNode::Block(body.clone());
                s.push_str(&self.emit(&block));
                s
            }
            ASTNode::Return(expr) => format!("return {};\n", self.emit_expr(expr)),
            ASTNode::Log(expr) => format!("console.log({});\n", self.emit_expr(expr)),

            // NEW: emit assignments/calls as statements (no extra parens)
            ASTNode::Assignment { name, value } => {
                format!("{} = {};\n", name, self.emit_expr(value))
            }
            ASTNode::Call { .. } => format!("{};\n", self.emit_expr(node)),

            ASTNode::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let mut s = String::new();
                s.push_str(&format!("if ({}) ", self.emit_expr(condition)));
                s.push_str(&self.wrap_stmt(then_branch));
                if let Some(e) = else_branch {
                    s.push_str(" else ");
                    s.push_str(&self.wrap_stmt(e));
                }
                s.push('\n');
                s
            }
            ASTNode::While { condition, body } => {
                let mut s = String::new();
                s.push_str(&format!("while ({}) ", self.emit_expr(condition)));
                s.push_str(&self.wrap_stmt(body));
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
                    let tmp = self.emit(i);
                    self.strip_trailing(tmp)
                } else {
                    String::new()
                };
                let cond_s = condition
                    .as_ref()
                    .map(|c| self.emit_expr(c))
                    .unwrap_or_default();
                let inc_s = increment
                    .as_ref()
                    .map(|i| self.emit_expr(i))
                    .unwrap_or_default();

                let mut s = String::new();
                s.push_str(&format!("for ({}; {}; {}) ", init_s, cond_s, inc_s));
                s.push_str(&self.wrap_stmt(body));
                s.push('\n');
                s
            }

            // Expression statements fallback
            ASTNode::BinaryExpr { .. }
            | ASTNode::UnaryExpr { .. }
            | ASTNode::Identifier(_)
            | ASTNode::NumberLiteral(_)
            | ASTNode::StringLiteral(_)
            | ASTNode::BooleanLiteral(_) => format!("{};\n", self.emit_expr(node)),

            // Quantum & Hieroglyphic placeholders
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
                    .map(|q| self.emit_expr(q))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({});\n", opname, args)
            }
            ASTNode::HieroglyphicOp { symbol, args } => {
                let a = args
                    .iter()
                    .map(|e| self.emit_expr(e))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("__glyph('{}', {});\n", symbol, a)
            }

            ASTNode::Error(msg) => format!("/* ERROR NODE: {} */\n", msg),
        }
    }

    fn emit_expr(&mut self, node: &ASTNode) -> String {
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
                format!("({}{})", self.op_str(op), self.emit_expr(expr))
            }
            ASTNode::BinaryExpr { op, left, right } => format!(
                "({} {} {})",
                self.emit_expr(left),
                self.op_str(op),
                self.emit_expr(right)
            ),

            // keep parens when used inside other expressions
            ASTNode::Assignment { name, value } => {
                format!("({} = {})", name, self.emit_expr(value))
            }

            ASTNode::Call { callee, args } => {
                let c = self.emit_expr(callee);
                let a = args
                    .iter()
                    .map(|e| self.emit_expr(e))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", c, a)
            }

            _ => "/* unsupported-expr */".to_string(),
        }
    }

    fn wrap_stmt(&mut self, n: &ASTNode) -> String {
        match n {
            ASTNode::Block(_) => self.emit(n),
            _ => {
                let mut s = String::new();
                s.push_str("{\n");
                self.indent += 1;
                s.push_str(&self.indent_str());
                s.push_str(&self.emit(n));
                self.indent -= 1;
                s.push_str("}");
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
    fn gen_let_and_log() {
        let ast = ASTNode::Program(vec![
            ASTNode::new_variable_decl("x", ASTNode::NumberLiteral(42.0)),
            ASTNode::new_log(ASTNode::Identifier("x".into())),
        ]);
        let mut g = CodeGenerator::new();
        let js = g.generate(&ast).unwrap();
        assert!(js.contains("let x = 42;"));
        assert!(js.contains("console.log(x);"));
    }

    #[test]
    fn gen_assignment_and_call() {
        let call = ASTNode::new_call(
            ASTNode::Identifier("add".into()),
            vec![ASTNode::NumberLiteral(2.0), ASTNode::NumberLiteral(3.0)],
        );
        let prog = ASTNode::Program(vec![ASTNode::new_assignment("x", call)]);
        let mut g = CodeGenerator::new();
        let js = g.generate(&prog).unwrap();
        assert!(js.contains("x = add(2, 3);"));
    }
}
