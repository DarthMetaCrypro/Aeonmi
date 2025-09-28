use aeonmi_project::core::lexer::Lexer;use aeonmi_project::core::parser::Parser as AeParser;use aeonmi_project::core::code_actions::suggest_actions;

#[test]
fn add_missing_let_action_present() {
    let src = "x = 1\n"; // assignment without decl
    let mut lex = Lexer::from_str(src); let toks = lex.tokenize().unwrap(); let mut p = AeParser::new(toks); let ast = p.parse().unwrap();
    let acts = suggest_actions(&ast); assert!(acts.iter().any(|a| a.kind=="addMissingLet"));
}

#[test]
fn inline_variable_action_present() {
    let src = "let x = 1; log(x);"; // single use
    let mut lex = Lexer::from_str(src); let toks = lex.tokenize().unwrap(); let mut p = AeParser::new(toks); let ast = p.parse().unwrap();
    let acts = suggest_actions(&ast); assert!(acts.iter().any(|a| a.kind=="inlineVariable"));
}
