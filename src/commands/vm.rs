use anyhow::Result;
use std::path::PathBuf;

pub fn start() -> Result<()> {
    println!("vm: start (placeholder)");
    Ok(())
}

pub fn stop() -> Result<()> {
    println!("vm: stop (placeholder)");
    Ok(())
}

pub fn status() -> Result<()> {
    println!("vm: status: running (placeholder)");
    Ok(())
}

pub fn reset() -> Result<()> {
    println!("vm: reset (placeholder)");
    Ok(())
}

pub fn snapshot(name: String) -> Result<()> {
    use std::env;
    use std::fs;
    let repo = env::current_dir()?;
    let vmdir = repo.join(".aeonmi");
    if !vmdir.exists() {
        fs::create_dir_all(&vmdir)?;
    }
    let snaps = vmdir.join("vm_snapshots");
    if !snaps.exists() {
        fs::create_dir_all(&snaps)?;
    }
    let dest = snaps.join(&name);
    if dest.exists() {
        println!("overwriting snapshot '{}'", name);
        fs::remove_dir_all(&dest)?;
    }
    // For simplicity, snapshot the whole repo into the snapshot dir
    let options = fs_extra::dir::CopyOptions::new();
    fs_extra::dir::copy(&repo, &dest, &options)?;
    println!("vm: snapshot '{}' created", name);
    Ok(())
}

pub fn restore(name: String) -> Result<()> {
    use std::env;
    use std::fs;
    let repo = env::current_dir()?;
    let snaps = repo.join(".aeonmi").join("vm_snapshots");
    let src = snaps.join(&name);
    if !src.exists() {
        anyhow::bail!("snapshot '{}' not found", name);
    }
    // For safety, copy snapshot to a temporary dir and then move
    let tmp = repo.join(".aeonmi").join("tmp_restore");
    if tmp.exists() {
        fs::remove_dir_all(&tmp)?;
    }
    let options = fs_extra::dir::CopyOptions::new();
    fs_extra::dir::copy(&src, &tmp, &options)?;
    // Now copy contents back into repo (overwriting)
    for entry in std::fs::read_dir(&tmp)? {
        let entry = entry?;
        let name = entry.file_name();
        let srcp = entry.path();
        let dst = repo.join(name);
        if dst.exists() {
            if dst.is_dir() {
                fs::remove_dir_all(&dst)?;
            } else {
                fs::remove_file(&dst)?;
            }
        }
        if srcp.is_dir() {
            fs_extra::dir::copy(&srcp, &dst, &options)?;
        } else {
            fs::copy(&srcp, &dst)?;
        }
    }
    fs::remove_dir_all(&tmp)?;
    println!("vm: restored snapshot '{}'", name);
    Ok(())
}

pub fn mount(dir: PathBuf) -> Result<()> {
    println!("vm: mount '{}' (placeholder)", dir.display());
    Ok(())
}
