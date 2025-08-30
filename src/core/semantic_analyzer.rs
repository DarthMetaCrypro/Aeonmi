// src/core/semantic_analyzer.rs
//! Minimal semantic analyzer:
//! - Tracks variable declarations per scope
//! - Errors on re-declaration in same scope (existing behavior)
//! - NEW: Errors on assignment to undeclared identifier
//! Next steps (planned incremental expansion):
//! 1. Track function call sites to emit warning for unused private (non-exported) functions.
//! 2. Basic type tagging (number, bool, string) and arithmetic / comparison operand checks.
//! 3. Simple return path consistency: warn if some paths lack return in a function that returns early elsewhere.
//! 4. Coercion rules scaffold (e.g. number <-> string in concatenation) with warnings.
//! 5. Quantum / glyph op arity validation.

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

#[derive(Clone, Copy, Debug, PartialEq)]
enum ValueType { Number, String, Bool, Unknown }

impl Default for ValueType { fn default() -> Self { ValueType::Unknown } }

#[derive(Default)]
struct VarInfo { line: usize, column: usize, used: bool, ty: ValueType }

pub struct SemanticAnalyzer {
    scopes: Vec<HashSet<String>>,
    var_meta: Vec<std::collections::HashMap<String, VarInfo>>, // parallel stack with metadata
    functions: HashMap<String, (usize, usize)>, // track function declarations (line,column) for duplicate detection
    used_functions: HashSet<String>,            // function call sites
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
            used_functions: HashSet::new(),
        }
    }

    pub fn analyze(&mut self, ast: &ASTNode) -> Result<(), String> {
        self.visit(ast, false);
        self.post_pass();
        self.flush_unused_warnings();
        if self.errors.is_empty() { Ok(()) } else { Err(self.errors.join("\n")) }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn analyze_with_spans(&mut self, ast: &ASTNode) -> Vec<SemanticDiagnostic> {
        self.visit(ast, true);
        self.post_pass();
        self.flush_unused_warnings();
        self.diags.clone()
    }

    fn post_pass(&mut self) {
        // Placeholder for future multi-pass checks (e.g., unused function detection)
        // Currently detects functions never referenced (simple heuristic: name never marked used as identifier)
        // This is conservative and may false-positive for indirect calls.
        for (name,(line,column)) in self.functions.clone() { // clone to avoid borrow issues
            // skip if any scope recorded it as used identifier
        if !self.used_functions.contains(&name) {
                self.diags.push(SemanticDiagnostic { message: format!("Unused function '{name}'"), line, column, len: name.len().max(1), severity: Severity::Warning });
            }
        }
    }

    // identifier_was_used removed (replaced by used_functions set)

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
                meta.insert(name.to_string(), VarInfo { line: l, column: c, used: false, ty: ValueType::Unknown });
            } else {
                meta.insert(name.to_string(), VarInfo { line: 0, column: 0, used: false, ty: ValueType::Unknown });
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
                let mut return_types: Vec<ValueType> = Vec::new();
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
                    if let ASTNode::Return(expr) = it { 
                        saw_return = true; 
                        let ty = self.expr_type(expr);
                        return_types.push(ty);
                    }
                    self.visit(it, capture);
                }
                self.end_scope();
                if saw_return {
                    // Path consistency heuristic: if last stmt not a Return warn missing terminal return.
                    if !matches!(body.last(), Some(ASTNode::Return(_))) {
                        if capture { self.diags.push(SemanticDiagnostic { message: format!("Not all code paths return a value in function '{name}'"), line: *line, column: *column, len: name.len().max(1), severity: Severity::Warning }); }
                    }
                    // Return type consistency (ignore Unknown)
                    let mut distinct: Vec<ValueType> = return_types.iter().copied().filter(|t| *t != ValueType::Unknown).collect();
                    distinct.sort_by(|a,b| (*a as u8).cmp(&(*b as u8)));
                    distinct.dedup();
                    if distinct.len() > 1 {
                        if capture { self.diags.push(SemanticDiagnostic { message: format!("Inconsistent return types in function '{name}'"), line: *line, column: *column, len: name.len().max(1), severity: Severity::Warning }); }
                    }
                }
            }
            ASTNode::VariableDecl { name, value, line, column } => {
                self.visit(value, capture);
                self.declare(name, Some(*line), Some(*column));
                let ty = self.expr_type(value);
                self.set_var_type(name, ty);
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
            ASTNode::BinaryExpr { op, left, right } => {
                self.visit(left, capture);
                self.visit(right, capture);
                self.check_binary(op, left, right, capture);
            }
            ASTNode::UnaryExpr { expr, .. } => self.visit(expr, capture),
            ASTNode::Call { callee, args } => {
                if let ASTNode::Identifier(n) = &**callee { self.used_functions.insert(n.clone()); }
                if let ASTNode::IdentifierSpanned { name, .. } = &**callee { self.used_functions.insert(name.clone()); }
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
            | ASTNode::HieroglyphicOp { .. }
            | ASTNode::Error(_) => {}
            ASTNode::QuantumOp { op, qubits } => {
                // Arity validation
                let qlen = qubits.len();
                let (min, kind_name) = match op {
                    crate::core::token::TokenKind::Superpose => (1, "superpose"),
                    crate::core::token::TokenKind::Entangle => (2, "entangle"),
                    crate::core::token::TokenKind::Measure => (1, "measure"),
                    crate::core::token::TokenKind::Dod => (1, "dod"),
                    _ => (0, "unknown")
                };
                if qlen < min {
                    let msg = format!("Quantum op '{kind_name}' expects >= {min} qubit(s) but got {qlen}");
                    self.errors.push(msg.clone());
                    if capture { self.diags.push(SemanticDiagnostic { message: msg, line: 0, column: 0, len: 1, severity: Severity::Error }); }
                }
            }
        }
    }

    fn mark_used(&mut self, name: &str) {
        for map in self.var_meta.iter_mut().rev() {
            if let Some(v) = map.get_mut(name) { v.used = true; return; }
        }
    }

    fn set_var_type(&mut self, name: &str, ty: ValueType) {
        for map in self.var_meta.iter_mut().rev() {
            if let Some(v) = map.get_mut(name) { v.ty = ty; return; }
        }
    }

    fn get_var_type(&self, name: &str) -> ValueType {
        for map in self.var_meta.iter().rev() {
            if let Some(v) = map.get(name) { return v.ty; }
        }
        ValueType::Unknown
    }

    fn expr_type(&self, node: &ASTNode) -> ValueType {
        use ValueType::*;
        match node {
            ASTNode::NumberLiteral(_) => Number,
            ASTNode::StringLiteral(_) => String,
            ASTNode::BooleanLiteral(_) => Bool,
            ASTNode::Identifier(n) => self.get_var_type(n),
            ASTNode::IdentifierSpanned { name, .. } => self.get_var_type(name),
            ASTNode::BinaryExpr { op, left, right } => {
                let lt = self.expr_type(left); let rt = self.expr_type(right);
                match op {
                    crate::core::token::TokenKind::Plus => {
                        if lt == String || rt == String { String } else if lt == Number && rt == Number { Number } else { Unknown }
                    }
                    crate::core::token::TokenKind::Minus | crate::core::token::TokenKind::Star | crate::core::token::TokenKind::Slash => {
                        if lt == Number && rt == Number { Number } else { Unknown }
                    }
                    crate::core::token::TokenKind::DoubleEquals | crate::core::token::TokenKind::NotEquals | crate::core::token::TokenKind::LessThan | crate::core::token::TokenKind::LessEqual | crate::core::token::TokenKind::GreaterThan | crate::core::token::TokenKind::GreaterEqual => Bool,
                    _ => Unknown
                }
            }
            ASTNode::UnaryExpr { op: _, expr } => self.expr_type(expr),
            ASTNode::Call { .. } => Unknown,
            _ => Unknown,
        }
    }

    fn check_binary(&mut self, op: &crate::core::token::TokenKind, left: &ASTNode, right: &ASTNode, capture: bool) {
        use crate::core::token::TokenKind as TK; use ValueType::*;
        let lt = self.expr_type(left); let rt = self.expr_type(right);
        match op {
            TK::Plus => {
                // Be permissive with Unknown types (parameters / unresolved) to avoid false positives.
                if lt == Number && rt == Number { return; }
                if lt == String && rt == String { return; }
                if lt == Unknown || rt == Unknown { return; }
                if (lt == String && rt == Number) || (lt == Number && rt == String) {
                    if capture { self.diags.push(SemanticDiagnostic { message: "Implicit number/string coercion in '+'".into(), line: 0, column: 0, len: 1, severity: Severity::Warning }); }
                } else { self.push_type_error("Invalid operands for '+'", capture); }
            }
            TK::Minus | TK::Star | TK::Slash => { if lt != Number || rt != Number { if lt != Unknown && rt != Unknown { self.push_type_error("Arithmetic operands must be numbers", capture); } } }
            TK::LessThan | TK::LessEqual | TK::GreaterThan | TK::GreaterEqual => { if lt != Number || rt != Number { if lt != Unknown && rt != Unknown { self.push_type_error("Comparison operands must be numbers", capture); } } }
            _ => {}
        }
    }

    fn push_type_error(&mut self, msg: &str, capture: bool) {
        self.errors.push(msg.to_string());
        if capture { self.diags.push(SemanticDiagnostic { message: msg.to_string(), line: 0, column: 0, len: 1, severity: Severity::Error }); }
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

    #[test]
    fn unreachable_after_return_warns() {
        let ast = ASTNode::Program(vec![
            ASTNode::new_function("f", vec![], vec![
                ASTNode::new_return(ASTNode::NumberLiteral(1.0)),
                ASTNode::new_variable_decl("z", ASTNode::NumberLiteral(2.0)),
            ]),
        ]);
        let mut a = SemanticAnalyzer::new();
        let diags = a.analyze_with_spans(&ast);
        assert!(diags.iter().any(|d| d.message.contains("Unreachable code after return")));
    }

    #[test]
    fn unused_variable_warning() {
        let ast = ASTNode::Program(vec![
            ASTNode::new_variable_decl("unused", ASTNode::NumberLiteral(0.0)),
        ]);
        let mut a = SemanticAnalyzer::new();
        let diags = a.analyze_with_spans(&ast);
        assert!(diags.iter().any(|d| d.message.contains("Unused variable 'unused'")));
    }
}
