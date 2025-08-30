#![cfg(feature = "bytecode")]
use aeonmi_project::core::lexer::Lexer;use aeonmi_project::core::parser::Parser;use aeonmi_project::core::ast::ASTNode;use aeonmi_project::core::bytecode::BytecodeCompiler;use aeonmi_project::core::vm_bytecode::VM;

fn eval_bytecode(src:&str)->Option<aeonmi_project::core::vm_bytecode::Value>{let mut lex=Lexer::from_str(src);let toks=lex.tokenize().unwrap();let mut p=Parser::new(toks);let ast=p.parse().unwrap();let chunk=BytecodeCompiler::new().compile(&ast);let mut vm=VM::new(&chunk);vm.run()}

#[test]fn arithmetic(){let v=eval_bytecode("let a=1; let b=2; return a+b*3;");match v { Some(aeonmi_project::core::vm_bytecode::Value::Number(n))=> assert_eq!(n,7.0),_=> panic!("unexpected {v:?}") }}
