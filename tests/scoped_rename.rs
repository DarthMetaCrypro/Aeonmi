use aeonmi_project::core::{lexer::Lexer, parser::Parser, incremental::parse_or_cached, scope_map::ScopeMap};

#[test]
fn scope_map_occurrences() {
    let src = "function foo(a){ let x = 1; { let x = 2; } x = 3; }";
    let ast = parse_or_cached(src).expect("parse");
    let sm = ScopeMap::build(&ast);
    // outer x should have at least 2 occurrences (decl + assignment)
    let mut decl_line_col = None;
    for (name, occs) in &sm.symbols { if name == "x" { decl_line_col = occs.iter().find(|o| o.column>0).map(|o|(o.line,o.column)); } }
    if let Some((dl,dc)) = decl_line_col {
        let occs = sm.occurrences_in_same_scope("x", dl, dc);
        assert!(occs.len()>=2);
    } else { panic!("decl not found"); }
}
