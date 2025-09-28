use aeonmi_project::core::lexer::Lexer;use aeonmi_project::core::parser::Parser as AeParser;use aeonmi_project::core::quantum_extract::{extract_circuit, circuit_to_pseudo_qasm};

#[test]
fn pseudo_qasm_contains_qreg() {
    let src = "superpose q0; entangle q0 q1;"; // simple quantum ops if grammar supports
    let mut lex = Lexer::from_str(src); let toks = lex.tokenize().unwrap(); let mut p = AeParser::new(toks); let ast = p.parse().unwrap();
    let circ = extract_circuit(&ast); let qasm = circuit_to_pseudo_qasm(&circ); assert!(qasm.contains("qreg q["));
}
