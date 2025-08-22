//! src/commands/format.rs
//! Batch formatter for .ai files with --check mode.

use std::fs;
use std::path::PathBuf;
use anyhow::Result;

use crate::core::formatter::format_ai;
use crate::io::atomic::atomic_write;

pub fn main(paths: Vec<PathBuf>, check: bool) -> Result<i32> {
    let mut changed = 0usize;
    for p in paths {
        let Ok(orig) = fs::read_to_string(&p) else {
            eprintln!("warn: cannot read {}", p.display());
            continue;
        };
        let formatted = format_ai(&orig);

        if check {
            if normalized(&orig) != normalized(&formatted) {
                println!("{}", p.display());
                changed += 1;
            }
        } else {
            if normalized(&orig) != normalized(&formatted) {
                atomic_write(&p, formatted.as_bytes())?;
                println!("formatted {}", p.display());
                changed += 1;
            }
        }
    }
    Ok(if changed == 0 { 0 } else { 1 })
}

fn normalized(s: &str) -> String {
    let mut t = s.replace("\r\n", "\n").replace('\r', "\n");
    if t.ends_with('\n') { t.pop(); }
    t
}