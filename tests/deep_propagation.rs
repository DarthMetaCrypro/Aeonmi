use aeonmi_project::core::incremental::compute_transitive_callers;

#[test]
fn deep_propagation_transitive() {
    // Build reverse edges for chain 0<-1<-2<-3 (i calls i-1), changed leaf 0 should collect all when deep=true
    // rev[i] = callers of i
    let rev = vec![vec![1], vec![2], vec![3], vec![]];
    let set_deep = compute_transitive_callers(0, &rev, true, 8);
    assert!(set_deep.contains(&1) && set_deep.contains(&2) && set_deep.contains(&3));
    // shallow with limit smaller than needed stops early (simulate deep=false, limit=1)
    let set_shallow = compute_transitive_callers(0, &rev, false, 1);
    // Should only have direct caller 1
    assert!(set_shallow.contains(&1) && !set_shallow.contains(&2) && !set_shallow.contains(&3));
}