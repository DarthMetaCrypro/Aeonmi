#![cfg(feature = "bytecode")]
use aeonmi_project::core::lexer::Lexer;use aeonmi_project::core::parser::Parser;use aeonmi_project::core::bytecode::BytecodeCompiler;use aeonmi_project::core::vm_bytecode::VM;

// Create intentionally deep recursion exceeding frame limit (default 256)
#[test]
fn recursion_overflow_guard() {
    let depth = 400; // > 256
    let mut src = String::from("fn dive(n){ if (n==0){ return 0; } return dive(n-1); } return dive(");
    src.push_str(&depth.to_string()); src.push_str(");");
    let mut lex=Lexer::from_str(&src); let toks=lex.tokenize().unwrap(); let mut p=Parser::new(toks); let ast=p.parse().unwrap(); let chunk=BytecodeCompiler::new().compile(&ast); let mut vm=VM::new(&chunk); let _=vm.run(); assert!(vm.stack_overflow, "expected stack overflow flag"); }
