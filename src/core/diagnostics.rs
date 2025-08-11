// src/core/diagnostics.rs
//! Pretty, colored, file+line diagnostics (minimal, no external parser).

use colored::Colorize;

pub struct Span {
    pub line: usize,
    pub col: usize,
    pub len: usize, // underline length (use 1 if unknown)
}

impl Span {
    pub fn single(line: usize, col: usize) -> Self {
        Self { line, col, len: 1 }
    }
}

pub fn print_error(filename: &str, source: &str, title: &str, span: Span) {
    eprintln!(
        "{} {}",
        "error:".bright_red().bold(),
        title.bright_white()
    );
    let (ln, col) = (span.line, span.col);
    let line_text = nth_line(source, ln).unwrap_or_default();

    // line number gutter
    let ln_str = format!("{:>4}", ln);
    eprintln!("{} {}", "-->".bright_blue(), format!("{}:{}:{}", filename, ln, col).bright_white());
    eprintln!(" {} {}", ln_str.dimmed(), "|".dimmed());
    eprintln!(
        "{} {} {}",
        ln_str.dimmed(),
        "|".dimmed(),
        line_text
    );

    // underline with ^^^^^
    let underline = " ".repeat(col.saturating_sub(1)) + &"^".repeat(span.len.max(1));
    eprintln!(
        " {} {} {}",
        " ".repeat(ln_str.len()).dimmed(),
        "|".dimmed(),
        underline.bright_red()
    );
    eprintln!();
}

fn nth_line(src: &str, n: usize) -> Option<String> {
    src.lines().nth(n.saturating_sub(1)).map(|s| s.to_string())
}
