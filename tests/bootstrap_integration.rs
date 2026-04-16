//! Integration test for `beck bootstrap`.
//!
//! Spawns the compiled `beck` binary under an isolated `$BECK_HOME`, asserts
//! the skills directory and manifest file are created, then runs it a
//! second time and confirms the manifest is untouched (idempotent).

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use beck::agents::manifest::{Manifest, SCHEMA_VERSION};

fn isolated_home() -> PathBuf {
    let base = std::env::temp_dir().join(format!(
        "beck-phase1-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let _ = fs::remove_dir_all(&base);
    base
}

fn run_bootstrap(home: &PathBuf) -> std::process::Output {
    let exe = env!("CARGO_BIN_EXE_beck");
    Command::new(exe)
        .arg("bootstrap")
        .env("BECK_HOME", home)
        .output()
        .expect("failed to spawn beck")
}

#[test]
fn bootstrap_creates_skills_dir_and_manifest_then_idempotent() {
    let home = isolated_home();

    // First invocation: creates everything.
    let out = run_bootstrap(&home);
    assert!(
        out.status.success(),
        "first bootstrap failed: stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let skills_dir = home.join("skills");
    let manifest_file = home.join(".beck-manifest.json");

    assert!(
        skills_dir.is_dir(),
        "skills home should exist at {skills_dir:?}"
    );
    assert!(
        manifest_file.is_file(),
        "manifest should exist at {manifest_file:?}"
    );

    // Manifest is valid JSON with schema_version: 1, entries: [].
    let loaded = Manifest::load(&manifest_file).expect("manifest should load");
    assert_eq!(loaded.schema_version, SCHEMA_VERSION);
    assert!(loaded.entries.is_empty());

    // Capture bytes for the idempotence check.
    let first_bytes = fs::read(&manifest_file).expect("read manifest");

    // Second invocation: must not rewrite the file.
    let out2 = run_bootstrap(&home);
    assert!(
        out2.status.success(),
        "second bootstrap failed: stdout={} stderr={}",
        String::from_utf8_lossy(&out2.stdout),
        String::from_utf8_lossy(&out2.stderr)
    );

    let second_bytes = fs::read(&manifest_file).expect("re-read manifest");
    assert_eq!(
        first_bytes, second_bytes,
        "second bootstrap must leave the existing manifest untouched"
    );

    // Cleanup.
    let _ = fs::remove_dir_all(&home);
}
