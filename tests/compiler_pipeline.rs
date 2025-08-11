use aeonmi_project::core::compiler::Compiler;

#[test]
fn pipeline_end_to_end_basic() {
    let code = r#"
        let x = 2 + 3;
        log(x);
    "#;

    let out = std::env::temp_dir().join("aeonmi_pipeline_out.js");
    let _ = std::fs::remove_file(&out);

    let c = Compiler::new();
    c.compile(code, out.to_str().unwrap()).expect("compile should succeed");

    let js = std::fs::read_to_string(&out).expect("output exists");
    assert!(js.contains("let x = (2 + 3);") || js.contains("let x = 2 + 3;"));
    assert!(js.contains("console.log(x);"));
}

#[test]
fn pipeline_quantum_and_glyph_ops() {
    // Quantum op + Hieroglyphic op as statements; ensure they pass through codegen.
    let code = r#"
        superpose(q1);
        𓀀(q1, 42);
    "#;

    let out = std::env::temp_dir().join("aeonmi_pipeline_qglyph_out.js");
    let _ = std::fs::remove_file(&out);

    let c = Compiler::new();
    c.compile(code, out.to_str().unwrap()).expect("compile should succeed");

    let js = std::fs::read_to_string(&out).expect("output exists");
    assert!(js.contains("superpose(q1);"));
    assert!(js.contains("__glyph('𓀀', q1, 42);"));
}
