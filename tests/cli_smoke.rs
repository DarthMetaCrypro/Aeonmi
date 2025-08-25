use std::fs;
use std::process::Command;

fn bin() -> String {
    // Cargo sets this environment variable for binary targets in integration tests
    env!("CARGO_BIN_EXE_aeonmi_project").to_string()
}

#[test]
fn cli_compiles_basic_file() {
    let src = r#"
        let x = 2 + 3;
        log(x);
    "#;
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("ok.ai");
    let out = dir.path().join("out.js");
    fs::write(&input, src).unwrap();

    let output = Command::new(bin())
        .arg("--tokens")
        .arg("--ast")
        .arg("--out")
        .arg(out.to_str().unwrap())
        .arg(input.to_str().unwrap())
        .output()
        .expect("failed to run aeonmi_project");

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let js = fs::read_to_string(&out).expect("output file should exist");
    assert!(
        js.contains("let x = (2 + 3);") || js.contains("let x = 2 + 3;"),
        "output JS missing expected code"
    );
    assert!(
        js.contains("console.log(x);"),
        "output JS missing expected console.log call"
    );
}

#[test]
fn cli_skips_semantic_when_flagged() {
    let src = "let x = 1; log(x);";
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("ok.ai");
    let out = dir.path().join("out.js");
    fs::write(&input, src).unwrap();

    let output = Command::new(bin())
        .arg("--no-sema")
        .arg("--out")
        .arg(out.to_str().unwrap())
        .arg(input.to_str().unwrap())
        .output()
        .expect("failed to run aeonmi_project");

    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Accept either stream; case-insensitive and allow minor phrasing differences
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
    .to_lowercase();

    assert!(
        combined.contains("semantic analyzer: skipped")
            || combined.contains("skipped by flag")
            || (combined.contains("semantic analyzer") && combined.contains("skipp"))
            || combined.contains("semantic analysis skipped"),
        "did not find expected skip message in output:\n{}",
        combined
    );
}

#[test]
fn cli_rejects_unsupported_emit() {
    let src = "let x = 1; log(x);";
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("ok.ai");
    fs::write(&input, src).unwrap();

    let output = Command::new(bin())
        .arg("--emit")
        .arg("wasm")
        .arg(input.to_str().unwrap())
        .output()
        .expect("failed to run aeonmi_project");

    assert!(
        !output.status.success(),
        "unexpected success running with unsupported emit kind"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Unsupported --emit kind"),
        "stderr did not contain expected error message"
    );
}
