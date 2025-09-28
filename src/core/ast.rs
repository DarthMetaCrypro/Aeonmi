//! Abstract Syntax Tree (AST) definitions for Aeonmi/QUBE/Titan.
//! Includes Assignment and Call nodes to support expression statements.

use crate::core::token::TokenKind;

/// Represents nodes in the Abstract Syntax Tree.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)] // Many variants used only in experimental passes / future features
pub enum ASTNode {
    // Program root
    Program(Vec<ASTNode>),
    // Declarations
    Function {
        name: String,
        line: usize,
        column: usize,
        params: Vec<FunctionParam>,
        body: Vec<ASTNode>,
    },
    VariableDecl {
        name: String,
        value: Box<ASTNode>,
        line: usize,
        column: usize,
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
        line: usize,
        column: usize,
    },
    Call {
        callee: Box<ASTNode>,
        args: Vec<ASTNode>,
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
    #[allow(dead_code)]
    Identifier(String),
    IdentifierSpanned { name: String, line: usize, column: usize, len: usize },
    NumberLiteral(f64),
    StringLiteral(String),
    BooleanLiteral(bool),
    // Quantum & Hieroglyphic
    QuantumOp {
        op: TokenKind,
        qubits: Vec<ASTNode>,
    },
    HieroglyphicOp {
        symbol: String,
        args: Vec<ASTNode>,
    },
    // Special
    #[allow(dead_code)]
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionParam {
    pub name: String,
    pub line: usize,
    pub column: usize,
}

impl ASTNode {
    // Utility constructors
    #[allow(dead_code)]
    pub fn new_function(name: &str, params: Vec<&str>, body: Vec<ASTNode>) -> Self {
        Self::Function {
            name: name.to_string(),
            line: 0,
            column: 0,
            params: params.into_iter().map(|p| FunctionParam { name: p.to_string(), line: 0, column: 0 }).collect(),
            body,
        }
    }
    pub fn new_function_at(name: &str, line: usize, column: usize, params: Vec<FunctionParam>, body: Vec<ASTNode>) -> Self {
        Self::Function { name: name.to_string(), line, column, params, body }
    }
    #[allow(dead_code)]
    pub fn new_variable_decl(name: &str, value: ASTNode) -> Self {
        Self::VariableDecl { name: name.to_string(), value: Box::new(value), line: 0, column: 0 }
    }
    pub fn new_variable_decl_at(name: &str, value: ASTNode, line: usize, column: usize) -> Self {
        Self::VariableDecl { name: name.to_string(), value: Box::new(value), line, column }
    }
    #[allow(dead_code)]
    pub fn new_assignment(name: &str, value: ASTNode) -> Self {
        Self::Assignment { name: name.to_string(), value: Box::new(value), line: 0, column: 0 }
    }
    pub fn new_assignment_at(name: &str, value: ASTNode, line: usize, column: usize) -> Self {
        Self::Assignment { name: name.to_string(), value: Box::new(value), line, column }
    }
    pub fn new_call(callee: ASTNode, args: Vec<ASTNode>) -> Self {
        Self::Call {
            callee: Box::new(callee),
            args,
        }
    }
    pub fn new_binary_expr(op: TokenKind, left: ASTNode, right: ASTNode) -> Self {
        Self::BinaryExpr {
            op,
            left: Box::new(left),
            right: Box::new(right),
        }
    }
    pub fn new_unary_expr(op: TokenKind, expr: ASTNode) -> Self {
        Self::UnaryExpr {
            op,
            expr: Box::new(expr),
        }
    }
    pub fn new_identifier_spanned(name: &str, line: usize, column: usize, len: usize) -> Self { Self::IdentifierSpanned { name: name.into(), line, column, len } }
    pub fn new_if(cond: ASTNode, then_branch: ASTNode, else_branch: Option<ASTNode>) -> Self {
        Self::If {
            condition: Box::new(cond),
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(Box::new),
        }
    }
    pub fn new_while(cond: ASTNode, body: ASTNode) -> Self {
        Self::While {
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
        Self::For {
            init: init.map(Box::new),
            condition: condition.map(Box::new),
            increment: increment.map(Box::new),
            body: Box::new(body),
        }
    }
    pub fn new_log(expr: ASTNode) -> Self {
        Self::Log(Box::new(expr))
    }
    pub fn new_return(expr: ASTNode) -> Self {
        Self::Return(Box::new(expr))
    }
    pub fn new_quantum_op(op: TokenKind, qubits: Vec<ASTNode>) -> Self {
        Self::QuantumOp { op, qubits }
    }
    pub fn new_hieroglyphic_op(symbol: &str, args: Vec<ASTNode>) -> Self {
        Self::HieroglyphicOp {
            symbol: symbol.to_string(),
            args,
        }
    }
}

// Unit tests for ASTNode types – works directly with your TokenKind
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
        if let ASTNode::If {
            condition,
            then_branch,
            else_branch,
        } = node
        {
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
            assert_eq!(qubits[0], ASTNode::Identifier("q1".into()));
        } else {
            panic!("Expected QuantumOp node");
        }
    }

    #[test]
    fn test_assignment_and_call_nodes() {
        let call = ASTNode::new_call(
            ASTNode::Identifier("f".into()),
            vec![ASTNode::NumberLiteral(1.0)],
        );
        let asn = ASTNode::new_assignment("x", call);
    let ASTNode::Assignment { name, value, .. } = asn else {
            panic!("Expected Assignment")
        };
        assert_eq!(name, "x");
    let ASTNode::Call { callee, args } = *value else {
            panic!("Expected Call")
        };
        assert_eq!(*callee, ASTNode::Identifier("f".into()));
        assert_eq!(args[0], ASTNode::NumberLiteral(1.0));
    }
}
