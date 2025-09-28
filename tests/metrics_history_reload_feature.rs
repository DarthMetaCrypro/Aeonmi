#[cfg(feature = "debug-metrics")]
#[test]
fn savings_history_reload_with_samples() {
    use std::process::Command;
    fn run(args: &[&str]) -> (i32,String) { let out = Command::new(env!("CARGO_BIN_EXE_aeonmi")).args(args).output().expect("run"); (out.status.code().unwrap_or(-1), String::from_utf8_lossy(&out.stdout).to_string()) }
    // Configure history cap and inject samples
    let (_c,_o)= run(&["metrics-config","--set-history-cap","12"]);
    for i in 0..6 { let partial = 100 + i*10; let full = 300 + i*20; let (_c2,_o2)= run(&["metrics-inject-savings","--partial", &partial.to_string(), "--full", &full.to_string()]); }
    // Flush, capture JSON, then reload
    let (_c3,_o3)= run(&["metrics-flush"]);
    let (_c4,json)= run(&["metrics-dump"]);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    let savings = v.get("savings").unwrap();
    let samples = savings.get("recent_samples").unwrap().as_array().unwrap();
    assert!(samples.len() >= 6);
    let recent_partial = savings.get("recent_window_partial_ns").and_then(|x| x.as_u64()).unwrap();
    assert!(recent_partial > 0);
}
