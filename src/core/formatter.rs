//! Canonical, idempotent .ai source formatter.
//!
//! Design notes:
//! - Token-agnostic but syntax-aware via simple rules.
//! - Stable spacing around punctuation and operators.
//! - Indentation with 4 spaces, newline rules around braces.
//! - Idempotent: format(format(src)) == format(src)

use std::fmt::Write as _;

#[derive(Debug, Clone, Copy)]
pub struct FormatOptions {
    /// Spaces per indent level.
    pub indent: usize,
    /// Max consecutive blank lines allowed.
    pub max_blank: usize,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self { indent: 4, max_blank: 1 }
    }
}

pub fn format_ai(src: &str) -> String {
    format_ai_with(src, &FormatOptions::default())
}

pub fn format_ai_with(src: &str, opt: &FormatOptions) -> String {
    // 1) Normalize line endings, trim trailing whitespace lines.
    let mut s = src.replace("\r\n", "\n").replace('\r', "\n");
    s = s
        .lines()
        .map(|l| l.trim_end())
        .collect::<Vec<_>>()
        .join("\n");

    // 2) Collapse >max_blank consecutive blank lines.
    let mut cleaned = String::new();
    let mut blank_run = 0usize;
    for line in s.lines() {
        if line.trim().is_empty() {
            blank_run += 1;
            if blank_run <= opt.max_blank {
                cleaned.push('\n');
            }
        } else {
            blank_run = 0;
            cleaned.push_str(line);
            cleaned.push('\n');
        }
    }
    // avoid trailing newline explosion
    while cleaned.ends_with("\n\n") {
        cleaned.pop();
    }

    // 3) Token-ish pass: enforce spacing around punctuation/operators.
    //    (We avoid full tokenization to keep this decoupled and robust.)
    let mut out = String::with_capacity(cleaned.len() + cleaned.len() / 10);
    let mut indent = 0usize;
    let mut i = 0usize;
    let bytes = cleaned.as_bytes();

    // helpers
    let mut was_space = false;
    let mut need_space = false;
    let mut just_wrote_newline = true;

    while i < bytes.len() {
        let ch = bytes[i] as char;

        // Simple string/char literal passthrough (handles quotes)
        if ch == '"' || ch == '\'' {
            let quote = ch;
            push_pending_space(&mut out, &mut need_space, &mut was_space);
            out.push(quote);
            i += 1;
            while i < bytes.len() {
                let c = bytes[i] as char;
                out.push(c);
                i += 1;
                if c == '\\' && i < bytes.len() {
                    // escape next
                    out.push(bytes[i] as char);
                    i += 1;
                    continue;
                }
                if c == quote { break; }
            }
            just_wrote_newline = false;
            continue;
        }

        match ch {
            // Braces/newline/indent rules
            '{' => {
                push_pending_space(&mut out, &mut need_space, &mut was_space);
                out.push(' ');
                out.push('{');
                out.push('\n');
                indent += 1;
                write_indent(&mut out, opt.indent, indent);
                just_wrote_newline = true;
                was_space = false;
                need_space = false;
                i += 1;
            }
            '}' => {
                // dedent first
                if indent > 0 { indent -= 1; }
                if !just_wrote_newline {
                    out.push('\n');
                }
                write_indent(&mut out, opt.indent, indent);
                out.push('}');
                just_wrote_newline = false;
                was_space = false;
                need_space = true;
                i += 1;
            }
            '(' | '[' => {
                push_pending_space(&mut out, &mut need_space, &mut was_space);
                out.push(ch);
                just_wrote_newline = false;
                was_space = false;
                need_space = false;
                i += 1;
            }
            ')' | ']' => {
                out.push(ch);
                just_wrote_newline = false;
                was_space = false;
                need_space = true;
                i += 1;
            }
            ',' => {
                out.push(',');
                out.push(' ');
                just_wrote_newline = false;
                was_space = true;
                need_space = false;
                i += 1;
            }
            ';' => {
                out.push(';');
                out.push('\n');
                write_indent(&mut out, opt.indent, indent);
                just_wrote_newline = true;
                was_space = false;
                need_space = false;
                i += 1;
            }
            ':' => {
                out.push(':');
                out.push(' ');
                was_space = true;
                need_space = false;
                just_wrote_newline = false;
                i += 1;
            }
            ' ' | '\t' => {
                if !just_wrote_newline && !was_space {
                    need_space = true;
                }
                i += 1;
            }
            '\n' => {
                if !just_wrote_newline {
                    out.push('\n');
                    just_wrote_newline = true;
                }
                write_indent(&mut out, opt.indent, indent);
                was_space = false;
                need_space = false;
                i += 1;
            }
            _ => {
                push_pending_space(&mut out, &mut need_space, &mut was_space);
                out.push(ch);
                just_wrote_newline = false;
                was_space = false;
                need_space = false;
                i += 1;
            }
        }
    }

    // final trim of trailing spaces and ensure single trailing newline
    let mut final_s = out
        .lines()
        .map(|l| l.trim_end())
        .collect::<Vec<_>>()
        .join("\n");
    if !final_s.ends_with('\n') {
        final_s.push('\n');
    }
    final_s
}

fn write_indent(out: &mut String, indent_spaces: usize, level: usize) {
    for _ in 0..(indent_spaces * level) { out.push(' '); }
}

fn push_pending_space(out: &mut String, need_space: &mut bool, was_space: &mut bool) {
    if *need_space {
        out.push(' ');
        *need_space = false;
        *was_space = true;
    }
}