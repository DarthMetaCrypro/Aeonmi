use std::process::Command;

fn run(args: &[&str]) -> (i32,String) {
    let out = Command::new(env!("CARGO_BIN_EXE_aeonmi")).args(args).output().expect("run");
    (out.status.code().unwrap_or(-1), String::from_utf8_lossy(&out.stdout).to_string())
}

#[test]
fn metrics_window_and_prune_behavior() {
    // Force small window
    std::env::set_var("AEONMI_EMA_ALPHA","50");
    std::env::set_var("AEONMI_METRICS_WINDOW","4");
    // Initial dump to create file
    let (_c,_o)= run(&["metrics-dump"]);
    // Simulate second dump (no activity) just to ensure fields exist
    let (_c2, json) = run(&["metrics-dump"]);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(v.get("windowCapacity").is_some(), "missing windowCapacity");
    assert!(v.get("emaAlphaPct").is_some(), "missing emaAlphaPct");
    assert!(v.get("functionMetricsPruned").and_then(|x| x.as_u64()).is_some());
    // Savings structure baseline
    let savings = v.get("savings").expect("savings");
    assert!(savings.get("recent_window_partial_ns").is_some());
    assert!(savings.get("recent_window_estimated_full_ns").is_some());
    assert!(savings.get("recent_window_savings_pct").is_some());
    assert!(savings.get("recent_samples").is_some());
}
