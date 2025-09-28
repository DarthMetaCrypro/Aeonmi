//! tests/format_snapshots.rs
//! Snapshot tests for canonical .ai formatting.

use aeonmi_project::core::formatter::format_ai;
use aeonmi_project as _;
use std::fs;
use std::path::PathBuf;

#[test]
fn examples_are_stably_formatted() {
    let examples = [
        "examples/hello.ai",
        "examples/control_flow.ai",
        "examples/functions.ai",
        "examples/glyph.ai",
    ];
    for ex in examples {
        let p = PathBuf::from(ex);
        if let Ok(src) = fs::read_to_string(&p) {
            let out = format_ai(&src);
            insta::assert_snapshot!(p.file_name().unwrap().to_string_lossy().as_ref(), &out);
        }
    }
}
