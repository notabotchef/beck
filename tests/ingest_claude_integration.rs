//! Integration test for `beck sync --from claude-code`.
//!
//! Lays out a fake HOME with two hand-written skills at
//! `~/.claude/skills/<name>/SKILL.md` and one existing beck symlink.
//! Runs the binary in dry-run, confirms it lists Create plans but
//! writes nothing. Runs again with `--write`, confirms the canonical
//! tree at `~/beck/skills/` now has byte-identical copies.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn unique_root(name: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!(
        "beck-ingest-e2e-{name}-{}-{}",
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

fn run(beck_home: &PathBuf, fake_home: &PathBuf, args: &[&str]) -> std::process::Output {
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
fn reverse_ingest_from_claude_code_dry_run_then_write() {
    let root = unique_root("claude-ingest");
    let beck_home = root.join("beck");
    let fake_home = root.join("home");
    let claude_skills = fake_home.join(".claude").join("skills");
    fs::create_dir_all(&claude_skills).unwrap();

    // bootstrap: creates <beck_home>/skills/ and the empty manifest.
    let out = run(&beck_home, &fake_home, &["bootstrap"]);
    assert_ok("bootstrap", &out);

    // Two hand-written skills living under ~/.claude/skills.
    let alpha_dir = claude_skills.join("alpha");
    fs::create_dir_all(&alpha_dir).unwrap();
    fs::write(
        alpha_dir.join("SKILL.md"),
        "---\nname: alpha\ndescription: hand-written\n---\nalpha body\n",
    )
    .unwrap();

    let beta_dir = claude_skills.join("beta");
    fs::create_dir_all(&beta_dir).unwrap();
    fs::write(
        beta_dir.join("SKILL.md"),
        "---\nname: beta\ndescription: also hand-written\n---\nbeta body\n",
    )
    .unwrap();

    // Dry-run first.
    let dry = run(
        &beck_home,
        &fake_home,
        &["sync", "--from", "claude-code", "--json"],
    );
    assert_ok("sync --from claude-code (dry-run)", &dry);
    let body = String::from_utf8_lossy(&dry.stdout);
    let report: serde_json::Value =
        serde_json::from_str(&body).expect("valid json on dry-run");
    assert_eq!(report["agent"], "claude-code");
    assert_eq!(report["written"], false);
    assert_eq!(report["created"].as_u64().unwrap(), 2);

    // Nothing should have been created on disk yet.
    assert!(!beck_home.join("skills").join("alpha").join("SKILL.md").exists());
    assert!(!beck_home.join("skills").join("beta").join("SKILL.md").exists());

    // Real write.
    let written = run(
        &beck_home,
        &fake_home,
        &["sync", "--from", "claude-code", "--write"],
    );
    assert_ok("sync --from claude-code --write", &written);

    let alpha_target = beck_home.join("skills").join("alpha").join("SKILL.md");
    let beta_target = beck_home.join("skills").join("beta").join("SKILL.md");
    assert!(alpha_target.exists());
    assert!(beta_target.exists());
    assert_eq!(
        fs::read_to_string(&alpha_target).unwrap(),
        fs::read_to_string(claude_skills.join("alpha").join("SKILL.md")).unwrap(),
    );
    assert_eq!(
        fs::read_to_string(&beta_target).unwrap(),
        fs::read_to_string(claude_skills.join("beta").join("SKILL.md")).unwrap(),
    );

    // Running the same write a second time is idempotent: both plans
    // become Skip because the target already matches.
    let again = run(
        &beck_home,
        &fake_home,
        &["sync", "--from", "claude-code", "--write", "--json"],
    );
    assert_ok("sync --write second run", &again);
    let body2 = String::from_utf8_lossy(&again.stdout);
    let report2: serde_json::Value = serde_json::from_str(&body2).unwrap();
    assert_eq!(report2["created"].as_u64().unwrap(), 0);
    assert_eq!(report2["skipped"].as_u64().unwrap(), 2);

    // Cleanup.
    let _ = fs::remove_dir_all(&root);
}
