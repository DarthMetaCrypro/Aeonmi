use std::path::PathBuf;

#[allow(dead_code)]
pub fn main_with_opts(inputs: Vec<PathBuf>, fix: bool) -> anyhow::Result<()> {
    let _ = (inputs, fix);
    println!("(lint) placeholder: linting...");
    Ok(())
}
