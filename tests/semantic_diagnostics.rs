use aeonmi_project::core::lexer::Lexer;
use aeonmi_project::core::parser::Parser as AeParser;
use aeonmi_project::core::semantic_analyzer::{SemanticAnalyzer, Severity};

fn gather(source: &str) -> Vec<(String, Severity)> {
    let mut lexer = Lexer::from_str(source);
    let tokens = lexer.tokenize().expect("lex");
    let mut parser = AeParser::new(tokens);
    let ast = parser.parse().expect("parse");
    let mut sema = SemanticAnalyzer::new();
    sema.analyze_with_spans(&ast)
        .into_iter()
        .map(|d| (d.message, d.severity))
        .collect()
}

#[test]
fn undeclared_and_unused() {
    let src = r#"
let x = 1;
let y = 2;
x = 3;
z = 4;
"#;
    let diags = gather(src);
    assert!(diags.iter().any(|(m,s)| matches!(s, Severity::Error) && m.contains("undeclared variable 'z'")), "expected undeclared variable z error: {diags:?}");
    assert!(diags.iter().any(|(m,s)| matches!(s, Severity::Warning) && m.contains("Unused variable 'y'")), "expected unused variable y warning: {diags:?}");
}

#[test]
fn redeclaration_error() {
    let src = r#"
let a = 1;
let a = 2;
"#;
    let diags = gather(src);
    assert!(diags.iter().any(|(m,s)| matches!(s, Severity::Error) && m.contains("Redeclaration of 'a'")), "expected redeclaration error: {diags:?}");
}

#[test]
fn duplicate_function_error() {
    let src = r#"
fn foo() { return 1; }
fn foo() { return 2; }
"#;
    let diags = gather(src);
    assert!(diags.iter().any(|(m,s)| matches!(s, Severity::Error) && m.contains("Duplicate function 'foo'")), "expected duplicate function error: {diags:?}");
}

#[test]
fn unreachable_code_warning() {
    let src = r#"
fn bar() { return 1; let z = 3; }
"#;
    let diags = gather(src);
    assert!(diags.iter().any(|(m,s)| matches!(s, Severity::Warning) && m.contains("Unreachable code after return")), "expected unreachable code warning: {diags:?}");
}
