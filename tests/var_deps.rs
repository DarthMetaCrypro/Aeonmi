use aeonmi_project::core::incremental::compute_var_deps;
use aeonmi_project::core::lexer::Lexer;
use aeonmi_project::core::parser::Parser as AeParser;
use aeonmi_project::core::ast::ASTNode;

fn parse(src: &str) -> ASTNode {
    let mut lex = Lexer::from_str(src);
    let tokens = lex.tokenize().expect("lex");
    let mut p = AeParser::new(tokens);
    p.parse().expect("parse")
}

#[test]
fn variable_dependency_reads_writes() {
    let src = r#"
function a() { x = 1; y = x; }
function b() { y = 2; z = y; }
function c() { z = x; }
"#;
    let ast = parse(src);
    let deps = compute_var_deps(&ast);
    // writers
    let wx = deps.writes.get("x").unwrap();
    assert!(wx.contains(&0)); // function a writes x
    let wy = deps.writes.get("y").unwrap();
    assert!(wy.contains(&0) && wy.contains(&1));
    let wz = deps.writes.get("z").unwrap();
    assert!(wz.contains(&1) && wz.contains(&2));
    // readers
    let rx = deps.reads.get("x").unwrap();
    assert!(rx.contains(&0) && rx.contains(&2));
    let ry = deps.reads.get("y").unwrap();
    assert!(ry.contains(&0) && ry.contains(&1));
    let rz = deps.reads.get("z").unwrap();
    assert!(rz.contains(&1));
}
