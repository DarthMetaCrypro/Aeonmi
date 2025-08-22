//! tests/format_snapshots.rs
//! Snapshot tests for canonical .ai formatting.

use std::fs;
use std::path::PathBuf;
use aeonmi as _; // ensure crate is linked
use aeonmi::core::formatter::format_ai;

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
            insta::assert_snapshot!(p.file_name().unwrap().to_string_lossy(), out);
        }
    }
}