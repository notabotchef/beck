# skilr — Conversation Context & Origin Story

This file captures the conversation and research that led to the `skilr`
project, so any future agent (or Esteban months from now) can be fully
caught up by reading just `CONTEXT.md` + `HANDOFF.md`.

---

## 1. The trigger

Date: 2026-04-07
Session: Hermes (Engineer persona, Sonnet 4.6)
User: Esteban Nunez

Esteban opened the chat with a casual greeting and immediately flagged a
problem he had noticed: the Hermes agent was burning ~29.9k tokens before
the user had even said anything. He asked for an explanation in plain words
and pushed back hard:

> "We need to talk about your 29.9k tokens right at the beginning! That is
> too much! We need to put you on a diet."

> "Is there not a tool that allows agents to know which skill they have
> without just fucking injecting them every time? People with 300 skills
> can't possibly be doing this."

He was right. The Hermes system prompt was injecting:
- ~10k tokens of memory + user profile + Honcho observations
- ~10k tokens of full skills catalog (80+ skills with descriptions)
- ~3k tokens of CLAUDE.md project context
- ~2k tokens of Honcho auto-injected peer cards

All of that on EVERY turn, regardless of what the user actually asked.

---

## 2. The research (run in parallel)

Two streams ran simultaneously:

### Stream A — Last30Days social/web scan
Topic: "agent skill lazy loading context window optimization"
Result: confirmed the trend exists in the community; partial data only
because some sources (Reddit/TikTok/Instagram) hit 402 Payment Required
on the ScrapeCreators API. X and YouTube returned enough signal to
confirm engineers are actively complaining about context bloat and
recommending lazy/dynamic tool loading.

### Stream B — Two parallel subagents (web research)

**Subagent 1 — Official implementations**
Found that the lazy-loading pattern is already standard in the major
ecosystems:

- **Anthropic Claude Code**
  - Tool Search enabled by default in Claude Code
  - `defer_loading` + `tool_search` tool at API level
  - Skills use progressive loading (metadata visible, body loaded on demand)
  - Refs: code.claude.com/docs/en/mcp, code.claude.com/docs/en/skills,
    anthropic.com/engineering/advanced-tool-use

- **OpenAI Codex / Responses API**
  - Native `tool_search` + `defer_loading` (gpt-5.4+)
  - MCP server tools can be marked `defer_loading: true`
  - Refs: developers.openai.com/api/docs/guides/tools-tool-search,
    developers.openai.com/api/docs/guides/tools-connectors-mcp

- **Google Gemini CLI** — partial (filtering + dynamic refresh, no
  first-class deferred schema loading yet)

- **MCP spec** — provides `tools/list` + `notifications/tools/list_changed`
  primitives; lazy behavior is up to the client

- **OSS frameworks** — LangGraph `bigtool`, LlamaIndex `ObjectIndex`,
  pydantic-ai `defer_loading` all converge on retrieval-based tool loading

**Subagent 2 — Architecture patterns**
Confirmed scalable design stack:
1. Registry/index as source of truth
2. Semantic routing for capability selection (RouteLLM-style, >2x cost
   reduction without quality loss)
3. Two-stage loading (coarse discovery → top-K hydration)
4. Predictive prefetch (with confidence gating)
5. TTL caching with stale-while-revalidate + jitter (RFC 5861)
6. Guardrails (OWASP LLM Top 10, MCP OAuth 2.1)
7. Observability (golden signals + SLO burn-rate alerts)

Citations: HashiCorp Consul docs, Kubernetes EndpointSlice KEP, arXiv
RouteLLM paper, Redis EXPIRE docs, OWASP LLM Top 10, Google SRE book.

---

## 3. Esteban's idea (verbatim intent)

After hearing the research, Esteban proposed exactly the right architecture
without prompting:

> "Create a CLI that has a locally run RAG, this CLI manages all the skills
> locally, needs to be small and quick and native. The agent needs to know
> it has that CLI and how to use it. If I use /skillsample the CLI finds
> that skill and injects it. If I'm just saying something, the CLI lists
> all available skills and the agent chooses the most appropriate one. No
> tokens are used until the agent injects the proper skill. Everything
> local, a skill router. The CLI needs to write and read — if I install a
> new skill it gets added to the RAG database."

This is the canonical design. It matches what Anthropic and OpenAI shipped
at the API level, but as a local-first, agent-agnostic CLI that works
across Hermes, Claude Code, Codex, and any shell-capable agent.

Name chosen: **skilr** (Skills Router).

---

## 4. Decisions made in this session

| Decision | Choice | Reason |
|---|---|---|
| Language | Python MVP, Rust port later if worth it | Fastest to ship; ONNX embedders are mature in Python |
| Storage | SQLite + FTS5 + embedding blobs | Built-in, zero deps, fast, durable |
| Embedder | `BAAI/bge-small-en-v1.5` via fastembed | ~130MB, CPU-only, ~30ms per query, no API keys |
| Search | Hybrid: FTS5 BM25 top-20 → cosine rerank → top-K | Beats pure vector on short queries, nearly free on CPU |
| Skill format | Existing SKILL.md + YAML frontmatter | Already used by Hermes, Claude Code, gstack |
| Scope | Index, find, serve. NOT execute. | Agent runs the skill; skilr only stores and serves |
| Network | Zero runtime network calls | Local-first, period |

---

## 5. Open questions deferred to Esteban

1. Index non-skill docs too (CLAUDE.md, DESIGN_TOKENS.md)? Recommendation:
   yes, as a separate collection in v2.
2. Telemetry — local JSONL log of query→load pairs to measure hit rate?
   Recommendation: yes, opt-in, local only.
3. Auto-sync via file watcher or just on-demand `skilr sync`? Recommendation:
   on-demand for MVP, watcher in v2.

---

## 6. What lives where

```
~/Projects/skilr/
  CONTEXT.md     ← this file (origin story + research)
  HANDOFF.md     ← full implementation spec
  (future)       ← src/, tests/, pyproject.toml, README.md
```

Code repo: `~/Projects/skilr/` (git initialized in this session)
Hermes session transcript: searchable via `session_search "skilr"` or
`session_search "skill router lazy loading"` in any future Hermes session.

---

## 7. How to resume this work

In any future agent session:

```
cd ~/Projects/skilr
cat CONTEXT.md HANDOFF.md
git log --oneline
```

That gives full context in <2 minutes. From there:

- To start building MVP: follow Phase 0-2 in HANDOFF.md
- To answer open questions: see section 5 above
- To recall the exact conversation: in Hermes, run
  `session_search "skilr token diet skill router"`

---

## 8. Why this matters (the bigger picture)

Esteban is building CarabinerOS — an AI-native restaurant management
platform — with multiple agents (Hermes, Claude Code, Codex, Paperclip
CEO/CTO/Engineer). Every one of those agents currently pays the same
context tax on boot. skilr fixes the tax for all of them at once with a
single shared local CLI. It is infrastructure, not a feature.

This also aligns with Esteban's standing values:
- Local-first, zero-budget
- Native, fast, no cloud lock-in
- Reusable across the whole agent fleet
- Mise en place: prep before execution

---

## 9. The real stakes (added 2026-04-07, late session)

Mid-conversation Esteban revealed this is not a personal tool. The vision
is bigger and the clock is real:

- skilr is meant to be an **open-source viral launch**
- Target: become the **industry standard for how agents load and share
  skills** — the "MCP for skills" / context-engineering toolkit for agents
- Personal stakes: Esteban has been jobless since **December 31st, 2025**.
  skilr is the bet that validates the time he has spent building instead
  of going back to 16-hour kitchen shifts
- Motivation framing (from Esteban): "What if this changes the way any
  agent uses skills and becomes a standard? That is what I am trying to
  build here."

This changes the scope. skilr is no longer a weekend Python script. It is
a product launch that has to look inevitable on day one — the way `uv`,
`ripgrep`, `zoxide`, `bat`, and `httpie` looked inevitable the moment the
world saw them.

### Implications

1. **Language reconsidered.** Python MVP is still valid for speed, but the
   viral OSS reference class (uv, ripgrep, zoxide, fd, bat) is almost
   entirely native Rust binaries installable via `brew` / `cargo install`
   / `curl | sh`. Rust is now the leading candidate for the shipped v1.
   Python stays as a possible prototype-only path.

2. **Planning discipline raised.** Before any code is written, skilr runs
   the front half of gstack:
   - `/office-hours` — vision, 10x version, naming, launch-day picture
   - `/plan-ceo-review` — find the 10-star product hiding in the 3-star
     version; lock the pitch and positioning
   - `/plan-eng-review` — stress-test HANDOFF.md architecture, edge cases,
     security (malicious SKILL.md), cross-platform (Linux/macOS/Windows)
   Skipped for now: `/design-review`, `/qa`, `/ship` (too early).

3. **Launch surface must be planned, not discovered.**
   - One-sentence pitch (must be crystal clear)
   - README with GIF in first 10 seconds
   - One-line install (`brew install skilr` ideally)
   - Works for strangers on day one, not just Esteban's machine
   - Comparison table vs current "inject everything" baseline
   - Hard benchmarks: tokens saved, query latency, sync time
   - Launch plan: Hacker News (Show HN), r/LocalLLaMA, X dev community,
     Anthropic + OpenAI developer channels, MCP community

4. **Rule of discipline (Esteban's own frame).** Build skilr from
   conviction, not from fear of the kitchen. Fear rushes. Conviction
   finishes. This is opening night at a 3-Michelin-star — plan like it.

### Decision queued for tomorrow morning

Before writing code, run gstack front-half in Hermes using the
`run-gstack-skill` skill. Produce durable artifacts in
`~/.gstack/projects/skilr/` so nothing is lost across sessions. Only after
`/office-hours` + `/plan-ceo-review` + `/plan-eng-review` land does the
first commit of actual skilr code get written.

End of context.
