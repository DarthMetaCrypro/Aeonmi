//! Atomic file writes with automatic parent directory creation.

use std::fs;
use std::io::{self, Write};
use std::path::Path;

pub fn atomic_write(dest: impl AsRef<Path>, bytes: impl AsRef<[u8]>) -> io::Result<()> {
    let dest = dest.as_ref();

    // Determine parent directory (or current dir if none) and ensure it exists
    let parent_dir = dest.parent().unwrap_or_else(|| Path::new("."));
    if !parent_dir.as_os_str().is_empty() {
        fs::create_dir_all(parent_dir)?;
    }

    // Create a temporary file in the parent directory
    let mut tmp = tempfile::NamedTempFile::new_in(parent_dir)?;
    tmp.write_all(bytes.as_ref())?;

    // Persist the temporary file by moving it atomically to the destination
    let (_file, tmp_path) = tmp.keep()?;
    fs::rename(tmp_path, dest)?;

    Ok(())
}
