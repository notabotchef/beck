# beck

*your agent's skills, at its beck and call.*

[![crates.io](https://img.shields.io/crates/v/beck.svg)](https://crates.io/crates/beck)
[![build](https://img.shields.io/github/actions/workflow/status/notabotchef/beck/ci.yml?branch=main)](https://github.com/notabotchef/beck/actions)
[![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

<p align="center">
  <img src="assets/hero.png" alt="beck — your agent's skills, at its beck and call." width="720">
</p>

Your agent loads every skill into its system prompt on every turn.
500+ skills = 21,000 tokens before you say a word.

beck indexes your skills and serves them on demand. 200 flat tokens.

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/notabotchef/beck/main/scripts/install.sh | sh
```

```bash
cargo install beck
```

## Quick start

```bash
beck sync                      # index your skills
beck query "transcribe audio"  # search (BM25, 4ms)
beck load whisper              # load the match
```

## MCP (Claude Code, Desktop, Cursor)

```bash
claude mcp add -s user beck /absolute/path/to/beck mcp
```

Two MCP tools: `skills_query` and `skills_load`. Session cost: ~200 tokens.

## The math

| Path | Tokens per turn |
|------|----------------|
| Inject everything | ~150,000 (300 skills) |
| **beck MCP** | **~200 (flat)** |

98% top-3 retrieval recall. Pure FTS5 BM25. No embeddings.
Eval harness in `tests/eval/`. Run `cargo test --features eval --bin eval`.

## Commands

| Command | What it does |
|---------|-------------|
| `beck sync` | Index SKILL.md files from `~/.hermes/skills` and `~/.claude/skills` |
| `beck query "<text>"` | BM25-ranked search across name, description, tags, body |
| `beck load <name>` | Print the full skill body |
| `beck mcp` | MCP server over stdio |
| `beck link` | Write skills once, install into every agent |
| `beck check` | Diagnose agents, orphans, collisions |
| `beck bench` | See your real token savings |
| `beck prompt` | Print the agent integration stub |

2.0 MB binary. Zero network calls. Zero telemetry. macOS + Linux.

## License

MIT OR Apache-2.0
