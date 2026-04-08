use beck::error::Result;

/// Start the MCP server on stdio. Blocks until the client disconnects.
/// See `src/mcp/` for the ServerHandler + ToolRouter implementation.
pub async fn handle() -> Result<()> {
    beck::mcp::start().await
}
