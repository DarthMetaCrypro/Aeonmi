//! Scope map builder for safer renames.
use crate::core::ast::ASTNode;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct ScopeEntry {
    pub scope_id: usize,
    pub line: usize,
    pub column: usize,
    pub is_def: bool,
}

#[derive(Debug, Default)]
pub struct ScopeMap {
    // name -> list of occurrences with scope id
    pub symbols: HashMap<String, Vec<ScopeEntry>>,
    // scope nesting (parent id)
    parents: Vec<Option<usize>>,
}

impl ScopeMap {
    pub fn build(ast: &ASTNode) -> Self {
        let mut sm = ScopeMap { symbols: HashMap::new(), parents: vec![None] };
        let mut stack: Vec<usize> = vec![0];
        visit(ast, &mut sm, &mut stack, 0);
        sm
    }
    pub fn occurrences_in_same_scope(&self, name: &str, line: usize, column: usize) -> Vec<(usize,usize,bool)> {
        let mut target_scope: Option<usize> = None;
        if let Some(entries) = self.symbols.get(name) {
            for e in entries {
                if e.line == line && e.column == column { target_scope = Some(e.scope_id); break; }
            }
        }
        let Some(ts) = target_scope else { return vec![] };
    if let Some(entries) = self.symbols.get(name) { entries.iter().filter(|e| e.scope_id == ts).map(|e| (e.line, e.column, e.is_def)).collect() } else { vec![] }
    }
}

fn visit(node: &ASTNode, sm: &mut ScopeMap, stack: &mut Vec<usize>, current: usize) {
    use ASTNode::*;
    match node {
        Program(items) => { for it in items { visit(it, sm, stack, current); } }
        Block(items) => {
            let new_id = sm.parents.len(); sm.parents.push(Some(current)); stack.push(new_id);
            for it in items { visit(it, sm, stack, new_id); }
            stack.pop();
        }
        Function { name, line, column, params, body, .. } => {
            record(sm, name, *line, *column, *stack.last().unwrap(), true);
            let new_id = sm.parents.len(); sm.parents.push(Some(current)); stack.push(new_id);
            for p in params { record(sm, &p.name, p.line, p.column, new_id, true); }
            for st in body { visit(st, sm, stack, new_id); }
            stack.pop();
        }
        VariableDecl { name, line, column, value } => { record(sm, name, *line, *column, *stack.last().unwrap(), true); visit(value, sm, stack, current); }
        Assignment { name, line, column, value } => { record(sm, name, *line, *column, *stack.last().unwrap(), false); visit(value, sm, stack, current); }
        If { condition, then_branch, else_branch } => { visit(condition, sm, stack, current); visit(then_branch, sm, stack, current); if let Some(e)=else_branch { visit(e, sm, stack, current); } }
        While { condition, body } => { visit(condition, sm, stack, current); visit(body, sm, stack, current); }
        For { init, condition, increment, body } => { if let Some(i)=init { visit(i, sm, stack, current); } if let Some(c)=condition { visit(c, sm, stack, current); } if let Some(inc)=increment { visit(inc, sm, stack, current); } visit(body, sm, stack, current); }
        BinaryExpr { left, right, .. } => { visit(left, sm, stack, current); visit(right, sm, stack, current); }
        UnaryExpr { expr, .. } => visit(expr, sm, stack, current),
        Call { callee, args } => { visit(callee, sm, stack, current); for a in args { visit(a, sm, stack, current); } }
        Return(expr) | Log(expr) => visit(expr, sm, stack, current),
        QuantumOp { qubits, .. } => { for q in qubits { visit(q, sm, stack, current); } }
        HieroglyphicOp { args, .. } => { for a in args { visit(a, sm, stack, current); } }
    Identifier(name) => { record(sm, name, 0,0, *stack.last().unwrap(), false); }
    IdentifierSpanned { name, line, column, len:_ } => { record(sm, name, *line, *column, *stack.last().unwrap(), false); }
        NumberLiteral(_) | StringLiteral(_) | BooleanLiteral(_) | Error(_) => {}
    }
}

fn record(sm: &mut ScopeMap, name: &str, line: usize, column: usize, scope_id: usize, is_def: bool) {
    sm.symbols.entry(name.to_string()).or_default().push(ScopeEntry { scope_id, line, column, is_def });
}