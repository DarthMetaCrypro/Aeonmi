//! Extract a simple quantum circuit timeline from AST.
use crate::core::ast::ASTNode;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct QuantumGate { pub gate: String, pub qubits: Vec<String>, pub line: usize }

#[derive(Debug, Serialize, Clone)]
pub struct QuantumCircuit { pub gates: Vec<QuantumGate>, pub qubit_count: usize }

pub fn extract_circuit(ast: &ASTNode) -> QuantumCircuit {
    let mut gates = Vec::new();
    let mut qubit_names: Vec<String> = Vec::new();
    walk(ast, &mut gates, &mut qubit_names);
    qubit_names.sort(); qubit_names.dedup();
    QuantumCircuit { gates, qubit_count: qubit_names.len() }
}

pub fn circuit_to_json(c: &QuantumCircuit) -> String { serde_json::to_string_pretty(c).unwrap_or_else(|_|"{}".into()) }

// Very small pseudo-QASM emitter (not full OpenQASM)
// Format:
// qreg q[<count>];\n
// gate lines: <gate> q[<index>]; or <gate> q[i],q[j]; preserving order qubits appear first time sorted.
pub fn circuit_to_pseudo_qasm(c: &QuantumCircuit) -> String {
    // Map qubit string names to indices stable sorted
    let mut names: Vec<String> = c.gates.iter().flat_map(|g| g.qubits.clone()).collect();
    names.sort(); names.dedup();
    let mut map = std::collections::HashMap::new();
    for (i,n) in names.iter().enumerate() { map.insert(n.clone(), i); }
    let mut out = format!("qreg q[{}];\n", names.len());
    for g in &c.gates { if !g.qubits.is_empty() { let idxs: Vec<String> = g.qubits.iter().filter_map(|q| map.get(q)).map(|i| format!("q[{}]", i)).collect(); out.push_str(&format!("{} {};// line{}\n", g.gate.to_lowercase(), idxs.join(","), g.line)); } }
    out
}

fn walk(node: &ASTNode, gates: &mut Vec<QuantumGate>, qubits: &mut Vec<String>) {
    match node {
        ASTNode::Program(items) | ASTNode::Block(items) => { for it in items { walk(it, gates, qubits); } }
        ASTNode::QuantumOp { op, qubits: qs } => {
            let qn: Vec<String> = qs.iter().filter_map(|q| match q { ASTNode::Identifier(name) => Some(name.clone()), ASTNode::IdentifierSpanned { name, .. } => Some(name.clone()), _ => None }).collect();
            for q in &qn { if !qubits.contains(q) { qubits.push(q.clone()); } }
            gates.push(QuantumGate { gate: format!("{:?}", op), qubits: qn, line: 0 });
        }
        ASTNode::Function { body, .. } => { for it in body { walk(it, gates, qubits); } }
        ASTNode::If { then_branch, else_branch, .. } => { walk(then_branch, gates, qubits); if let Some(e)=else_branch { walk(e, gates, qubits); } }
        ASTNode::While { body, .. } => walk(body, gates, qubits),
        ASTNode::For { body, .. } => walk(body, gates, qubits),
        ASTNode::Log(expr) | ASTNode::Return(expr) => walk(expr, gates, qubits),
        ASTNode::Assignment { value, .. } | ASTNode::VariableDecl { value, .. } => walk(value, gates, qubits),
        ASTNode::BinaryExpr { left, right, .. } => { walk(left, gates, qubits); walk(right, gates, qubits); }
        ASTNode::UnaryExpr { expr, .. } => walk(expr, gates, qubits),
        ASTNode::Call { callee, args } => { walk(callee, gates, qubits); for a in args { walk(a, gates, qubits); } }
        _ => {}
    }
}
