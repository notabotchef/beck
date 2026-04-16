//! Integration test for `beck check` against the real ClaudeCodeAdapter.
//!
//! Lays out a contrived state that exercises every classifier branch:
//!
//! - one beck-managed file (installed via `beck link`)
//! - one foreign file (user-authored, living at the claude target dir)
//! - one orphan manifest entry (manifest points at a path that was
//!   deleted out from under beck)
//!
//! Then runs `beck check --json` and asserts the report shape.
//! Finishes by running `beck check --prune` and confirming the orphan
//! entry is dropped from the manifest.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use beck::agents::manifest::Manifest;

fn unique_root(name: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!(
        "beck-check-e2e-{name}-{}-{}",
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
fn check_classifies_beck_managed_foreign_and_orphan_then_prunes() {
    let root = unique_root("three-state");
    let beck_home = root.join("beck");
    let fake_home = root.join("home");
    let claude_skills = fake_home.join(".claude").join("skills");
    fs::create_dir_all(fake_home.join(".claude")).unwrap();

    // bootstrap
    let out = run(&beck_home, &fake_home, &["bootstrap"]);
    assert_ok("bootstrap", &out);

    // Canonical source for a legitimate skill that will be linked.
    let skill_dir = beck_home.join("skills").join("caveman");
    fs::create_dir_all(&skill_dir).unwrap();
    fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: caveman\ndescription: e2e sample\n---\nbody\n",
    )
    .unwrap();

    // Link it: creates the beck-managed symlink at
    // `<claude>/skills/caveman/SKILL.md`.
    let out = run(&beck_home, &fake_home, &["link", "--agent", "claude-code"]);
    assert_ok("link", &out);
    let linked_target = claude_skills.join("caveman").join("SKILL.md");
    assert!(
        linked_target.exists(),
        "link should create {linked_target:?}"
    );

    // Foreign file: user-authored SKILL.md at a different subdir of
    // the claude skills root. beck did not install it and will not
    // remove it.
    let foreign_dir = claude_skills.join("user-handmade");
    fs::create_dir_all(&foreign_dir).unwrap();
    fs::write(foreign_dir.join("SKILL.md"), b"user's own rule\n").unwrap();

    // Orphan: splice a manifest entry by hand that points at a file
    // that never existed.
    let manifest_file = beck_home.join(".beck-manifest.json");
    let mut manifest = Manifest::load(&manifest_file).unwrap();
    manifest.add(beck::agents::manifest::Entry {
        skill: "ghost".into(),
        agent: "claude-code".into(),
        target: claude_skills.join("ghost").join("SKILL.md"),
        mode: beck::agents::manifest::InstallMode::Symlink,
        sha256: "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".into(),
        installed_at: "2026-04-11T00:00:00Z".into(),
    });
    manifest.save(&manifest_file).unwrap();

    // Run `beck check --json` and parse the report.
    let out = run(&beck_home, &fake_home, &["check", "--json"]);
    assert_ok("check", &out);
    let body = String::from_utf8_lossy(&out.stdout).to_string();
    let report: serde_json::Value = serde_json::from_str(&body).expect("valid json");

    let detected = report["adapters_detected"]
        .as_array()
        .expect("adapters_detected array");
    assert!(
        detected.iter().any(|v| v == "claude-code"),
        "expected claude-code in detected, body={body}"
    );

    assert_eq!(
        report["beck_managed"].as_u64().unwrap(),
        1,
        "expected exactly one beck-managed file"
    );

    let foreign = report["foreign"].as_array().unwrap();
    assert_eq!(foreign.len(), 1);
    assert!(
        foreign[0]["path"]
            .as_str()
            .unwrap()
            .contains("user-handmade"),
        "foreign[0]={:?}",
        foreign[0]
    );

    let orphans = report["orphans"].as_array().unwrap();
    assert_eq!(orphans.len(), 1);
    assert_eq!(orphans[0]["skill"].as_str().unwrap(), "ghost");

    // --prune should remove the orphan entry from the manifest only.
    let out = run(&beck_home, &fake_home, &["check", "--prune"]);
    assert_ok("check --prune", &out);
    let pruned = Manifest::load(&manifest_file).unwrap();
    assert!(
        pruned.entries.iter().all(|e| e.skill != "ghost"),
        "ghost should have been pruned, got entries={:?}",
        pruned.entries
    );
    assert_eq!(
        pruned.entries.len(),
        1,
        "the legitimate caveman entry should still be there"
    );

    // The foreign file is untouched.
    assert!(foreign_dir.join("SKILL.md").exists());

    // Cleanup.
    let _ = fs::remove_dir_all(&root);
}
