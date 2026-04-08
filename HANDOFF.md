# beck — Local Skills Router CLI

> **RENAME NOTICE (2026-04-07):** This project was originally named `skilr`.
> It was renamed to `beck` during `/plan-ceo-review` after a research pass
> confirmed `beck` is dual-clear on crates.io + homebrew-core and the English
> idiom "at your beck and call" literally describes the product: skills
> summoned on demand. Git history preserves the original `skilr` commits.

**Handoff document v2 (post plan-eng-review)**
For: implementing agent
Date: 2026-04-07
Status: READY TO IMPLEMENT
Branch to create: feat/beck-mvp
Authoritative scope: ~/.gstack/projects/beck/ceo-plans/2026-04-07-beck.md
Engineering review: ~/.gstack/projects/beck/plan-eng-review-20260407.md

This HANDOFF replaces the original Python-based handoff. The earlier scope (Python, fastembed, hybrid search, 8+ commands) is dead. Read the engineering review report alongside this file. If anything here disagrees with that report, the report wins.

---

## 1. Problem (unchanged from v1)

Agents today dump their full skill catalog into the system prompt on every turn. With 80+ skills, that is ~10-15k tokens of pure overhead BEFORE the user says a word. At 300 skills the pattern is unusable. Anthropic shipped Tool Search + defer_loading in Claude Code. OpenAI shipped tool_search + defer_loading in gpt-5.4+. LangGraph bigtool, LlamaIndex ObjectIndex, and pydantic-ai defer_loading converged on the same fix. None of them are local, agent-agnostic, or zero-rewrite. beck is.

---

## 2. What beck is (v0)

A single static Rust binary that:
1. Indexes SKILL.md files from configured roots into a local SQLite + FTS5 database.
2. Serves them on demand via 7 shell commands AND an MCP server.
3. Makes zero network calls at runtime.

The agent never sees a giant `<available_skills>` block in its system prompt. Instead it sees one line from `beck prompt`:

> You have a local skills router: `beck`. `beck query` to search, `beck load` to inject one. Or call the skills/query and skills/load tools over MCP if connected.

When the agent needs a skill, it runs `beck query "task description"`, gets ranked matches, then `beck load <name>` to pull the body into context.

---

## 3. v0 scope (LOCKED by CEO plan)

Stack: Rust, SQLite + FTS5 BM25 (no embeddings), single static binary, macOS + Linux only.

Commands (7 total, no more):
1. `beck sync [--force]`
2. `beck list [--json]`
3. `beck query "<text>" [--top N=3] [--json]`
4. `beck load <name>`
5. `beck mcp` — long-lived MCP server over stdio (rmcp), exposes BOTH MCP tools (skills/query, skills/load) AND MCP resources (skill://<name>)
6. `beck prompt`
7. `beck bench`

Every command supports `--json`.

Out of scope for v0 (do not implement): watch, init, doctor, status, add, remove, show, embeddings, hybrid search, ONNX, file watcher, Windows, telemetry, shared multi-user DB, skill execution.

Timeline: 5-7 days from first commit to first `cargo publish`.

---

## 4. Stack picks (LOCKED by plan-eng-review section 11)

- Language: Rust stable
- sqlite client: rusqlite features ["bundled","fts5"]
- arg parser: clap features ["derive"]
- yaml parser: serde_yaml (fallback serde_yml fork if unmaintained)
- async runtime: tokio features ["rt","io-std","macros"], ONLY behind cargo feature `mcp`
- MCP SDK: rmcp 1.3.x features ["server","transport-io"], behind feature `mcp` (default = on)
- Other deps: walkdir, sha2, anyhow, thiserror, directories, tracing, tracing-subscriber
- [profile.release]: opt-level="z", lto="fat", codegen-units=1, strip=true, panic="abort"
- Binary size budget: <6MB stripped (relaxed from <2MB; see review section 11). Frame as smaller than ripgrep.

---

## 5. Crate layout

```
beck/
  Cargo.toml             (workspace root)
  rust-toolchain.toml    (stable channel pin)
  crates/
    beck-core/           (lib: db, sync, query, skill model, frontmatter parser)
    beck-cli/            (bin: clap, dispatches to core, default-on feature mcp)
  tests/
    fixtures/skills/     (50-100 real SKILL.md files, committed)
    eval/queries.toml    (50-query accuracy test set)
  scripts/
    install.sh           (curl install with sha256 verify)
  .github/workflows/
    ci.yml               (5-cell matrix)
    release.yml          (on tag v*)
  README.md
  LICENSE-MIT
  LICENSE-APACHE
  CHANGELOG.md
  HANDOFF.md (this file)
  CONTEXT.md
```

---

## 6. Phase plan (5-7 days)

### Phase 0 — Eval gate (DAY 1, MANDATORY)

This is the FIRST commit on feat/beck-mvp. Before writing the production schema.

Phase 0a fixture corpus (2h): mkdir -p tests/fixtures/skills. Find SKILL.md files under ~/.hermes/skills and ~/.claude/skills, copy each into tests/fixtures/skills/<dirname>/SKILL.md. Manually scan for leaked secrets / personal data; redact. Target 50-100 files. Commit as "tests: import fixture corpus".

Phase 0b query set (2h, Esteban writes): tests/eval/queries.toml with 50 entries. 20 exact-keyword, 20 paraphrase, 10 adversarial. Each: text, expected_top1, expected_in_top3.

Phase 0c eval harness (1h): crates/beck-core/src/eval.rs behind `#[cfg(feature = "eval")]`. Reads queries.toml, builds a fresh in-memory test DB from fixtures, runs query(), computes top-1 and top-3 recall. Asserts top-3 >= 85%. `cargo test --features eval eval::accuracy_gate` must pass.

Phase 0d gate decision:
- top-3 >= 85%: continue to Phase 1.
- top-3 65-84%: continue, but add a TODO to README to relabel narrative to "10x token reduction".
- top-3 < 65%: STOP. Ping main agent for scope reopen.

### Phase 1 — Workspace + DB + sync (DAY 1-2)

1. cargo new --lib crates/beck-core; cargo new crates/beck-cli; workspace Cargo.toml.
2. beck-core::db: open(path), migrate (FTS5 schema), upsert_skill, prune_missing.
3. Schema: skills table (id, name, path UNIQUE, description, tags, content_hash, last_indexed) + UNIQUE INDEX on name + skills_fts virtual table FTS5(name, description, tags, body) with content=skills + sync triggers.
4. beck-core::sync: walkdir each root, parse frontmatter, sha256, upsert. Last-wins on duplicate name across roots, log warning. Symlink loop detection via canonical-path set.
5. beck-core::skill::Frontmatter: name (Option<String>), description (Option<String>), tags (Option<Vec<String>>). Unknown fields ignored.
6. beck-cli::commands::sync wires it up. Hard-coded config path via directories crate.

### Phase 2 — Query + load + list (DAY 2-3)

1. beck-core::query::search(query, top): FTS5 BM25 across name + description + tags + body, weighted name 4x, description 2x, tags 1.5x, body 1x. Return Vec<Match>.
2. beck-cli::commands::query: text and --json.
3. beck-cli::commands::load: SELECT path WHERE name=?, read file, print body.
4. beck-cli::commands::list: SELECT name, description ORDER BY name. text and --json.

After Phase 2, eval gate runs against the real implementation. Must still pass top-3 >= 85%.

### Phase 3 — prompt + bench (DAY 3, half day)

1. beck-cli::commands::prompt: print canonical agent integration stub (text or --json).
2. beck-cli::commands::bench: SELECT count, sum(length(description)). Compute estimated tokens-saved-per-turn (chars/4 heuristic, with --explain for the math). Print formatted output.

### Phase 4 — beck mcp (DAY 4)

1. crates/beck-cli/src/mcp.rs behind feature `mcp` (default).
2. tokio::main, rmcp Server with stdio transport.
3. Tools: skills/query (params: query string, top u32 default 3), skills/load (params: name string).
4. Resources: list_resources returns one Resource per skill with URI skill://<name>, mime text/markdown. read_resource parses URI, calls beck-core load.
5. Integration test: spawn beck mcp as subprocess, send initialize, tools/list, tools/call skills/query, resources/list, resources/read; assert responses.

### Phase 5 — README + install + CI + release (DAY 5-6)

1. README.md with: 10s GIF placeholder, comparison table (vs nothing / Manual / Claude Code Tool Search / LangGraph bigtool), one-line install per platform, agent integration stub copied from beck prompt, Trust model paragraph, Privacy paragraph, Authoring skills section with canonical schema.
2. scripts/install.sh: detect OS+arch, download release tarball, verify sha256, install to /usr/local/bin or ~/.local/bin.
3. .github/workflows/ci.yml: 5-cell matrix (ubuntu-22.04 gnu, ubuntu-latest musl, ubuntu-latest aarch64-gnu, macos-13 x86_64, macos-latest aarch64). cargo build --release, cargo test, strip + size gate. Separate jobs for cargo deny + fmt/clippy.
4. .github/workflows/release.yml: on tag v*, build all 5, sign SHA256SUMS, gh release create, cargo publish.
5. Homebrew tap repo at notabotchef/homebrew-beck with Formula/beck.rb (template in plan-eng-review section 10).

### Phase 6 — Launch prep (DAY 7)

1. Tag v0.1.0, push, watch release.yml.
2. Verify cargo install beck on a clean machine.
3. Verify brew tap notabotchef/beck && brew install beck.
4. Verify curl install.
5. Hand back to main agent for Show HN sequencing.

---

## 7. Forced decisions already made (do not relitigate)

- P4: FTS5-only, gated by Phase 0 eval. See section 11 of the engineering review.
- P7-remainder: beck mcp ships BOTH MCP tools AND MCP resources.
- Duplicate skill name policy: last-wins by config order, warn.
- Binary size budget: <6MB stripped (pending Esteban sign-off; default to <6MB).
- Frontmatter schema: name + description + tags, unknown fields ignored.
- Cross-platform: macOS Intel + Apple Silicon + Linux glibc + Linux musl. Windows v0.2.

---

## 8. Hard requirements

- Zero network calls at runtime.
- Single binary (beck). MCP behind default-on cargo feature.
- Cold start <50ms for non-mcp commands.
- Hot query <50ms.
- Sync 100 skills <5s cold, <500ms no-op.
- Binary size <6MB stripped.
- Every command supports --json.
- Every command snapshot-tested.
- Phase 0 eval must pass before Phase 1 starts.

---

## 9. Non-goals (resist scope creep)

No GUI, no daemon, no HTTP server, no embeddings in v0, no Windows in v0, no telemetry, no skill execution, no custom skill format, no auto-updater, no shared multi-user DB.

---

## 10. Success criteria

- cargo install beck succeeds on a clean macOS or Linux machine in <5s.
- beck sync completes in <5s for 100 skills.
- beck query returns the expected skill in top-3 for >=85% of the 50-query eval set.
- beck mcp passes the integration test (initialize, tools/list, tools/call, resources/list, resources/read).
- Binary size after strip is <6MB.
- README has the comparison table, install one-liner, demo GIF placeholder, trust model and privacy paragraphs.
- v0.1.0 tag, GitHub release, crates.io publish, homebrew tap formula all green.

---

## 11. Where to read more

1. CEO plan (authoritative scope): ~/.gstack/projects/beck/ceo-plans/2026-04-07-beck.md
2. Engineering review (your design doc): ~/.gstack/projects/beck/plan-eng-review-20260407.md
3. Origin story / research / stakes: ~/Projects/beck/CONTEXT.md
4. Office-hours design doc (forcing-question reasoning): ~/.gstack/projects/beck/estebannunez-main-design-20260407-2110.md

End of handoff v2.
