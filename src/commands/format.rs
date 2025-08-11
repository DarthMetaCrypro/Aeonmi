use std::path::PathBuf;

pub fn main_with_opts(inputs: Vec<PathBuf>, check: bool) -> anyhow::Result<()> {
    let _ = (inputs, check);
    println!("(format) placeholder: formatting...");
    Ok(())
}
