use std::io::Write;
use std::process::{Command, Stdio};

fn bin() -> String { env!("CARGO_BIN_EXE_aeonmi_project").to_string() }

#[test]
fn shell_run_native_skips_js_emit() {
    // Launch shell with a temp dir
    let dir = tempfile::tempdir().unwrap();
    let ai_path = dir.path().join("demo.ai");
    std::fs::write(&ai_path, "let a = 5; let b = a * 2; log(b);").unwrap();

    let mut child = Command::new(bin())
        .current_dir(dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start shell");

    // Send run --native command then exit
    {
        let stdin = child.stdin.as_mut().expect("stdin");
        writeln!(stdin, "run --native {}", ai_path.display()).unwrap();
        writeln!(stdin, "exit").unwrap();
    }

    let output = child.wait_with_output().expect("shell run");
    assert!(output.status.success(), "shell exit status not success: stderr=\n{}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("10"), "expected native result in stdout: {stdout}");
    assert!(!dir.path().join("aeonmi.run.js").exists(), "unexpected JS file emitted in shell native run mode");
}
