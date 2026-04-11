//! Round-trip integration test: the hardest correctness signal.
//!
//! Flow:
//! 1. Create a canonical skill at `<beck_home>/skills/caveman/SKILL.md`.
//! 2. `beck link --agent claude-code` installs a symlink at
//!    `~/.claude/skills/caveman/SKILL.md`.
//! 3. Simulate data loss: delete the canonical source while the
//!    symlink still points to the now-dangling path.
//! 4. Physically replace the symlink with a copy of the original bytes
//!    (as if the user had typed the file back by hand into
//!    `~/.claude/skills/caveman/SKILL.md`).
//! 5. `beck sync --from claude-code --write` must bring the file back
//!    into `<beck_home>/skills/caveman/SKILL.md` with the exact same
//!    bytes the user saw in the source before step 3.
//!
//! The invariant this test pins is: if you lose `~/beck/skills/` but
//! your Claude Code dir still has a valid SKILL.md (beck-managed or
//! not), ingest can rebuild the canonical tree from it.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn unique_root(name: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!(
        "beck-round-trip-{name}-{}-{}",
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
fn link_wipe_ingest_restores_canonical_body_bytewise() {
    let root = unique_root("link-wipe-ingest");
    let beck_home = root.join("beck");
    let fake_home = root.join("home");
    let claude_skills = fake_home.join(".claude").join("skills");
    fs::create_dir_all(fake_home.join(".claude")).unwrap();

    // Step 1: bootstrap + canonical source.
    let out = run(&beck_home, &fake_home, &["bootstrap"]);
    assert_ok("bootstrap", &out);

    let skill_dir = beck_home.join("skills").join("caveman");
    fs::create_dir_all(&skill_dir).unwrap();
    let body = "---\nname: caveman\ndescription: round trip source\n---\ncave body\n";
    fs::write(skill_dir.join("SKILL.md"), body).unwrap();

    // Step 2: link it into the fake claude dir.
    let out = run(&beck_home, &fake_home, &["link", "--agent", "claude-code"]);
    assert_ok("link", &out);
    let claude_target = claude_skills.join("caveman").join("SKILL.md");
    assert!(claude_target.exists());

    // Step 3 + 4: wipe the canonical source and replace the symlink
    // with a real file carrying the original bytes. This simulates
    // the user having lost `~/beck/skills/` but still having their
    // `~/.claude/skills/caveman/SKILL.md` around.
    fs::remove_file(&claude_target).unwrap();
    fs::remove_file(skill_dir.join("SKILL.md")).unwrap();
    fs::remove_dir(&skill_dir).unwrap();
    fs::write(&claude_target, body).unwrap();

    // Step 5: ingest back into `<beck_home>/skills/`.
    let out = run(
        &beck_home,
        &fake_home,
        &["sync", "--from", "claude-code", "--write"],
    );
    assert_ok("sync --from claude-code --write", &out);

    let restored = beck_home.join("skills").join("caveman").join("SKILL.md");
    assert!(restored.exists(), "ingest should restore canonical source");
    let restored_bytes = fs::read(&restored).unwrap();
    assert_eq!(
        restored_bytes,
        body.as_bytes(),
        "restored canonical body must be byte-identical to the original"
    );

    // Cleanup.
    let _ = fs::remove_dir_all(&root);
}
