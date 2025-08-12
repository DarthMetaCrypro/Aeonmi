use std::path::PathBuf;

#[allow(dead_code)]
pub fn main_with_opts(inputs: Vec<PathBuf>, check: bool) -> anyhow::Result<()> {
    let _ = (inputs, check);
    println!("(format) placeholder: formatting...");
    Ok(())
}
