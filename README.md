# beck

*your agent's skills, at its beck and call.*

[![crates.io](https://img.shields.io/crates/v/beck.svg)](https://crates.io/crates/beck)
[![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![build](https://img.shields.io/github/actions/workflow/status/notabotchef/beck/ci.yml?branch=main)](https://github.com/notabotchef/beck/actions)

## The pitch

beck is a single static Rust binary that indexes `SKILL.md` files on disk and
serves the right one on demand, both as a CLI and as an MCP server. The
problem it solves is simple. Today, AI agents inject the full body (or a long
description) of every skill you have into their system prompt on every single
turn, because that is the only way the agent "knows" the skill exists. A
power user with 300 skills burns north of 100,000 tokens per turn on a catalog
the agent mostly ignores.

beck replaces that eager-load pattern with query-on-demand. The agent calls
`skills_query` when it wants a skill, gets back three BM25-ranked names, then
calls `skills_load` on the one it actually needs. MCP session-start cost is
**a flat ~200 tokens regardless of how many skills you have**. On my machine,
with 547 skills indexed, `beck bench` reports **~21,000 tokens saved per agent
turn, 99% of the baseline**.

Zero network calls. Zero telemetry. Zero daemons. One 2.0 MB binary.

## Demo

```bash
$ beck sync
indexed 368 skills into ~/Library/Application Support/beck/skills.db
  118  /Users/you/.hermes/skills
  250  /Users/you/.claude/skills

$ beck bench
beck saves you ~10833 tokens per agent turn (98% of the baseline)
  skills indexed:              153
  baseline inject-all tokens:  11033
  beck MCP session tokens:     200  (flat)

$ beck query "transcribe audio"
whisper
  OpenAI's general-purpose speech recognition model. Supports 99 languages...
audiocraft-audio-generation
  PyTorch library for audio generation including text-to-music (MusicGen)...
songsee
  Generate spectrograms and audio feature visualizations (mel, chroma, MFCC...

$ beck load whisper
# Whisper - Robust Speech Recognition

OpenAI's multilingual speech recognition model.

## When to use Whisper
...
```

That transcript is real output from my actual machine against my actual skill
library. An animated screencast will land here before the v0.1.0 tag.

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/notabotchef/beck/main/scripts/install.sh | sh
cargo install beck                                   # from source via crates.io
brew install notabotchef/beck/beck                   # coming soon
```

The shipped binary is **2.0 MB stripped on Apple Silicon**, smaller than `fd`
(~2.4 MB), smaller than `bat` (~3.5 MB), and a fraction of `ripgrep`.

## How to use it

beck has two personalities: the **MCP router** (v0.1) and the **universal
skills directory** (v0.2). The MCP router is the query-on-demand path that
kills the token cost. The universal directory is the opposite direction: it
is the single place you edit your skills, and beck installs them into every
agent that speaks its own format.

### v0.1 commands (MCP router)

```
beck sync                index ~/.hermes/skills and ~/.claude/skills into the local db
beck list                show every indexed skill, one per line
beck query "<text>"      rank matching skills by BM25
beck load <name>         print the full body of a skill
beck prompt              print the agent integration stub to paste into a system prompt
beck bench               estimate how many tokens beck saves you per agent turn
beck mcp                 start the MCP server on stdio (for Claude Code, Claude Desktop, Cursor, Codex)
```

`beck sync` on my machine indexes 153 unique SKILL.md files in **805 ms cold**.
Hot `beck query` calls return in **4 ms**. Everything lives in a single
SQLite file with an FTS5 index.

### v0.2 commands (universal skills directory)

v0.2 adds four commands. Write your skill once at
`~/beck/skills/<name>/SKILL.md`, run `beck link`, and beck installs it into
every agent you have (Claude Code today, more coming).

```
beck bootstrap           create ~/beck/skills/ and the manifest
beck link [--agent N]    install every skill into every detected agent
beck unlink --all        remove every beck-installed file (foreign files are safe)
beck check [--json]      diagnose agents, orphans, foreign files, case collisions
beck sync --from claude-code --write    reverse-ingest skills you already wrote into ~/beck/skills/
```

Quick start for v0.2:

```bash
$ beck bootstrap
initialized beck home at /Users/you/beck
  skills: /Users/you/beck/skills
  manifest: /Users/you/beck/.beck-manifest.json

$ mkdir -p ~/beck/skills/caveman && cat > ~/beck/skills/caveman/SKILL.md <<'EOF'
---
name: caveman
description: ultra-compressed communication mode
---

# caveman

Drop articles, drop pronouns, keep verbs. Speak as few words as possible.
EOF

$ beck link
linked 1 targets:
caveman
  claude-code -> /Users/you/.claude/skills/caveman/SKILL.md

$ beck check
detected agents: claude-code
manifest: ok
beck-managed files: 1
```

One source of truth (`~/beck/skills/`), every agent gets a copy in its
native format. `beck link` is idempotent: running it twice reports "skipped"
because the source sha256 has not drifted. `beck unlink --all` only removes
files beck installed and will never touch a file the user wrote by hand.

### Supported agents in v0.2

| Agent | Target on macOS / Linux | Install mode | Status |
|-------|-------------------------|--------------|--------|
| Claude Code | `~/.claude/skills/<name>/SKILL.md` | Symlink | shipping |
| Cursor | per-project `.cursor/rules/` only | deferred (no user-global rules dir) | v0.3 candidate |
| Windsurf | TBD | not researched | v0.3 candidate |
| Cline | TBD | not researched | v0.3 candidate |

Cursor dropped out of v0.2 because there is no user-global rules directory:
every Cursor rule lives in a specific project. beck installs globally by
design, so Cursor lands in v0.3 once we ship a per-project install mode or
Cursor ships a global rules location. Track the decision in
`.rune/plan-beck-link-spec.md` section 0.

### Uninstalling cleanly

`beck unlink --all` is the whole uninstall story on the agent side. It reads
the manifest at `~/beck/.beck-manifest.json`, walks every entry, and removes
the file beck installed. Files the user wrote by hand are never touched.
Run `beck check` afterward to confirm the expected state.

```bash
$ beck unlink --all
unlinked 1:
  caveman/claude-code -> /Users/you/.claude/skills/caveman/SKILL.md

$ beck check
detected agents: claude-code
manifest: ok
beck-managed files: 0
```

To remove beck itself, delete the canonical source tree too:

```bash
rm -rf ~/beck
cargo uninstall beck       # or: brew uninstall beck
```

## Why you want this

Three ways an agent can reach the same pile of skills. Token cost per turn:

| Path                                  | Tokens per turn     | Notes                                       |
|---------------------------------------|---------------------|---------------------------------------------|
| Inject everything (status quo)        | ~150,000            | 300 skills, average 500-token description   |
| beck CLI path (shell out per query)   | ~15,000             | Agent still sees a stub frontmatter line per skill, ~50 tokens x 300 |
| beck MCP path (tools only)            | **~200 (flat)**     | Two tool schemas. Flat for 5 skills or 5,000 |
| beck universal install (v0.2)         | native agent cost   | `beck link` drops files into every agent's native dir; agent loads via its own mechanism |

The CLI path is an honest improvement over the baseline, roughly 10x. The MCP
path is the wedge. Two tools, two JSON schemas, done. beck exposes no
resources precisely because `resources/list` would reintroduce the 27,000-
token session-start cost we are trying to kill. See `TODOS.md` erratum 1 if
you want the math.

Phase 0 retrieval quality on 118 real fixture skills and a 50-query test set:
**top-3 recall 98.0%, top-1 recall 92.0%**. Pure FTS5 BM25 with column
weights, no embeddings, no reranker. The 50-query set is checked in at
`tests/eval/queries.toml` and the eval harness is `cargo run --features eval
--bin eval`.

## Agent integration

Verified end-to-end against two real agents during development: **Codex**
worked on the first try, **Claude Code** worked after a bug fix, both with
screenshots in the commit log.

### Claude Code

```bash
claude mcp add -s user beck /absolute/path/to/beck mcp
```

The `-s user` flag makes the MCP server available in every project. Use the
absolute path to the `beck` binary, not just `beck`, because Claude Code's
spawned subprocess may not inherit your shell PATH.

### Claude Desktop

Edit `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "beck": { "command": "/absolute/path/to/beck", "args": ["mcp"] }
  }
}
```

Restart Claude Desktop. On a brand new install you want the absolute path to
`beck`, not just `beck`, because Claude Desktop on macOS does not inherit
your shell PATH. This is a real gotcha, not paranoia.

### Codex

Edit `~/.codex/config.toml`:

```toml
[mcp_servers.beck]
command = "/absolute/path/to/beck"
args    = ["mcp"]
```

Same absolute-path rule applies.

### Everyone else

`beck prompt` prints a short system-prompt stub you can paste into any agent
that speaks MCP but doesn't ship a config GUI. If your agent does not speak
MCP at all, `beck query` and `beck load` work fine as shell commands, and the
CLI path numbers above apply.

## Authoring skills

A `SKILL.md` file is a YAML frontmatter block followed by free markdown:

```markdown
---
name: whisper
description: OpenAI's multilingual speech recognition model.
tags: [audio, transcription, openai]
---

# Whisper - Robust Speech Recognition

Body goes here. Anything you want. Code blocks, headings, links.
```

The frontmatter keys are all optional. `name` defaults to the parent
directory name. `description` feeds the query path. `tags` are indexed but
not required. Unknown keys are ignored without warning.

beck's frontmatter schema is the minimum common denominator of Hermes,
Claude Code skills, and gstack skills. If you already have a pile of SKILL.md
files for any of those tools, they Just Work with beck, no migration needed.
Point `beck sync` at them and you are done.

By default `beck sync` walks `~/.hermes/skills` and `~/.claude/skills`. You
can add more roots via the config file (see `beck doctor` in v0.1).

## Trust model

beck is a content server, not an executor. It reads `SKILL.md` files from
disk and returns their contents. It does not run any code, evaluate any
macros, or interpret any instructions inside a skill. What the agent does
with the skill body after beck returns it is the agent's problem, and
sandboxing tool calls remains the agent's job.

beck makes zero network calls at runtime. You can verify with
`lsof -p $(pgrep beck)` or `sudo lsof -i -P | grep beck`. The only I/O is
reads against your filesystem and reads/writes against the local SQLite
database.

## Privacy

No telemetry. No phone home. No analytics. No crash reporting. No update
check. beck never opens a socket.

On macOS the database lives at
`~/Library/Application Support/beck/skills.db`. On Linux it lives at
`~/.local/share/beck/skills.db`. Deleting that file is the entire uninstall
story for the data side. Removing the binary itself (`cargo uninstall beck`
or `brew uninstall beck`) finishes the job.

## Non-goals

- No GUI.
- No daemon.
- No HTTP server.
- No embeddings in v0 (gated on the Phase 0 eval, which FTS5 passed at 98%).
- No Windows in v0 (Mac + Linux only).
- No skill execution. beck never runs code from a skill body.
- No telemetry, ever.
- No custom skill format. SKILL.md with YAML frontmatter is the whole spec.
- No auto-updater.
- No shared multi-user database. One user, one `~/Library/Application Support/beck/skills.db`.

## Roadmap

**v0.1.0** (shipped). MCP router, `beck sync / list / query / load / prompt
/ bench / mcp`, release binaries for Linux (gnu + musl, x86_64 + aarch64)
and macOS (Intel + Apple Silicon), Homebrew tap.

**v0.2.0** (shipping now). Universal skills directory. `~/beck/skills/` as
the single source of truth. New commands: `beck bootstrap`, `beck link`,
`beck unlink`, `beck check`, and `beck sync --from <agent>` for reverse
ingest. Claude Code is the one shipping adapter. Additive: every v0.1
command still works exactly the same.

**v0.3.0**. Cursor adapter (pending per-project install mode), Windsurf,
Cline, OpenCode, and Continue adapters as demand-gated. `beck-spec`
companion repo that formalizes the SKILL.md frontmatter convention so
other tools can adopt it.

**v1.0.0**. Homebrew-core merged. Schema frozen. Semver guarantee. Windows
support lands around here if the Mac + Linux story is solid.

## Contributing

Patches welcome. Issues welcome. See `CONTRIBUTING.md` for the ground rules.
The primary repo lives at **https://github.com/notabotchef/beck**, with a
backup mirror at **https://gitlab.com/knifecode/beck**.

beck is built by a former exec chef (Alinea Group, Roister) who now writes
Rust. Mise en place shows up in the review checklist: read the plan, prep the
ground, then cook. If a PR feels half-plated, expect to send it back around.

## License

MIT OR Apache-2.0, at your option.
