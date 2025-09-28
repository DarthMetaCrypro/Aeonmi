//! Minimal type system scaffold.
//! Provides primitive types and a simple inference + checking routine.

use crate::core::ast::ASTNode;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TypeKind { Number, Boolean, String, Void, Unknown }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeInfo {
    pub ty: TypeKind,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDiagnostic { pub message: String, pub line: usize, pub column: usize }

pub struct TypeContext {
    scopes: Vec<std::collections::HashMap<String, TypeKind>>,
    pub diags: Vec<TypeDiagnostic>,
    functions: std::collections::HashMap<String, (Vec<TypeKind>, TypeKind)>, // name -> (param types, return type)
}

impl TypeContext {
    pub fn new() -> Self { Self { scopes: vec![Default::default()], diags: vec![], functions: Default::default() } }
    fn begin_scope(&mut self){ self.scopes.push(Default::default()); }
    fn end_scope(&mut self){ self.scopes.pop(); }
    fn declare(&mut self, name: &str, ty: TypeKind) { if let Some(s) = self.scopes.last_mut() { s.insert(name.to_string(), ty); } }
    fn lookup(&self, name: &str) -> TypeKind { for s in self.scopes.iter().rev() { if let Some(t) = s.get(name) { return t.clone(); } } TypeKind::Unknown }
    fn update_if_unknown(&mut self, name: &str, ty: &TypeKind) {
        if *ty == TypeKind::Unknown || *ty == TypeKind::Void { return; }
        for s in self.scopes.iter().rev() {
            if let Some(slot) = s.get(name) { if *slot != TypeKind::Unknown { return; } }
        }
        if let Some(s) = self.scopes.last_mut() { if let Some(slot) = s.get_mut(name) { if *slot == TypeKind::Unknown { *slot = ty.clone(); } } }
    }

    pub fn infer_program(&mut self, ast: &ASTNode) { self.visit(ast); }

    fn visit(&mut self, node: &ASTNode) -> TypeKind {
        match node {
            ASTNode::Program(items) => { for it in items { self.visit(it); } TypeKind::Void }
            ASTNode::Block(items) => { self.begin_scope(); for it in items { self.visit(it); } self.end_scope(); TypeKind::Void }
            ASTNode::Function { name, params, body, .. } => {
                self.begin_scope();
                // Predeclare params
                for p in params { self.declare(&p.name, TypeKind::Unknown); }
                let mut ret_type: TypeKind = TypeKind::Void;
                for it in body {
                    if let ASTNode::Return(expr) = it { ret_type = self.visit(expr); } else { self.visit(it); }
                }
                let param_types: Vec<TypeKind> = params.iter().map(|p| self.lookup(&p.name)).collect();
                self.functions.insert(name.clone(), (param_types, ret_type.clone()));
                self.end_scope();
                TypeKind::Void
            }
            ASTNode::VariableDecl { name, value, line, column } => { let t = self.visit(value); self.declare(name, t.clone()); if t==TypeKind::Void { self.diags.push(TypeDiagnostic{ message: format!("Variable '{name}' initialized with void"), line:*line, column:*column }); } TypeKind::Void }
            ASTNode::Assignment { name, value, line, column } => { let lhs = self.lookup(name); let rhs = self.visit(value); if lhs!=TypeKind::Unknown && lhs!=rhs && rhs!=TypeKind::Unknown { self.diags.push(TypeDiagnostic { message: format!("Type mismatch assigning {rhs:?} to {lhs:?}"), line:*line, column:*column }); } else if lhs==TypeKind::Unknown { self.update_if_unknown(name, &rhs); } TypeKind::Void }
            ASTNode::Return(expr) => { self.visit(expr); TypeKind::Void }
            ASTNode::Log(expr) => { self.visit(expr); TypeKind::Void }
            ASTNode::If { condition, then_branch, else_branch } => { let ct = self.visit(condition); if ct!=TypeKind::Boolean && ct!=TypeKind::Unknown { self.diags.push(TypeDiagnostic { message: "If condition not boolean".into(), line:0, column:0 }); } self.visit(then_branch); if let Some(e)=else_branch { self.visit(e); } TypeKind::Void }
            ASTNode::While { condition, body } => { let ct=self.visit(condition); if ct!=TypeKind::Boolean && ct!=TypeKind::Unknown { self.diags.push(TypeDiagnostic { message: "While condition not boolean".into(), line:0, column:0 }); } self.visit(body); TypeKind::Void }
            ASTNode::For { init, condition, increment, body } => { if let Some(i)=init { self.visit(i); } if let Some(c)=condition { let ct=self.visit(c); if ct!=TypeKind::Boolean && ct!=TypeKind::Unknown { self.diags.push(TypeDiagnostic { message: "For condition not boolean".into(), line:0, column:0 }); } } if let Some(inc)=increment { self.visit(inc); } self.visit(body); TypeKind::Void }
            ASTNode::BinaryExpr { op, left, right } => {
                let lt=self.visit(left); let rt=self.visit(right);
                use crate::core::token::TokenKind::*;
                let result = match op {
                    Plus | Minus | Star | Slash => {
                        if lt==TypeKind::Unknown && rt==TypeKind::Number { return TypeKind::Number; }
                        if rt==TypeKind::Unknown && lt==TypeKind::Number { return TypeKind::Number; }
                        if lt!=TypeKind::Number || rt!=TypeKind::Number { self.diags.push(TypeDiagnostic { message: "Arithmetic on non-number".into(), line:0, column:0 }); TypeKind::Unknown } else { TypeKind::Number }
                    }
                    DoubleEquals | NotEquals => {
                        if lt!=rt && lt!=TypeKind::Unknown && rt!=TypeKind::Unknown { self.diags.push(TypeDiagnostic { message: "Equality between different types".into(), line:0, column:0 }); }
                        TypeKind::Boolean
                    }
                    GreaterThan | GreaterEqual | LessThan | LessEqual => {
                        if lt!=TypeKind::Number || rt!=TypeKind::Number { self.diags.push(TypeDiagnostic { message: "Comparison on non-number".into(), line:0, column:0 }); }
                        TypeKind::Boolean
                    }
                    _ => { if lt!=rt && lt!=TypeKind::Unknown && rt!=TypeKind::Unknown { TypeKind::Unknown } else { lt } }
                };
                result
            }
            ASTNode::UnaryExpr { expr, .. } => self.visit(expr),
            ASTNode::Call { callee, args } => {
                // Only support direct identifier calls for now
                let (fname, f_info) = match &**callee { ASTNode::Identifier(n) | ASTNode::IdentifierSpanned { name: n, .. } => {
                    let info = self.functions.get(n).cloned(); (n.clone(), info)
                }, _ => (String::new(), None) };
                for a in args { self.visit(a); }
                if let Some((param_tys, ret_ty)) = f_info {
                    // simple arity check
                    if param_tys.len() != args.len() { self.diags.push(TypeDiagnostic { message: format!("Call arity mismatch for {fname}"), line:0, column:0 }); }
                    ret_ty
                } else { TypeKind::Unknown }
            }
            ASTNode::Identifier(name) => self.lookup(name),
            ASTNode::IdentifierSpanned { name, .. } => self.lookup(name),
            ASTNode::NumberLiteral(_) => TypeKind::Number,
            ASTNode::StringLiteral(_) => TypeKind::String,
            ASTNode::BooleanLiteral(_) => TypeKind::Boolean,
            ASTNode::QuantumOp { .. } => TypeKind::Void,
            ASTNode::HieroglyphicOp { .. } => TypeKind::Void,
            ASTNode::Error(_) => TypeKind::Unknown,
        }
    }
}
