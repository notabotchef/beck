use anyhow::Result;
use rusqlite::Connection;

pub struct Db {
    pub conn: Connection,
}

impl Db {
    /// Open an in-memory database with the beck schema applied.
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        migrate(&conn)?;
        Ok(Self { conn })
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
