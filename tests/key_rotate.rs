use std::process::Command;

// Helper to run cli (aeonmi) with args returning stdout
fn run(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_aeonmi")).args(args).output().expect("run aeonmi");
    (out.status.code().unwrap_or(-1), String::from_utf8_lossy(&out.stdout).to_string(), String::from_utf8_lossy(&out.stderr).to_string())
}

#[test]
fn key_rotation_preserves_plaintext() {
    // Use temp config dir to isolate keys file
    let td = tempfile::tempdir().unwrap();
    std::env::set_var("AEONMI_CONFIG_DIR", td.path());
    // Set a key
    let (_c,_o,_e)= run(&["key-set","testprov","ABC123"]);
    // Get it
    let (_c2,o2,_e2)= run(&["key-get","testprov"]);
    assert!(o2.trim()=="ABC123");
    // Rotate in JSON mode
    let (_c3,o3,_e3)= run(&["key-rotate","--json"]);
    let v: serde_json::Value = serde_json::from_str(&o3).unwrap();
    assert!(v.get("rotated").and_then(|x| x.as_u64()).unwrap_or(0) >= 1);
    // Get again
    let (_c4,o4,_e4)= run(&["key-get","testprov"]);
    assert!(o4.trim()=="ABC123");
}
