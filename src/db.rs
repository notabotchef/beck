use std::path::Path;

use anyhow::Result;
use rusqlite::Connection;

pub struct Db {
    pub conn: Connection,
}

impl Db {
    /// Open an in-memory database with the beck schema applied.
    /// Used by the Phase 0 eval harness and unit tests.
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        migrate(&conn)?;
        Ok(Self { conn })
    }

    /// Open or create a persistent SQLite file at `path` with the beck schema.
    /// The parent directory must already exist.
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        migrate(&conn)?;
        Ok(Self { conn })
    }

    /// Number of indexed skills.
    pub fn count(&self) -> Result<i64> {
        let n: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM skills", [], |r| r.get(0))?;
        Ok(n)
    }

    /// Total bytes of description text across all skills.
    /// Used by `beck bench` for the tokens-saved calculation.
    pub fn description_bytes(&self) -> Result<i64> {
        let n: i64 = self.conn.query_row(
            "SELECT COALESCE(SUM(length(description)), 0) FROM skills",
            [],
            |r| r.get(0),
        )?;
        Ok(n)
    }

    /// Total bytes of body text across all skills.
    pub fn body_bytes(&self) -> Result<i64> {
        let n: i64 = self.conn.query_row(
            "SELECT COALESCE(SUM(length(body)), 0) FROM skills",
            [],
            |r| r.get(0),
        )?;
        Ok(n)
    }

    /// Wipe all skills. Called by `beck sync` to rebuild from scratch.
    pub fn clear(&self) -> Result<()> {
        self.conn.execute("DELETE FROM skills", [])?;
        Ok(())
    }
}

fn migrate(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS skills (
            id           INTEGER PRIMARY KEY,
            name         TEXT NOT NULL,
            path         TEXT NOT NULL UNIQUE,
            description  TEXT NOT NULL DEFAULT '',
            tags         TEXT NOT NULL DEFAULT '',
            body         TEXT NOT NULL DEFAULT ''
        );
        CREATE UNIQUE INDEX IF NOT EXISTS skills_name_unique ON skills(name);

        CREATE VIRTUAL TABLE IF NOT EXISTS skills_fts USING fts5(
            name, description, tags, body,
            content='skills',
            content_rowid='id',
            tokenize='unicode61 remove_diacritics 2'
        );

        CREATE TRIGGER IF NOT EXISTS skills_ai AFTER INSERT ON skills BEGIN
            INSERT INTO skills_fts(rowid, name, description, tags, body)
            VALUES (new.id, new.name, new.description, new.tags, new.body);
        END;
        CREATE TRIGGER IF NOT EXISTS skills_ad AFTER DELETE ON skills BEGIN
            INSERT INTO skills_fts(skills_fts, rowid, name, description, tags, body)
            VALUES('delete', old.id, old.name, old.description, old.tags, old.body);
        END;
        CREATE TRIGGER IF NOT EXISTS skills_au AFTER UPDATE ON skills BEGIN
            INSERT INTO skills_fts(skills_fts, rowid, name, description, tags, body)
            VALUES('delete', old.id, old.name, old.description, old.tags, old.body);
            INSERT INTO skills_fts(rowid, name, description, tags, body)
            VALUES (new.id, new.name, new.description, new.tags, new.body);
        END;
        "#,
    )?;
    Ok(())
}
