use aeonmi_project::core::qpoly::QPolyMap;

#[test]
fn qpoly_basic_replacements() {
    let m = QPolyMap::default();
    assert_eq!(m.apply_line("a -> b"),       "a → b");
    assert_eq!(m.apply_line("x <= 3"),       "x ≤ 3");
    assert_eq!(m.apply_line("y != 2"),       "y ≠ 2");
    assert_eq!(m.apply_line("|0> then |1>"), "∣0⟩ then ∣1⟩");
}

#[test]
fn qpoly_longest_match_wins() {
    let m = QPolyMap::default();
    // Ensure multi-char chords beat substrings
    assert_eq!(m.apply_line("a <=> b"), "a ⇔ b");
    assert_eq!(m.apply_line("<<< >>>"), "⟪ ⟫");
}
