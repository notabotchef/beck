use serde_json::json;

use beck::db::Db;
use beck::error::{CliError, Result};
use beck::paths;

pub async fn handle(name: &str, json_out: bool) -> Result<()> {
    let db_path = paths::db_path()?;
    if !db_path.exists() {
        return Err(CliError::Validation(
            "no database found. Run `beck sync` first.".into(),
        ));
    }
    let db = Db::open(&db_path).map_err(CliError::Other)?;
    let row: std::result::Result<(String, String, String), rusqlite::Error> = db.conn.query_row(
        "SELECT name, path, body FROM skills WHERE name = ?1",
        [name],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    );
    match row {
        Ok((name, path, body)) => {
            if json_out {
                let payload = json!({
                    "name": name,
                    "path": path,
                    "body": body,
                });
                println!(
                    "{}",
                    serde_json::to_string_pretty(&payload).unwrap_or_default()
                );
            } else {
                print!("{body}");
            }
            Ok(())
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            Err(CliError::NotFound(format!("skill not found: {name}")))
        }
        Err(e) => Err(CliError::Db(e)),
    }
}
