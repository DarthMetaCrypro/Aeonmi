use aeonmi_project::core::compiler::Compiler;

#[test]
fn function_decl_and_body() {
    let code = r#"
        function add(a, b) { 
            let sum = a + b; 
            log(sum); 
            return sum; 
        }
        let r = 1 + 2;
        log(r);
    "#;

    let out = std::env::temp_dir().join("aeonmi_func_out.js");
    let _ = std::fs::remove_file(&out);

    let c = Compiler::new();
    c.compile(code, out.to_str().unwrap()).expect("compile should succeed");

    let js = std::fs::read_to_string(&out).expect("output exists");

    // Function signature + body structure
    assert!(js.contains("function add(a, b)"));
    assert!(js.contains("let sum = (a + b);"));
    assert!(js.contains("console.log(sum);"));
    assert!(js.contains("return sum;"));

    // Post-function code
    assert!(js.contains("let r = (1 + 2);"));
    assert!(js.contains("console.log(r);"));
}
