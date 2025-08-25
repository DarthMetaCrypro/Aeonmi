use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Ensure the tauri.conf.json is available in OUT_DIR for generate_context!
    let out = env::var("OUT_DIR").expect("OUT_DIR not set");
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let src = PathBuf::from(manifest_dir).join("tauri.conf.json");
    let dst = PathBuf::from(out).join("tauri.conf.json");
    fs::copy(&src, &dst).expect("failed to copy tauri.conf.json to OUT_DIR");
    println!("cargo:rerun-if-changed={}", src.display());
}
