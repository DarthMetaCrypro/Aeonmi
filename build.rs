#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    // Set icon (ensure assets/icon.ico exists and is a proper ICO)
    res.set_icon("assets/icon.ico");
    // Optional metadata (can be expanded)
    res.set("FileDescription", "Aeonmi Quantum/AI Language Shell");
    res.set("ProductName", "Aeonmi");
    res.set("LegalCopyright", "Copyright (C) 2025 Aeonmi");
    if let Err(e) = res.compile() {
        eprintln!("winres compile warning: {e}");
    }
}

#[cfg(not(windows))]
fn main() {}
