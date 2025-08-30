//! Ensures top-level let bindings are lowered into synthesized main and execute in native VM.
use std::process::Command;
use std::path::PathBuf;

#[test]
fn top_level_lets_execute() {
    // Prepare a tiny program
    let program = "let a = 10; let b = a + 5; log(a); log(b);";
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("mini.ai");
    std::fs::write(&file_path, program).unwrap();

    // Resolve binary path: tests set CARGO_BIN_EXE_<name> per cargo naming; fallback to target/debug/Aeonmi.exe
    let exe = std::env::var("CARGO_BIN_EXE_Aeonmi").ok()
        .map(PathBuf::from)
        .or_else(|| {
            let p = PathBuf::from("target").join("debug").join(if cfg!(windows) {"Aeonmi.exe"} else {"Aeonmi"});
            if p.exists() { Some(p) } else { None }
        })
        .expect("Aeonmi binary path not found");
    let output = Command::new(&exe)
        .arg("run")
        .arg(&file_path)
        .arg("--native")
        .env("AEONMI_NATIVE", "1")
        .output()
        .expect("failed to run Aeonmi binary");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("10"), "stdout missing first value: {stdout}");
    assert!(stdout.contains("15"), "stdout missing second value: {stdout}");
    assert!(output.status.success(), "process failed: status={:?} stderr={} stdout={}", output.status.code(), String::from_utf8_lossy(&output.stderr), stdout);
}

// (Removed custom assert_env macro; using direct logic above.)