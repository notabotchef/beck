# Show HN Draft — beck

## Title
Show HN: beck – local skills router for AI agents (2 MB Rust binary, 200 flat tokens)

## Body

Hey HN,

I built beck because my AI agent was burning 21,000 tokens on every turn loading my entire skills catalog into the system prompt.

The problem: agents like Claude Code inject every available skill (SKILL.md files) into context on every turn. With 500+ skills, that's 100K+ tokens of overhead before you say a word. Anthropic and OpenAI both shipped "tool search" fixes, but they're provider-specific and require code changes.

beck is the local, agent-agnostic fix. It's a single static Rust binary (2.0 MB) that:

1. Indexes SKILL.md files from disk into SQLite + FTS5
2. Serves them on demand via MCP (two tools: query + load)
3. Replaces the 21K-token eager-load with a flat ~200 tokens per session

How it works:

```bash
# index your skills
beck sync

# search (BM25 ranked, 4ms)
beck query "transcribe audio"

# load the match
beck load whisper
```

Or connect it as an MCP server to Claude Code, Claude Desktop, Cursor, etc:

```bash
claude mcp add -s user beck /path/to/beck mcp
```

v0.2 adds `beck link` — a universal skills directory. Write your skill once, install it into every agent you use. One source of truth, native format per agent.

Stats (real, from my machine):
- 547 skills indexed, 98% top-3 retrieval recall
- 805ms cold sync, 4ms hot query
- 2.0 MB binary (smaller than fd, a fraction of ripgrep)
- Zero network calls, zero telemetry, zero API keys

The eval harness is in the repo (tests/eval/queries.toml, 50 queries against 118 fixture skills). Run it yourself.

```
curl -fsSL https://raw.githubusercontent.com/notabotchef/beck/main/scripts/install.sh | sh
```

Built in Rust with rusqlite (bundled FTS5), clap, rmcp (MCP SDK). macOS + Linux. MIT + Apache 2.0.

Feedback welcome. Especially on: other agents to support (Cursor adapter is next), and whether the MCP resources surface should come back in a future version.

Repo: https://github.com/notabotchef/beck

## Notes for posting

- Title stays under 80 chars
- No emojis, no "game-changer", no "revolutionary"
- Lead with the problem, not the solution
- Numbers are verifiable (bench output, eval harness in repo)
- The "feedback welcome" ending invites discussion
- Mentioning Cursor adapter signals roadmap without over-promising
