// Simple snapshot-like test ensuring lexer identifies basic tokens for Monaco mapping.
use aeonmi_project::core::lexer::Lexer;

#[test]
fn tokenize_basic_sample() {
    let src = "let x = 42; if x > 3 { log(x); }";
    let mut lex = Lexer::from_str(src);
    let tokens = lex.tokenize().expect("tokenize");
    // Just assert presence/order of a few key tokens
    let kinds: Vec<_> = tokens.iter().map(|t| format!("{:?}", t.kind)).collect();
    assert!(kinds.starts_with(&["Let".into(), "Identifier".into()]));
    assert!(kinds.iter().any(|k| k == "Number"));
    assert!(kinds.iter().any(|k| k == "If"));
    assert!(kinds.iter().any(|k| k == "GreaterThan"));
}