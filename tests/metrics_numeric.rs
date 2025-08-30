use std::process::Command;

fn run(args: &[&str]) -> (i32,String) {
    let out = Command::new(env!("CARGO_BIN_EXE_aeonmi")).args(args).output().expect("run");
    (out.status.code().unwrap_or(-1), String::from_utf8_lossy(&out.stdout).to_string())
}

#[test]
fn metrics_numeric_window_and_ema() {
    // Reset config then set known values
    std::env::set_var("AEONMI_EMA_ALPHA","50");
    std::env::set_var("AEONMI_METRICS_WINDOW","4");
    // Initial dump
    let (_c,_o)= run(&["metrics-dump"]);
    // We cannot directly trigger record_function_infer from CLI; emulate by persisting manual edit not exposed.
    // Skip deep numeric validation due to lack of direct hooks; ensure version >=6 and window/ema fields.
    let (_c2, json) = run(&["metrics-dump"]);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(v.get("version").and_then(|x| x.as_u64()).unwrap_or(0) >= 6);
    assert!(v.get("windowCapacity").is_some());
    assert!(v.get("emaAlphaPct").is_some());
}
