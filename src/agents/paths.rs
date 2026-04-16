//! Skills-home path resolution for the `beck-link` layer.
//!
//! These are DIFFERENT from `crate::paths`, which resolves the XDG data dir
//! for the SQLite index. This module resolves the user-visible `~/beck/`
//! directory that holds canonical SKILL.md sources and the manifest file.
//!
//! Precedence:
//!   1. `$BECK_HOME` environment variable (tests and power users)
//!   2. `$HOME/beck` via `dirs::home_dir()`
//!
//! All three accessors return a `CliError::Validation` if neither source
//! resolves. Nothing panics, nothing touches disk.

use std::path::PathBuf;

use crate::error::{CliError, Result};

const ENV_OVERRIDE: &str = "BECK_HOME";
const HOME_SUBDIR: &str = "beck";
const SKILLS_SUBDIR: &str = "skills";
const MANIFEST_FILE: &str = ".beck-manifest.json";

/// Root of the user's beck home. Honors `$BECK_HOME`, otherwise `$HOME/beck`.
pub fn beck_home() -> Result<PathBuf> {
    if let Some(raw) = std::env::var_os(ENV_OVERRIDE) {
        let path = PathBuf::from(raw);
        if path.as_os_str().is_empty() {
            return Err(CliError::Validation(format!(
                "{ENV_OVERRIDE} is set but empty"
            )));
        }
        return Ok(path);
    }

    let home = dirs::home_dir()
        .ok_or_else(|| CliError::Validation("could not resolve home dir (HOME not set)".into()))?;
    Ok(home.join(HOME_SUBDIR))
}

/// Directory where canonical SKILL.md sources live: `<beck_home>/skills/`.
pub fn skills_home() -> Result<PathBuf> {
    Ok(beck_home()?.join(SKILLS_SUBDIR))
}

/// Path to the manifest file: `<beck_home>/.beck-manifest.json`.
pub fn manifest_path() -> Result<PathBuf> {
    Ok(beck_home()?.join(MANIFEST_FILE))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Env vars are process-global. Serialize tests that touch BECK_HOME so
    // they do not race each other under `cargo test` default parallelism.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    struct EnvGuard {
        previous: Option<std::ffi::OsString>,
    }

    impl EnvGuard {
        fn set(value: &str) -> Self {
            let previous = std::env::var_os(ENV_OVERRIDE);
            // Safety: this is a test helper. We hold ENV_LOCK for the
            // duration of any test that calls this, so no other thread in
            // this process touches BECK_HOME concurrently.
            unsafe { std::env::set_var(ENV_OVERRIDE, value) };
            Self { previous }
        }

        fn unset() -> Self {
            let previous = std::env::var_os(ENV_OVERRIDE);
            unsafe { std::env::remove_var(ENV_OVERRIDE) };
            Self { previous }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.previous {
                Some(v) => unsafe { std::env::set_var(ENV_OVERRIDE, v) },
                None => unsafe { std::env::remove_var(ENV_OVERRIDE) },
            }
        }
    }

    #[test]
    fn beck_home_honors_env_override() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvGuard::set("/tmp/beck-paths-test-override");

        let home = beck_home().expect("beck_home should resolve with override");
        assert_eq!(home, PathBuf::from("/tmp/beck-paths-test-override"));
    }

    #[test]
    fn skills_home_is_subdir_of_beck_home() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvGuard::set("/tmp/beck-paths-test-skills");

        let skills = skills_home().expect("skills_home should resolve");
        assert_eq!(skills, PathBuf::from("/tmp/beck-paths-test-skills/skills"));
    }

    #[test]
    fn manifest_path_lives_inside_beck_home() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvGuard::set("/tmp/beck-paths-test-manifest");

        let manifest = manifest_path().expect("manifest_path should resolve");
        assert_eq!(
            manifest,
            PathBuf::from("/tmp/beck-paths-test-manifest/.beck-manifest.json")
        );
    }

    #[test]
    fn empty_env_override_is_rejected() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvGuard::set("");

        let err = beck_home().expect_err("empty override should error");
        match err {
            CliError::Validation(_) => {}
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn fallback_to_home_dir_when_override_absent() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvGuard::unset();

        // This test only runs meaningfully when HOME is set. CI always has
        // HOME set, so we expect success.
        if dirs::home_dir().is_some() {
            let home = beck_home().expect("beck_home should fall back to HOME");
            assert!(
                home.ends_with("beck"),
                "expected path ending in 'beck', got {home:?}"
            );
        }
    }
}
