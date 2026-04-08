use serde_json::json;

use beck::db::Db;
use beck::error::{CliError, Result};
use beck::paths;
use beck::sync as core_sync;

pub async fn handle(_force: bool, json_out: bool) -> Result<()> {
    let roots = paths::default_roots();
    if roots.is_empty() {
        return Err(CliError::Validation(
            "no skill roots found. Expected ~/.hermes/skills or ~/.claude/skills.".into(),
        ));
    }
    let db_path = paths::db_path()?;
    let db = Db::open(&db_path).map_err(CliError::Other)?;
    db.clear().map_err(CliError::Other)?;

    let mut total = 0usize;
    let mut per_root: Vec<(String, usize)> = Vec::new();
    for root in &roots {
        let n = core_sync::sync_root(&db, root).map_err(CliError::Other)?;
        per_root.push((root.display().to_string(), n));
        total += n;
    }

    if json_out {
        let payload = json!({
            "indexed": total,
            "db": db_path.display().to_string(),
            "roots": per_root.iter().map(|(p, n)| json!({"path": p, "indexed": n})).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&payload).unwrap_or_default());
    } else {
        println!("indexed {total} skills into {}", db_path.display());
        for (path, n) in &per_root {
            println!("  {n:>4}  {path}");
        }
    }
    Ok(())
}
