use aeonmi_project::core::compiler::Compiler;

#[test]
fn control_flow_if_while_for() {
    // Test control flow constructs: if, while, for.
    // Conditions here are numbers/booleans only (no '==' operator tested yet).

    let code = r#"
        let x = 0;
        if (true) { log(1); } else { log(0); }
        while (0) { log(999); }        // falsy; codegen should still emit while(0) { ... }
        for (let i = 0; 1; i + 1) {    // truthy condition; we just assert code structure in JS
            log(i);
        }
    "#;

    let out = std::env::temp_dir().join("aeonmi_cf_out.js");

    if out.exists() {
        std::fs::remove_file(&out).expect("Failed to remove existing output file");
    }

    // Compile Aeonmi source code to output JS file.
    let compiler = Compiler::new();
    compiler
        .compile(code, out.to_str().expect("Valid output path"))
        .expect("Compilation should succeed");

    // Read generated JS code.
    let js = std::fs::read_to_string(&out).expect("Output JS file should exist");

    // Assertions to verify code structure in output.
    assert!(
        js.contains("let x = 0;"),
        "JS output missing variable initialization"
    );
    assert!(js.contains("if (true)"), "JS output missing if condition");
    assert!(
        js.contains("console.log(1);"),
        "JS output missing then branch log"
    );
    assert!(
        js.contains("console.log(0);"),
        "JS output missing else branch log"
    );
    assert!(
        js.contains("while (0)"),
        "JS output missing while loop structure"
    );
    assert!(
        js.contains("for (let i = 0; 1; (i + 1))"),
        "JS output missing normalized for loop structure"
    );
}
