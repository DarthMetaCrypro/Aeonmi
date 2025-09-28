#![cfg(feature = "bytecode")]
use aeonmi_project::core::lexer::Lexer;use aeonmi_project::core::parser::Parser;use aeonmi_project::core::bytecode::BytecodeCompiler;use aeonmi_project::core::vm_bytecode::{VM,Value};
fn eval(src:&str)->Option<Value>{let mut l=Lexer::from_str(src);let toks=l.tokenize().unwrap();let mut p=Parser::new(toks);let ast=p.parse().unwrap();let chunk=BytecodeCompiler::new().compile(&ast);let mut vm=VM::new(&chunk);vm.run()}

#[test]
fn fold_arithmetic(){let v=eval("return 2+3*4;");match v {Some(Value::Number(n))=>assert_eq!(n,14.0),_=>panic!("bad {v:?}")}}

#[test]
fn fold_string_concat(){let v=eval("return \"a\" + \"b\";");match v {Some(Value::String(s))=>assert_eq!(s,"ab"),_=>panic!("bad {v:?}")}}

#[test]
fn implicit_null_return(){let v=eval("fn foo(a){ let x = a + 1; } return foo(1);");match v {Some(Value::Null)=>{},_=>panic!("expected null got {v:?}")}}
