use aeonmi_project::core::incremental::{reset_metrics_full, record_function_infer, FUNCTION_METRICS, record_reinfer_event, CALL_GRAPH_METRICS, persist_metrics, load_metrics};
use std::fs;

#[test]
fn function_metrics_record_and_reset() {
    reset_metrics_full();
    record_function_infer(0, 1000);
    record_function_infer(0, 500);
    record_function_infer(1, 200);
    {
        let fm = FUNCTION_METRICS.lock().unwrap();
        let m0 = fm.get(&0).unwrap();
        assert_eq!(m0.runs, 2);
        assert_eq!(m0.total_ns, 1500);
        let m1 = fm.get(&1).unwrap();
        assert_eq!(m1.runs, 1);
        assert_eq!(m1.total_ns, 200);
    }
    record_reinfer_event(3);
    {
        let cg = CALL_GRAPH_METRICS.lock().unwrap();
        assert_eq!(cg.reinfer_events, 3);
    }
}

#[test]
fn metrics_persist_and_load_round_trip() {
    reset_metrics_full();
    record_function_infer(2, 42);
    record_reinfer_event(1);
    persist_metrics();
    // Mutate in-memory then reload from disk to ensure overwrite
    {
        let mut fm = FUNCTION_METRICS.lock().unwrap();
        fm.clear();
    }
    load_metrics();
    {
        let fm = FUNCTION_METRICS.lock().unwrap();
        assert!(fm.get(&2).is_some());
    }
    // clean up metrics file to reduce test interference
    let _ = fs::remove_file("aeonmi_metrics.json");
}
