# beck TODOS

Living backlog of features, errata, and ideas deferred from v0. Each entry
should say what it is, why it is here (and not in v0), and what triggers it
leaving this file. Ordered by rough priority within each section.

---

## Errata against plan-eng-review-20260407

### ERRATUM 1: v0 MCP surface is tools-only, not tools + per-skill resources

**Source decision:** plan-eng-review section 11 "P7-remainder", which said
`beck mcp` should expose every indexed skill as an MCP resource
(`skill://<name>`), alongside `skills_query` and `skills_load` tools.

**Why it was reversed (2026-04-08):** the math is fatal. MCP clients call
`resources/list` at session start and the full list becomes part of the
agent's system prompt. A beck user with 300 indexed skills would pay
roughly `300 * 90 = 27,000` tokens at session start just for beck's
resource catalog. That is worse than the original "inject everything"
disease beck exists to cure.

**New v0 decision:** tools only. Two tools: `skills_query` (params: query,
top) and `skills_load` (params: name). No `resources/list`, no
`resources/read`. Session-start cost is ~200 tokens flat regardless of
skill count. This preserves beck's launch narrative ("flat 200 tokens, no
matter how many skills you have") and does not paint into a corner: a
resource surface can be added in v0.1 or v0.2 without breaking the v0
API.

**Status:** locked. Phase 4 implements the tools-only MCP server.

---

## Deferred features (v0.1 and later)

### `beck stub` — opt-in file rewriter for non-MCP agents (v0.2)

**Idea (credit: Esteban, 2026-04-08):** after `beck sync` indexes a skill,
offer an opt-in command `beck stub` that replaces each original
`SKILL.md` on disk with a tiny stub containing only the frontmatter plus
a pointer back at beck:

```markdown
---
name: stable-diffusion-image-generation
description: State-of-the-art text-to-image generation with Stable Diffusion...
---

This skill is managed by beck. To load the full body, run:

  beck load stable-diffusion-image-generation

Or call the `skills_load` MCP tool with name=stable-diffusion-image-generation.
```

Original bodies are backed up to `~/.local/share/beck/originals/<name>.md`
before any rewrite. The command is idempotent: re-running it detects
already-stubbed files and skips them. An inverse `beck unstub` restores
from the backup.

**Why this is powerful:** any tool that reads `SKILL.md` files directly
(Hermes, Claude Code without MCP, gstack, file-based skill loaders) now
sees the tiny stub instead of a 500-line skill body. Token savings reach
agents that don't speak MCP, with zero code changes on their side.

**Why it is deferred from v0:**
1. Destructive-by-default operations kill first-run trust. The v0 sync
   command must remain read-only or Show HN buries us.
2. Source of truth becomes ambiguous: the stub is on disk, the body is in
   SQLite, and users editing the stub creates a conflict model we have
   not designed.
3. Most serious users have `~/.claude/skills` or `~/.hermes/skills` under
   git. Stubbing 200 files produces a 200-file diff the user has to
   either commit (losing originals in git history forever) or constantly
   fight.
4. Outside the 7-command v0 fence.

**What it needs before it ships:**
- Explicit opt-in prompt on first run ("This will rewrite N files. Backup
  to X. Continue?")
- Backup + restore tested end-to-end on a dirty repo
- `beck stub --dry-run` that shows the planned diff without touching
  files
- README section explaining the tradeoff and the trust model
- A decision on how re-sync handles new skills (auto-stub? require
  explicit re-stub?)
- Probably a `.beckignore` file so users can exclude specific skills

**Trigger to leave TODOS.md:** after v0.1.0 is released and stable,
promote to v0.2 planning.

---

### `beck watch` — file watcher auto-sync (v0.1)

Deferred by CEO plan. Walk the roots continuously via `notify` crate, run
`sync` on change events. Debounce.

### `beck init` — interactive first-run setup (v0.1)

Deferred by CEO plan. Guide users through selecting roots, checking
agent integration, writing the prompt stub to their agent config.

### `beck doctor` / `beck status` (v0.1)

Deferred by CEO plan. Print config + db paths, every root, integrity_check,
count per root.

### `beck list --duplicates` (v0.1)

Surfaces skills where the last-wins dup policy is shadowing an earlier
entry. Helps users diagnose "why is the wrong skill loading."

### Opt-in embedding reranker (v0.2)

Gated by a second Phase 0 eval. The current FTS5-only path passes 98%
top-3 on the 50-query set, so this is pure upside rather than a fix.
Ship ONNX-bundled BGE-small behind `beck embed-enable`, keep the
embedding optional, never ship in the default build.

### Non-skill doc indexing (v0.2)

Index `CLAUDE.md`, `DESIGN_TOKENS.md`, etc. as a separate collection
from skills. Same FTS5 backend, different table.

### Spec repo `beck-spec` (v0.3)

Formalize the SKILL.md frontmatter convention as a spec other tools can
adopt. The "transition from a CLI to a standard" milestone.

### Windows support (v0.2 or v0.3)

`dirs` crate handles it, but the install story and CI matrix doubles.
Deferred until the Mac+Linux story is solid.

### Telemetry opt-in (decision pending)

Local-only JSONL log of query → load pairs. Never sent. User decision on
whether to ship at all.

---

## Housekeeping

### Switch from `directories` to `dirs` (DONE 2026-04-08)

Mateonunez/nucleo uses `dirs` instead of `directories`. Simpler, fewer
indirections, same API surface for our needs. Adopted in Phase 1.

### CLAUDE.md / CONTEXT.md namespace cleanup

Both files still reference `github.com/estebannunez/beck` in several
places. The real public URL is `gitlab.com/knifecode/beck` (GitHub is
invisible to the public due to the Nunezchef account flag). Not urgent.
Fix in a single grep-and-replace session before launch week.

### github-flagged-account-recovery

The fixture corpus at `tests/fixtures/skills/github-flagged-account-recovery/`
contains the exact playbook. Run it in a separate session before v0.1.0
launch so that the "publish to crates.io yourself" path is unblocked for
future releases. Blocked today; not blocking Phase 1.

End of TODOS.
