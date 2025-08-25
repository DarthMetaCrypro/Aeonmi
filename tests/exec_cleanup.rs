use std::{fs, path::Path, process::Command};

fn bin() -> String { env!("CARGO_BIN_EXE_aeonmi_project").to_string() }

#[test]
fn exec_ai_removes_temp_by_default() {
    let ai = "temp_cleanup.ai";
    fs::write(ai, "let a = 1\nlog a\n").unwrap();
    let status = Command::new(bin()).args(["exec", ai, "--no-run"]).status().unwrap();
    assert!(status.success());
    assert!(!Path::new("__exec_tmp.js").exists(), "temp js should be removed");
}

#[test]
fn exec_ai_keeps_temp_with_flag() {
    let ai = "temp_keep.ai";
    fs::write(ai, "let a = 2\nlog a\n").unwrap();
    let status = Command::new(bin()).args(["exec", ai, "--keep-temp", "--no-run"]).status().unwrap();
    assert!(status.success());
    assert!(Path::new("__exec_tmp.js").exists(), "temp js should remain when --keep-temp");
    let _ = fs::remove_file("__exec_tmp.js");
}
