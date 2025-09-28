use aeonmi_project::core::lexer::Lexer;

/// Make sure we surface a couple more error shapes that got added recently.
#[test]
fn unterminated_string_reports() {
    let mut lx = Lexer::from_str("let s = \"oops;");
    let err = lx.tokenize().unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("Unterminated string") && msg.contains(":"),
        "got: {msg}"
    );
}

#[test]
fn bad_qubit_literal_reports() {
    // starts a qubit literal but never closes with '>'
    let mut lx = Lexer::from_str("|psi");
    let err = lx.tokenize().unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("Invalid qubit literal") || msg.contains("qubit"),
        "got: {msg}"
    );
}
