use serde_json::json;

use beck::db::Db;
use beck::error::{CliError, Result};
use beck::paths;

pub async fn handle(json_out: bool) -> Result<()> {
    let db_path = paths::db_path()?;
    if !db_path.exists() {
        return Err(CliError::Validation(
            "no database found. Run `beck sync` first.".into(),
        ));
    }
    let db = Db::open(&db_path).map_err(CliError::Other)?;
    let mut stmt = db
        .conn
        .prepare("SELECT name, description FROM skills ORDER BY name")?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
    let mut items: Vec<(String, String)> = Vec::new();
    for row in rows {
        items.push(row?);
    }

    if json_out {
        let arr = items
            .iter()
            .map(|(n, d)| json!({"name": n, "description": d}))
            .collect::<Vec<_>>();
        println!("{}", serde_json::to_string_pretty(&arr).unwrap_or_default());
    } else {
        for (name, desc) in &items {
            let first_line = desc.lines().next().unwrap_or("").trim();
            let short = if first_line.len() > 100 {
                format!("{}...", &first_line[..97])
            } else {
                first_line.to_string()
            };
            println!("{name}  {short}");
        }
    }
    Ok(())
}
