#![cfg(feature = "bytecode")]
use aeonmi_project::core::lexer::Lexer;use aeonmi_project::core::parser::Parser;use aeonmi_project::core::bytecode::BytecodeCompiler;use aeonmi_project::core::vm_bytecode::{VM,Value};
fn eval(src:&str)->Option<Value>{let mut lex=Lexer::from_str(src);let toks=lex.tokenize().unwrap();let mut p=Parser::new(toks);let ast=p.parse().unwrap();let chunk=BytecodeCompiler::new().compile(&ast);let mut vm=VM::new(&chunk);vm.run()}
#[test]fn if_else_true(){let v=eval("let a=1; if a { return 10; } else { return 20; }");match v {Some(Value::Number(n))=>assert_eq!(n,10.0),_=>panic!("bad {v:?}")}}
#[test]fn if_else_false(){let v=eval("let a=0; if a { return 10; } else { return 20; }");match v {Some(Value::Number(n))=>assert_eq!(n,20.0),_=>panic!("bad {v:?}")}}
