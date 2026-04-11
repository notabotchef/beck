# beck + Claude Code

This is the per-adapter notes file for the `claude-code` adapter that ships
in beck v0.2. If you are writing documentation for another agent, copy this
file as a template and fill in the agent-specific sections.

## Target location

| OS | Path |
|----|------|
| macOS | `~/.claude/skills/<name>/SKILL.md` |
| Linux | `~/.claude/skills/<name>/SKILL.md` |
| Windows | not supported in v0.2 |

Both OSes use the same layout. The dir `<name>` is the same as the folder
name under `~/beck/skills/<name>/`. If your canonical source is at
`~/beck/skills/caveman/SKILL.md`, beck installs
`~/.claude/skills/caveman/SKILL.md` as a symlink back to it.

## Install mode

Symlink only. No format transform. Claude Code reads the canonical SKILL.md
byte-for-byte through the symlink, which means:

- You can edit `~/beck/skills/caveman/SKILL.md` directly and Claude Code
  sees the new content the next time it reads the file.
- There is no drift between the source and what the agent sees. `sha256`
  in the manifest is the sha of the source bytes; beck never copies.
- If your filesystem cannot create symlinks (iCloud sync on some setups,
  exotic FS flags), v0.2 will error out. Copy-mode is on the v0.3 roadmap.

## Detection

`beck check` considers Claude Code "detected" if `~/.claude/` exists.
A bare `~/.claude/` directory counts: you do not need the `skills/`
subdirectory to already exist. `beck link` will create it.

## Hot reload

Claude Code reads skills at agent start time. After running `beck link`
for the first time, you may need to restart the agent for it to pick up
the new file. Subsequent edits to the canonical source do not require a
restart for the next agent session to see them (they are re-read on each
session start).

## Gotchas

1. **Absolute path required for the MCP binary.** Unrelated to the `link`
   command, but mentioned because new users hit it: if you also want the
   MCP router, use the absolute path to `beck` in your Claude Code
   MCP config. Claude Code's spawned subprocess does not always inherit
   your shell PATH.
2. **Namespaced skills use hyphens in v0.2.** A skill named
   `gstack/benchmark` under `~/beck/skills/` would install as
   `~/.claude/skills/gstack/benchmark/SKILL.md` in Claude Code's native
   nested layout, but beck v0.2 treats the skills home as flat.
   Workaround: name the folder `gstack-benchmark` instead. Nested
   namespaces land in v0.3 when we wire per-agent name translation.
3. **iCloud-synced home.** If `~/beck/skills/` lives under an
   iCloud-synced directory, every `beck link` creates a symlink and
   iCloud may try to resolve it as a real file. Prefer a non-synced home
   (the default `~/beck/` in your real home directory) for beck's
   canonical source tree.

## Manual smoke test

```bash
# Start from a clean slate.
beck bootstrap

# Create a skill.
mkdir -p ~/beck/skills/hello-beck
cat > ~/beck/skills/hello-beck/SKILL.md <<'EOF'
---
name: hello-beck
description: simple test skill for the beck adapter
---

# hello-beck

If you can read this, the beck link adapter is working.
EOF

# Link it in.
beck link --agent claude-code

# Verify the symlink.
ls -la ~/.claude/skills/hello-beck/
#   lrwxr-xr-x  SKILL.md -> /Users/you/beck/skills/hello-beck/SKILL.md

# Reading through the symlink should give you the original body.
cat ~/.claude/skills/hello-beck/SKILL.md

# Open Claude Code, ask the agent to list your skills. `hello-beck`
# should be among them.

# Clean up.
beck unlink --skill hello-beck
```

## Reverse ingest

If you already have hand-written SKILL.md files at `~/.claude/skills/`,
beck can pull them into the canonical tree:

```bash
# Dry-run first.
beck sync --from claude-code

# Execute.
beck sync --from claude-code --write
```

Ingest skips every entry whose symlink already points back into
`~/beck/skills/` (those skills originally came FROM beck). It will only
pick up hand-written files the user created directly in the Claude Code
dir. Conflicts on the canonical side are reported and require `--force`
to overwrite.
