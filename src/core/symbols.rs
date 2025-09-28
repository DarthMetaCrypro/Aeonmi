//! Symbol extraction for outline / navigation.
use crate::core::ast::{ASTNode, FunctionParam};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct SymbolInfo {
    pub kind: SymbolKind,
    pub name: String,
    pub line: usize,
    pub column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all="lowercase")]
pub enum SymbolKind { Function, Variable, Parameter }

pub fn collect_symbols(ast: &ASTNode) -> Vec<SymbolInfo> {
    let mut out = Vec::new();
    visit(ast, &mut out);
    out
}

fn visit(node: &ASTNode, out: &mut Vec<SymbolInfo>) {
    match node {
        ASTNode::Program(items) | ASTNode::Block(items) => { for it in items { visit(it, out); } }
        ASTNode::Function { name, line, column, params, body } => {
            out.push(SymbolInfo { kind: SymbolKind::Function, name: name.clone(), line: *line, column: *column, end_line: *line, end_column: *column + name.len().max(1), });
            for FunctionParam { name, line, column } in params { out.push(SymbolInfo { kind: SymbolKind::Parameter, name: name.clone(), line: *line, column: *column, end_line: *line, end_column: *column + name.len().max(1) }); }
            for st in body { visit(st, out); }
        }
        ASTNode::VariableDecl { name, line, column, .. } => {
            out.push(SymbolInfo { kind: SymbolKind::Variable, name: name.clone(), line: *line, column: *column, end_line: *line, end_column: *column + name.len().max(1) });
        }
        ASTNode::Assignment { .. }
        | ASTNode::Return(_)
        | ASTNode::Log(_)
        | ASTNode::If { .. }
        | ASTNode::While { .. }
        | ASTNode::For { .. }
        | ASTNode::BinaryExpr { .. }
        | ASTNode::UnaryExpr { .. }
        | ASTNode::Call { .. }
        | ASTNode::Identifier(_)
        | ASTNode::IdentifierSpanned { .. }
        | ASTNode::NumberLiteral(_)
        | ASTNode::StringLiteral(_)
        | ASTNode::BooleanLiteral(_)
        | ASTNode::QuantumOp { .. }
        | ASTNode::HieroglyphicOp { .. }
        | ASTNode::Error(_) => {}
    }
}
