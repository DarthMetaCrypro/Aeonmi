use aeonmi_project::core::incremental::{parse_or_cached, parse_or_partial};

#[test]
fn single_node_edit_triggers_partial() {
    let src1 = "function a() { let x = 1; }\nfunction b() { let y = 2; }";
    let _ = parse_or_cached(src1).unwrap();
    let src2 = "function a() { let x = 1; }\nfunction b() { let y = 3; }"; // edit inside b
    let (_ast, partial) = parse_or_partial(src2).unwrap();
    assert!(partial, "expected partial reparse");
}
