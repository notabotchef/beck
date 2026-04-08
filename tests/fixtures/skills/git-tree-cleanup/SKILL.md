---
name: git-tree-cleanup
description: "Untangle a messy git working tree with mixed uncommitted changes from multiple sessions/agents. Audit, categorize, selectively commit in logical groups, and restore a clean main branch. Use when git status shows dozens/hundreds of dirty files from different workstreams."
tags: [git, cleanup, triage, branches, stash, reorg]
triggers:
  - "git tree is a mess"
  - "clean up the repo"
  - "too many uncommitted changes"
  - "working tree is dirty"
  - "lost track of changes"
  - "multiple agents left uncommitted work"
---

# Git Tree Cleanup

Restore a clean git tree when multiple sessions or agents have left a mess of uncommitted changes, dead worktrees, and stale branches.

## When to Use

- `git status` shows dozens or hundreds of dirty files
- Changes span multiple concerns (docs, code, config, plugins)
- Previous sessions didn't commit or close properly
- Agent worktrees or branches are abandoned

## Step-by-Step

### 1. Audit the Full State

Gather everything before touching anything:

```bash
git status
git branch -a --sort=-committerdate | head -30
git log --oneline --all --graph -20
git stash list
git worktree list
```

Count the damage:
```bash
git status --short | wc -l                    # total dirty files
git status --short | grep '^ D' | wc -l      # deletions
git status --short | grep '^??' | wc -l      # untracked
git status --short | grep '^ M' | wc -l      # modified
```

### 2. Check for Prior Session Context

Use `session_search` to find what the user was doing when the mess was created. This tells you whether deletions are intentional (e.g., a file reorg where files were moved, not nuked).

### 3. Verify File Moves (if reorg detected)

When files appear deleted + new files appear in a different structure, verify nothing was lost:

```python
# Compare basenames of deleted files vs new files
# Account for filename quoting issues with spaces/special chars
# Files with spaces show differently in git status vs filesystem
```

**PITFALL**: `git status --short` quotes filenames with spaces. Use `find` to list new files and compare basenames. A few "orphans" may just be quoting mismatches — verify with `find . -name "filename"`.

### 4. Prune Dead Worktrees

```bash
git worktree prune -v
```

Delete branches that pointed to pruned worktrees:
```bash
git branch -d <dead-branch-name>
```

### 5. Separate Changes by Category

Classify each dirty file into logical groups:
- **docs** — documentation moves, READMEs, reorgs
- **infra** — docker, env, config
- **frontend** — UI components, hooks, config
- **backend** — API handlers, extensions, prompts
- **plugins** — new or modified plugins
- **state** — .rune/, progress files

### 6. Create a Clean Branch and Stage Selectively

**IMPORTANT**: Do NOT use `git checkout stash -- <path>` to selectively apply from a stash — it only restores tracked file modifications, NOT untracked new files (which are stored in a separate stash tree). This will silently miss entire new directories.

**Correct approach** — pop the stash fully, then selectively stage:

```bash
# Stash everything
git stash push -u -m "WIP: mixed changes"

# Create clean branch off main
git checkout main && git pull origin main
git checkout -b chore/cleanup

# Pop stash (restores all files)
git stash pop

# Stage by category and commit
git add docs/
git commit -m "chore(docs): description"

git add frontend/ docker-compose.dev.yml
git commit -m "fix: frontend connectivity"

# ... etc for each logical group
```

### 7. Handle Submodule Dirt

If a submodule shows as dirty:
```bash
cd engine/submodule-name
git status --short          # see what changed inside
git add . && git commit     # commit inside submodule first
cd ../..
git add engine/submodule-name
git commit -m "fix: update submodule — description"
```

### 8. Handle Nested Git Repos

Plugins or tools with their own `.git` directory will show as `modified (untracked content)`. This is cosmetic — it means the nested repo has uncommitted files. Either:
- Commit inside the nested repo, or
- Ignore it (harmless in the parent)

Do NOT try `git add` on a nested repo — it won't stage the contents.

### 9. Merge to Main and Push

```bash
git checkout main
git merge chore/cleanup --no-ff -m "Merge cleanup: description"
git push origin main
```

### 10. Clean Up Stale Branches

```bash
git branch -d <merged-branch>       # delete merged branches
git stash drop stash@{N}            # drop old stashes if safe
```

Keep feature branches that have unmerged work. Delete everything else.

### 11. Update Progress

Update `.rune/progress.md` (or equivalent) with what was committed, what branches remain, and what's next.

## Pitfalls

1. **`git checkout stash -- path/` misses untracked files** — Always pop the full stash, then selectively stage. The untracked files in a stash are stored in a separate tree that `checkout` doesn't access.

2. **Filename quoting with spaces** — `git status` wraps filenames with special characters in quotes. When comparing deleted vs new files, use `find` on the filesystem for the new files and `os.path.basename` matching.

3. **Submodule "-dirty" suffix** — A submodule commit hash showing as `abc1234-dirty` means the submodule has local changes. Commit inside the submodule first, then update the parent's reference.

4. **Don't commit .env changes carelessly** — Review `usr/.env` or any env file diffs before staging. They may contain secrets or machine-specific paths.

5. **Check session history first** — Deletions that look alarming (100+ files) may be intentional moves from a reorg session that didn't finish committing.

6. **Multi-agent repos accumulate invisible debt** — When tools like Paperclip or autonomous agents work on a repo, they often don't close sessions cleanly. Expect: partial reorgs, mixed concerns on one branch, stale worktrees from dead agent sessions. Always audit before assuming the worst.

7. **`git add docs/` after a move detects renames** — When files were moved (not deleted), staging the entire directory at once lets git detect renames with similarity scores. This produces a much cleaner commit than staging deletes and adds separately.
