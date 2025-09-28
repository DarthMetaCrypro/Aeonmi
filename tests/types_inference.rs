use aeonmi_project::core::{lexer::Lexer, parser::Parser, types::TypeContext};

#[test]
fn arithmetic_type_check() {
    let src = "let a = 1; let b = 2; let c = a + b;";
    let mut lexer = Lexer::from_str(src);
    let tokens = lexer.tokenize().expect("lex");
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().expect("parse");
    let mut ctx = TypeContext::new();
    ctx.infer_program(&ast);
    assert!(ctx.diags.is_empty(), "Unexpected diagnostics: {:?}", ctx.diags);
}
