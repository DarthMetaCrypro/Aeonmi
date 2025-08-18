use aeonmi_project::core::compiler::Compiler;

#[test]
fn control_flow_if_while_for() {
    // Note: conditions use numbers/booleans only (no == yet).
    let code = r#"
        let x = 0;
        if (true) { log(1); } else { log(0); }

        while (0) { log(999); }        // falsy; codegen should still emit while(0) { ... }

        for (let i = 0; 1; i + 1) {    // truthy condition; we just assert code structure in JS
            log(i);
        }
    "#;

    let out = std::env::temp_dir().join("aeonmi_cf_out.js");
    let _ = std::fs::remove_file(&out);

    let c = Compiler::new();
    c.compile(code, out.to_str().unwrap())
        .expect("compile should succeed");

    let js = std::fs::read_to_string(&out).expect("output exists");

    // Basic structure checks
    assert!(js.contains("let x = 0;"));
    assert!(js.contains("if (true)")); // then/else blocks emitted
    assert!(js.contains("console.log(1);"));
    assert!(js.contains("console.log(0);"));
    assert!(js.contains("while (0)")); // structure only
    assert!(js.contains("for (let i = 0; 1; (i + 1))")); // normalized by codegen
}
