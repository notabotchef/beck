//! End-to-end integration test for the Claude Code adapter.
//!
//! Lays out a tempdir that looks like:
//!
//! ```text
//! <tmp>/home/
//!   ├── .claude/               (target agent home)
//!   └── beck/
//!       └── skills/
//!           └── caveman/
//!               └── SKILL.md   (the canonical source)
//! ```
//!
//! Drives the adapter through plan → install → read-through-symlink →
//! uninstall. The test owns `$HOME` for the duration and restores it on
//! drop.
//!
//! Isolated in its own integration file so that parallel unit tests in
//! `src/agents/claude_code.rs` do not race this one for the `HOME` env
//! var. `cargo test` runs each integration test binary sequentially by
//! default, but we still lock on a process-wide mutex because the
//! adapter unit tests and this test both poke `HOME`.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use beck::agents::adapter::Adapter;
use beck::agents::claude_code::ClaudeCodeAdapter;
use beck::agents::manifest::InstallMode;
use beck::agents::skill::Skill;

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
    let base = std::env::temp_dir().join(format!("beck-cc-e2e-{name}"));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    base
}

#[test]
fn install_then_read_through_symlink_then_uninstall() {
    let _lock = ENV_LOCK.lock().unwrap();
    let root = tempdir("round-trip");

    // Build the fake home layout.
    let home = root.join("home");
    let claude = home.join(".claude");
    fs::create_dir_all(&claude).unwrap();

    let skills_home = home.join("beck").join("skills");
    let skill_dir = skills_home.join("caveman");
    fs::create_dir_all(&skill_dir).unwrap();
    let source = skill_dir.join("SKILL.md");
    let body = "---\nname: caveman\ndescription: cc integration sample\n---\ncontent of caveman\n";
    fs::write(&source, body).unwrap();

    let _guard = HomeGuard::set(&home);

    // Discover the skill from the canonical source.
    let skills = Skill::discover_in(&skills_home).unwrap();
    assert_eq!(skills.len(), 1, "expected one skill in fake home");
    let skill = &skills[0];
    assert_eq!(skill.name, "caveman");

    // Plan → install.
    let adapter = ClaudeCodeAdapter::new();
    assert!(adapter.detect(), "detect() should see the .claude dir");

    let plan = adapter.plan(skill).expect("plan");
    let expected_target = claude.join("skills").join("caveman").join("SKILL.md");
    assert_eq!(plan.target, expected_target);
    assert_eq!(plan.mode, InstallMode::Symlink);
    assert!(plan.transform.is_none());

    let entry = adapter.install(&plan).expect("install");
    assert_eq!(entry.skill, "caveman");
    assert_eq!(entry.agent, "claude-code");
    assert_eq!(entry.target, expected_target);
    assert_eq!(entry.mode, InstallMode::Symlink);
    assert_eq!(entry.sha256.len(), 64);
    assert!(entry.installed_at.ends_with('Z'));

    // A real symlink is on disk.
    let meta = fs::symlink_metadata(&expected_target).unwrap();
    assert!(meta.file_type().is_symlink());

    // Reading through the symlink yields the source body.
    let through = fs::read_to_string(&expected_target).unwrap();
    assert_eq!(through, body);

    // install() a second time is a no-op and returns an equivalent entry.
    let entry2 = adapter.install(&plan).expect("idempotent");
    assert_eq!(entry2.skill, entry.skill);
    assert_eq!(entry2.target, entry.target);
    assert_eq!(entry2.sha256, entry.sha256);

    // Uninstall the entry.
    adapter.uninstall(&entry).expect("uninstall");
    assert!(
        !expected_target.exists(),
        "symlink should be removed after uninstall"
    );

    // The canonical source is never touched.
    assert!(source.exists());
    assert_eq!(fs::read_to_string(&source).unwrap(), body);
}
