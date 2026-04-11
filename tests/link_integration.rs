//! Integration test for `beck link` and `beck unlink` against a real
//! ClaudeCodeAdapter.
//!
//! Spawns the compiled binary under an isolated `$BECK_HOME` and `$HOME`,
//! writes a fake SKILL.md, runs `beck bootstrap`, then `beck link
//! --agent claude-code`, verifies the symlink exists at the fake
//! `~/.claude/skills/...` path, runs `beck link` a second time (must be
//! idempotent), then `beck unlink --all` and verifies the symlink is
//! gone.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use beck::agents::manifest::Manifest;

fn unique_root(name: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!(
        "beck-link-e2e-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    base
}

fn beck_bin() -> &'static str {
    env!("CARGO_BIN_EXE_beck")
}

fn run(beck_home: &Path, fake_home: &Path, args: &[&str]) -> std::process::Output {
    Command::new(beck_bin())
        .args(args)
        .env("BECK_HOME", beck_home)
        .env("HOME", fake_home)
        .output()
        .expect("failed to spawn beck")
}

fn assert_ok(label: &str, out: &std::process::Output) {
    assert!(
        out.status.success(),
        "{label} failed (exit {:?}): stdout={} stderr={}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn bootstrap_link_unlink_round_trip_against_claude_code_adapter() {
    let root = unique_root("round-trip");
    let beck_home = root.join("beck");
    let fake_home = root.join("home");
    let claude_skills = fake_home.join(".claude").join("skills");
    fs::create_dir_all(fake_home.join(".claude")).unwrap();

    // bootstrap creates <beck_home>/skills and the manifest.
    let out = run(&beck_home, &fake_home, &["bootstrap"]);
    assert_ok("bootstrap", &out);

    let skills_dir = beck_home.join("skills");
    assert!(skills_dir.is_dir());
    let manifest_path = beck_home.join(".beck-manifest.json");
    assert!(manifest_path.is_file());

    // Drop a canonical skill under `<beck_home>/skills/caveman/SKILL.md`.
    let skill_dir = skills_dir.join("caveman");
    fs::create_dir_all(&skill_dir).unwrap();
    let source = skill_dir.join("SKILL.md");
    fs::write(
        &source,
        "---\nname: caveman\ndescription: e2e integration sample\n---\nhello from beck link\n",
    )
    .unwrap();

    // Dry-run should write nothing to the claude target.
    let dry = run(
        &beck_home,
        &fake_home,
        &["link", "--agent", "claude-code", "--dry-run"],
    );
    assert_ok("link --dry-run", &dry);
    let expected_target = claude_skills.join("caveman").join("SKILL.md");
    assert!(
        !expected_target.exists(),
        "dry-run must not create {expected_target:?}"
    );

    // First real link.
    let first = run(&beck_home, &fake_home, &["link", "--agent", "claude-code"]);
    assert_ok("link first", &first);

    let meta = fs::symlink_metadata(&expected_target).expect("symlink exists");
    assert!(meta.file_type().is_symlink(), "target must be a symlink");
    let link_target = fs::read_link(&expected_target).unwrap();
    assert_eq!(link_target, source);

    // Reading through the symlink gets us the canonical source body.
    let through = fs::read_to_string(&expected_target).unwrap();
    assert!(through.contains("hello from beck link"));

    // Manifest has exactly one entry.
    let mf = Manifest::load(&manifest_path).unwrap();
    assert_eq!(mf.entries.len(), 1);
    assert_eq!(mf.entries[0].skill, "caveman");
    assert_eq!(mf.entries[0].agent, "claude-code");

    // Second link is idempotent: manifest entry count stays at one.
    let second = run(&beck_home, &fake_home, &["link", "--agent", "claude-code"]);
    assert_ok("link second", &second);
    let mf2 = Manifest::load(&manifest_path).unwrap();
    assert_eq!(mf2.entries.len(), 1);

    // Human output on second run mentions "skipped" since the source
    // sha256 did not change.
    let stdout = String::from_utf8_lossy(&second.stdout);
    assert!(
        stdout.contains("skipped"),
        "expected 'skipped' in stdout, got: {stdout}"
    );

    // JSON output sanity check on a link call.
    let json_out = run(
        &beck_home,
        &fake_home,
        &["link", "--agent", "claude-code", "--json"],
    );
    assert_ok("link --json", &json_out);
    let body = String::from_utf8_lossy(&json_out.stdout);
    assert!(body.contains("\"dry_run\""));

    // Unlink everything.
    let unlink = run(&beck_home, &fake_home, &["unlink", "--all"]);
    assert_ok("unlink --all", &unlink);

    assert!(
        !expected_target.exists(),
        "symlink should be removed after unlink --all"
    );

    // Manifest is empty.
    let mf3 = Manifest::load(&manifest_path).unwrap();
    assert!(mf3.entries.is_empty());

    // Canonical source is untouched.
    assert!(source.exists());
    let unchanged = fs::read_to_string(&source).unwrap();
    assert!(unchanged.contains("hello from beck link"));

    // Cleanup.
    let _ = fs::remove_dir_all(&root);
}
