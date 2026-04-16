//! beck - local skills router CLI for AI agents.
//!
//! Phase 1 of the v0 build. Seven commands. Single binary. MCP server stubs
//! through to Phase 4. Layout follows mateonunez/nucleo: flat `src/`, typed
//! CliError, clap derive tree, tokio::main async dispatch. Shared modules
//! live in src/lib.rs so the `eval` harness can reuse them.

use clap::{Parser, Subcommand};

use beck::error::{CliError, print_error_json};
mod banner;
mod commands;

#[derive(Parser, Debug)]
#[command(
    name = "beck",
    version,
    about = "Your agent's skills, at its beck and call.",
    long_about = "beck indexes SKILL.md files on disk and serves the right one on demand, so agents stop burning tokens on skill metadata in their system prompts.",
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Walk configured roots, index every SKILL.md into the local database.
    /// With `--from <agent>`, reverse-ingest skills from an agent's
    /// native directory into `~/beck/skills/` (dry-run by default).
    Sync {
        /// Force a full rebuild even if nothing appears to have changed.
        #[arg(long)]
        force: bool,
        /// Emit JSON instead of human text.
        #[arg(long)]
        json: bool,
        /// Reverse-ingest from this agent into `~/beck/skills/`.
        #[arg(long)]
        from: Option<String>,
        /// Execute the ingest plan. Without this, dry-run only.
        #[arg(long)]
        write: bool,
    },

    /// List every indexed skill.
    List {
        #[arg(long)]
        json: bool,
    },

    /// Search indexed skills by free-text query.
    Query {
        /// Free-text search query.
        text: String,
        /// Number of results to return.
        #[arg(long, default_value_t = 3)]
        top: usize,
        #[arg(long)]
        json: bool,
    },

    /// Print the full body of a skill by name.
    Load {
        /// Exact skill name (use `beck list` to discover names).
        name: String,
        #[arg(long)]
        json: bool,
    },

    /// Print the agent integration stub to paste into a system prompt.
    Prompt {
        #[arg(long)]
        json: bool,
    },

    /// Estimate how many tokens beck saves you per agent turn.
    Bench {
        /// Show the math behind the number.
        #[arg(long)]
        explain: bool,
        #[arg(long)]
        json: bool,
    },

    /// Start the MCP server on stdio.
    Mcp,

    /// Initialize the beck home directory (`~/beck/skills/` + manifest).
    Bootstrap {
        #[arg(long)]
        json: bool,
    },

    /// Install every skill under `~/beck/skills/` into every detected agent.
    Link {
        /// Only install into this agent (e.g. `claude-code`).
        #[arg(long)]
        agent: Option<String>,
        /// Print the plan without touching disk.
        #[arg(long)]
        dry_run: bool,
        /// Re-install a beck-managed target whose source sha256 has drifted.
        #[arg(long)]
        force: bool,
        /// Emit JSON instead of human text.
        #[arg(long)]
        json: bool,
    },

    /// Remove beck-managed installs from one or more agents.
    Unlink {
        /// Only remove entries for this skill.
        #[arg(long)]
        skill: Option<String>,
        /// Only remove entries for this agent.
        #[arg(long)]
        agent: Option<String>,
        /// Remove every entry in the manifest.
        #[arg(long)]
        all: bool,
        /// Emit JSON instead of human text.
        #[arg(long)]
        json: bool,
    },

    /// Diagnose beck installs: detect agents, foreign files, orphans, collisions.
    Check {
        /// Scan disk and overwrite the manifest from what beck finds.
        #[arg(long)]
        rebuild_manifest: bool,
        /// Drop manifest entries whose target file is gone.
        #[arg(long)]
        prune: bool,
        /// Emit JSON instead of human text.
        #[arg(long)]
        json: bool,
    },
}

#[tokio::main]
async fn main() {
    // Show banner on bare `beck` or `beck --help` (TTY only).
    let raw_args: Vec<String> = std::env::args().collect();
    let is_help_or_bare = raw_args.len() == 1
        || raw_args.iter().any(|a| a == "--help" || a == "-h");
    if is_help_or_bare {
        banner::maybe_print();
    }

    let cli = Cli::parse();

    let result: Result<(), CliError> = match cli.command {
        Command::Sync {
            force,
            json,
            from,
            write,
        } => commands::sync::handle(force, json, from, write).await,
        Command::List { json } => commands::list::handle(json).await,
        Command::Query { text, top, json } => commands::query::handle(&text, top, json).await,
        Command::Load { name, json } => commands::load::handle(&name, json).await,
        Command::Prompt { json } => commands::prompt::handle(json).await,
        Command::Bench { explain, json } => commands::bench::handle(explain, json).await,
        Command::Mcp => commands::mcp::handle().await,
        Command::Bootstrap { json } => commands::bootstrap::handle(json).await,
        Command::Link {
            agent,
            dry_run,
            force,
            json,
        } => commands::link::handle(agent, dry_run, force, json).await,
        Command::Unlink {
            skill,
            agent,
            all,
            json,
        } => commands::unlink::handle(skill, agent, all, json).await,
        Command::Check {
            rebuild_manifest,
            prune,
            json,
        } => commands::check::handle(rebuild_manifest, prune, json).await,
    };

    if let Err(err) = result {
        print_error_json(&err);
        std::process::exit(err.exit_code());
    }
}
