#![cfg(feature = "bytecode")]
use aeonmi_project::core::lexer::Lexer;use aeonmi_project::core::parser::Parser;use aeonmi_project::core::bytecode::BytecodeCompiler;use aeonmi_project::core::vm_bytecode::VM;

#[test]
fn respects_max_frames_env() {
    std::env::set_var("AEONMI_MAX_FRAMES","8");
    let src = "fn dive(n){ if (n==0){ return 0; } return dive(n-1); } return dive(50);"; // depth > 8
    let mut l=Lexer::from_str(src);let toks=l.tokenize().unwrap();let mut p=Parser::new(toks);let ast=p.parse().unwrap();let chunk=BytecodeCompiler::new().compile(&ast);let mut vm=VM::new(&chunk);let _=vm.run();assert!(vm.stack_overflow, "expected overflow with small frame limit");
    std::env::remove_var("AEONMI_MAX_FRAMES");
}
