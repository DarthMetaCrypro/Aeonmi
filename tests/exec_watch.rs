use std::{fs, process::Command};

fn bin() -> String { env!("CARGO_BIN_EXE_aeonmi_project").to_string() }

#[test]
fn exec_watch_once_env_breaks_loop() {
    let ai = "watch_once.ai";
    fs::write(ai, "let x = 1\nlog x\n").unwrap();
    let mut cmd = Command::new(bin());
    cmd.args(["exec", ai, "--watch", "--no-run"]);
    cmd.env("AEONMI_WATCH_ONCE", "1");
    let status = cmd.status().unwrap();
    assert!(status.success());
}
