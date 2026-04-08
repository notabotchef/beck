use anyhow::Result;

use crate::db::Db;

#[derive(Debug, Clone)]
pub struct Match {
    pub name: String,
    pub description: String,
    pub score: f64,
}

/// BM25 ranked FTS5 search with per-column weights:
/// name 4.0, description 2.0, tags 1.5, body 1.0.
pub fn search(db: &Db, query: &str, top: usize) -> Result<Vec<Match>> {
    let sanitized = sanitize_query(query);
    if sanitized.is_empty() {
        return Ok(Vec::new());
    }
    let sql = "SELECT s.name, s.description, bm25(skills_fts, 4.0, 2.0, 1.5, 1.0) AS score
               FROM skills_fts
               JOIN skills s ON s.id = skills_fts.rowid
               WHERE skills_fts MATCH ?1
               ORDER BY score ASC
               LIMIT ?2";
    let mut stmt = db.conn.prepare(sql)?;
    let rows = stmt.query_map(rusqlite::params![sanitized, top as i64], |r| {
        Ok(Match {
            name: r.get(0)?,
            description: r.get(1)?,
            score: r.get(2)?,
        })
    })?;
    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

/// Break a free-text query into FTS5-safe tokens, OR-joined.
/// Drops punctuation and tokens shorter than 2 chars.
fn sanitize_query(q: &str) -> String {
    let tokens: Vec<String> = q
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() >= 2)
        .map(|t| format!("\"{}\"", t.to_lowercase()))
        .collect();
    tokens.join(" OR ")
}
