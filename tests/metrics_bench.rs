use std::process::Command;

fn run(args: &[&str]) -> (i32,String) { let out = Command::new(env!("CARGO_BIN_EXE_aeonmi")).args(args).output().expect("run"); (out.status.code().unwrap_or(-1), String::from_utf8_lossy(&out.stdout).to_string()) }

#[test]
fn metrics_bench_generates_functions() {
    let (_c,_o)= run(&["metrics-bench","--functions","3","--samples","5","--reset"]);
    let (_c2,json)= run(&["metrics-dump"]);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    let fm = v.get("functionMetrics").unwrap().as_object().unwrap();
    assert!(fm.len() >= 3, "expected at least 3 function metrics after bench");
}
