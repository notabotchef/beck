use anyhow::{Context, Result};
use std::collections::HashSet;
use std::path::Path;
use walkdir::WalkDir;

use crate::db::Db;
use crate::frontmatter;

/// Walk a root directory, parse every SKILL.md, upsert into the db.
/// Last-wins on duplicate name, matching the beck duplicate policy.
pub fn sync_root(db: &Db, root: &Path) -> Result<usize> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut count = 0usize;
    for entry in WalkDir::new(root).follow_links(false).into_iter().flatten() {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.file_name() != "SKILL.md" {
            continue;
        }
        let path = entry.path();
        let contents = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let (fm, body) = frontmatter::parse(&contents);
        let parent_dir = path
            .parent()
            .and_then(|p| p.file_name())
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let name = fm
            .name
            .clone()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(parent_dir);
        if name.is_empty() {
            continue;
        }
        let description = fm.description.clone().unwrap_or_default();
        let tags = fm.tags.clone().map(|v| v.join(" ")).unwrap_or_default();
        let path_str = path.to_string_lossy().to_string();

        // Last-wins on duplicate name: remove prior row with same name, insert fresh.
        if !seen.insert(name.clone()) {
            db.conn
                .execute("DELETE FROM skills WHERE name = ?1", [&name])
                .context("delete prior dup")?;
        } else {
            // Also defend against a crash-and-reopen: drop any pre-existing row.
            db.conn
                .execute("DELETE FROM skills WHERE name = ?1", [&name])
                .ok();
        }
        db.conn.execute(
            "INSERT INTO skills (name, path, description, tags, body) VALUES (?1, ?2, ?3, ?4, ?5)",
            (&name, &path_str, &description, &tags, &body),
        )?;
        count += 1;
    }
    Ok(count)
}
