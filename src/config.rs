use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub fn default_config_path() -> Option<PathBuf> {
    // ~\Users\you\.aeonmi\qpoly.toml on Windows; ~/.aeonmi/qpoly.toml elsewhere
    dirs_next::home_dir().map(|h| h.join(".aeonmi").join("qpoly.toml"))
}

pub fn resolve_config_path(cli_path: &Option<PathBuf>) -> Option<PathBuf> {
    if let Some(p) = cli_path {
        return Some(p.clone());
    }
    default_config_path()
}

#[allow(dead_code)]
pub fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Create config parent dir {}", parent.display()))?;
    }
    Ok(())
}
