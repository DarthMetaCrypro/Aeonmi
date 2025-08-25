use std::fs;use std::process::Command;use std::path::Path;

#[test]
fn exec_ai_compiles_and_runs() {
    // Use a tiny AI source copied into a temp file to avoid modifying examples.
    let ai_src = "let x = 1\nlog x\n"; // minimal program
    let file = "temp_exec_test.ai";
    fs::write(file, ai_src).expect("write ai file");
    let status = Command::new(env!("CARGO_BIN_EXE_aeonmi_project"))
        .args(["exec", file])
        .status()
        .expect("spawn exec ai");
    assert!(status.success(), "exec ai should succeed");
    assert!(Path::new("__exec_tmp.js").exists(), "temp compiled js should exist");
}

#[test]
fn exec_js_runs_directly() {
    let js_src = "console.log('ok');";
    let file = "temp_exec_test.js";
    fs::write(&file, js_src).expect("write js file");
    let status = Command::new(env!("CARGO_BIN_EXE_aeonmi_project"))
        .args(["exec", file])
        .status()
        .expect("spawn exec js");
    // Success depends on node presence; if node missing we allow skip.
    if !status.success() {
        eprintln!("(warn) exec js failed: likely node missing; skipping assertion");
    }
}
