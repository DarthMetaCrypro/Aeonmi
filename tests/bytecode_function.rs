#![cfg(feature = "bytecode")]
use aeonmi_project::core::lexer::Lexer;use aeonmi_project::core::parser::Parser;use aeonmi_project::core::bytecode::BytecodeCompiler;use aeonmi_project::core::vm_bytecode::{VM,Value};
fn eval(src:&str)->Option<Value>{let mut l=Lexer::from_str(src);let toks=l.tokenize().unwrap();let mut p=Parser::new(toks);let ast=p.parse().unwrap();let chunk=BytecodeCompiler::new().compile(&ast);let mut vm=VM::new(&chunk);vm.run()}
#[test]fn function_call(){let v=eval("fn add1(a){ return a+1; } return add1(41);");match v {Some(Value::Number(n))=>assert_eq!(n,42.0),_=>panic!("bad {v:?}")}}
