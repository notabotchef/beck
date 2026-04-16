# beck — Landing Page Copy

## Hero

**Headline:** Your agent burns 21,000 tokens before you say a word.

**Subheadline:** beck indexes your skills on disk and serves them on demand. 200 flat tokens per session, no matter how many skills you have. Single static Rust binary. Zero network calls.

**CTA:** `curl -fsSL https://raw.githubusercontent.com/notabotchef/beck/main/scripts/install.sh | sh`

---

## Value Props

**1. Flat token cost, forever.**
Your agent loads every skill into its system prompt on every turn. 50 skills? 10,000 tokens. 300 skills? 100,000 tokens. beck replaces that with two MCP tools. 200 tokens. Whether you have 5 skills or 5,000.

**2. Local. Static. Done.**
One 2.0 MB binary. SQLite + FTS5 on disk. No servers, no API keys, no cloud, no telemetry. Install it, run `beck sync`, forget about it.

**3. Works with what you have.**
Claude Code, Claude Desktop, Cursor, Codex. If your agent speaks MCP, beck speaks to it. If it reads SKILL.md files, `beck link` installs them in its native format.

---

## Features

- **beck sync** — Walks your skill directories, parses frontmatter, indexes into SQLite FTS5. 547 skills in under a second.
- **beck query "task description"** — BM25-ranked search across name, description, tags, and body. Weighted scoring. Returns in 4ms.
- **beck load <name>** — Full SKILL.md body, ready to inject. No truncation, no summaries.
- **beck mcp** — MCP server over stdio. Two tools: `skills_query` and `skills_load`. Works in Claude Code, Claude Desktop, any MCP client.
- **beck link** — Universal skills directory. Write your skill once, install into every agent. Idempotent. Foreign files untouched.
- **beck bench** — See your actual token savings. Not marketing math, real numbers against your indexed skills.
- **beck check** — Diagnostic tool. Detects agents, finds orphans, flags collisions, reports manifest health.

---

## The Math

| Path | Tokens per turn | With 300 skills |
|------|----------------|-----------------|
| Inject everything (status quo) | ~150,000 | You're paying for this right now |
| beck MCP (tools only) | **200 (flat)** | Same cost with 300 or 3,000 |

**Top-3 recall: 98%. Top-1 recall: 92%.** Pure FTS5 BM25, no embeddings, no reranker. The eval harness is in the repo. Run it yourself.

---

## Social Proof

beck is new. No testimonials yet. But the eval numbers are real:

- 547 skills indexed across two directories
- 118 fixture skills in the test corpus
- 50-query accuracy test set, checked in at `tests/eval/queries.toml`
- Binary size: 2.0 MB (smaller than `fd`, a fraction of `ripgrep`)
- 805ms cold sync. 4ms hot query.

---

## Bottom CTA

```bash
curl -fsSL https://raw.githubusercontent.com/notabotchef/beck/main/scripts/install.sh | sh
beck sync
beck bench
```

See how many tokens you're wasting. Then stop.
