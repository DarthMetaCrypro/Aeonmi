// src/core/semantic_analyzer.rs
//! Minimal semantic analyzer:
//! - Tracks variable declarations per scope
//! - Errors on re-declaration in same scope (existing behavior)
//! - NEW: Errors on assignment to undeclared identifier

use std::collections::HashSet;
use crate::core::ast::ASTNode;

#[derive(Default)]
pub struct SemanticAnalyzer {
    scopes: Vec<HashSet<String>>,
    errors: Vec<String>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self { scopes: vec![HashSet::new()], errors: vec![] }
    }

    pub fn analyze(&mut self, ast: &ASTNode) -> Result<(), String> {
        self.visit(ast);
        if self.errors.is_empty() { Ok(()) } else { Err(self.errors.join("\n")) }
    }

    fn begin_scope(&mut self) { self.scopes.push(HashSet::new()); }
    fn end_scope(&mut self) { self.scopes.pop(); }

    fn declare(&mut self, name: &str) {
        let scope = self.scopes.last_mut().unwrap();
        if scope.contains(name) {
            self.errors.push(format!("Redeclaration of '{}'", name));
        } else {
            scope.insert(name.to_string());
        }
    }

    fn is_declared(&self, name: &str) -> bool {
        for scope in self.scopes.iter().rev() {
            if scope.contains(name) { return true; }
        }
        false
    }

    fn visit(&mut self, node: &ASTNode) {
        match node {
            ASTNode::Program(items) => {
                for it in items { self.visit(it); }
            }
            ASTNode::Block(items) => {
                self.begin_scope();
                for it in items { self.visit(it); }
                self.end_scope();
            }
            ASTNode::Function { name: _, params, body } => {
                self.begin_scope();
                for p in params { self.declare(p); }
                for it in body { self.visit(it); }
                self.end_scope();
            }
            ASTNode::VariableDecl { name, value } => {
                self.visit(value);
                self.declare(name);
            }
            ASTNode::Assignment { name, value } => {
                if !self.is_declared(name) {
                    self.errors.push(format!("Assignment to undeclared variable '{}'", name));
                }
                self.visit(value);
            }
            ASTNode::Return(expr) |
            ASTNode::Log(expr) |
            ASTNode::While { condition: expr, body: _ } => {
                self.visit(expr);
            }
            ASTNode::If { condition, then_branch, else_branch } => {
                self.visit(condition);
                self.visit(then_branch);
                if let Some(e) = else_branch { self.visit(e); }
            }
            ASTNode::For { init, condition, increment, body } => {
                self.begin_scope();
                if let Some(i) = init { self.visit(i); }
                if let Some(c) = condition { self.visit(c); }
                if let Some(inc) = increment { self.visit(inc); }
                self.visit(body);
                self.end_scope();
            }
            ASTNode::BinaryExpr { left, right, .. } => {
                self.visit(left);
                self.visit(right);
            }
            ASTNode::UnaryExpr { expr, .. } => self.visit(expr),
            ASTNode::Call { callee, args } => {
                self.visit(callee);
                for a in args { self.visit(a); }
            }

            // literals / identifiers / quantum/glyph / error
            ASTNode::Identifier(_) |
            ASTNode::NumberLiteral(_) |
            ASTNode::StringLiteral(_) |
            ASTNode::BooleanLiteral(_) |
            ASTNode::QuantumOp {..} |
            ASTNode::HieroglyphicOp {..} |
            ASTNode::Error(_) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ast::ASTNode;

    #[test]
    fn redeclare_fails() {
        let ast = ASTNode::Program(vec![
            ASTNode::new_variable_decl("x", ASTNode::NumberLiteral(1.0)),
            ASTNode::new_variable_decl("x", ASTNode::NumberLiteral(2.0)),
        ]);
        let mut a = SemanticAnalyzer::new();
        assert!(a.analyze(&ast).is_err());
    }

    #[test]
    fn assignment_to_undeclared_fails() {
        let ast = ASTNode::Program(vec![
            ASTNode::new_assignment("x", ASTNode::NumberLiteral(1.0)),
        ]);
        let mut a = SemanticAnalyzer::new();
        assert!(a.analyze(&ast).is_err());
    }

    #[test]
    fn assignment_to_declared_ok() {
        let ast = ASTNode::Program(vec![
            ASTNode::new_variable_decl("x", ASTNode::NumberLiteral(1.0)),
            ASTNode::new_assignment("x", ASTNode::NumberLiteral(2.0)),
        ]);
        let mut a = SemanticAnalyzer::new();
        assert!(a.analyze(&ast).is_ok());
    }
}
