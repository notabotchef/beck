//! The Claude Code adapter: the reference `Adapter` implementation.
//!
//! Target layout on disk:
//! ```text
//! ~/.claude/skills/<name>/SKILL.md   (symlink back to ~/beck/skills/<name>/SKILL.md)
//! ```
//!
//! This adapter ships only in v0.2. Cursor, Windsurf, Cline, OpenCode,
//! and Continue are deferred to v0.3 (see
//! `.rune/plan-beck-link-spec.md` §0).
//!
//! Invariants this file enforces:
//! 1. `install()` never overwrites a foreign file. If `target` exists and
//!    is not a symlink pointing to our source, we refuse with a
//!    `CliError::Validation`.
//! 2. `install()` is idempotent on a target that already points at the
//!    same source.
//! 3. `uninstall()` never deletes a regular file. It verifies the path is
//!    a symlink pointing at the expected source before removing it.
//! 4. The target root is resolved from `dirs::home_dir()` so that tests
//!    can set `$HOME` to a tempdir and drive the adapter end to end.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::agents::adapter::{Adapter, InstallPlan};
use crate::agents::manifest::{Entry, InstallMode};
use crate::agents::paths::beck_home;
use crate::agents::skill::Skill;
use crate::error::{CliError, Result};

const AGENT_NAME: &str = "claude-code";
const CLAUDE_HOME_SUBDIR: &str = ".claude";
const SKILLS_SUBDIR: &str = "skills";
const SKILL_FILE_NAME: &str = "SKILL.md";

/// The Claude Code adapter. Stateless: the unit struct has no fields.
#[derive(Debug, Default, Clone, Copy)]
pub struct ClaudeCodeAdapter;

impl ClaudeCodeAdapter {
    pub const fn new() -> Self {
        Self
    }

    /// Resolve `~/.claude/`. Separate from `target_root()` so that
    /// `detect()` can reuse the same resolution without caring about the
    /// `skills` subdir.
    fn claude_home() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| {
            CliError::Validation("could not resolve home dir (HOME not set)".into())
        })?;
        Ok(home.join(CLAUDE_HOME_SUBDIR))
    }
}

impl Adapter for ClaudeCodeAdapter {
    fn name(&self) -> &'static str {
        AGENT_NAME
    }

    fn detect(&self) -> bool {
        match Self::claude_home() {
            Ok(path) => path.exists(),
            Err(_) => false,
        }
    }

    fn target_root(&self) -> Result<PathBuf> {
        Ok(Self::claude_home()?.join(SKILLS_SUBDIR))
    }

    fn plan(&self, skill: &Skill) -> Result<InstallPlan> {
        let target = self.target_root()?.join(&skill.name).join(SKILL_FILE_NAME);
        Ok(InstallPlan {
            source: skill.source_path.clone(),
            target,
            mode: InstallMode::Symlink,
            transform: None,
        })
    }

    fn install(&self, plan: &InstallPlan) -> Result<Entry> {
        // Only symlink installs are supported in v0.2. Copy mode is a
        // Phase 3 concern (when we teach the adapter to fall back on
        // filesystems that reject symlinks).
        if plan.mode != InstallMode::Symlink {
            return Err(CliError::Validation(
                "ClaudeCodeAdapter only supports symlink mode in v0.2".into(),
            ));
        }

        // Make sure the parent dir exists before we touch the leaf.
        if let Some(parent) = plan.target.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                CliError::Validation(format!(
                    "mkdir -p {} failed: {e}",
                    parent.display()
                ))
            })?;
        }

        // If the target already exists, fall into one of three branches:
        // - beck-managed symlink pointing at the SAME source → no-op.
        // - beck-managed symlink pointing at a DIFFERENT source → refuse
        //   and tell the caller to re-link. Phase 4 `link --force` will
        //   handle the re-point.
        // - anything else → refuse, this is a foreign file.
        if let Ok(meta) = fs::symlink_metadata(&plan.target) {
            if meta.file_type().is_symlink() {
                let current = fs::read_link(&plan.target).map_err(|e| {
                    CliError::Validation(format!(
                        "failed to read symlink at {}: {e}",
                        plan.target.display()
                    ))
                })?;
                if current == plan.source {
                    // Idempotent: already points at us.
                    return build_entry(&plan.source, &plan.target);
                }
                return Err(CliError::Validation(format!(
                    "target {} already linked to {}, not {}",
                    plan.target.display(),
                    current.display(),
                    plan.source.display()
                )));
            }
            return Err(CliError::Validation(format!(
                "target exists at {}, not beck-managed",
                plan.target.display()
            )));
        }

        // Create the symlink. Unix only in v0.2. Windows support is a
        // v0.3 concern and requires junctions instead.
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&plan.source, &plan.target).map_err(|e| {
                CliError::Validation(format!(
                    "symlink {} -> {} failed: {e}",
                    plan.target.display(),
                    plan.source.display()
                ))
            })?;
        }

        #[cfg(not(unix))]
        {
            return Err(CliError::Validation(
                "ClaudeCodeAdapter symlink install requires Unix in v0.2".into(),
            ));
        }

        build_entry(&plan.source, &plan.target)
    }

    fn uninstall(&self, entry: &Entry) -> Result<()> {
        if entry.agent != AGENT_NAME {
            return Err(CliError::Validation(format!(
                "entry agent {} does not match {AGENT_NAME}",
                entry.agent
            )));
        }

        let meta = match fs::symlink_metadata(&entry.target) {
            Ok(meta) => meta,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // Nothing to remove. Phase 5 `check` surfaces this as an
                // orphan; Phase 4 `unlink` is happy to drop the manifest
                // entry regardless.
                return Ok(());
            }
            Err(e) => {
                return Err(CliError::Validation(format!(
                    "cannot stat {} during uninstall: {e}",
                    entry.target.display()
                )));
            }
        };

        if !meta.file_type().is_symlink() {
            return Err(CliError::Validation(format!(
                "refusing to remove non-symlink at {}",
                entry.target.display()
            )));
        }

        // Verify the link points somewhere inside our beck skills home.
        // If someone manually repointed it, bail. Phase 5 swapped the
        // string-component heuristic for an actual path-prefix check
        // against `beck_home()?/skills`.
        if !link_points_into_beck_home(&entry.target)? {
            return Err(CliError::Validation(format!(
                "refusing to remove symlink at {}: no longer points into beck skills home",
                entry.target.display()
            )));
        }

        fs::remove_file(&entry.target).map_err(|e| {
            CliError::Validation(format!(
                "remove_file {} failed: {e}",
                entry.target.display()
            ))
        })?;

        // Best-effort: if the parent `<name>/` dir is now empty, remove
        // it too so `~/.claude/skills/` does not accumulate empty husks.
        if let Some(parent) = entry.target.parent() {
            let _ = fs::remove_dir(parent);
        }

        Ok(())
    }

    fn list_managed(&self) -> Result<Vec<PathBuf>> {
        let root = match self.target_root() {
            Ok(r) => r,
            Err(_) => return Ok(Vec::new()),
        };
        if !root.exists() {
            return Ok(Vec::new());
        }

        let skills_root = beck_home()?.join("skills");
        let mut out = Vec::new();

        // Walk `<target_root>/<skill_name>/SKILL.md` one level deep.
        for entry in fs::read_dir(&root)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let candidate = entry.path().join(SKILL_FILE_NAME);
            let Ok(meta) = fs::symlink_metadata(&candidate) else {
                continue;
            };
            if !meta.file_type().is_symlink() {
                continue;
            }
            if link_resolves_under(&candidate, &skills_root) {
                out.push(candidate);
            }
        }

        out.sort();
        Ok(out)
    }

    fn rebuild_entry(&self, target: &Path) -> Result<Entry> {
        let meta = fs::symlink_metadata(target).map_err(|e| {
            CliError::Validation(format!(
                "cannot stat {}: {e}",
                target.display()
            ))
        })?;
        if !meta.file_type().is_symlink() {
            return Err(CliError::Validation(format!(
                "cannot rebuild entry, {} is not a symlink",
                target.display()
            )));
        }
        let source = fs::read_link(target).map_err(|e| {
            CliError::Validation(format!(
                "cannot read symlink {}: {e}",
                target.display()
            ))
        })?;
        build_entry(&source, target)
    }

    fn ingest(&self) -> Result<Vec<Skill>> {
        let root = match self.target_root() {
            Ok(r) => r,
            Err(_) => return Ok(Vec::new()),
        };
        if !root.exists() {
            return Ok(Vec::new());
        }

        // Skills that are already symlinks back into beck were created
        // BY beck. Ingest walks the other direction: it pulls in skills
        // a user hand-wrote under `~/.claude/skills/`. So we skip any
        // entry whose SKILL.md is a symlink resolving under
        // `beck_home()?/skills/`.
        let beck_skills_root = beck_home().ok().map(|h| h.join("skills"));

        let mut out = Vec::new();
        for entry in fs::read_dir(&root)? {
            let entry = entry?;
            let ft = entry.file_type()?;
            if !ft.is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            let candidate = entry.path().join(SKILL_FILE_NAME);

            // Only consider actual SKILL.md files. If it is a symlink
            // into beck already, skip: that skill already lives in the
            // canonical home.
            let Ok(meta) = fs::symlink_metadata(&candidate) else {
                continue;
            };
            if meta.file_type().is_symlink()
                && let Some(ref beck_root) = beck_skills_root
                && link_resolves_under(&candidate, beck_root)
            {
                continue;
            }

            // Load the skill. `from_path` uses the parent dir as the
            // skill name, which is what we want for Claude Code's
            // `<name>/SKILL.md` layout.
            if let Ok(mut skill) = Skill::from_path(&candidate) {
                // Override the name to the directory under target_root,
                // in case the file is nested deeper.
                skill.name = name;
                out.push(skill);
            }
        }

        out.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(out)
    }
}

/// Does `link_path`, when its immediate symlink target is resolved,
/// live somewhere under `beck_skills_root`? This is the accurate
/// replacement for the old `link_points_into_beck` string heuristic.
/// Both sides are canonicalized, which handles `/var` vs `/private/var`
/// on macOS tempdirs.
fn link_resolves_under(link_path: &Path, beck_skills_root: &Path) -> bool {
    let Ok(target) = fs::read_link(link_path) else {
        return false;
    };
    let resolved = if target.is_absolute() {
        target
    } else {
        match link_path.parent() {
            Some(parent) => parent.join(&target),
            None => return false,
        }
    };
    let Ok(canon_target) = fs::canonicalize(&resolved) else {
        return false;
    };
    let Ok(canon_root) = fs::canonicalize(beck_skills_root) else {
        return false;
    };
    canon_target.starts_with(&canon_root)
}

/// Standalone version used by `uninstall`: canonicalizes the link
/// target and checks containment under `beck_home()?/skills`.
fn link_points_into_beck_home(link_path: &Path) -> Result<bool> {
    let skills_root = beck_home()?.join("skills");
    Ok(link_resolves_under(link_path, &skills_root))
}

/// Build a manifest `Entry` for a freshly installed (or already
/// beck-managed) symlink. Hashes the source file, not the target.
fn build_entry(source: &Path, target: &Path) -> Result<Entry> {
    let bytes = fs::read(source).map_err(|e| {
        CliError::Validation(format!(
            "failed to read source {} for entry hash: {e}",
            source.display()
        ))
    })?;

    let skill_name = source
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .ok_or_else(|| {
            CliError::Validation(format!(
                "cannot derive skill name from source {}",
                source.display()
            ))
        })?
        .to_string();

    Ok(Entry {
        skill: skill_name,
        agent: AGENT_NAME.into(),
        target: target.to_path_buf(),
        mode: InstallMode::Symlink,
        sha256: sha256_hex(&bytes),
        installed_at: rfc3339_now(),
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    use std::fmt::Write;
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest.iter() {
        let _ = write!(out, "{byte:02x}");
    }
    out
}

/// Minimal RFC3339 UTC timestamp formatter: `YYYY-MM-DDTHH:MM:SSZ`.
///
/// We intentionally do not depend on `chrono` or `time`. The implementation
/// uses the civil-from-days algorithm by Howard Hinnant
/// (http://howardhinnant.github.io/date_algorithms.html), which is public
/// domain and widely reused.
fn rfc3339_now() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    format_rfc3339(secs)
}

fn format_rfc3339(unix_secs: i64) -> String {
    let days = unix_secs.div_euclid(86_400);
    let secs_in_day = unix_secs.rem_euclid(86_400);
    let hour = secs_in_day / 3600;
    let minute = (secs_in_day % 3600) / 60;
    let second = secs_in_day % 60;

    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z"
    )
}

/// Civil-from-days (Hinnant). Returns `(year, month, day)` for a given
/// count of days since 1970-01-01.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365; // [0, 399]
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    let year = if m <= 2 { y + 1 } else { y };
    (year, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Env vars are process-global. Every test that pokes HOME serializes
    // through this mutex, mirroring the pattern in `agents::paths::tests`.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    struct HomeGuard {
        previous: Option<std::ffi::OsString>,
    }

    impl HomeGuard {
        fn set(value: &Path) -> Self {
            let previous = std::env::var_os("HOME");
            unsafe { std::env::set_var("HOME", value) };
            Self { previous }
        }
    }

    impl Drop for HomeGuard {
        fn drop(&mut self) {
            match &self.previous {
                Some(v) => unsafe { std::env::set_var("HOME", v) },
                None => unsafe { std::env::remove_var("HOME") },
            }
        }
    }

    fn tempdir(name: &str) -> PathBuf {
        let base = std::env::temp_dir().join(format!("beck-cc-tests-{name}"));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        base
    }

    /// Lay out `<root>/beck/skills/<name>/SKILL.md`, a fake HOME with a
    /// `.claude` dir, and return `(home, source_path, skill)`.
    fn fake_world(root: &Path, skill_name: &str) -> (PathBuf, PathBuf, Skill) {
        let home = root.join("home");
        fs::create_dir_all(home.join(".claude")).unwrap();

        let skills_home = home.join("beck").join("skills");
        let skill_dir = skills_home.join(skill_name);
        fs::create_dir_all(&skill_dir).unwrap();
        let source = skill_dir.join("SKILL.md");
        fs::write(
            &source,
            "---\nname: sample\ndescription: a test skill\n---\nbody\n",
        )
        .unwrap();

        let skill = Skill::from_path(&source).unwrap();
        (home, source, skill)
    }

    #[test]
    fn plan_targets_claude_skills_dir() {
        let _lock = ENV_LOCK.lock().unwrap();
        let root = tempdir("plan");
        let (home, source, skill) = fake_world(&root, "caveman");
        let _guard = HomeGuard::set(&home);

        let plan = ClaudeCodeAdapter.plan(&skill).unwrap();
        assert_eq!(plan.source, source);
        assert_eq!(
            plan.target,
            home.join(".claude")
                .join("skills")
                .join("caveman")
                .join("SKILL.md")
        );
        assert_eq!(plan.mode, InstallMode::Symlink);
        assert!(plan.transform.is_none());
    }

    #[test]
    fn detect_true_when_claude_dir_exists() {
        let _lock = ENV_LOCK.lock().unwrap();
        let root = tempdir("detect-yes");
        let home = root.join("home");
        fs::create_dir_all(home.join(".claude")).unwrap();
        let _guard = HomeGuard::set(&home);

        assert!(ClaudeCodeAdapter.detect());
    }

    #[test]
    fn detect_false_when_claude_dir_missing() {
        let _lock = ENV_LOCK.lock().unwrap();
        let root = tempdir("detect-no");
        let home = root.join("home");
        fs::create_dir_all(&home).unwrap();
        let _guard = HomeGuard::set(&home);

        assert!(!ClaudeCodeAdapter.detect());
    }

    #[test]
    fn install_creates_symlink_and_returns_entry() {
        let _lock = ENV_LOCK.lock().unwrap();
        let root = tempdir("install");
        let (home, source, skill) = fake_world(&root, "caveman");
        let _guard = HomeGuard::set(&home);

        let adapter = ClaudeCodeAdapter;
        let plan = adapter.plan(&skill).unwrap();
        let entry = adapter.install(&plan).unwrap();

        assert_eq!(entry.skill, "caveman");
        assert_eq!(entry.agent, "claude-code");
        assert_eq!(entry.mode, InstallMode::Symlink);
        assert_eq!(entry.target, plan.target);
        assert_eq!(entry.sha256.len(), 64);
        assert!(entry.installed_at.ends_with('Z'));

        let meta = fs::symlink_metadata(&plan.target).unwrap();
        assert!(meta.file_type().is_symlink());

        let link = fs::read_link(&plan.target).unwrap();
        assert_eq!(link, source);

        // Reading through the symlink gets us the source body.
        let contents = fs::read_to_string(&plan.target).unwrap();
        assert!(contents.contains("body"));
    }

    #[test]
    fn install_is_idempotent_for_same_source() {
        let _lock = ENV_LOCK.lock().unwrap();
        let root = tempdir("idempotent");
        let (home, _source, skill) = fake_world(&root, "caveman");
        let _guard = HomeGuard::set(&home);

        let adapter = ClaudeCodeAdapter;
        let plan = adapter.plan(&skill).unwrap();
        let first = adapter.install(&plan).unwrap();
        let second = adapter.install(&plan).unwrap();

        assert_eq!(first.target, second.target);
        assert_eq!(first.sha256, second.sha256);
        assert_eq!(first.skill, second.skill);

        // Still exactly one symlink.
        let link = fs::read_link(&plan.target).unwrap();
        assert!(link.ends_with("caveman/SKILL.md"));
    }

    #[test]
    fn install_refuses_foreign_regular_file() {
        let _lock = ENV_LOCK.lock().unwrap();
        let root = tempdir("foreign");
        let (home, _source, skill) = fake_world(&root, "caveman");
        let _guard = HomeGuard::set(&home);

        // User has their own file sitting at the target path already.
        let target = home
            .join(".claude")
            .join("skills")
            .join("caveman")
            .join("SKILL.md");
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(&target, b"user's own skill").unwrap();

        let adapter = ClaudeCodeAdapter;
        let plan = adapter.plan(&skill).unwrap();
        let err = adapter.install(&plan).expect_err("should refuse");
        match err {
            CliError::Validation(msg) => {
                assert!(msg.contains("not beck-managed"), "msg={msg}");
            }
            other => panic!("expected Validation, got {other:?}"),
        }

        // The foreign file is untouched.
        assert_eq!(fs::read(&target).unwrap(), b"user's own skill");
    }

    #[test]
    fn install_refuses_symlink_pointing_elsewhere() {
        let _lock = ENV_LOCK.lock().unwrap();
        let root = tempdir("retargeted");
        let (home, _source, skill) = fake_world(&root, "caveman");
        let _guard = HomeGuard::set(&home);

        // Something else at the target path: a symlink to a random file.
        let other = root.join("unrelated.md");
        fs::write(&other, b"unrelated").unwrap();
        let target = home
            .join(".claude")
            .join("skills")
            .join("caveman")
            .join("SKILL.md");
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(&other, &target).unwrap();

        let adapter = ClaudeCodeAdapter;
        let plan = adapter.plan(&skill).unwrap();
        let err = adapter.install(&plan).expect_err("should refuse");
        assert!(matches!(err, CliError::Validation(_)));

        // The foreign symlink is untouched.
        let link = fs::read_link(&target).unwrap();
        assert_eq!(link, other);
    }

    #[test]
    fn uninstall_removes_beck_symlink() {
        let _lock = ENV_LOCK.lock().unwrap();
        let root = tempdir("uninstall-ok");
        let (home, _source, skill) = fake_world(&root, "caveman");
        let _guard = HomeGuard::set(&home);

        let adapter = ClaudeCodeAdapter;
        let plan = adapter.plan(&skill).unwrap();
        let entry = adapter.install(&plan).unwrap();

        adapter.uninstall(&entry).unwrap();

        assert!(
            !plan.target.exists(),
            "expected symlink removed at {}",
            plan.target.display()
        );
    }

    #[test]
    fn uninstall_refuses_regular_file() {
        let _lock = ENV_LOCK.lock().unwrap();
        let root = tempdir("uninstall-file");
        let (home, _source, _skill) = fake_world(&root, "caveman");
        let _guard = HomeGuard::set(&home);

        let target = home
            .join(".claude")
            .join("skills")
            .join("caveman")
            .join("SKILL.md");
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(&target, b"real file, not a symlink").unwrap();

        let entry = Entry {
            skill: "caveman".into(),
            agent: AGENT_NAME.into(),
            target: target.clone(),
            mode: InstallMode::Symlink,
            sha256: "deadbeef".into(),
            installed_at: "2026-04-11T02:55:00Z".into(),
        };

        let err = ClaudeCodeAdapter.uninstall(&entry).expect_err("refuse");
        match err {
            CliError::Validation(msg) => assert!(msg.contains("non-symlink")),
            other => panic!("expected Validation, got {other:?}"),
        }
        assert!(target.exists(), "file should not have been deleted");
    }

    #[test]
    fn uninstall_refuses_symlink_pointing_outside_beck() {
        let _lock = ENV_LOCK.lock().unwrap();
        let root = tempdir("uninstall-foreign-link");
        let (home, _source, _skill) = fake_world(&root, "caveman");
        let _guard = HomeGuard::set(&home);

        // Install a symlink that does NOT point into beck/skills.
        let other = root.join("unrelated.md");
        fs::write(&other, b"unrelated").unwrap();
        let target = home
            .join(".claude")
            .join("skills")
            .join("caveman")
            .join("SKILL.md");
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(&other, &target).unwrap();

        let entry = Entry {
            skill: "caveman".into(),
            agent: AGENT_NAME.into(),
            target: target.clone(),
            mode: InstallMode::Symlink,
            sha256: "deadbeef".into(),
            installed_at: "2026-04-11T02:55:00Z".into(),
        };

        let err = ClaudeCodeAdapter.uninstall(&entry).expect_err("refuse");
        assert!(matches!(err, CliError::Validation(_)));
        assert!(target.exists(), "symlink should survive");
    }

    #[test]
    fn uninstall_is_silent_on_missing_target() {
        let _lock = ENV_LOCK.lock().unwrap();
        let root = tempdir("uninstall-missing");
        let home = root.join("home");
        fs::create_dir_all(home.join(".claude").join("skills")).unwrap();
        let _guard = HomeGuard::set(&home);

        let entry = Entry {
            skill: "caveman".into(),
            agent: AGENT_NAME.into(),
            target: home
                .join(".claude")
                .join("skills")
                .join("caveman")
                .join("SKILL.md"),
            mode: InstallMode::Symlink,
            sha256: "deadbeef".into(),
            installed_at: "2026-04-11T02:55:00Z".into(),
        };

        // Missing target is not an error: Phase 5 `check` surfaces the
        // orphan manifest entry instead.
        ClaudeCodeAdapter.uninstall(&entry).unwrap();
    }

    #[test]
    fn uninstall_refuses_entry_from_wrong_agent() {
        let adapter = ClaudeCodeAdapter;
        let entry = Entry {
            skill: "caveman".into(),
            agent: "cursor".into(),
            target: PathBuf::from("/nonexistent"),
            mode: InstallMode::Symlink,
            sha256: "deadbeef".into(),
            installed_at: "2026-04-11T02:55:00Z".into(),
        };
        let err = adapter.uninstall(&entry).expect_err("wrong agent");
        assert!(matches!(err, CliError::Validation(_)));
    }

    #[test]
    fn format_rfc3339_known_epoch() {
        // 0 → 1970-01-01T00:00:00Z
        assert_eq!(format_rfc3339(0), "1970-01-01T00:00:00Z");
        // 1_700_000_000 → 2023-11-14T22:13:20Z
        assert_eq!(
            format_rfc3339(1_700_000_000),
            "2023-11-14T22:13:20Z"
        );
        // 1_800_000_000 → 2027-01-15T08:00:00Z
        assert_eq!(
            format_rfc3339(1_800_000_000),
            "2027-01-15T08:00:00Z"
        );
    }

    #[test]
    fn link_resolves_under_accepts_symlink_into_beck_home() {
        let _lock = ENV_LOCK.lock().unwrap();
        let root = tempdir("link-resolves-ok");
        let skills_root = root.join("beck").join("skills");
        let source_dir = skills_root.join("caveman");
        fs::create_dir_all(&source_dir).unwrap();
        let source = source_dir.join("SKILL.md");
        fs::write(&source, b"body").unwrap();

        let link_parent = root.join("other");
        fs::create_dir_all(&link_parent).unwrap();
        let link = link_parent.join("SKILL.md");
        std::os::unix::fs::symlink(&source, &link).unwrap();

        assert!(link_resolves_under(&link, &skills_root));
    }

    #[test]
    fn link_resolves_under_rejects_unrelated_target() {
        let _lock = ENV_LOCK.lock().unwrap();
        let root = tempdir("link-resolves-no");
        let skills_root = root.join("beck").join("skills");
        fs::create_dir_all(&skills_root).unwrap();

        let other = root.join("unrelated.md");
        fs::write(&other, b"unrelated").unwrap();

        let link_parent = root.join("other");
        fs::create_dir_all(&link_parent).unwrap();
        let link = link_parent.join("SKILL.md");
        std::os::unix::fs::symlink(&other, &link).unwrap();

        assert!(!link_resolves_under(&link, &skills_root));
    }

    #[test]
    fn list_managed_returns_symlinks_that_resolve_into_beck() {
        let _lock = ENV_LOCK.lock().unwrap();
        let root = tempdir("list-managed");
        let (home, _source, skill) = fake_world(&root, "caveman");
        // Override BECK_HOME too so beck_home()?.join("skills") lines
        // up with the fake layout under the fake HOME.
        let previous_beck = std::env::var_os("BECK_HOME");
        unsafe {
            std::env::set_var("BECK_HOME", home.join("beck"));
        }
        let _guard = HomeGuard::set(&home);

        let adapter = ClaudeCodeAdapter;
        let plan = adapter.plan(&skill).unwrap();
        adapter.install(&plan).unwrap();

        let managed = adapter.list_managed().unwrap();
        assert_eq!(managed.len(), 1, "managed={managed:?}");
        assert_eq!(managed[0], plan.target);

        // Restore BECK_HOME.
        unsafe {
            match previous_beck {
                Some(v) => std::env::set_var("BECK_HOME", v),
                None => std::env::remove_var("BECK_HOME"),
            }
        }
    }

    #[test]
    fn ingest_returns_handwritten_skills_and_skips_beck_symlinks() {
        let _lock = ENV_LOCK.lock().unwrap();
        let root = tempdir("ingest");
        let home = root.join("home");
        fs::create_dir_all(home.join(".claude").join("skills")).unwrap();

        // Canonical beck home for the link-skip case.
        let beck = home.join("beck");
        fs::create_dir_all(beck.join("skills").join("compress")).unwrap();
        let beck_source = beck.join("skills").join("compress").join("SKILL.md");
        fs::write(&beck_source, "---\nname: compress\n---\nbody\n").unwrap();

        // Hand-written skill directly under ~/.claude/skills (not a
        // symlink): ingest SHOULD pick this up.
        let handwritten_dir = home.join(".claude").join("skills").join("handwritten");
        fs::create_dir_all(&handwritten_dir).unwrap();
        fs::write(
            handwritten_dir.join("SKILL.md"),
            "---\nname: handwritten\ndescription: user authored\n---\nuser body\n",
        )
        .unwrap();

        // Symlink from ~/.claude/skills/compress/SKILL.md → beck source.
        // ingest SHOULD skip this: it is already beck-managed.
        let claude_compress_dir = home.join(".claude").join("skills").join("compress");
        fs::create_dir_all(&claude_compress_dir).unwrap();
        std::os::unix::fs::symlink(
            &beck_source,
            claude_compress_dir.join("SKILL.md"),
        )
        .unwrap();

        // Override BECK_HOME so the ingest-side `link_resolves_under`
        // check sees our fake beck root.
        let previous_beck = std::env::var_os("BECK_HOME");
        unsafe {
            std::env::set_var("BECK_HOME", &beck);
        }
        let _home_guard = HomeGuard::set(&home);

        let skills = ClaudeCodeAdapter.ingest().unwrap();

        unsafe {
            match previous_beck {
                Some(v) => std::env::set_var("BECK_HOME", v),
                None => std::env::remove_var("BECK_HOME"),
            }
        }

        assert_eq!(
            skills.len(),
            1,
            "expected only the handwritten skill, got {skills:?}"
        );
        assert_eq!(skills[0].name, "handwritten");
        assert!(skills[0].body.contains("user body"));
    }

    #[test]
    fn rebuild_entry_from_disk_produces_matching_sha() {
        let _lock = ENV_LOCK.lock().unwrap();
        let root = tempdir("rebuild-entry");
        let (home, source, skill) = fake_world(&root, "caveman");
        let _guard = HomeGuard::set(&home);

        let adapter = ClaudeCodeAdapter;
        let plan = adapter.plan(&skill).unwrap();
        let original = adapter.install(&plan).unwrap();

        let rebuilt = adapter.rebuild_entry(&plan.target).unwrap();
        assert_eq!(rebuilt.skill, original.skill);
        assert_eq!(rebuilt.agent, original.agent);
        assert_eq!(rebuilt.target, original.target);
        assert_eq!(rebuilt.sha256, original.sha256);

        // Source is still there, target is still a symlink pointing at it.
        assert!(source.exists());
    }
}
