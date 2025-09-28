use std::collections::VecDeque;

#[test]
fn metrics_ema_window_savings_and_pruning() {
    use aeonmi_project::core::incremental::{reset_metrics_full, set_ema_alpha, set_window_capacity, record_function_infer, build_metrics_json, FUNCTION_METRICS, FunctionInferenceMetric, session_start_epoch_ms, record_savings, SAVINGS_METRICS};
    // Reset all state
    reset_metrics_full();
    // Configure EMA alpha = 50, window size = 4
    set_ema_alpha(50);
    set_window_capacity(4);
    // Record sequence of durations for function index 0
    record_function_infer(0, 100); // runs=1 ema=100
    record_function_infer(0, 200); // ema=(100*50+200*50)/100=150
    record_function_infer(0, 300); // ema=(150*50+300*50)/100=225
    {
        let fm = FUNCTION_METRICS.lock().unwrap();
        let m = fm.get(&0).unwrap();
        assert_eq!(m.runs, 3);
        assert_eq!(m.ema_ns, 225, "EMA after three samples incorrect (got {})", m.ema_ns);
        assert_eq!(m.window.len(), 3);
    }
    // Add 4th and 5th samples to test rolling eviction
    record_function_infer(0, 500); // window now 4 elements
    record_function_infer(0, 700); // evict 100, window contains 200,300,500,700
    let json = build_metrics_json();
    let func = json.get("functionMetrics").and_then(|o| o.get("0")).expect("func 0 json");
    let window_avg = func.get("window_avg_ns").and_then(|v| v.as_u64()).unwrap();
    // Expected window avg = (200+300+500+700)/4 = 425
    assert_eq!(window_avg, 425, "window_avg_ns expected 425 got {}", window_avg);

    // Savings metrics validation
    reset_metrics_full();
    // Record two savings samples: (partial, est_full)
    record_savings(100, 300); // savings 200
    record_savings(120, 400); // savings 280 cumulative savings 480 est_full 700
    let json2 = build_metrics_json();
    let savings = json2.get("savings").unwrap();
    let cum_savings = savings.get("cumulative_savings_ns").and_then(|v| v.as_u64()).unwrap();
    assert_eq!(cum_savings, 480);
    let recent_pct = savings.get("recent_window_savings_pct").and_then(|v| v.as_f64()).unwrap();
    // Expected 480/700 * 100 = 68.571... allow small tolerance
    assert!((recent_pct - 68.571).abs() < 0.1, "recent_window_savings_pct off: {}", recent_pct);

    // Pruning: insert an old metric predating session start
    reset_metrics_full();
    let session_start = session_start_epoch_ms();
    {
        let mut fm = FUNCTION_METRICS.lock().unwrap();
        let mut m = FunctionInferenceMetric::default();
        m.last_run_epoch_ms = session_start.saturating_sub(1); // older than session
        m.runs = 1; m.total_ns = 100; m.last_ns = 100; m.ema_ns = 100; m.window = VecDeque::from(vec![100]);
        fm.insert(42, m);
    }
    let json3 = build_metrics_json();
    let pruned = json3.get("functionMetricsPruned").and_then(|v| v.as_u64()).unwrap();
    assert!(pruned >= 1, "expected at least one pruned metric");
    assert!(json3.get("functionMetrics").unwrap().get("42").is_none(), "old metric should be pruned and absent");

    // Ensure savings history persists through build (no panic when empty)
    {
        let sm = SAVINGS_METRICS.lock().unwrap();
        assert!(sm.history.len() <= sm.history_cap);
    }
}
