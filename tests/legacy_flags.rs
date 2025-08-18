use std::process::Command;

#[test]
fn legacy_top_level_flags_still_work() {
    let cmd = Command::new(env!("CARGO_BIN_EXE_aeonmi_project"))
        .args(["--tokens", "--out", "legacy.js", "examples/hello.ai"])
        .output()
        .expect("spawn");
    assert!(
        cmd.status.success(),
        "legacy flags should work without subcommand"
    );
    let stdout = String::from_utf8_lossy(&cmd.stdout);
    assert!(stdout.contains("=== Tokens ==="));
}
