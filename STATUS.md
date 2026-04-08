# beck - session status (2026-04-08)

## Current state, one screen

- **Phase 0:** eval gate passed at **98.0% top-3 recall** on 118 real
  fixtures and 50 queries. FTS5-only, no embeddings. Gate verdict: PASS,
  no narrative pivot.
- **Phase 1:** shipped. Single-crate Rust binary, 7 clap subcommands,
  on-disk SQLite + FTS5. **1.6 MB stripped binary.** `beck sync` indexes
  Esteban's 368 real SKILL.md files in 805 ms cold, `beck query` hot
  path is 4 ms. `beck bench` reports **~10,833 tokens saved per agent
  turn (98% of baseline)** on his actual 153-unique-skill library.
- **Phase 4 (MCP server):** NOT STARTED. The `beck mcp` subcommand is
  a stub that returns a validation error. Phase 4 implements the
  real rmcp 1.3 tools-only server (two tools: `skills_query`,
  `skills_load`, no resources per `TODOS.md` erratum 1).
- **Phase 5 (README, install.sh, release.yml, Homebrew tap):** NOT
  STARTED. Needs the OSS launch research that died to API 529s
  earlier, retry in a non-congested session.

## Canonical URLs

| | |
|---|---|
| Public repo | **https://github.com/notabotchef/beck** |
| Backup mirror | `git@gitlab.com:knifecode/beck.git` (private, pushed alongside github) |
| Published crate | https://crates.io/crates/beck (0.0.1 placeholder) |
| Profile | https://github.com/notabotchef |

The GitHub account `Nunezchef` was shadowbanned (account-flagged in
GitHub's spam filter: could push code but could not authorize OAuth
apps, and every `Nunezchef/*` URL returned 404 to logged-out traffic).
It has been replaced entirely by `notabotchef`, which is the only
public-facing identity going forward. The old `Nunezchef/beck` repo
was deleted on 2026-04-08, and the `Nunezchef` account has been
removed from the local `gh` CLI.

## Git state

```
main           0559ca9  chore(main): ignore .rune/
feat/beck-mvp  bab1e81  chore: ignore .rune/ session metrics
```

Both branches pushed to:
- `github` → `git@github.com:notabotchef/beck.git` (primary, public)
- `gitlab` → `git@gitlab.com:knifecode/beck.git` (backup, private)

There is no `origin` remote by design. `git push github` and
`git push gitlab` are explicit.

## crates.io

`beck 0.0.1` published 2026-04-08, owner `notabotchef`. The 0.0.1
crate's metadata still points `repository` at
`https://gitlab.com/knifecode/beck` because that was the URL at
publish time, and crates.io does not allow post-publish metadata
updates. When `0.1.0` is published, the repository field will have
been already corrected in-tree to
`https://github.com/notabotchef/beck` (done in this session).

**SECURITY REMINDER:** the crates.io API token Esteban pasted in
chat earlier today (`cio2Zht...`, first 7 chars shown for
identification only) must be revoked. Go to
https://crates.io/me, API Tokens, revoke it, issue a new one.
Future `cargo publish` will need the new token.

## Layout at repo root

```
Cargo.toml                single-crate, rusqlite 0.39, rmcp 1.3, edition 2024
src/
  lib.rs                  module reexports for eval + main
  main.rs                 clap derive tree, 7 subcommands, tokio::main
  consts.rs               4 identity constants (nucleo scheme)
  error.rs                CliError with distinct exit codes + JSON
  paths.rs                XDG data_dir, default roots, isolates `dirs`
  db.rs                   in_memory() + open() + count + clear + bytes
  sync.rs                 walkdir + frontmatter + last-wins dup
  query.rs                FTS5 BM25 with col weights (name 4x, desc 2x)
  frontmatter.rs          YAML parser, name/description/tags
  commands/
    sync.rs  list.rs  query.rs  load.rs  prompt.rs  bench.rs  mcp.rs
  bin/eval.rs             Phase 0 harness, gated by `eval` cargo feature
tests/
  fixtures/skills/        118 real SKILL.md files from ~/.hermes/skills
  eval/queries.toml       50 queries with known top-1
CLAUDE.md                 standing context (unchanged this session)
CONTEXT.md                origin story (unchanged this session)
HANDOFF.md                v2 build plan (small URL fix this session)
TODOS.md                  backlog + errata
STATUS.md                 this file
```

## One-liner verification

```
cd ~/Projects/beck
source "$HOME/.cargo/env"
./target/release/beck sync
./target/release/beck bench
cargo run --release --features eval --bin eval
```

Expected output:
- `indexed 368 skills into ~/Library/Application Support/beck/skills.db`
- `beck saves you ~10833 tokens per agent turn (98% of the baseline)`
- `top-3 recall: 49/50 = 98.0%   VERDICT: PASS`

## Decisions locked this session

1. **Layout:** single-crate, flat `src/`, lib+bin. Workspace from Phase 0
   is gone. Matches `mateonunez/nucleo`.
2. **Edition:** `2024`, `rust-version = 1.85`.
3. **Stack:** rusqlite 0.39 bundled, clap 4.6 derive+string, rmcp 1.3
   server, tokio 1.50 full, dirs 6, serde_yaml 0.9, walkdir 2,
   anyhow 1, thiserror 2, sha2 0.10.
4. **MCP cargo feature gate:** REMOVED. MCP is part of the wedge,
   ships in every build. Binary is 1.6 MB anyway.
5. **MCP surface v0:** tools-only (`skills_query`, `skills_load`). The
   plan-eng-review "tools + per-skill resources" decision was reversed
   because it would have cost ~27k tokens at MCP session start for a
   300-skill power user. See `TODOS.md` erratum 1.
6. **Repository URL:** `https://github.com/notabotchef/beck` in-tree,
   GitLab backup via a second remote.
7. **`beck stub` disk-rewrite idea (Esteban, 2026-04-08):** queued for
   v0.2 as an opt-in command. Full design in `TODOS.md`.
8. **Binary size target:** informally revised to `<2 MB` from `<6 MB`
   because actual release build is 1.6 MB. Launch narrative can now
   say "smaller than fd or bat" instead of "smaller than ripgrep."

## Next work

- **Phase 4:** `beck mcp` rmcp 1.3 server (tools-only). Scariest
  remaining unknown because rmcp 1.3 is 5 weeks old, but
  `mateonunez/nucleo` has a working tools-only example to crib from.
- **Phase 5:** README (with GIF), `install.sh`, `release.yml`
  (hand-rolled, nucleo two-phase flow), Homebrew tap. Needs the OSS
  launch research that died to API 529s earlier.
- **Research retries:** rerun the two background agents in a
  non-congested session and fold their findings into Phase 5.
- **Housekeeping:** token rotation (above), eventually a `0.1.0`
  publish once Phase 4 + 5 land.

End of status.
