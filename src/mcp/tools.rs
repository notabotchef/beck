//! MCP tool router. Two tools exposed to any MCP client:
//!
//! - `skills_query` — free-text search, returns ranked matches
//! - `skills_load`  — exact-name lookup, returns the full skill body
//!
//! Each tool opens a fresh rusqlite connection per call. rusqlite
//! connections are not Send + Sync, and the overhead of reopening
//! (~1 ms on an SSD) is negligible next to the MCP wire roundtrip.
//!
//! Resources are intentionally NOT exposed. See TODOS.md erratum 1:
//! a resources/list that returns every indexed skill would cost a
//! 300-skill power user ~27k tokens at session start, which defeats
//! the whole point of beck.

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::{tool, tool_router};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::db::Db;
use crate::paths;
use crate::query as core_query;

#[derive(Clone)]
pub struct BeckServer {
    pub tool_router: ToolRouter<Self>,
}

impl BeckServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
}

fn default_top() -> usize {
    3
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct QueryParams {
    /// Free-text query describing the task you need a skill for.
    pub query: String,
    /// Maximum number of results to return. Default 3.
    #[serde(default = "default_top")]
    pub top: usize,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct LoadParams {
    /// Exact skill name. Use skills_query first to find candidates.
    pub name: String,
}

#[tool_router]
impl BeckServer {
    /// Search indexed skills by free-text query.
    #[tool(
        name = "skills_query",
        description = "Search locally indexed skills by free-text query. Returns up to `top` ranked matches, each with name, short description, and BM25 score. Always call this first when you need a skill but do not know its exact name, then call skills_load with the chosen name to fetch the full body."
    )]
    async fn tool_query(&self, Parameters(params): Parameters<QueryParams>) -> String {
        let result = (|| -> anyhow::Result<String> {
            let db_path = paths::db_path().map_err(|e| anyhow::anyhow!("{e}"))?;
            if !db_path.exists() {
                return Ok(json!({
                    "error": "beck has not been synced yet. Run `beck sync` from a shell to index skills."
                })
                .to_string());
            }
            let db = Db::open(&db_path)?;
            let matches = core_query::search(&db, &params.query, params.top)?;
            let arr: Vec<_> = matches
                .iter()
                .map(|m| {
                    json!({
                        "name": m.name,
                        "description": m.description,
                        "score": m.score,
                    })
                })
                .collect();
            Ok(serde_json::to_string(&json!({ "matches": arr }))
                .unwrap_or_else(|_| "[]".to_string()))
        })();
        match result {
            Ok(s) => s,
            Err(e) => json!({ "error": format!("{e}") }).to_string(),
        }
    }

    /// Load the full body of a skill by exact name.
    #[tool(
        name = "skills_load",
        description = "Load the full markdown body of a skill by exact name. Use skills_query first to find candidate names. Returns {name, path, body} as JSON. Returns {error} if no skill matches the given name."
    )]
    async fn tool_load(&self, Parameters(params): Parameters<LoadParams>) -> String {
        let result = (|| -> anyhow::Result<String> {
            let db_path = paths::db_path().map_err(|e| anyhow::anyhow!("{e}"))?;
            if !db_path.exists() {
                return Ok(json!({
                    "error": "beck has not been synced yet. Run `beck sync` from a shell to index skills."
                })
                .to_string());
            }
            let db = Db::open(&db_path)?;
            let row: std::result::Result<(String, String, String), rusqlite::Error> =
                db.conn.query_row(
                    "SELECT name, path, body FROM skills WHERE name = ?1",
                    [&params.name],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
                );
            match row {
                Ok((name, path, body)) => Ok(json!({
                    "name": name,
                    "path": path,
                    "body": body,
                })
                .to_string()),
                Err(rusqlite::Error::QueryReturnedNoRows) => Ok(json!({
                    "error": format!("skill not found: {}", params.name)
                })
                .to_string()),
                Err(e) => Err(e.into()),
            }
        })();
        match result {
            Ok(s) => s,
            Err(e) => json!({ "error": format!("{e}") }).to_string(),
        }
    }
}
