// tests/assign_and_calls.rs
use aeonmi_project::core::compiler::Compiler;
use std::fs;

#[test]
fn assignment_and_function_call_pipeline() {
    let code = r#"
        function add(a, b) { return a + b; }
        let x = 1;
        x = add(x, 2);
        log(x);
    "#;

    let out = std::env::temp_dir().join("aeonmi_assign_call_out.js");
    let _ = fs::remove_file(&out);

    let c = Compiler::new();
    c.compile(code, out.to_str().unwrap()).expect("compile should succeed");

    let js = fs::read_to_string(&out).expect("output exists");
    assert!(js.contains("function add(a, b)"));
    assert!(js.contains("let x = 1;"));
    assert!(js.contains("x = add(x, 2);"));
    assert!(js.contains("console.log(x);"));
}
