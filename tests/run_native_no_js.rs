use std::process::Command;
use std::fs;

fn bin() -> String {
    env!("CARGO_BIN_EXE_aeonmi_project").to_string()
}

#[test]
fn native_run_does_not_emit_js() {
    let dir = tempfile::tempdir().unwrap();
    let program = "let a = 1; let b = a + 2; log(b);"; // expect 3
    let input = dir.path().join("prog.ai");
    fs::write(&input, program).unwrap();

    let output = Command::new(bin())
        .arg("run")
        .arg("--native")
        .arg(input.to_str().unwrap())
        .current_dir(dir.path())
        .output()
        .expect("failed to execute aeonmi_project run --native");

    assert!(output.status.success(), "native run failed: stdout=\n{}\nstderr=\n{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("3"), "expected computed value in stdout: {stdout}");

    // Ensure no JS artifact was produced in working directory
    let js_path = dir.path().join("aeonmi.run.js");
    assert!(!js_path.exists(), "unexpected JS output created in native run: {:?}", js_path);
}
