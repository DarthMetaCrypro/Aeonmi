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
fn arithmetic_type_error() {
    let src = r#"\nlet a = 1;\nlet b = "s";\nlet c = a - b;\n"#; // minus between number and string -> error
    let diags = gather(src);
    assert!(diags.iter().any(|(m,s)| matches!(s, Severity::Error) && m.contains("Arithmetic operands must be numbers")), "expected arithmetic type error, got: {:?}", diags);
}

#[test]
fn plus_string_number_warning() {
    let src = r#"\nlet a = 1;\nlet b = "s";\nlet c = a + b;\n"#; // implicit coercion warning
    let diags = gather(src);
    assert!(diags.iter().any(|(m,s)| matches!(s, Severity::Warning) && m.contains("Implicit number/string coercion")), "expected coercion warning, got: {:?}", diags);
}

#[test]
fn unused_function_warning() {
    let src = r#"\nfn foo() { return 1; }\n"#; // no call -> warning
    let diags = gather(src);
    assert!(diags.iter().any(|(m,s)| matches!(s, Severity::Warning) && m.contains("Unused function 'foo'")), "expected unused function warning, got: {:?}", diags);
}

#[test]
fn used_function_no_warning() {
    let src = r#"\nfn foo() { return 1; }\nlet x = foo();\n"#; // called -> no unused warning
    let diags = gather(src);
    assert!(!diags.iter().any(|(m,_s)| m.contains("Unused function 'foo'")), "did not expect unused function warning, got: {:?}", diags);
}
