use aeonmi_project::core::lexer::Lexer;
use aeonmi_project::core::parser::Parser;
use aeonmi_project::core::code_generator::CodeGenerator;

#[test]
fn arithmetic_and_comparisons_precedence() {
    // 1 + (2*3) == 7  => true; check emitted parens order
    let code = "let x = 1 + 2 * 3 == 7;";
    let mut lx = Lexer::new(code);
    let toks = lx.tokenize().unwrap();
    let mut p = Parser::new(toks);
    let ast = p.parse().unwrap();

    let mut cg = CodeGenerator::new();
    let js = cg.generate(&ast).unwrap();

    assert!(
        js.contains("let x = ((1 + (2 * 3)) == 7);")
            || js.contains("let x = (1 + (2 * 3)) == 7;")
            || js.contains("let x = ((1 + (2 * 3)) == 7);")
    );
}

#[test]
fn unary_then_binary() {
    let code = "let y = -1 + 2;";
    let mut lx = Lexer::new(code);
    let toks = lx.tokenize().unwrap();
    let mut p = Parser::new(toks);
    let ast = p.parse().unwrap();

    let mut cg = CodeGenerator::new();
    let js = cg.generate(&ast).unwrap();

    assert!(js.contains("let y = ((-1) + 2);") || js.contains("let y = (-1 + 2);"));
}
