use serde_json::json;

use beck::db::Db;
use beck::error::{CliError, Result};
use beck::paths;

/// Rough chars-per-token heuristic. OpenAI tokenizer averages ~4 chars per
/// token for English text; Anthropic is close enough. Refinable in v0.1.
const CHARS_PER_TOKEN: f64 = 4.0;

/// What a dumb agent system prompt typically injects per skill: full
/// description plus ~30 chars of formatting ("- {name}: {description}\n").
/// This is the "inject everything" baseline beck saves you from.
const PER_SKILL_FORMAT_OVERHEAD: i64 = 30;

pub async fn handle(explain: bool, json_out: bool) -> Result<()> {
    let db_path = paths::db_path()?;
    if !db_path.exists() {
        return Err(CliError::Validation(
            "no database found. Run `beck sync` first.".into(),
        ));
    }
    let db = Db::open(&db_path).map_err(CliError::Other)?;
    let count = db.count().map_err(CliError::Other)?;
    let desc_bytes = db.description_bytes().map_err(CliError::Other)?;
    let body_bytes = db.body_bytes().map_err(CliError::Other)?;

    let baseline_bytes = desc_bytes + (count * PER_SKILL_FORMAT_OVERHEAD);
    let baseline_tokens = (baseline_bytes as f64 / CHARS_PER_TOKEN).round() as i64;

    // beck's MCP tool surface is 2 tools, ~200 tokens total, flat regardless
    // of skill count. Tools-only path (see STATUS.md erratum for v0 surface).
    let beck_session_tokens: i64 = 200;
    let saved = baseline_tokens - beck_session_tokens;
    let pct = if baseline_tokens > 0 {
        (saved as f64 / baseline_tokens as f64 * 100.0).round()
    } else {
        0.0
    };

    if json_out {
        let payload = json!({
            "skills_indexed": count,
            "baseline_tokens_per_turn": baseline_tokens,
            "beck_tokens_per_turn": beck_session_tokens,
            "tokens_saved_per_turn": saved,
            "percent_saved": pct,
            "description_bytes": desc_bytes,
            "body_bytes": body_bytes,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).unwrap_or_default()
        );
    } else {
        println!("beck saves you ~{saved} tokens per agent turn ({pct}% of the baseline)");
        println!("  skills indexed:              {count}");
        println!("  baseline inject-all tokens:  {baseline_tokens}");
        println!("  beck MCP session tokens:     {beck_session_tokens}  (flat)");
    }

    if explain {
        println!();
        println!("--- math ---");
        println!("chars_per_token            = {CHARS_PER_TOKEN}");
        println!("per_skill_format_overhead  = {PER_SKILL_FORMAT_OVERHEAD}  (bytes of '- {{name}}: ' framing)");
        println!("description_bytes          = {desc_bytes}");
        println!("body_bytes                 = {body_bytes}  (not counted, beck only replaces the catalog metadata)");
        println!(
            "baseline_bytes             = description_bytes + (skill_count * per_skill_format_overhead)"
        );
        println!("                           = {desc_bytes} + ({count} * {PER_SKILL_FORMAT_OVERHEAD}) = {baseline_bytes}");
        println!("baseline_tokens            = baseline_bytes / chars_per_token = {baseline_tokens}");
        println!("beck_session_tokens        = 200  (2 MCP tools * ~100 tokens each, flat)");
        println!("tokens_saved_per_turn      = baseline_tokens - beck_session_tokens = {saved}");
    }

    Ok(())
}
