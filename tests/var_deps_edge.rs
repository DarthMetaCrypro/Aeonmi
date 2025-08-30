use aeonmi_project::core::incremental::compute_var_deps;
use aeonmi_project::core::lexer::Lexer;
use aeonmi_project::core::parser::Parser;

fn parse(code: &str) -> aeonmi_project::core::ast::ASTNode {
    let mut lex = Lexer::from_str(code);
    let toks = lex.tokenize().expect("lex");
    let mut p = Parser::new(toks);
    p.parse().expect("parse")
}

#[test]
fn var_deps_loops() {
    let code = r#"
function f() {
  let x = 0;
  while (x < 3) { x = x + 1; }
  for (let i = 0; i < 2; i = i + 1) { x = x + i; }
  log(x);
}
"#;
    let ast = parse(code);
    let deps = compute_var_deps(&ast);
    // Expect function index 0 writes x and i; reads x/i inside conditions
    assert!(deps.writes.get("x").map(|s| s.contains(&0)).unwrap_or(false));
    assert!(deps.reads.get("x").map(|s| s.contains(&0)).unwrap_or(false));
}

#[test]
fn var_deps_fanout_shared_variable() {
    let code = r#"
function a() { shared = 1; }
function b() { shared = 2; }
function c() { shared = 3; }
function d() { log(shared); }
"#;
    let ast = parse(code);
    let deps = compute_var_deps(&ast);
    let writes = deps.writes.get("shared").cloned().unwrap_or_default();
    let reads = deps.reads.get("shared").cloned().unwrap_or_default();
    // a,b,c indices 0,1,2 write; d index 3 reads
    assert!(writes.contains(&0) && writes.contains(&1) && writes.contains(&2));
    assert!(reads.contains(&3));
    // ensure no accidental self-read for writers (they only write in simple body)
    assert!(!reads.contains(&0) && !reads.contains(&1) && !reads.contains(&2));
}