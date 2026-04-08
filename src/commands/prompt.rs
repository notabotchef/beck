use serde_json::json;

use beck::error::Result;

/// The agent integration stub. Users paste this into their agent system
/// prompt to replace a giant `<available_skills>` block.
const STUB: &str = "\
You have a local skills router called `beck`. Do not load every skill into \
your context; query beck on demand.

- To search: run `beck query \"<task description>\" --json` and pick the \
  best match by name.
- To load the full skill: run `beck load <name>` and treat the output as \
  the skill body.
- If an MCP client is available, the same operations are exposed as \
  `skills_query` and `skills_load` tools over stdio.
";

pub async fn handle(json_out: bool) -> Result<()> {
    if json_out {
        let payload = json!({"prompt": STUB});
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).unwrap_or_default()
        );
    } else {
        print!("{STUB}");
    }
    Ok(())
}
