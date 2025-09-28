use std::path::PathBuf;

#[allow(dead_code)]
pub fn main_with_opts(inputs: Vec<PathBuf>, fix: bool) -> anyhow::Result<()> {
    use std::fs;
    use anyhow::Context;

    let mut problems = 0usize;
    for p in inputs {
        let content = fs::read_to_string(&p).with_context(|| format!("reading {}", p.display()))?;
        let mut out = String::new();
        let mut changed = false;
        for line in content.lines() {
            let trimmed_end = line.trim_end();
            if trimmed_end.len() != line.len() {
                problems += 1;
                changed = true;
            }
            out.push_str(trimmed_end);
            out.push('\n');
            // simple style: top-level lines that look like statements should end with ';'
            if !trimmed_end.ends_with(';') && trimmed_end.starts_with("let ") {
                problems += 1;
                if fix {
                    out.pop();
                    out.push_str(";");
                    out.push('\n');
                    changed = true;
                }
            }
        }
        if fix && changed {
            fs::write(&p, out).with_context(|| format!("writing {}", p.display()))?;
            println!("fixed {}", p.display());
        } else if problems > 0 {
            println!("{}: {} problems", p.display(), problems);
        }
    }
    if problems > 0 { std::process::exit(1); }
    Ok(())
}
