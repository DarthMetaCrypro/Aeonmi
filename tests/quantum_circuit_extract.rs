use aeonmi_project::core::{lexer::Lexer, parser::Parser, quantum_extract::extract_circuit};

#[test]
fn extract_simple_circuit() {
    let src = "superpose(q1); entangle(q1, q2); measure(q2);";
    let mut lexer = Lexer::from_str(src);
    let tokens = lexer.tokenize().expect("lex");
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().expect("parse");
    let circ = extract_circuit(&ast);
    assert_eq!(circ.gates.len(), 3);
    assert!(circ.qubit_count >= 2);
}
