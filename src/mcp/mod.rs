//! beck MCP server. Stdio transport, tools-only surface.
//!
//! Exposes two tools: `skills_query` and `skills_load`. No resources —
//! see TODOS.md erratum 1 for the token-math reasoning.
//!
//! Pattern adapted from mateonunez/nucleo's src/mcp/.

pub mod tools;

use rmcp::ServiceExt;
use rmcp::handler::server::ServerHandler;
use rmcp::model::{Implementation, ServerInfo};
use rmcp::tool_handler;

use crate::consts::APP_NAME;
use crate::error::CliError;
use tools::BeckServer;

#[tool_handler]
impl ServerHandler for BeckServer {
    fn get_info(&self) -> ServerInfo {
        let mut info = ServerInfo::default();
        info.server_info = Implementation::new(APP_NAME, env!("CARGO_PKG_VERSION"));
        info.instructions = Some(
            "beck is a local skills router. Use `skills_query` to search indexed \
             SKILL.md files by free-text description, then `skills_load` to fetch \
             the full body of the chosen skill. beck is populated by running \
             `beck sync` from the shell, which walks ~/.hermes/skills and \
             ~/.claude/skills by default."
                .into(),
        );
        info
    }
}

/// Start the MCP server on stdio transport. Blocks until the client disconnects.
pub async fn start() -> std::result::Result<(), CliError> {
    let server = BeckServer::new();
    let transport = (tokio::io::stdin(), tokio::io::stdout());
    let service = server
        .serve(transport)
        .await
        .map_err(|e| CliError::Other(anyhow::anyhow!("MCP server error: {e}")))?;
    service
        .waiting()
        .await
        .map_err(|e| CliError::Other(anyhow::anyhow!("MCP server stopped: {e}")))?;
    Ok(())
}
