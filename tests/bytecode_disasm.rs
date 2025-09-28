#![cfg(feature = "bytecode")]
use aeonmi_project::core::lexer::Lexer;use aeonmi_project::core::parser::Parser;use aeonmi_project::core::bytecode::{BytecodeCompiler,disassemble};

fn compile_disasm(src:&str)->String{let mut l=Lexer::from_str(src);let toks=l.tokenize().unwrap();let mut p=Parser::new(toks);let ast=p.parse().unwrap();let chunk=BytecodeCompiler::new().compile(&ast);disassemble(&chunk)}

#[test]
fn disassembler_basic_layout(){let out=compile_disasm("fn add(a,b){ return a+b; } return add(1,2);");assert!(out.contains("== constants"),"missing constants header");assert!(out.contains("fn#0 add"),"missing function signature");assert!(out.lines().any(|l| l.contains("LOAD_CONST")),"expected at least one LOAD_CONST");assert!(out.lines().any(|l| l.contains("CALL")),"expected CALL opcode");assert!(out.trim().ends_with("RETURN"),"expected final RETURN");}
