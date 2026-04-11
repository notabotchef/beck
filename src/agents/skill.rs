//! The canonical in-memory view of a SKILL.md file living under
//! `~/beck/skills/<name>/SKILL.md`.
//!
//! A `Skill` is the input to every adapter. Adapters translate it into a
//! file another agent reads. For the Claude Code adapter this translation
//! is the identity: the source file is symlinked byte-for-byte.
//!
//! Contract is locked in `.rune/plan-beck-link-spec.md` §1. Do not add
//! fields without bumping the spec.

use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::error::{CliError, Result};
use crate::frontmatter::{self, Frontmatter};

/// A SKILL.md loaded from disk: name, source path, parsed frontmatter,
/// body, and a content hash of the raw bytes. The hash is used by the
/// manifest to detect drift between install time and now.
#[derive(Debug, Clone)]
pub struct Skill {
    /// Folder name under `<skills_home>/`. Matches the containing dir, not
    /// the `name:` field inside the frontmatter.
    pub name: String,
    /// Absolute path to the on-disk SKILL.md.
    pub source_path: PathBuf,
    /// Parsed YAML frontmatter. Empty if the file has none or fails parse.
    pub frontmatter: Frontmatter,
    /// Markdown body after the closing `---` fence.
    pub body: String,
    /// Lowercase hex sha256 of the full raw file bytes.
    pub sha256: String,
}

impl Skill {
    /// Load a `Skill` from an absolute SKILL.md path.
    ///
    /// The skill name is taken from the parent directory name. That means
    /// `<skills_home>/caveman/SKILL.md` produces `name = "caveman"`, matching
    /// the folder convention Claude Code already uses at
    /// `~/.claude/skills/caveman/SKILL.md`.
    ///
    /// Errors:
    /// - `CliError::Io` if the file is missing or unreadable.
    /// - `CliError::Validation` if the path has no parent directory (e.g.
    ///   passing `/SKILL.md` directly).
    pub fn from_path(path: &Path) -> Result<Self> {
        let bytes = fs::read(path)?;
        let contents = String::from_utf8_lossy(&bytes).into_owned();
        let (frontmatter, body) = frontmatter::parse(&contents);

        let name = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .ok_or_else(|| {
                CliError::Validation(format!(
                    "cannot derive skill name from path: {}",
                    path.display()
                ))
            })?
            .to_string();

        Ok(Self {
            name,
            source_path: path.to_path_buf(),
            frontmatter,
            body,
            sha256: sha256_hex(&bytes),
        })
    }

    /// Walk `skills_home` and return every `SKILL.md` found one level deep.
    ///
    /// The layout is: `<skills_home>/<name>/SKILL.md`. Files at other
    /// depths are ignored. Nested namespaces like `gstack/benchmark` are a
    /// v0.3 concern and are deliberately not traversed here.
    pub fn discover_in(skills_home: &Path) -> Result<Vec<Self>> {
        let mut out = Vec::new();

        if !skills_home.exists() {
            return Ok(out);
        }

        let entries = fs::read_dir(skills_home)?;
        for entry in entries {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if !file_type.is_dir() {
                continue;
            }
            let skill_md = entry.path().join("SKILL.md");
            if skill_md.is_file() {
                out.push(Self::from_path(&skill_md)?);
            }
        }

        out.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(out)
    }
}

/// Lowercase hex encoding of `Sha256::digest(bytes)`. The hex crate would
/// add ~3KB for no reason, so write the formatter inline.
fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest.iter() {
        // SAFETY: `write!` into a String never fails.
        use std::fmt::Write;
        let _ = write!(out, "{byte:02x}");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tempdir(name: &str) -> PathBuf {
        let base = std::env::temp_dir().join(format!("beck-skill-tests-{name}"));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        base
    }

    fn write_skill(root: &Path, name: &str, contents: &str) -> PathBuf {
        let dir = root.join(name);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("SKILL.md");
        fs::write(&path, contents).unwrap();
        path
    }

    const SAMPLE: &str = "---\n\
name: caveman\n\
description: ultra compressed communication\n\
tags: [compression, style]\n\
---\n\
body goes here\n";

    #[test]
    fn from_path_parses_fixture() {
        let dir = tempdir("from-path");
        let path = write_skill(&dir, "caveman", SAMPLE);

        let skill = Skill::from_path(&path).expect("load");
        assert_eq!(skill.name, "caveman");
        assert_eq!(skill.source_path, path);
        assert_eq!(
            skill.frontmatter.description.as_deref(),
            Some("ultra compressed communication")
        );
        assert_eq!(
            skill.frontmatter.tags.as_ref().map(|v| v.len()),
            Some(2)
        );
        assert!(skill.body.contains("body goes here"));
        // sha256 is 64 hex chars, all lowercase.
        assert_eq!(skill.sha256.len(), 64);
        assert!(
            skill.sha256.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        );
    }

    #[test]
    fn from_path_tolerates_missing_frontmatter() {
        let dir = tempdir("no-fm");
        let path = write_skill(&dir, "plain", "just a body\n");

        let skill = Skill::from_path(&path).expect("load");
        assert_eq!(skill.name, "plain");
        assert!(skill.frontmatter.description.is_none());
        assert!(skill.frontmatter.tags.is_none());
        assert_eq!(skill.body, "just a body\n");
    }

    #[test]
    fn from_path_hashes_raw_bytes_deterministically() {
        let dir = tempdir("hash");
        let path_a = write_skill(&dir, "a", "same contents\n");
        let path_b = write_skill(&dir, "b", "same contents\n");

        let a = Skill::from_path(&path_a).unwrap();
        let b = Skill::from_path(&path_b).unwrap();
        assert_eq!(a.sha256, b.sha256);

        // Different bytes → different hash.
        let path_c = write_skill(&dir, "c", "different contents\n");
        let c = Skill::from_path(&path_c).unwrap();
        assert_ne!(a.sha256, c.sha256);
    }

    #[test]
    fn from_path_errors_on_missing_file() {
        let dir = tempdir("missing");
        let path = dir.join("nope").join("SKILL.md");
        let err = Skill::from_path(&path).expect_err("should fail");
        assert!(matches!(err, CliError::Io(_)));
    }

    #[test]
    fn discover_in_finds_top_level_skills() {
        let dir = tempdir("discover");
        write_skill(&dir, "alpha", "a\n");
        write_skill(&dir, "beta", "b\n");
        write_skill(&dir, "gamma", SAMPLE);

        // Stray file at the top level is ignored.
        fs::write(dir.join("README.md"), b"not a skill").unwrap();

        // A subdir without a SKILL.md is ignored.
        fs::create_dir_all(dir.join("empty-dir")).unwrap();

        let skills = Skill::discover_in(&dir).unwrap();
        assert_eq!(skills.len(), 3);

        // Sorted by name.
        assert_eq!(skills[0].name, "alpha");
        assert_eq!(skills[1].name, "beta");
        assert_eq!(skills[2].name, "gamma");
    }

    #[test]
    fn discover_in_handles_missing_home() {
        let dir = tempdir("missing-home");
        let nonexistent = dir.join("not-created");
        let skills = Skill::discover_in(&nonexistent).unwrap();
        assert!(skills.is_empty());
    }

    #[test]
    fn sha256_hex_is_lowercase_and_64_chars() {
        let out = sha256_hex(b"hello");
        assert_eq!(out.len(), 64);
        assert!(out.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
        // Known vector: sha256("hello") =
        // 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824
        assert_eq!(
            out,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }
}
