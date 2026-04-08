---
name: import-external-agent-tools
description: Import skills and workflows from external AI tool repos (Rune, oh-my-openagent, etc.) into the Hermes skill library. Covers compiler-based imports (Rune), direct SKILL.md extraction, and selective filtering.
tags: [hermes, skills, rune, import, setup]
---

# Importing External Agent Tools into Hermes

Use when a user points you at a GitHub repo containing an agent framework, skill library, or workflow system and wants to "install" or "learn from" it.

## Step 1: Detect what kind of repo it is

```bash
ls <repo>/
cat <repo>/README.md | head -40
```

Look for:
- A compiler/build system (Rune has `compiler/bin/rune.js build`)
- Raw SKILL.md files (oh-my-openagent `.opencode/skills/`)
- Slash command definitions (`.claude/commands/`, `.opencode/command/`)
- Platform-specific formats (Claude Code, Cursor, Windsurf, OpenCode)

## Step 2: Compiler-based repos (e.g. Rune)

If the repo has a build system with a `--platform generic` or equivalent target:

```bash
node compiler/bin/rune.js --help   # check available platforms
node compiler/bin/rune.js build --platform generic --output /tmp/rune-generic
ls /tmp/rune-generic/.ai/rules/    # compiled output
```

Then selectively import the skills you actually need — don't import everything:

```python
import os

src = "/tmp/rune-generic/.ai/rules"
dst_base = os.path.expanduser("~/.hermes/skills/rune")
os.makedirs(dst_base, exist_ok=True)

core = ["rune-cook", "rune-team", "rune-debug", "rune-plan",
        "rune-sentinel", "rune-launch", "rune-scaffold", "rune-perf",
        "rune-review", "rune-brainstorm", "rune-completion-gate",
        "rune-ba", "rune-db"]

for skill in core:
    src_file = os.path.join(src, f"{skill}.md")
    if not os.path.exists(src_file):
        continue
    dst_dir = os.path.join(dst_base, skill)
    os.makedirs(dst_dir, exist_ok=True)
    with open(src_file) as f:
        content = f.read()
    if not content.startswith("---"):
        name = skill.replace("rune-", "")
        header = f"---\nname: {skill}\ndescription: Rune {name} workflow\ntags: [rune, {name}]\n---\n\n"
        content = header + content
    with open(os.path.join(dst_dir, "SKILL.md"), "w") as f:
        f.write(content)
```

## Step 3: Raw SKILL.md repos (e.g. oh-my-openagent)

Find all SKILL.md files and evaluate each one:

```bash
find <repo> -name "SKILL.md" | sort
```

For each, read the frontmatter `description` to decide if it's worth importing. Import selectively — only what's genuinely reusable. Copy to `~/.hermes/skills/<category>/<skill-name>/SKILL.md`. Include supporting scripts if present.

## Step 4: Extract non-skill learnings

Beyond skills, look for:
- `docs/guide/` — architecture and model-matching insights worth saving to memory
- Agent role definitions — useful for understanding how to delegate tasks
- Anti-patterns sections — worth noting

Save durable insights to memory, not as skills.

## Step 5: Verify import

```python
# Use mcp_skills_list with category filter to confirm
skills_list(category="rune")  # or whatever category you used
```

## Pitfalls

- Rune's compiler ignores `--output` and always writes to `.ai/rules/` inside the repo dir. Check there, not the output path.
- Don't import everything — be selective. Utility/internal skills from external repos rarely map to Hermes workflows.
- Skills with platform-specific tool names (task(), background_output(), etc.) need mental mapping to Hermes equivalents (mcp_delegate_task, mcp_process).
- Pre-publish / npm-specific skills are usually not worth importing unless the user publishes npm packages.

## Working directory note

Always run compiler commands from inside the cloned repo directory.
