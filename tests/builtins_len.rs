use aeonmi_project::core::compiler::Compiler;

#[test]
fn len_builtin_emits_helper_and_invocation() {
    let code = r#"
        let name = "Aeonmi";
        let n = len(name);
        log(n);
    "#;

    let out = std::env::temp_dir().join("aeonmi_len_helper.js");
    if out.exists() {
        std::fs::remove_file(&out).expect("failed to remove stale output");
    }

    let compiler = Compiler::new();
    compiler
        .compile(code, out.to_str().expect("valid output path"))
        .expect("compilation should succeed");

    let js = std::fs::read_to_string(&out).expect("output JS exists");
    assert!(
        js.contains("const __aeonmi_len"),
        "helper prelude missing: {}",
        js
    );
    assert!(
        js.contains("let n = __aeonmi_len(name);"),
        "helper call not rewritten: {}",
        js
    );
    assert!(
        js.contains("console.log(n);"),
        "log should use rewritten binding: {}",
        js
    );
}
