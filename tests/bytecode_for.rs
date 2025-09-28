#![cfg(feature = "bytecode")]
use aeonmi_project::core::lexer::Lexer;use aeonmi_project::core::parser::Parser;use aeonmi_project::core::bytecode::BytecodeCompiler;use aeonmi_project::core::vm_bytecode::{VM,Value};
fn eval(src:&str)->Option<Value>{let mut l=Lexer::from_str(src);let toks=l.tokenize().unwrap();let mut p=Parser::new(toks);let ast=p.parse().unwrap();let chunk=BytecodeCompiler::new().compile(&ast);let mut vm=VM::new(&chunk);vm.run()}

#[test]fn for_sum(){let v=eval("let s=0; for(let i=0; i<5; i=i+1){ s = s + i; } return s;");match v {Some(Value::Number(n))=>assert_eq!(n,10.0),_=>panic!("bad {v:?}")}}
#[test]fn for_dce(){let v=eval("for(; false; ){ return 99; } return 7;");match v {Some(Value::Number(n))=>assert_eq!(n,7.0),_=>panic!("bad {v:?}")}}
