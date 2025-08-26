use aeonmi_project::core::incremental::{reset_metrics_full, record_partial_savings, SAVINGS_METRICS};

#[test]
fn savings_record_accumulates() {
    reset_metrics_full();
    record_partial_savings(10, 30); // savings 20
    record_partial_savings(5,  15); // savings 10 (total 30)
    let sm = SAVINGS_METRICS.lock().unwrap().clone();
    assert_eq!(sm.cumulative_partial_ns, 15);
    assert_eq!(sm.cumulative_estimated_full_ns, 45);
    assert_eq!(sm.cumulative_savings_ns, 30);
}

#[test]
fn savings_ignore_negative() {
    reset_metrics_full();
    // estimated lower than partial -> should be ignored
    record_partial_savings(50, 40);
    let sm = SAVINGS_METRICS.lock().unwrap().clone();
    assert_eq!(sm.cumulative_partial_ns, 0);
    assert_eq!(sm.cumulative_estimated_full_ns, 0);
    assert_eq!(sm.cumulative_savings_ns, 0);
}
