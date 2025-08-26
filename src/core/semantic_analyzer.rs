// src/core/semantic_analyzer.rs
//! Minimal semantic analyzer:
//! - Tracks variable declarations per scope
//! - Errors on re-declaration in same scope (existing behavior)
//! - NEW: Errors on assignment to undeclared identifier

use crate::core::ast::{ASTNode, FunctionParam};
use std::collections::{HashSet, HashMap};

#[derive(Debug, Clone)]
pub struct SemanticDiagnostic {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub len: usize,
    pub severity: Severity,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Severity { Error, Warning }

#[derive(Default)]
struct VarInfo { line: usize, column: usize, used: bool }

pub struct SemanticAnalyzer {
    scopes: Vec<HashSet<String>>,
    var_meta: Vec<std::collections::HashMap<String, VarInfo>>, // parallel stack with metadata
    functions: HashMap<String, (usize, usize)>, // track function declarations (line,column) for duplicate detection
    errors: Vec<String>,            // legacy string list for existing callers
    diags: Vec<SemanticDiagnostic>, // unified diagnostics (errors + warnings)
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashSet::new()],
            var_meta: vec![std::collections::HashMap::new()],
            errors: vec![],
            diags: vec![],
            functions: HashMap::new(),
        }
    }

    pub fn analyze(&mut self, ast: &ASTNode) -> Result<(), String> {
        self.visit(ast, false);
        self.flush_unused_warnings();
        if self.errors.is_empty() { Ok(()) } else { Err(self.errors.join("\n")) }
    }

    pub fn analyze_with_spans(&mut self, ast: &ASTNode) -> Vec<SemanticDiagnostic> {
        self.visit(ast, true);
        self.flush_unused_warnings();
        self.diags.clone()
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashSet::new());
        self.var_meta.push(std::collections::HashMap::new());
    }
    fn end_scope(&mut self) {
        self.scopes.pop();
        if let Some(map) = self.var_meta.pop() {
            // Emit warnings for unused variables in this scope
            for (name, info) in map.into_iter() {
                if !info.used {
                    let msg = format!("Unused variable '{}'", name);
                    self.diags.push(SemanticDiagnostic { message: msg, line: info.line, column: info.column, len: name.len().max(1), severity: Severity::Warning });
                }
            }
        }
    }

    fn declare(&mut self, name: &str, line: Option<usize>, column: Option<usize>) {
        let scope = self.scopes.last_mut().unwrap();
        let meta = self.var_meta.last_mut().unwrap();
        if scope.contains(name) {
            let msg = format!("Redeclaration of '{}'", name);
            self.errors.push(msg.clone());
            if let (Some(l), Some(c)) = (line, column) {
                self.diags.push(SemanticDiagnostic { message: msg, line: l, column: c, len: name.len().max(1), severity: Severity::Error });
            }
        } else {
            scope.insert(name.to_string());
            if let (Some(l), Some(c)) = (line, column) {
                meta.insert(name.to_string(), VarInfo { line: l, column: c, used: false });
            } else {
                meta.insert(name.to_string(), VarInfo { line: 0, column: 0, used: false });
            }
        }
    }

    fn is_declared(&self, name: &str) -> bool {
        for scope in self.scopes.iter().rev() {
            if scope.contains(name) {
                return true;
            }
        }
        false
    }

    fn visit(&mut self, node: &ASTNode, capture: bool) {
        match node {
            ASTNode::Program(items) => {
                for it in items {
                    self.visit(it, capture);
                }
            }
            ASTNode::Block(items) => {
                self.begin_scope();
                for it in items {
                    self.visit(it, capture);
                }
                self.end_scope();
            }
            ASTNode::Function { name, line, column, params, body } => {
                // duplicate function detection
                if let Some((prev_l, prev_c)) = self.functions.get(name) {
                    let msg = format!("Duplicate function '{name}' (previous at {prev_l}:{prev_c})");
                    self.errors.push(msg.clone());
                    if capture { self.diags.push(SemanticDiagnostic { message: msg, line: *line, column: *column, len: name.len().max(1), severity: Severity::Error }); }
                } else {
                    self.functions.insert(name.clone(), (*line, *column));
                }
                self.begin_scope();
                for FunctionParam { name, line, column } in params {
                    self.declare(name, Some(*line), Some(*column));
                    // Mark parameters immediately as used? Keep as unused to surface warnings if not referenced.
                }
                let mut saw_return = false;
                for it in body {
                    if saw_return {
                        // unreachable code warning
                        if capture {
                            // approximate span: use start location if available via pattern matching
                            let (l,c) = match it { ASTNode::VariableDecl { line, column, .. } => (*line,*column),
                                                   ASTNode::Assignment { line, column, .. } => (*line,*column),
                                                   ASTNode::Function { line, column, .. } => (*line,*column),
                                                   ASTNode::IdentifierSpanned { line, column, .. } => (*line,*column),
                                                   _ => (0,0) };
                            self.diags.push(SemanticDiagnostic { message: "Unreachable code after return".into(), line: l, column: c, len: 1, severity: Severity::Warning });
                        }
                        // still traverse in case of further symbol usage (optionally skip)
                        self.visit(it, capture);
                        continue;
                    }
                    if matches!(it, ASTNode::Return(_)) { saw_return = true; }
                    self.visit(it, capture);
                }
                self.end_scope();
            }
            ASTNode::VariableDecl { name, value, line, column } => {
                self.visit(value, capture);
                self.declare(name, Some(*line), Some(*column));
            }
            ASTNode::Assignment { name, value, line, column } => {
                if !self.is_declared(name) {
                    let msg = format!("Assignment to undeclared variable '{}'", name);
                    self.errors.push(msg.clone());
                    if capture {
                        self.diags.push(SemanticDiagnostic { message: msg, line: *line, column: *column, len: name.len().max(1), severity: Severity::Error });
                    }
                }
                // write counts as a use
                self.visit(value, capture);
                self.mark_used(name);
            }
            ASTNode::Return(expr)
            | ASTNode::Log(expr)
            | ASTNode::While {
                condition: expr,
                body: _,
            } => {
                self.visit(expr, capture);
            }
            ASTNode::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.visit(condition, capture);
                self.visit(then_branch, capture);
                if let Some(e) = else_branch {
                    self.visit(e, capture);
                }
            }
            ASTNode::For {
                init,
                condition,
                increment,
                body,
            } => {
                self.begin_scope();
                if let Some(i) = init {
                    self.visit(i, capture);
                }
                if let Some(c) = condition {
                    self.visit(c, capture);
                }
                if let Some(inc) = increment {
                    self.visit(inc, capture);
                }
                self.visit(body, capture);
                self.end_scope();
            }
            ASTNode::BinaryExpr { left, right, .. } => {
                self.visit(left, capture);
                self.visit(right, capture);
            }
            ASTNode::UnaryExpr { expr, .. } => self.visit(expr, capture),
            ASTNode::Call { callee, args } => {
                self.visit(callee, capture);
                for a in args {
                    self.visit(a, capture);
                }
            }

            // literals / identifiers / quantum/glyph / error
            ASTNode::Identifier(name) => { self.mark_used(name); }
            ASTNode::IdentifierSpanned { name, .. } => { self.mark_used(name); }
            ASTNode::NumberLiteral(_)
            | ASTNode::StringLiteral(_)
            | ASTNode::BooleanLiteral(_)
            | ASTNode::QuantumOp { .. }
            | ASTNode::HieroglyphicOp { .. }
            | ASTNode::Error(_) => {}
        }
    }

    fn mark_used(&mut self, name: &str) {
        for map in self.var_meta.iter_mut().rev() {
            if let Some(v) = map.get_mut(name) { v.used = true; return; }
        }
    }

    fn flush_unused_warnings(&mut self) {
        // Trigger end_scope logic for remaining scopes without popping global (avoid double warnings)
        // We'll only process the top-most (innermost) because outer scopes handled during normal popping.
        // Global scope warnings will be generated here.
        if self.var_meta.len() == 1 {
            if let Some(global) = self.var_meta.last() {
                for (name, info) in global.iter() {
                    if !info.used {
                        let msg = format!("Unused variable '{}'", name);
                        self.diags.push(SemanticDiagnostic { message: msg, line: info.line, column: info.column, len: name.len().max(1), severity: Severity::Warning });
                    }
                }
            }
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
        let ast = ASTNode::Program(vec![ASTNode::new_assignment(
            "x",
            ASTNode::NumberLiteral(1.0),
        )]);
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
