#![cfg(feature = "bytecode")]
use std::process::Command;

#[test]
fn cli_emits_opt_stats_json() {
    // Write a small temp file with obvious folding opportunities
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("fold.ai");
    std::fs::write(&file_path, "fn f(){ return 1+2+3; } return f();").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_aeonmi_project"))
        .arg("run")
        .arg(&file_path)
        .arg("--opt-stats-json")
        .output()
        .expect("failed to run cli");
    assert!(output.status.success(), "cli failed: stderr={}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("const_folds"), "missing const_folds in json: {stdout}");
    assert!(stdout.contains("chain_folds"));
}
