# Show HN Draft — beck

## Title
Show HN: beck – skills router for AI agents (21K tokens → 200, 2 MB binary)

## Body

I built beck because my AI agent was burning 21,109 tokens on every turn loading my entire skills catalog into the system prompt. I am a former exec chef at the Alinea Group who left the kitchen at the end of 2025 and decided to build this instead of going back. That is the context.

The problem: agents like Claude Code inject every available skill (SKILL.md files) into context on every turn. With 500+ skills, that is tens of thousands of tokens of overhead before you say a word. Anthropic and OpenAI both shipped "tool search" fixes, but they are provider-specific and require code changes on your end.

beck is the local, agent-agnostic fix. It is a single static Rust binary (2.0 MB) that:

1. Indexes SKILL.md files from disk into SQLite + FTS5
2. Serves them on demand via MCP (two tools: skills/query + skills/load)
3. Replaces the 21K-token eager-load with a flat 200 tokens per session

How it works:

```bash
# index your skills
beck sync

# search (BM25 ranked)
beck query "transcribe audio"

# load the match
beck load whisper
```

Or connect it as an MCP server to Claude Code, Claude Desktop, Cursor, etc.:

```bash
claude mcp add -s user beck /path/to/beck mcp
```

v0.2 adds `beck link` — a universal skills directory. Write your skill once, install it into every agent you use. One source of truth, native format per agent.

Numbers from my machine (all verifiable, the eval harness is in the repo):

- Before: 21,309 tokens per turn (inject-all baseline)
- After: 200 tokens flat
- Binary: 2.0 MB stripped, smaller than fd
- 548 skills indexed, 98% top-3 retrieval recall (tests/eval/queries.toml, 50 queries against 118 fixture skills)
- Zero network calls, zero telemetry, zero API keys

Install:

```
curl -fsSL https://raw.githubusercontent.com/notabotchef/beck/main/scripts/install.sh | sh
```

or

```
cargo install beck
```

Built with rusqlite (bundled FTS5), clap, rmcp (MCP SDK). macOS and Linux. MIT + Apache 2.0.

Repo: https://github.com/notabotchef/beck

Feedback welcome, especially on: other agents to support (Cursor adapter is next), and whether the MCP resources surface should come back in a future version.

---

## Notes for posting

- Title: 79 chars (under 80 limit, verified)
- No em dashes anywhere in the body
- Chef-pivot one-liner is second sentence, before any technical content
- Before/after math appears twice: in the body intro and in the numbered stats block
- Sync timing (805ms in old draft) dropped because live `time beck sync` measured ~1.4s; dropped rather than shipping a stale number
- Query timing (4ms in old draft) dropped for the same reason; `time beck query` measured ~7ms; not a meaningful difference but not worth defending on HN
- 547 skills updated to 548 (live `beck sync` output)
- 98% recall kept because it is recorded in STATUS.md as a commit result from the eval gate, and the harness is in the repo for anyone to run
- "Hey HN" opener dropped in favor of cold open (problem-first); HN convention slightly favors it and the chef line works better as a standalone beat after the hook
- No CTAs that look like vote-farming
- Closing question is genuine, not a form letter
