use anyhow::Result;
use std::path::PathBuf;
use std::fs;
use std::io::Write;
use chrono::Local;

pub fn new_file(path: Option<PathBuf>) -> Result<()> {
    let target = path.unwrap_or_else(|| PathBuf::from("untitled.ai"));
    if target.exists() {
        println!("new: '{}' already exists (leaving unchanged)", target.display());
        return Ok(());
    }
    if let Some(parent) = target.parent() { if !parent.as_os_str().is_empty() { fs::create_dir_all(parent)?; } }
    let now = Local::now().format("%Y-%m-%d %H:%M:%S");
    let template = format!(r#"// Aeonmi source created {now}
let greeting = "Hello Aeonmi";
function square(x) {{ return x * x; }}
let total = 0;
for let i = 0; i < 5; i = i + 1 {{ total = total + square(i); }}
log(greeting, total);
"#);
    let mut f = fs::File::create(&target)?;
    f.write_all(template.as_bytes())?;
    println!("new: created '{}'", target.display());
    Ok(())
}

pub fn open(path: PathBuf) -> Result<()> {
    println!("open: '{}' (placeholder)", path.display());
    Ok(())
}

pub fn save(path: Option<PathBuf>) -> Result<()> {
    if let Some(p) = path {
        println!("save: '{}' (placeholder)", p.display());
    } else {
        println!("save: current buffer (placeholder)");
    }
    Ok(())
}

pub fn save_as(path: PathBuf) -> Result<()> {
    println!("saveas: '{}' (placeholder)", path.display());
    Ok(())
}

pub fn close(path: Option<PathBuf>) -> Result<()> {
    if let Some(p) = path {
        println!("close: '{}' (placeholder)", p.display());
    } else {
        println!("close: current buffer (placeholder)");
    }
    Ok(())
}

pub fn import(path: PathBuf) -> Result<()> {
    println!("import: '{}' (placeholder)", path.display());
    Ok(())
}

pub fn export(path: PathBuf, format: Option<String>) -> Result<()> {
    println!("export: '{}' as '{}' (placeholder)", path.display(), format.unwrap_or_else(|| "auto".into()));
    Ok(())
}

pub fn upload(path: PathBuf) -> Result<()> {
    use std::fs;
    use std::env;

    let src = if path.is_absolute() { path } else { env::current_dir()?.join(path) };
    if !src.exists() {
        anyhow::bail!("upload failed: source '{}' does not exist", src.display());
    }

    let repo_root = env::current_dir()?;
    let uploads_dir = repo_root.join("uploads");
    if !uploads_dir.exists() {
        fs::create_dir_all(&uploads_dir)?;
    }

    if src.is_dir() {
        let options = fs_extra::dir::CopyOptions::new();
        let to = uploads_dir.join(src.file_name().unwrap_or_else(|| std::ffi::OsStr::new("uploaded_dir")));
        fs_extra::dir::copy(&src, &to, &options)?;
        println!("uploaded dir '{}' -> '{}'", src.display(), to.display());
        return Ok(());
    }

    let filename = src
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "uploaded.bin".into());
    let dst = uploads_dir.join(filename);
    fs::copy(&src, &dst)?;
    println!("uploaded '{}' -> '{}'", src.display(), dst.display());
    Ok(())
}

pub fn download(path: PathBuf) -> Result<()> {
    use std::fs;
    use std::env;

    let repo_root = env::current_dir()?;
    let src = if path.is_absolute() { path } else { repo_root.join(&path) };
    if !src.exists() {
        anyhow::bail!("download failed: source '{}' does not exist in workspace", src.display());
    }

    let downloads_dir = repo_root.join("downloads");
    if !downloads_dir.exists() {
        fs::create_dir_all(&downloads_dir)?;
    }

    if src.is_dir() {
        let options = fs_extra::dir::CopyOptions::new();
        let to = downloads_dir.join(src.file_name().unwrap_or_else(|| std::ffi::OsStr::new("downloaded_dir")));
        fs_extra::dir::copy(&src, &to, &options)?;
        println!("downloaded dir '{}' -> '{}'", src.display(), to.display());
        return Ok(());
    }

    let filename = src
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "downloaded.bin".into());
    let dst = downloads_dir.join(filename);
    fs::copy(&src, &dst)?;
    println!("downloaded '{}' -> '{}'", src.display(), dst.display());
    Ok(())
}
