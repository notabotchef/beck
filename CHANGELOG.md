# Changelog

All notable changes to beck are listed here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and beck tries to follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-04-11 (beck-link)

Additive release. Every v0.1 command and behavior is untouched.

### Added

- `beck bootstrap` creates `~/beck/skills/` and a fresh
  `~/beck/.beck-manifest.json`. Idempotent on re-run.
- `beck link [--agent <name>] [--dry-run] [--force] [--json]` installs
  every skill under `~/beck/skills/` into every detected agent's
  native skills directory. Transactional per skill: a failure on a
  later adapter rolls back earlier adapters for the same skill. The
  manifest is saved atomically once at the end.
- `beck unlink [--skill <name>] [--agent <name>] [--all] [--json]`
  removes beck-installed files. Manifest-driven. Refuses without an
  explicit scope to prevent accidental mass deletes. Foreign files at
  the adapter's target are never touched.
- `beck check [--rebuild-manifest] [--prune] [--json]` is the
  diagnostic command. Detects every agent, classifies files at each
  target root as beck-managed, foreign, or orphan, flags
  case-insensitive name collisions in the skills home, and reports
  manifest health (Ok, Missing, Corrupt, VersionUnsupported).
- `beck sync --from <agent> [--write] [--force]` reverse-ingest:
  walks an agent's native skills directory and pulls handwritten
  skills into `~/beck/skills/`. Dry-run by default. Conflict-aware:
  refuses to overwrite an existing canonical source whose bytes
  differ from the ingested copy unless `--force` is passed.
- `ClaudeCodeAdapter` is the first (and, in v0.2, only) adapter.
  Symlink install at `~/.claude/skills/<name>/SKILL.md`. No format
  transform: Claude Code reads the canonical SKILL.md byte-for-byte.
- Manifest schema v1 at `~/beck/.beck-manifest.json`. Atomic writes
  via tmp + rename.
- `Adapter` trait in `src/agents/adapter.rs` with seven methods:
  `name`, `detect`, `target_root`, `plan`, `install`, `uninstall`,
  and two Phase 5 additions, `list_managed` and `rebuild_entry`.
  Default impls for `ingest`, `list_managed`, and `rebuild_entry`
  so new adapters can land without rewriting the trait.

### Changed

- `beck sync` now has two modes. Default behavior (no `--from`)
  walks the configured roots and rebuilds the SQLite FTS5 index
  exactly like v0.1. Pass `--from <agent>` to flip into reverse
  ingest.
- `ClaudeCodeAdapter::uninstall` verifies that the symlink at the
  target resolves under `beck_home()?/skills/` before removing it.
  This is an accuracy fix over the old string-component heuristic
  and also means beck will never remove a symlink a user manually
  repointed to their own file.

### Deferred

- Cursor adapter is deferred to v0.3. Phase 0 research confirmed
  that Cursor has no user-global rules directory: every rule lives
  under a specific project's `.cursor/rules/`. Installing
  beck-managed rules globally for Cursor would require a different
  UX (per-project install mode, or waiting for Cursor to ship a
  user-global rules dir).
- Windsurf, Cline, OpenCode, and Continue adapters are demand-gated
  v0.3 candidates. None are researched yet.

### Hard guarantees this release does NOT break

- Every v0.1 command keeps working. Nothing renamed, nothing
  removed.
- `beck sync` without `--from` is byte-equivalent to v0.1.
- MCP tool schema (`skills_query`, `skills_load`) is unchanged.
- Binary size stays under the 6 MB cap. v0.2 ships at 2.14 MB
  stripped on Apple Silicon, up from 2.0 MB in v0.1.

## [0.1.0] - 2026-04-10

First production release. MCP router, `beck sync / list / query /
load / prompt / bench / mcp`. Release binaries for Linux (gnu +
musl, x86_64 + aarch64) and macOS (Intel + Apple Silicon).
