//! Simple code actions scaffold.
#![allow(dead_code)] // Entire module is a forward-looking scaffold not yet wired.
//! Provides structure for future intelligent refactors.

use crate::core::ast::ASTNode;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeAction {
    pub title: String,
    pub kind: String,
    pub line: usize,
    pub column: usize,
}

pub fn suggest_actions(ast: &ASTNode) -> Vec<CodeAction> {
    let mut v = Vec::new();
    collect(ast, &mut v);
    // Post-pass heuristics for new actions
    add_missing_let_actions(ast, &mut v);
    add_inline_variable_actions(ast, &mut v);
    v
}

fn collect(node: &ASTNode, out: &mut Vec<CodeAction>) {
    match node {
        ASTNode::Program(items) | ASTNode::Block(items) => {
            for it in items { collect(it, out); }
        }
        ASTNode::Function { name, line, column, body, .. } => {
            // Suggest extract function if function body large (>5 statements)
            if body.len() > 5 { out.push(CodeAction { title: format!("Extract part of '{name}' into function"), kind: "extractFunction".into(), line: *line, column: *column }); }
            for it in body { collect(it, out); }
        }
        ASTNode::VariableDecl { name, line, column, .. } => {
            out.push(CodeAction { title: format!("Rename variable '{name}'"), kind: "rename".into(), line: *line, column: *column });
        }
        ASTNode::Assignment { line, column, .. } => {
            out.push(CodeAction { title: "Introduce variable".into(), kind: "introduceVariable".into(), line: *line, column: *column });
        }
        _ => {}
    }
}

// Heuristic: assignment to identifier with no prior VariableDecl of same name -> suggest Add missing let
fn add_missing_let_actions(ast: &ASTNode, out: &mut Vec<CodeAction>) {
    use std::collections::HashSet;
    let mut declared: HashSet<String> = HashSet::new();
    fn walk(node: &ASTNode, declared: &mut std::collections::HashSet<String>, out: &mut Vec<CodeAction>) {
        match node {
            ASTNode::VariableDecl { name, .. } => { declared.insert(name.clone()); }
            ASTNode::Assignment { name, line, column, .. } => {
                if !declared.contains(name) { out.push(CodeAction { title: format!("Add missing 'let' for '{name}'"), kind: "addMissingLet".into(), line: *line, column: *column }); declared.insert(name.clone()); }
            }
            ASTNode::Program(items) | ASTNode::Block(items) => { for it in items { walk(it, declared, out); } }
            ASTNode::Function { body, .. } => { for it in body { walk(it, &mut declared.clone(), out); } }
            ASTNode::If { then_branch, else_branch, .. } => { walk(then_branch, &mut declared.clone(), out); if let Some(e)=else_branch { walk(e, &mut declared.clone(), out); } }
            ASTNode::While { body, .. } | ASTNode::For { body, .. } => { walk(body, &mut declared.clone(), out); }
            _ => {}
        }
    }
    walk(ast, &mut declared, out);
}

// Heuristic: variable declared then used exactly once -> suggest inline variable at decl site
fn add_inline_variable_actions(ast: &ASTNode, out: &mut Vec<CodeAction>) {
    use std::collections::HashMap;
    #[derive(Default)] struct Info { decl: Option<(usize,usize)>, uses: usize }
    let mut map: HashMap<String, Info> = HashMap::new();
    fn scan(node: &ASTNode, map: &mut std::collections::HashMap<String, Info>) {
        match node {
            ASTNode::VariableDecl { name, line, column, value: _ } => { map.entry(name.clone()).or_default().decl = Some((*line,*column)); }
            ASTNode::IdentifierSpanned { name, line: _, column: _, .. } => {
                let e = map.entry(name.clone()).or_default(); if e.decl.is_some() { e.uses += 1; }
            }
            ASTNode::Identifier(name) => { let e = map.entry(name.clone()).or_default(); if e.decl.is_some() { e.uses += 1; } }
            ASTNode::Program(items) | ASTNode::Block(items) => { for it in items { scan(it, map); } }
            ASTNode::Function { body, .. } => { for it in body { scan(it, map); } }
            ASTNode::If { then_branch, else_branch, .. } => { scan(then_branch, map); if let Some(e)=else_branch { scan(e, map); } }
            ASTNode::While { body, .. } | ASTNode::For { body, .. } => { scan(body, map); }
            ASTNode::Assignment { value, .. } | ASTNode::Return(value) | ASTNode::Log(value) => { scan(value, map); }
            ASTNode::BinaryExpr { left, right, .. } => { scan(left, map); scan(right, map); }
            ASTNode::UnaryExpr { expr, .. } => { scan(expr, map); }
            ASTNode::Call { callee, args } => { scan(callee, map); for a in args { scan(a, map); } }
            _ => {}
        }
    }
    scan(ast, &mut map);
    for (name, info) in map.iter() { if info.uses == 1 { if let Some((l,c)) = info.decl { out.push(CodeAction { title: format!("Inline variable '{name}'"), kind: "inlineVariable".into(), line: l, column: c }); } } }
}
