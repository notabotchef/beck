//! XDG-style path resolution. Keeps `dirs` isolated to one module so the
//! rest of the crate can be tested with arbitrary paths.

use std::path::PathBuf;

use crate::consts::APP_DIR;
use crate::error::{CliError, Result};

/// `~/.local/share/beck/` on Linux, `~/Library/Application Support/beck/` on macOS.
pub fn data_dir() -> Result<PathBuf> {
    let base = dirs::data_dir()
        .ok_or_else(|| CliError::Validation("could not resolve XDG data_dir".into()))?;
    Ok(base.join(APP_DIR))
}

/// Path to the SQLite database file. Creates the parent dir if missing.
pub fn db_path() -> Result<PathBuf> {
    let dir = data_dir()?;
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join("skills.db"))
}

/// Default skill roots to walk on `beck sync` with no explicit config.
/// Mirrors the CEO plan: `~/.hermes/skills` and `~/.claude/skills`.
/// Roots that do not exist on disk are silently skipped.
pub fn default_roots() -> Vec<PathBuf> {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return Vec::new(),
    };
    let candidates = [
        home.join(".hermes").join("skills"),
        home.join(".claude").join("skills"),
    ];
    candidates
        .into_iter()
        .filter(|p| p.exists())
        .collect()
}
