// src/core/ast.rs
//! Abstract Syntax Tree (AST) definitions for Aeonmi/QUBE/Titan
//! Now includes Assignment and Call nodes.

use crate::core::token::TokenKind;

/// Represents nodes in the Abstract Syntax Tree
#[derive(Debug, Clone, PartialEq)]
pub enum ASTNode {
    // Program root
    Program(Vec<ASTNode>),

    // Declarations
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<ASTNode>,
    },
    VariableDecl {
        name: String,
        value: Box<ASTNode>,
    },

    // Statements / simple stmt-like exprs
    Block(Vec<ASTNode>),
    Return(Box<ASTNode>),
    Log(Box<ASTNode>),

    // Control flow
    If {
        condition: Box<ASTNode>,
        then_branch: Box<ASTNode>,
        else_branch: Option<Box<ASTNode>>,
    },
    While {
        condition: Box<ASTNode>,
        body: Box<ASTNode>,
    },
    For {
        init: Option<Box<ASTNode>>,
        condition: Option<Box<ASTNode>>,
        increment: Option<Box<ASTNode>>,
        body: Box<ASTNode>,
    },

    // Expressions
    Assignment {
        name: String,
        value: Box<ASTNode>,
    },
    Call {
        callee: Box<ASTNode>,
        args: Vec<Box<ASTNode>>,
    },
    BinaryExpr {
        op: TokenKind,
        left: Box<ASTNode>,
        right: Box<ASTNode>,
    },
    UnaryExpr {
        op: TokenKind,
        expr: Box<ASTNode>,
    },
    Identifier(String),
    NumberLiteral(f64),
    StringLiteral(String),
    BooleanLiteral(bool),

    // Quantum & Hieroglyphic
    QuantumOp {
        op: TokenKind,
        qubits: Vec<Box<ASTNode>>,
    },
    HieroglyphicOp {
        symbol: String,
        args: Vec<Box<ASTNode>>,
    },

    // Special
    Error(String),
}

impl ASTNode {
    // Utility constructors
    pub fn new_function(name: &str, params: Vec<&str>, body: Vec<ASTNode>) -> Self {
        ASTNode::Function {
            name: name.to_string(),
            params: params.into_iter().map(String::from).collect(),
            body,
        }
    }
    pub fn new_variable_decl(name: &str, value: ASTNode) -> Self {
        ASTNode::VariableDecl {
            name: name.to_string(),
            value: Box::new(value),
        }
    }
    pub fn new_assignment(name: &str, value: ASTNode) -> Self {
        ASTNode::Assignment {
            name: name.to_string(),
            value: Box::new(value),
        }
    }
    pub fn new_call(callee: ASTNode, args: Vec<ASTNode>) -> Self {
        ASTNode::Call {
            callee: Box::new(callee),
            args: args.into_iter().map(Box::new).collect(),
        }
    }
    pub fn new_binary_expr(op: TokenKind, left: ASTNode, right: ASTNode) -> Self {
        ASTNode::BinaryExpr {
            op,
            left: Box::new(left),
            right: Box::new(right),
        }
    }
    pub fn new_unary_expr(op: TokenKind, expr: ASTNode) -> Self {
        ASTNode::UnaryExpr {
            op,
            expr: Box::new(expr),
        }
    }
    pub fn new_if(cond: ASTNode, then_branch: ASTNode, else_branch: Option<ASTNode>) -> Self {
        ASTNode::If {
            condition: Box::new(cond),
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(Box::new),
        }
    }
    pub fn new_while(cond: ASTNode, body: ASTNode) -> Self {
        ASTNode::While {
            condition: Box::new(cond),
            body: Box::new(body),
        }
    }
    pub fn new_for(
        init: Option<ASTNode>,
        condition: Option<ASTNode>,
        increment: Option<ASTNode>,
        body: ASTNode,
    ) -> Self {
        ASTNode::For {
            init: init.map(Box::new),
            condition: condition.map(Box::new),
            increment: increment.map(Box::new),
            body: Box::new(body),
        }
    }
    pub fn new_log(expr: ASTNode) -> Self {
        ASTNode::Log(Box::new(expr))
    }
    pub fn new_return(expr: ASTNode) -> Self {
        ASTNode::Return(Box::new(expr))
    }
    pub fn new_quantum_op(op: TokenKind, qubits: Vec<ASTNode>) -> Self {
        ASTNode::QuantumOp {
            op,
            qubits: qubits.into_iter().map(Box::new).collect(),
        }
    }
    pub fn new_hieroglyphic_op(symbol: &str, args: Vec<ASTNode>) -> Self {
        ASTNode::HieroglyphicOp {
            symbol: symbol.to_string(),
            args: args.into_iter().map(Box::new).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::token::TokenKind;

    #[test]
    fn test_if_else_node() {
        let cond = ASTNode::BooleanLiteral(true);
        let then_b = ASTNode::NumberLiteral(1.0);
        let else_b = ASTNode::NumberLiteral(0.0);
        let node = ASTNode::new_if(cond.clone(), then_b.clone(), Some(else_b.clone()));
        if let ASTNode::If { condition, then_branch, else_branch } = node {
            assert_eq!(*condition, cond);
            assert_eq!(*then_branch, then_b);
            assert_eq!(*else_branch.unwrap(), else_b);
        } else {
            panic!("Expected If node");
        }
    }

    #[test]
    fn test_quantum_op_node() {
        let qop = ASTNode::new_quantum_op(TokenKind::Superpose, vec![ASTNode::Identifier("q1".into())]);
        if let ASTNode::QuantumOp { op, qubits } = qop {
            assert_eq!(op, TokenKind::Superpose);
            assert_eq!(*qubits[0], ASTNode::Identifier("q1".into()));
        } else {
            panic!("Expected QuantumOp node");
        }
    }

    #[test]
    fn test_assignment_and_call_nodes() {
        let call = ASTNode::new_call(ASTNode::Identifier("f".into()), vec![ASTNode::NumberLiteral(1.0)]);
        let asn = ASTNode::new_assignment("x", call);
        if let ASTNode::Assignment { name, value } = asn {
            assert_eq!(name, "x");
            if let ASTNode::Call { callee, args } = *value {
                assert_eq!(*callee, ASTNode::Identifier("f".into()));
                assert_eq!(*args[0], ASTNode::NumberLiteral(1.0));
            } else {
                panic!("Expected Call in assignment value");
            }
        } else {
            panic!("Expected Assignment node");
        }
    }
}
