#![cfg(feature = "bytecode")]
use aeonmi_project::core::lexer::Lexer;use aeonmi_project::core::parser::Parser;use aeonmi_project::core::bytecode::BytecodeCompiler;use aeonmi_project::core::vm_bytecode::{VM,Value};
fn eval(src:&str)->Option<Value>{let mut l=Lexer::from_str(src);let toks=l.tokenize().unwrap();let mut p=Parser::new(toks);let ast=p.parse().unwrap();let chunk=BytecodeCompiler::new().compile(&ast);let mut vm=VM::new(&chunk);vm.run()}

#[test]
fn logical_and_short_circuit(){let v=eval("let a=true; let b=false; return a && b; ");match v {Some(Value::Bool(b))=>assert!(!b),_=>panic!("bad {v:?}")}}
#[test]
fn logical_or_short_circuit(){let v=eval("let a=true; let b=false; return a || b; ");match v {Some(Value::Bool(b))=>assert!(b),_=>panic!("bad {v:?}")}}
