use beck::error::{CliError, Result};

/// Stub for Phase 1. Phase 4 fills in the real rmcp server with two tools
/// (skills_query, skills_load) and NO resources. See STATUS.md for the
/// resources-decision erratum.
pub async fn handle() -> Result<()> {
    Err(CliError::Validation(
        "beck mcp is not implemented in Phase 1 yet. Phase 4 (HANDOFF.md) ships the rmcp server."
            .into(),
    ))
}
