#![cfg(feature = "bytecode")]
use aeonmi_project::core::lexer::Lexer;use aeonmi_project::core::parser::Parser;use aeonmi_project::core::bytecode::BytecodeCompiler;use aeonmi_project::core::vm_bytecode::{VM,Value};
fn eval(src:&str)->Option<Value>{let mut l=Lexer::from_str(src);let toks=l.tokenize().unwrap();let mut p=Parser::new(toks);let ast=p.parse().unwrap();let chunk=BytecodeCompiler::new().compile(&ast);let mut vm=VM::new(&chunk);vm.run()}

#[test]fn fold_add_chain(){let v=eval("return 1+2+3+4+5;");match v {Some(Value::Number(n))=>assert_eq!(n,15.0),_=>panic!("bad {v:?}")}}
#[test]fn fold_mul_chain(){let v=eval("return 2*3*4; ");match v {Some(Value::Number(n))=>assert_eq!(n,24.0),_=>panic!("bad {v:?}")}}
#[test]fn fold_bool_and_chain(){let v=eval("return true && true && false && true; ");match v {Some(Value::Bool(b))=>assert!(!b),_=>panic!("bad {v:?}")}}
#[test]fn fold_bool_or_chain(){let v=eval("return false || false || true || false; ");match v {Some(Value::Bool(b))=>assert!(b),_=>panic!("bad {v:?}")}}
