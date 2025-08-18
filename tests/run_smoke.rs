use std::path::Path;
use std::process::Command;

#[test]
fn run_subcommand_compiles_even_without_node() {
    let out = "aeonmi.run.js";
    let status = Command::new(env!("CARGO_BIN_EXE_aeonmi_project"))
        .args(["run", "examples/hello.ai", "--out", out])
        .status()
        .expect("failed to spawn");
    assert!(status.success(), "run subcommand should succeed");
    assert!(Path::new(out).exists(), "compiled output should exist");
}
