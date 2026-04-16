# beck — Social Posts

## Twitter/X Thread

**1/7**
Your AI agent loads every skill into its system prompt on every single turn.

300 skills = 100,000 tokens before you type a word.

beck fixes this. 200 flat tokens. Whether you have 5 skills or 5,000.

**2/7**
What beck actually is: a single 2.0 MB Rust binary.

Indexes SKILL.md files on disk into SQLite FTS5. Serves them on demand via MCP.

Zero network calls. Zero telemetry. Zero API keys.

`curl -fsSL https://raw.githubusercontent.com/notabotchef/beck/main/scripts/install.sh | sh`

**3/7**
Two commands changed my workflow:

`beck sync` — indexes 547 skills in 800ms
`beck query "transcribe audio"` — 4ms, returns ranked matches

Agent sees the result, calls `beck load whisper`, gets the full skill body.

**4/7**
The MCP integration is two tools. That's it.

`skills_query` — search
`skills_load` — inject

Session-start cost: ~200 tokens. Flat. Forever.

No resources list that re-introduces the 27,000-token problem we're solving.

**5/7**
Not just an MCP server. v0.2 adds `beck link`:

Write your skill once at `~/beck/skills/<name>/SKILL.md`
Run `beck link`
beck installs it into every agent you have, in their native format.

One source of truth. Every agent gets a copy.

**6/7**
Numbers, not vibes:

- 98% top-3 retrieval recall (pure FTS5 BM25, no embeddings)
- 2.0 MB binary (smaller than fd, a fraction of ripgrep)
- 805ms cold sync, 4ms hot query
- 118-fixture eval harness in the repo, run it yourself

**7/7**
Install beck:

```bash
curl -fsSL https://raw.githubusercontent.com/notabotchef/beck/main/scripts/install.sh | sh
beck sync
beck bench
```

See how many tokens you're wasting. Then stop.

---

## LinkedIn Post

I spent the last week building a tool I needed and couldn't find.

The problem: AI agents like Claude Code load every skill (plugin/tool) into their system prompt on every turn. I have 547 skills. That's 21,000+ tokens before I type a single word.

The industry answer is "tool search" — Anthropic shipped it, OpenAI shipped it, LangGraph has bigtool. But they're all provider-specific and require code changes.

I wanted something local, agent-agnostic, zero-rewrite.

So I built beck: a single 2.0 MB Rust binary that indexes SKILL.md files on disk and serves them on demand via MCP. Two tools: search and load. 200 flat tokens per session regardless of skill count.

It also doubles as a universal skills directory. Write your skill once, install it into every agent you use (Claude Code, Codex, etc.) in their native format.

Open source. MIT + Apache 2.0. Zero network calls, zero telemetry.

98% top-3 retrieval recall on a 50-query eval set. Pure FTS5 BM25, no embeddings. The eval harness is in the repo — run it yourself.

github.com/notabotchef/beck

---

## Product Hunt Tagline

Stop burning 100K tokens. Query skills on demand.
