//! QPolygraphic keymap: built-in defaults + optional TOML config.
//!
//! - `QPolyMap::default()` → small built-in chords
//! - `QPolyMap::from_toml_file(path)` → load user chords
//! - `QPolyMap::from_user_default_or_builtin()` → ~/.aeonmi/qpoly.toml if present
//! - `apply_line(&self, s)` → replace chords with glyphs (longest-first)

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Clone, Debug)]
pub struct QPolyMap {
    /// Ordered (longest-first) replacements for simple chords -> glyphs/tokens.
    rules: Vec<(String, String)>,
}

// ----- Config TOML -----

#[derive(Debug, Deserialize)]
struct QPolyConfig {
    #[serde(default)]
    rules: Vec<QPolyRule>,
}

#[derive(Debug, Deserialize)]
struct QPolyRule {
    chord: String,
    glyph: String,
}

impl QPolyMap {
    pub fn default() -> Self {
        // Keep it tiny; expand later. Longest first matters when chords overlap.
        let mut rules = vec![
            // long first
            ("<<<".to_string(), "⟪".to_string()),
            (">>>".to_string(), "⟫".to_string()),
            ("<=>".to_string(), "⇔".to_string()),
            // arrows & comps
            ("->".to_string(),  "→".to_string()),
            ("<-".to_string(),  "←".to_string()),
            ("<=".to_string(),  "≤".to_string()),
            (">=".to_string(),  "≥".to_string()),
            ("!=".to_string(),  "≠".to_string()),
            ("==".to_string(),  "＝".to_string()),
            // QUBE-ish
            ("::".to_string(),  "∷".to_string()),
            (":=".to_string(),  "≔".to_string()),
            // quantum hints
            ("|0>".to_string(), "∣0⟩".to_string()),
            ("|1>".to_string(), "∣1⟩".to_string()),
        ];
        // Sort longest-first (no type inference headaches)
        rules.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
        Self { rules }
    }

    /// Load from TOML file.
    pub fn from_toml_file(path: &Path) -> Result<Self> {
        let txt = fs::read_to_string(path)
            .with_context(|| format!("reading {}", path.display()))?;
        let cfg: QPolyConfig = toml::from_str(&txt)
            .with_context(|| format!("parsing {}", path.display()))?;

        let mut rules: Vec<(String, String)> = cfg
            .rules
            .into_iter()
            .map(|r| (r.chord, r.glyph))
            .collect();

        // If user file is empty, fall back to builtin so the editor still works.
        if rules.is_empty() {
            return Ok(Self::default());
        }

        rules.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
        Ok(Self { rules })
    }

    /// Use ~/.aeonmi/qpoly.toml if present; otherwise built-in.
    pub fn from_user_default_or_builtin() -> Self {
        if let Some(p) = default_config_path() {
            if p.exists() {
                if let Ok(m) = Self::from_toml_file(&p) {
                    return m;
                }
                eprintln!("(warn) failed loading {}, using builtin map", p.display());
            }
        }
        Self::default()
    }

    /// Apply simple chord replacements to one line.
    pub fn apply_line(&self, s: &str) -> String {
        let mut out = s.to_owned();
        for (k, v) in &self.rules {
            if out.contains(k) {
                out = out.replace(k, v);
            }
        }
        out
    }
}

// ----- helpers for default path + fs -----

/// ~/.aeonmi/qpoly.toml
pub fn default_config_path() -> Option<PathBuf> {
    dirs_next::home_dir().map(|h| h.join(".aeonmi").join("qpoly.toml"))
}

pub fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    Ok(())
}
