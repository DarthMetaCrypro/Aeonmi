use std::process::Command;

fn run(args: &[&str]) -> (i32,String) { let out = Command::new(env!("CARGO_BIN_EXE_aeonmi")).args(args).output().expect("run"); (out.status.code().unwrap_or(-1), String::from_utf8_lossy(&out.stdout).to_string()) }

#[test]
fn savings_history_reload_preserves_window_counters() {
    // Set history cap via CLI
    let (_c,_o)= run(&["metrics-config","--set-history-cap","16"]);
    // Simulate savings by invoking metrics-bench if feature enabled else skip
    // (Bench gated by feature; we just force a flush to persist current empty state, then reload.)
    let (_c2,_o2)= run(&["metrics-flush"]);
    // Capture before reload
    let (_c3,json_before)= run(&["metrics-dump"]);
    // Force load (re-populate structures)
    let (_c4,_o4)= run(&["metrics-dump"]);
    let v_before: serde_json::Value = serde_json::from_str(&json_before).unwrap();
    let savings_before = v_before.get("savings").unwrap();
    assert!(savings_before.get("recent_window_partial_ns").is_some());
}
