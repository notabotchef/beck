# beck — Local Skills Router CLI

> **RENAME NOTICE (2026-04-07):** This project was originally named `skilr`.
> It was renamed to `beck` during `/plan-ceo-review` after a research pass
> confirmed `beck` is dual-clear on crates.io + homebrew-core and the English
> idiom "at your beck and call" literally describes the product: skills
> summoned on demand. All documents below have been updated to reference
> `beck`. Git history preserves the original `skilr` commits.


**Handoff document**
Author: Hermes (Engineer persona)
For: Esteban Nunez / next implementing agent
Date: 2026-04-07
Status: SPEC — DO NOT IMPLEMENT YET. Blocked on gstack front-half
(office-hours → plan-ceo-review → plan-eng-review). See CONTEXT.md §9.

Mission framing: beck is an open-source viral launch targeting the
agent-tooling ecosystem. Goal is to become the standard for how agents
load and share skills. Treat every decision as "will this look inevitable
on day one of the Show HN post?"

---

## 1. Problem

Agents today dump their full skill catalog into the system prompt on every turn.
With 80+ skills, that is ~10-15k tokens of pure overhead BEFORE the user says a word.
At 300 skills this pattern is unusable.

Research (OpenAI `tool_search` + `defer_loading`, Anthropic Skills progressive
loading, LangGraph `bigtool`, LlamaIndex `ObjectIndex`, pydantic-ai
`defer_loading`) all converge on the same fix:

> Keep skill metadata out of the prompt. Load full skill content on demand,
> only when the agent decides it is relevant.

`beck` is our local, native, zero-token implementation of that pattern.

---

## 2. What beck is

A tiny local CLI + RAG index that manages all agent skills on the machine.
The agent does not see any skills in its system prompt. Instead it sees ONE
line: "You have a `beck` CLI. Use it."

When the agent needs a skill:
1. Runs `beck query "task description"` → gets top-K matches (name + score)
2. Runs `beck load <name>` → full SKILL.md prints to stdout → enters context
3. Done. Only the ONE relevant skill is in context. Everything else stays on disk.

Zero tokens are spent until the agent actively pulls a skill.

---

## 3. Hard requirements

- Local only. No network calls at runtime.
- Native feel. Fast startup (<200ms cold), fast query (<50ms).
- Small footprint. Single binary or single-file Python, SQLite for storage.
- Read AND write. `beck add` ingests new skills immediately.
- Self-healing. Detects stale entries via content hash and reindexes.
- Agent-agnostic. Works from any shell-capable agent: Hermes, Claude Code,
  Codex CLI, Ollama wrappers, Gemini CLI, raw bash.
- Zero API keys. No OpenAI, no cloud embeddings.

---

## 4. Architecture

```
~/.config/beck/
  skills.db          # SQLite with FTS5 + embedding blobs
  config.toml        # skill dirs, embed model, top_k, etc.
  model/             # cached ONNX embedding model (downloaded once)

Scanned skill sources (configurable):
  ~/.hermes/skills/**/SKILL.md
  ~/.claude/skills/**/SKILL.md
  ~/.config/beck/skills/**/SKILL.md   (user-local)
```

### Storage schema (SQLite)

```sql
CREATE TABLE skills (
  id            INTEGER PRIMARY KEY,
  name          TEXT UNIQUE NOT NULL,
  path          TEXT NOT NULL,
  category      TEXT,
  description   TEXT,
  tags          TEXT,          -- comma separated
  content_hash  TEXT NOT NULL, -- sha256 of SKILL.md
  embedding     BLOB NOT NULL, -- float32 vector
  last_indexed  INTEGER,       -- unix ts
  last_used     INTEGER,
  use_count     INTEGER DEFAULT 0
);

CREATE VIRTUAL TABLE skills_fts USING fts5(
  name, description, tags, content,
  content='skills', content_rowid='id'
);
```

### Search strategy (hybrid, dead simple)

1. BM25 lexical search via FTS5 → top 20 candidates
2. Rerank top 20 by cosine similarity against query embedding
3. Return top K (default 3) with scores

This beats pure vector search on short skill queries and is nearly free on CPU.

### Embedding model

Default: `BAAI/bge-small-en-v1.5` via `fastembed` (ONNX, ~130MB, CPU-only, ~30ms per query on M-series).
Alternative (if available): `Qwen/Qwen3-Embedding-0.6B` for multilingual.
Must run locally. No HTTP calls after initial model download.

---

## 5. CLI surface

```
beck sync [--force]
  Walk all configured skill directories, embed new/changed SKILL.md files,
  prune deleted ones. Idempotent. Uses content_hash to skip unchanged files.

beck add <path>
  Ingest a single skill file or directory. Embeds and upserts immediately.

beck list [--category X] [--json]
  Print all indexed skills: name | category | 1-line description.
  This is the "cheap discovery" call.

beck query "<text>" [--top N=3] [--json]
  Hybrid search (FTS5 + cosine). Prints ranked matches with scores.

beck load <name> [--section N]
  Print full SKILL.md content to stdout. This is what the agent pipes into
  its own context. Updates last_used + use_count.

beck show <name>
  Metadata only (no body). For debugging.

beck remove <name>
  Unindex a skill (does NOT delete the source file).

beck status
  DB path, count, last sync, avg query latency, cache size.

beck doctor
  Verify model present, DB healthy, paths readable. Prints fix hints.
```

Exit codes: 0 ok, 1 not found, 2 config error, 3 index corrupt.
All commands support `--json` for machine consumption by agents.

---

## 6. Agent integration (the whole point)

Replace the giant `<available_skills>` block in every agent system prompt with
~50 tokens:

```
You have a local skills router: `beck`.
- `beck list` — see what skills exist (name + 1-line description)
- `beck query "<task>"` — semantic search, returns top matches
- `beck load <name>` — inject the full skill instructions into context
Use it proactively when a task looks like it matches a known workflow.
Never guess skill contents — always `beck load` before following a skill.
```

Slash-command shortcut:
- User types `/deploy` or `/skill deploy` → agent runs `beck load deploy` directly.
- User describes a task → agent runs `beck query` first, then `beck load` on the top hit.

---

## 7. Implementation plan (MVP → v1)

### Phase 0 — Scaffold (30 min)
- `pyproject.toml` with deps: `click`, `fastembed`, `numpy`, `tomli`, `rich`
- Entry point: `beck = beck.cli:main`
- `pipx install -e .` for local install

### Phase 1 — DB + sync (1-2h)
- SQLite schema above, FTS5 enabled
- `beck sync` walks configured dirs, parses YAML frontmatter from SKILL.md,
  computes sha256, embeds, upserts
- Handles deletes (skills in DB not on disk → prune)
- `beck list` works

### Phase 2 — Query + load (1h)
- `beck query` hybrid search (FTS5 top 20 → cosine rerank → top K)
- `beck load` prints full file, bumps use_count
- `--json` output mode

### Phase 3 — Watch + polish (1h)
- `beck add <path>` for one-shot ingestion
- `beck doctor`, `beck status`
- Config file at `~/.config/beck/config.toml` with skill dir list

### Phase 4 — Agent wiring (30 min)
- Replace `<available_skills>` injection in Hermes with the 50-token stub
- Add a test prompt: "deploy the frontend" → verify agent calls `beck query`
  then `beck load` instead of guessing

### Phase 5 — Optional v2
- File watcher (`watchdog`) for auto-sync on SKILL.md changes
- LRU cache of recently loaded skills (in-memory, per shell session)
- Usage analytics: which skills get picked, which queries miss
- Cross-agent share: same DB used by Hermes + Claude Code + Codex

---

## 8. Config file (example)

```toml
# ~/.config/beck/config.toml
[general]
db_path      = "~/.config/beck/skills.db"
model        = "BAAI/bge-small-en-v1.5"
top_k        = 3
cache_model  = "~/.config/beck/model"

[[sources]]
path    = "~/.hermes/skills"
enabled = true

[[sources]]
path    = "~/.claude/skills"
enabled = true

[[sources]]
path    = "~/.config/beck/skills"
enabled = true
```

---

## 9. Non-goals (resist scope creep)

- No web UI.
- No multi-user auth.
- No cloud sync (local-first, period).
- No skill EXECUTION — beck only stores, finds, and serves skill text. The
  agent runs the skill.
- No custom skill format — uses the existing SKILL.md + YAML frontmatter that
  Hermes / Claude Code / gstack already follow.

---

## 10. Success criteria

- Hermes system prompt drops from ~30k tokens to <5k tokens at boot.
- `beck query` returns correct skill in top 3 for ≥90% of realistic task
  phrasings across a 50-query eval set.
- `beck sync` completes in <5s for 100 skills on M-series CPU.
- Adding a new skill file and running `beck sync` makes it queryable
  within one command, no restart.
- Works from Hermes, Claude Code, and plain bash with zero changes to the
  tool itself.

---

## 11. Open questions for Esteban

1. Preferred language: Python (fastest to ship, heavier) or Rust (true native
   binary, longer to build)? **UPDATED 2026-04-07:** beck is now scoped as a
   viral OSS launch. Reference class (uv, ripgrep, zoxide, fd, bat) is almost
   entirely native Rust distributed via `brew` / `cargo install` / `curl | sh`.
   **Leading recommendation: Rust for shipped v1.** Python only as a throwaway
   prototype if speed-to-first-demo matters more than launch polish. Final
   call happens during `/office-hours` + `/plan-eng-review` tomorrow.
2. Should beck ALSO index non-skill docs (CLAUDE.md, DESIGN_TOKENS.md,
   CONTRIBUTING.md)? That would let the agent pull project context on demand
   the same way. Recommendation: **yes, as a separate collection in v2**.
3. Telemetry: opt-in local log of query→load pairs so we can measure hit rate
   and improve the embedder prompt? Recommendation: **yes, local JSONL only**.

---

## 12. Next action

**BLOCKED on gstack front-half.** Do not spawn an implementing agent yet.

Tomorrow morning, in a fresh Hermes session, run the `run-gstack-skill`
skill to execute these three phases in order. Save all artifacts to
`~/.gstack/projects/beck/`:

1. `/office-hours` — lock the vision, 10x version, name, tagline,
   launch-day picture. Answers: who is this for, what does "done" mean,
   what does day-1-of-launch look like.
2. `/plan-ceo-review` — find the 10-star product hiding in the 3-star
   version. Lock the pitch, positioning, and comparison frame (vs current
   "inject everything" baseline). Produce the one-sentence pitch.
3. `/plan-eng-review` — stress-test this HANDOFF.md. Cover:
   - Rust vs Python final call
   - Malicious SKILL.md / prompt-injection surface
   - Cross-platform (macOS, Linux, Windows)
   - Install story (`brew`, `cargo install`, `curl | sh`, `pipx`)
   - Benchmarks plan (tokens saved, query latency, sync time)
   - SKILL.md format compatibility (Hermes, Claude Code, gstack)

Only after those three land does the first commit of actual beck code
get written. Suggested post-planning prompt to the implementing agent:

> Read ~/Projects/beck/CONTEXT.md and HANDOFF.md and all artifacts under
> ~/.gstack/projects/beck/. Implement Phase 0-2 on branch `feat/beck-mvp`
> in the language chosen during /plan-eng-review. Ship `sync`, `list`,
> `query`, `load`, `status`. Include README with GIF placeholder, install
> instructions, comparison table vs "inject everything" baseline, and a
> tests/ folder with 10 sample SKILL.md fixtures. Report back with install
> steps and a recorded demo of `beck query "deploy frontend"` returning
> the right skill.

End of handoff.
