use serde_json::json;

use beck::db::Db;
use beck::error::{CliError, Result};
use beck::paths;
use beck::query as core_query;

pub async fn handle(text: &str, top: usize, json_out: bool) -> Result<()> {
    if text.trim().is_empty() {
        return Err(CliError::Validation("query text is empty".into()));
    }
    let db_path = paths::db_path()?;
    if !db_path.exists() {
        return Err(CliError::Validation(
            "no database found. Run `beck sync` first.".into(),
        ));
    }
    let db = Db::open(&db_path).map_err(CliError::Other)?;
    let matches = core_query::search(&db, text, top).map_err(CliError::Other)?;

    if json_out {
        let arr = matches
            .iter()
            .map(|m| {
                json!({
                    "name": m.name,
                    "description": m.description,
                    "score": m.score,
                })
            })
            .collect::<Vec<_>>();
        println!("{}", serde_json::to_string_pretty(&arr).unwrap_or_default());
    } else if matches.is_empty() {
        println!("no matches");
    } else {
        for m in &matches {
            let first = m.description.lines().next().unwrap_or("").trim();
            let short = if first.len() > 100 {
                format!("{}...", &first[..97])
            } else {
                first.to_string()
            };
            println!("{}\n  {}", m.name, short);
        }
    }
    Ok(())
}
