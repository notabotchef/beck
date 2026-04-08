# CLAUDE.md — beck

You are working on **beck**, a local skills router CLI for AI agents.
This file is your standing context. Read it first on every session.

---

## What beck is (one sentence)

beck is the local, agent-agnostic skills layer for AI agents: a single static
Rust binary that indexes SKILL.md files on disk and serves the right one on
demand (as a CLI AND an MCP server), so agents stop burning 10k+ tokens of
skill metadata in their system prompts.

Tagline: "your agent's skills, at its beck and call."

---

## Project stakes

- Open-source viral launch, reference class: uv / ripgrep / zoxide / fd / bat
- The user (Esteban Nunez, former exec chef, Alinea Group / Roister) has been
  jobless since December 31 2025. beck is the bet that validates his runway.
- Zero budget. Local-first. No cloud. No API keys. No telemetry in v0.
- Treat every decision as "will this look inevitable on day one of the
  Show HN post?" This is the house rule.

---

## Rename event

This project was originally named `skilr`. It was renamed to **beck** on
2026-04-07 during /plan-ceo-review. Git history preserves the original
skilr commits. All documents have been updated to beck. Do not relitigate
the name. It is locked.

---

## Read these files in this order before doing anything

1. `~/Projects/beck/CONTEXT.md`
   Origin story, research, stakes. Includes rename banner at top.

2. `~/.gstack/projects/beck/ceo-plans/2026-04-07-beck.md`
   **AUTHORITATIVE SCOPE.** This is the source of truth. If anything in
   CONTEXT.md or HANDOFF.md conflicts with this, the CEO plan wins.

3. `~/.gstack/projects/beck/plan-eng-review-20260407.md`
   Engineering review report. 17 sections. Answers all 7 forced decisions
   (P4 eval plan, P7-remainder MCP shape, Rust stack picks, frontmatter
   schema, duplicate-name policy, binary size, CI matrix).

4. `~/.gstack/projects/beck/estebannunez-main-design-20260407-2110.md`
   Office-hours design doc with the forcing-question reasoning trail.
   Read last; it's the why, the other three are the what.

5. `~/Projects/beck/HANDOFF.md` (v2)
   The 7-phase build plan. This is your operational script. Phase 0 is
   the eval gate and MUST happen before any FTS5 schema commit.

---

## Locked decisions (do not relitigate any of these)

| # | Decision | Value |
|---|----------|-------|
| Name | beck (renamed from skilr 2026-04-07) |
| Language | Rust (static binary, cargo + brew + curl\|sh) |
| Storage | SQLite + FTS5 BM25 (no embeddings in v0, gated by P4 eval) |
| Commands v0 | sync, list, query, load, mcp, prompt, bench (7 total) |
| Format | SKILL.md + YAML frontmatter (name / description / tags; ignore unknown) |
| Dup policy | last-wins by config order, always warn |
| MCP exposure | BOTH tools AND resources (tools: skills/query + skills/load; resource URI: skill://<name>) |
| rmcp version | 1.3.x |
| Stack | rusqlite bundled+fts5 static, clap derive, serde_yaml, tokio gated behind `mcp` feature |
| Binary size | <6MB stripped (NOT <2MB) — framed as "smaller than ripgrep" |
| CI matrix | ubuntu-22.04 gnu, ubuntu-latest musl, ubuntu-latest aarch64-gnu, macos-13 Intel, macos-latest Apple Silicon |
| Positioning | "the skills layer for AI agents" (not "fast local skill router") |
| Launch | quiet seed to mateonunez + agent0 48h before public Show HN |
| Non-goals | No GUI, no daemon, no HTTP server, no Windows v0, no telemetry, no Python, no embeddings in v0 |

---

## Standing rules from the user (Esteban)

- **Mise en place** — prep before execution. Read everything before coding.
- **Unreasonable hospitality** — zero tolerance for half-assed work. Quality over speed.
- **No debugging inline** — if you hit a bug, branch, use rune-debug skill, report back. Do not pollute the main branch or main chat with debug thrash.
- **CEO plan is authoritative** — if you think scope should expand, flag it, do not silently add.
- **No scope creep** — the non-goals list is a fence, not a suggestion.
- **No em dashes in writing** — use commas, periods, or "...".
- **No AI vocabulary** — delve, robust, comprehensive, nuanced, crucial, underscore, foster, showcase, etc. Say what you mean.
- **Concrete over abstract** — name the file, the function, the line, the command.
- **Connect to the user** — say what the real user experiences, not what the code does.

---

## Git state as of handoff (2026-04-07 late session)

Branch `main`:
  - `8abbe8f` rename: skilr → beck
  - `ed2602c` Update scope: viral OSS launch, gstack front-half required before code
  - `d0feb75` Initial: HANDOFF + CONTEXT for skilr (Skills Router CLI)

Branch `review/plan-eng-review-20260407` (currently checked out):
  - `b1a4c75` handoff: rewrite v2 for Rust CEO scope (7 commands, MCP, 5-7 day)

**First action for a new session:**
1. `git checkout main`
2. `git merge review/plan-eng-review-20260407 --ff-only` (HANDOFF v2 lands on main)
3. `git checkout -b feat/beck-mvp`
4. Begin Phase 0 of HANDOFF.md (eval gate, fixture corpus, 50-query set) BEFORE any Cargo.toml or schema commit.

---

## Three blockers before first code commit

1. **P4 fixture corpus does not exist yet.** Phase 0 in HANDOFF.md is mandatory.
   It's the FIRST commit on `feat/beck-mvp`. Build a corpus of 50-100 real
   SKILL.md files (pull from `~/.hermes/skills/**/SKILL.md` and
   `~/.claude/skills/**/SKILL.md` into `tests/fixtures/`), write a 50-query
   test set, implement the eval harness, run it, record the top-3 recall
   number. The 3-way decision tree is in plan-eng-review section 11 under
   "P4 - Pure FTS5 accuracy gate."

2. **Binary size budget <2MB → <6MB** NEEDS USER SIGN-OFF.
   The old office-hours doc said <2MB. The plan-eng-review subagent proved
   this is not realistic with rusqlite bundled + clap + serde_yaml + rmcp +
   tokio. Realistic floor: 4-6MB stripped. Recommendation: accept <6MB,
   reframe launch narrative as "smaller than ripgrep." Esteban has NOT
   formally signed off on this change yet as of the handoff moment.
   **Action:** ask Esteban once at the start of your first session. Default
   if he does not answer within 24h: <6MB. Do not block Phase 0 on this.

3. **crates.io "beck" + GitHub repo name reservation.** Esteban action, not
   yours. 2 minutes each. Blocks day-0 launch but not Phase 0. Remind him
   once at the start of the session, then carry on.

---

## How the user prefers to delegate

- For long-running or complex tasks: spawn a subagent with a fresh branch
  and a named purpose. Report back with findings + branch name.
- For debugging specifically: the `/ctl` (check the logs) shortcut is
  Esteban's pattern. In this project you probably won't hit it before
  launch.
- Keep the main chat context clean. The user aggressively manages context
  and will thank you for delegating.

---

## How to write when the work is done or gated

- Lead with the point. What shipped, what is gated, what needs Esteban.
- Short paragraphs. Name files and line numbers.
- If something is broken, say it plainly.
- End with the next action.

---

## If you are confused about anything

STOP and ask Esteban one question via whatever the Claude Code equivalent
of AskUserQuestion is. Do not guess. User sovereignty is sacred on this
project.

---

## Skills you may or may not have (do not assume)

If you are Claude Code, you may have gstack skills installed at
`~/.claude/skills/gstack/`. The three that were used to plan beck are:
- office-hours (done, design doc exists)
- plan-ceo-review (done, CEO plan exists)
- plan-eng-review (done, review report exists)

You should NOT re-run any of these. All three artifacts are written.
Your job is to EXECUTE from them.

---

End of CLAUDE.md. Now read the five files listed above, in order.
