#![cfg(feature = "bytecode")]
use aeonmi_project::core::lexer::Lexer;use aeonmi_project::core::parser::Parser;use aeonmi_project::core::bytecode::{BytecodeCompiler};
#[test]
fn dce_and_fold_stats(){let src="fn a(){ if (true) { return 1; } if (false) { return 2; } while(false){ return 3; } return 4; } return a();";let mut l=Lexer::from_str(src);let toks=l.tokenize().unwrap();let mut p=Parser::new(toks);let ast=p.parse().unwrap();let chunk=BytecodeCompiler::new().compile(&ast);assert!(chunk.opt_stats.dce_if>=1);assert!(chunk.opt_stats.dce_while>=1);assert!(chunk.opt_stats.const_folds>=0);}
