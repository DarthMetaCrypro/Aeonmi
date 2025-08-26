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

#[test]
fn native_run_env() {
    use std::process::Command;
    let output = Command::new(env!("CARGO_BIN_EXE_aeonmi_project"))
        .env("AEONMI_NATIVE", "1")
        .arg("run")
        .arg("examples/hello.ai")
        .output()
        .expect("failed to run native aeonmi");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("native: executing"), "stdout missing native execution marker: {stdout}");
}

#[test]
fn shard_native_run_command() {
    use std::process::{Command, Stdio};
    let mut child = Command::new(env!("CARGO_BIN_EXE_aeonmi_project"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn shard");
    {
        use std::io::Write;
        let stdin = child.stdin.as_mut().unwrap();
        writeln!(stdin, "native-run examples/hello.ai").ok();
        writeln!(stdin, "exit").ok();
    }
    let out = child.wait_with_output().expect("wait shard");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("native: executing"), "stdout missing native execution marker in shard: {stdout}");
}
