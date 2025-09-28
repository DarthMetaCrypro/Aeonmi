// Integration test: ensures metrics JSON includes new schema v5 fields.
// Runs `aeonmi metrics-dump` after ensuring metrics file exists.

use std::process::Command;

#[test]
fn metrics_json_includes_v5_fields() {
    // Run metrics-dump
    let output = Command::new(env!("CARGO_BIN_EXE_aeonmi"))
        .arg("metrics-dump")
        .output()
        .expect("failed to run aeonmi metrics-dump");
    assert!(output.status.success(), "metrics-dump did not succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let val: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    let version = val.get("version").and_then(|v| v.as_u64()).unwrap_or(0);
    assert!(version >= 5, "expected metrics version >=5, got {}", version);
    let savings = val.get("savings").and_then(|v| v.as_object()).expect("savings object");
    assert!(savings.contains_key("cumulative_savings_pct"), "missing cumulative_savings_pct");
    assert!(savings.contains_key("cumulative_partial_pct"), "missing cumulative_partial_pct");
    // functionMetrics should be an object
    val.get("functionMetrics").and_then(|v| v.as_object()).expect("functionMetrics object");
}
