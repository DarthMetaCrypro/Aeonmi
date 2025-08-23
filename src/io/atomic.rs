//! Atomic file writes with automatic parent creation.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub fn atomic_write(dest: impl AsRef<Path>, bytes: impl AsRef<[u8]>) -> io::Result<()> {
    let dest = dest.as_ref();
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut tmp = tempfile::NamedTempFile::new_in(parent.unwrap_or_else(|| Path::new(".")))?;
    tmp.write_all(bytes.as_ref())?;
    let (_file, tmp_path) = tmp.keep()?;
    fs::rename(tmp_path, dest)?;
    Ok(())
}