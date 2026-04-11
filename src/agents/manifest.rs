//! The beck manifest: a JSON file at `<beck_home>/.beck-manifest.json`
//! tracking every file beck has installed into an agent's directory.
//!
//! Contract (locked in `.rune/plan-beck-link-spec.md` §3):
//!
//! ```json
//! {
//!   "schema_version": 1,
//!   "entries": [
//!     {
//!       "skill": "caveman",
//!       "agent": "claude-code",
//!       "target": "/abs/path/to/SKILL.md",
//!       "mode": "symlink",
//!       "sha256": "3f2a...c1",
//!       "installed_at": "2026-04-11T02:55:00Z"
//!     }
//!   ]
//! }
//! ```
//!
//! Invariants:
//! - `schema_version` is ALWAYS serialized, even on an empty manifest.
//! - Writes are atomic: write to `<path>.tmp`, fsync, rename.
//! - No global mutable state; callers pass `&mut Manifest` explicitly.

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{CliError, Result};

/// Current on-disk schema version. Bump on any breaking change and teach
/// `Manifest::load` to reject or migrate older versions.
pub const SCHEMA_VERSION: u32 = 1;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum InstallMode {
    Symlink,
    Copy,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Entry {
    /// Skill name, matches the folder under `~/beck/skills/`.
    pub skill: String,
    /// Agent identifier (e.g. `"claude-code"`).
    pub agent: String,
    /// Absolute path beck wrote to on disk.
    pub target: PathBuf,
    /// Whether beck created a symlink or a copy at `target`.
    pub mode: InstallMode,
    /// sha256 of the source SKILL.md at install time.
    pub sha256: String,
    /// RFC3339 timestamp of the install.
    pub installed_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Manifest {
    pub schema_version: u32,
    pub entries: Vec<Entry>,
}

impl Default for Manifest {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            entries: Vec::new(),
        }
    }
}

impl Manifest {
    /// Build an empty manifest at the current schema version. Does not touch
    /// the filesystem.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Load a manifest from disk. Returns `Validation` on corrupt JSON or on
    /// an unsupported `schema_version`. Returns the raw io error on missing
    /// files and permission errors.
    pub fn load(path: &Path) -> Result<Self> {
        let bytes = fs::read(path)?;
        let manifest: Manifest = serde_json::from_slice(&bytes).map_err(|_| {
            CliError::Validation(
                "manifest corrupt, run beck check --rebuild-manifest".into(),
            )
        })?;

        if manifest.schema_version != SCHEMA_VERSION {
            return Err(CliError::Validation(format!(
                "manifest schema v{} unsupported, beck only knows v{}",
                manifest.schema_version, SCHEMA_VERSION
            )));
        }

        Ok(manifest)
    }

    /// Atomic save: write to `<path>.tmp`, fsync the file, rename onto the
    /// final path. A crash between the write and the rename leaves the
    /// previous manifest (or nothing) intact. Never call `to_writer` on the
    /// final path directly.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let tmp_path = tmp_sibling(path);

        // Scope the handle so it is flushed, synced, and dropped before the
        // rename. On macOS and Linux, rename(2) is atomic within a single
        // filesystem; cross-device renames bubble up as io errors with
        // context for the caller.
        {
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&tmp_path)?;

            let json = serde_json::to_vec_pretty(self).map_err(|e| {
                CliError::Validation(format!("failed to serialize manifest: {e}"))
            })?;
            file.write_all(&json)?;
            file.write_all(b"\n")?;
            file.sync_all()?;
        }

        fs::rename(&tmp_path, path)?;

        // Best-effort: fsync the parent directory so the rename is durable.
        // Failures here are non-fatal; the rename itself already succeeded.
        if let Some(parent) = path.parent() {
            let _ = File::open(parent).and_then(|f| f.sync_all());
        }

        Ok(())
    }

    /// Append an entry. Does not dedupe; the caller is responsible for
    /// checking via `find` first.
    pub fn add(&mut self, entry: Entry) {
        self.entries.push(entry);
    }

    /// Remove the first entry matching `(skill, agent)` and return it.
    pub fn remove(&mut self, skill: &str, agent: &str) -> Option<Entry> {
        let idx = self
            .entries
            .iter()
            .position(|e| e.skill == skill && e.agent == agent)?;
        Some(self.entries.remove(idx))
    }

    /// Borrow the first entry matching `(skill, agent)`.
    pub fn find(&self, skill: &str, agent: &str) -> Option<&Entry> {
        self.entries
            .iter()
            .find(|e| e.skill == skill && e.agent == agent)
    }
}

fn tmp_sibling(path: &Path) -> PathBuf {
    let mut os = path.as_os_str().to_os_string();
    os.push(".tmp");
    PathBuf::from(os)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn sample_entry(skill: &str, agent: &str) -> Entry {
        Entry {
            skill: skill.into(),
            agent: agent.into(),
            target: PathBuf::from(format!("/tmp/{skill}/SKILL.md")),
            mode: InstallMode::Symlink,
            sha256: "deadbeef".into(),
            installed_at: "2026-04-11T02:55:00Z".into(),
        }
    }

    fn tempdir(name: &str) -> PathBuf {
        let base = std::env::temp_dir().join(format!("beck-manifest-tests-{name}"));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        base
    }

    #[test]
    fn empty_manifest_has_schema_version() {
        let m = Manifest::empty();
        assert_eq!(m.schema_version, SCHEMA_VERSION);
        assert!(m.entries.is_empty());

        // schema_version must serialize even on empty entries.
        let json = serde_json::to_string(&m).unwrap();
        assert!(json.contains("\"schema_version\":1"));
        assert!(json.contains("\"entries\":[]"));
    }

    #[test]
    fn save_then_load_round_trip() {
        let dir = tempdir("round-trip");
        let path = dir.join(".beck-manifest.json");

        let mut m = Manifest::empty();
        m.add(sample_entry("caveman", "claude-code"));
        m.add(sample_entry("compress", "claude-code"));

        m.save(&path).expect("save");
        let loaded = Manifest::load(&path).expect("load");

        assert_eq!(loaded, m);
        assert_eq!(loaded.entries.len(), 2);
        assert_eq!(loaded.entries[0].skill, "caveman");
    }

    #[test]
    fn save_is_atomic_no_leftover_tmp() {
        let dir = tempdir("atomic");
        let path = dir.join(".beck-manifest.json");

        let m = Manifest::empty();
        m.save(&path).expect("save");

        let tmp = tmp_sibling(&path);
        assert!(!tmp.exists(), "tmp file should be renamed away");
        assert!(path.exists());
    }

    #[test]
    fn save_overwrites_existing_atomically() {
        let dir = tempdir("overwrite");
        let path = dir.join(".beck-manifest.json");

        let mut first = Manifest::empty();
        first.add(sample_entry("one", "claude-code"));
        first.save(&path).unwrap();

        let mut second = Manifest::empty();
        second.add(sample_entry("two", "claude-code"));
        second.save(&path).unwrap();

        let loaded = Manifest::load(&path).unwrap();
        assert_eq!(loaded.entries.len(), 1);
        assert_eq!(loaded.entries[0].skill, "two");
    }

    #[test]
    fn add_find_remove_cycle() {
        let mut m = Manifest::empty();
        assert!(m.find("caveman", "claude-code").is_none());

        m.add(sample_entry("caveman", "claude-code"));
        assert!(m.find("caveman", "claude-code").is_some());

        let removed = m.remove("caveman", "claude-code");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().skill, "caveman");

        // Second remove returns None.
        assert!(m.remove("caveman", "claude-code").is_none());
        assert!(m.find("caveman", "claude-code").is_none());
    }

    #[test]
    fn find_discriminates_by_agent() {
        let mut m = Manifest::empty();
        m.add(sample_entry("caveman", "claude-code"));
        m.add(sample_entry("caveman", "cursor"));

        assert!(m.find("caveman", "claude-code").is_some());
        assert!(m.find("caveman", "cursor").is_some());
        assert!(m.find("caveman", "nonexistent").is_none());

        let removed = m.remove("caveman", "cursor").unwrap();
        assert_eq!(removed.agent, "cursor");
        assert_eq!(m.entries.len(), 1);
        assert_eq!(m.entries[0].agent, "claude-code");
    }

    #[test]
    fn load_rejects_corrupt_json() {
        let dir = tempdir("corrupt");
        let path = dir.join(".beck-manifest.json");
        fs::write(&path, b"not valid json {{{").unwrap();

        let err = Manifest::load(&path).expect_err("corrupt should error");
        match err {
            CliError::Validation(msg) => assert!(msg.contains("manifest corrupt")),
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn load_rejects_unknown_schema_version() {
        let dir = tempdir("schema");
        let path = dir.join(".beck-manifest.json");
        fs::write(&path, b"{\"schema_version\":999,\"entries\":[]}").unwrap();

        let err = Manifest::load(&path).expect_err("unknown schema should error");
        match err {
            CliError::Validation(msg) => {
                assert!(msg.contains("v999"));
                assert!(msg.contains("v1"));
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn install_mode_serializes_lowercase() {
        let entry = sample_entry("x", "claude-code");
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"mode\":\"symlink\""));

        let copy_entry = Entry {
            mode: InstallMode::Copy,
            ..sample_entry("y", "claude-code")
        };
        let json = serde_json::to_string(&copy_entry).unwrap();
        assert!(json.contains("\"mode\":\"copy\""));
    }
}
