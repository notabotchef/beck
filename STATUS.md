# beck - session status (2026-04-08)

Session owner: Claude Opus 4.6. Esteban offline.

## TL;DR

Phase 0 eval gate **passes at 98% top-3 recall** on pure FTS5.
GitHub repo is live. Crates.io reservation is **blocked on one paste**
from you (cargo login token). Nothing is ambiguous, nothing is half-done.
Stopping here per instructions. Greenlight Phase 1 when you are back.

## What shipped

### 1. GitHub repo created
- **URL:** https://github.com/Nunezchef/beck
- **Branches pushed:** `main`, `feat/beck-mvp`
- **Deviation from your instructions:** you asked for
  `estebannunez/beck`. The active `gh` account on this machine is
  `Nunezchef`, and `gh repo create estebannunez/beck` failed with
  `Nunezchef cannot create a repository for EstebanNunez`. I took your
  explicit fallback path (`gh repo create beck --public --source . --push`)
  which created it under `Nunezchef/beck`. If you want it under an
  `estebannunez` namespace instead, you have three options:
  1. Transfer the repo via GitHub UI (Settings -> Transfer ownership).
  2. Create a new `estebannunez` GitHub user/org and re-run.
  3. Live with `Nunezchef/beck`.
  CLAUDE.md, CONTEXT.md, and the ceo-plan still reference
  `estebannunez/beck` in various places. I did NOT touch them (not in
  scope for this session). Flag this when you decide on the namespace.

### 2. crates.io name reservation (placeholder 0.0.1)
- **Branch:** `chore/crates-reservation` (merged ff-only into `main`,
  pushed, commit `b0fac2f`).
- **Files added:** `Cargo.toml`, `src/lib.rs`, `README.md`, `.gitignore`.
  Note: `src/lib.rs` was removed on `feat/beck-mvp` when the workspace
  layout was introduced. The placeholder package layout only lives on
  `main` until you are ready to merge the real code.
- **Cargo.toml.repository** points at
  `https://github.com/Nunezchef/beck` (not `estebannunez/beck`) because
  the repo was created there. Change this before the real publish if
  the namespace moves.
- **Dry run:** `cargo publish --dry-run --allow-dirty` was **clean**.
  Log tail:
  ```
  Packaging beck v0.0.1 (...)
  Packaged 9 files, 30.5KiB (13.2KiB compressed)
  Verifying beck v0.0.1 (...)
  Finished `dev` profile ...
  Uploading beck v0.0.1 (...)
  warning: aborting upload due to dry run
  ```
- **Real publish: BLOCKED.** No `~/.cargo/credentials.toml`. Per your
  instructions, I stopped here instead of guessing. To reserve the name,
  paste this in a terminal:
  ```
  cargo login
  ```
  then paste the API token from https://crates.io/me (requires GitHub
  sign-in). After that, from `~/Projects/beck` on `main`:
  ```
  cargo publish
  ```
  This will irrevocably claim `beck` on crates.io at version 0.0.1. The
  real `0.1.0` can land on top later with the full workspace code.
- **New dependency I had to install to get this far:** `rustup` and
  `cargo` were not present on this machine. I installed the official
  Rust toolchain via `curl --proto '=https' --tlsv1.2 -sSf
  https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile
  minimal`. Default profile, minimal, $HOME-local. Non-destructive. Added
  `stable-aarch64-apple-darwin` (rustc 1.94.1). Remove with
  `rm -rf ~/.cargo ~/.rustup` if undesired. You need to `source
  "$HOME/.cargo/env"` in new shells until your shell init picks it up.

### 3. Phase 0 eval gate (the number you wanted)

**Branch:** `feat/beck-mvp`, commit `e8c6b84`.

**Fixture corpus:** 118 real `SKILL.md` files copied from
`~/.hermes/skills/` (excluding the mirrored copies under `.cursor/`,
`.opencode/`, `.agents/`, `.slate/`, `.factory/`, `.openclaw/`). Scanned
for secrets (api keys, bearer tokens, passwords, private keys). Four
hits were all placeholder/documentation text, not real secrets
(`test123`, `ghp_xxxxxxxxxxxxxxxxxxxx`, `sk-xxxxxxxxxxxxxxxxxxxx`,
`<password>` example). Safe to commit. 118 is slightly over the 50-100
target but more data = stronger signal. I did not mix in gstack mirror
skills under `~/.claude/skills/gstack` because they are all variants of
each other and would inflate the dataset with synthetic duplicates.

**Query set:** 50 queries in `tests/eval/queries.toml`.
- 20 exact-keyword
- 20 paraphrase
- 10 adversarial (short words, typos, tempting-but-wrong distractors)
Every `expected_top1` is the real frontmatter `name:` of a committed
fixture. I initially had 8 mismatches where I used the directory name
instead of the frontmatter name; the first eval run flagged them as
false misses (the retrieval was actually correct). I fixed all 8 in
the same commit.

**Harness:** `crates/beck-core/src/bin/eval.rs`, gated behind the
`eval` cargo feature. Run with:
```
cargo run -p beck-core --features eval --bin eval
```
Builds an in-memory SQLite + FTS5 index, walks the fixtures once via
`walkdir`, parses YAML frontmatter, upserts with the last-wins dup
policy, then runs every query through BM25 with per-column weights
name=4.0, description=2.0, tags=1.5, body=1.0 (matches HANDOFF Phase 2
spec).

**Results:**

```
indexed 118 skills from tests/fixtures/skills
running 50 queries

=== Phase 0 eval results ===
top-1 recall: 46/50 = 92.0%
top-3 recall: 49/50 = 98.0%

by category:
  adversarial  n=10  top-1=100.0%  top-3=100.0%
  exact        n=20  top-1= 95.0%  top-3=100.0%
  paraphrase   n=20  top-1= 85.0%  top-3= 95.0%

gate: top-3 >= 85% required to proceed FTS5-only
VERDICT: PASS. Proceed to Phase 1 as planned.
```

**Only miss (1/50):** `"schedule a todo on my iphone"` expected
`apple-reminders`. The apple-reminders description is
`"Manage Apple Reminders via remindctl CLI (list, add, complete,
delete)."` which shares zero terms with the query. This is the exact
case where hybrid embeddings would win, and the exact case where plan-
eng-review said "top-3 65-84% means pivot narrative." We are at 98%,
nowhere near that cliff. A single pure-paraphrase miss on 50 is within
expected FTS5 ceiling.

**Gate decision per plan-eng-review section 11:** GO, FTS5-only,
no narrative pivot, no embeddings in v0. The 85% bar is cleared by
13 points. v0.2 can still add opt-in ONNX embeddings if you want; this
number just says you do not need them to launch.

### 4. Binary size decision
Locked at **<6MB stripped** per your default. Framing: "smaller than
ripgrep" (rg is ~5.6MB on Linux, ~6.2MB on macOS). No action needed
from you; this is noted in HANDOFF.md already.

## What got built (concrete files)

```
Cargo.toml                              workspace root, opt-z/lto=fat/strip
crates/beck-core/Cargo.toml             deps from plan-eng-review section 3
crates/beck-core/src/lib.rs             module reexports
crates/beck-core/src/frontmatter.rs     YAML parser, name/description/tags
crates/beck-core/src/db.rs              in-memory SQLite + FTS5 schema + triggers
crates/beck-core/src/sync.rs            walkdir, parse, upsert, last-wins dup
crates/beck-core/src/query.rs           BM25 ranked search with col weights
crates/beck-core/src/bin/eval.rs        Phase 0 harness (feature-gated)
tests/fixtures/skills/*/SKILL.md        118 fixtures
tests/eval/queries.toml                 50 queries
.gitignore                              /target, .DS_Store
```

Total: 129 files, ~36k lines (most of it the fixture corpus itself).

## What did NOT get done (by design)

Per your instruction to stop at Phase 0: no production FTS5 schema on
disk, no `beck sync` CLI, no `beck-cli` crate, no MCP server, no prompt
command, no bench command, no CI workflow, no Homebrew tap, no
release.yml, no README. All of that is Phase 1+ from HANDOFF.md and
waits for your greenlight.

I also did not touch `CLAUDE.md`, `CONTEXT.md`, `HANDOFF.md`, or the
ceo-plan, even where they reference the wrong namespace
(`estebannunez/beck` vs actual `Nunezchef/beck`). Out of scope for
this session. Fixable in a single session once you decide on the
final namespace.

## Git state when this was written

```
main              b0fac2f  chore: reserve crates.io name with placeholder beck 0.0.1
feat/beck-mvp     e8c6b84  feat(phase-0): eval gate passes at 98% top-3 recall
chore/crates-reservation  b0fac2f  (already merged to main)
review/plan-eng-review-20260407  b1a4c75  (historical, can be deleted)
```

Both `main` and `feat/beck-mvp` are pushed to
`https://github.com/Nunezchef/beck`.

## Blockers for you to unblock

1. **Paste a cargo token.** Run `cargo login`, paste the crates.io API
   token from https://crates.io/me, then `cd ~/Projects/beck && git
   checkout main && cargo publish`. This reserves the name. Takes 2
   minutes. Do it before any typosquatter sees the GitHub repo.

2. **Decide on the GitHub namespace.** Stay at `Nunezchef/beck`, or
   transfer to an `estebannunez` account. The decision ripples into
   `Cargo.toml.repository`, `README.md`, and every doc that mentions
   the repo URL. If you stay at Nunezchef, zero changes needed; if you
   move, a single grep-and-replace session fixes it.

3. **CLAUDE.md / CONTEXT.md namespace cleanup** (low priority). Five
   files mention `estebannunez/beck`. Not urgent.

## Phase 1 preview (do not start without greenlight)

Per `HANDOFF.md` section 6, Phase 1 is:
1. Create the `beck-cli` crate alongside `beck-core`, wire up `clap`
   derive.
2. Replace in-memory DB with on-disk at
   `directories::ProjectDirs::from("dev","beck","beck")`.
3. Implement `beck sync` end-to-end against the real
   `~/.hermes/skills` + `~/.claude/skills` roots.
4. `beck query` + `beck list` + `beck load`.
5. Re-run the eval against the persisted DB; must still pass >=85%
   top-3.

Estimated effort from here: 1-2 days with focused sessions. Phase 4
(`beck mcp`) is the scary one, everything else is well-trodden Rust.

## One-command verification for future you

From repo root, in a shell where rust is on PATH:
```
source "$HOME/.cargo/env" && \
  cd ~/Projects/beck && \
  git checkout feat/beck-mvp && \
  cargo run -p beck-core --features eval --bin eval
```
Expected last lines:
```
top-3 recall: 49/50 = 98.0%
VERDICT: PASS. Proceed to Phase 1 as planned.
```

End of status.
