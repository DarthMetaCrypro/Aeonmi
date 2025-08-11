use aeonmi_project::core::compiler::Compiler;

#[test]
fn equality_and_comparison_in_if_and_while() {
    let code = r#"
        let a = 3 + 4 * 2;       // precedence: 3 + (4 * 2) = 11
        let b = 11;
        if (a == b) { log(1); } else { log(0); }

        // comparison chain
        if (a >= 10) { log(2); }

        // while with comparison (just structure check)
        while (a > 0) { a + 1; } // (no assignment support yet)
    "#;

    let out = std::env::temp_dir().join("aeonmi_cmp_out.js");
    let _ = std::fs::remove_file(&out);

    let c = Compiler::new();
    c.compile(code, out.to_str().unwrap()).expect("compile should succeed");

    let js = std::fs::read_to_string(&out).expect("output exists");

    // precedence on arithmetic
    assert!(js.contains("let a = (3 + (4 * 2));"));
    // equality
    assert!(js.contains("if ((a == b)) "));
    // >=
    assert!(js.contains("if ((a >= 10)) "));
    // while >
    assert!(js.contains("while ((a > 0)) "));
}
