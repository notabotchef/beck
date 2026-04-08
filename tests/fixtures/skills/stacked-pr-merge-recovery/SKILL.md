---
name: stacked-pr-merge-recovery
description: Recover from squash-merging a stacked PR. When you merge a PR whose base is another feature branch (not main), the squash commit lands on the stacked base, NOT on main. Use this skill to detect the situation and re-merge the content into main correctly.
version: 1.0.0
author: Hermes
---

# Stacked PR Merge Recovery

## When to use

You have a stack of PRs like:
- PR #7: `feat/branch-a` -> `main`
- PR #8: `feat/branch-b` -> `feat/branch-a`
- PR #9: `feat/branch-c` -> `feat/branch-b`

You merged PR #7 into main (correct). Then you squash-merged PR #8 thinking it would land on main. **It did not.** It squashed onto `feat/branch-a`, which means `main` still doesn't have PR #8's content.

Symptoms:
- `gh pr view 8 --json state` shows `MERGED`
- `git log origin/main` does NOT contain the PR #8 commit
- `git log origin/feat/branch-a` shows a new squash commit ahead of main
- New endpoints/code from PR #8 are not visible after spinning up against main

## Detection

```bash
# Find the squash commit on the stacked base
git fetch origin
git log --oneline origin/main..origin/feat/branch-a
```

If you see commits there that match PR #8's content, you have stacked merge drift.

## Recovery

Don't try to revert and re-target #8. It's already merged on the stacked base. Open a NEW PR from the stacked base to main with just the squash commit as its delta:

```bash
gh pr create \
  --repo OWNER/REPO \
  --base main \
  --head feat/branch-a \
  --title "feat: <same title> (re-merge to main)" \
  --body "Re-merge of the squashed #8 commit from feat/branch-a into main.

Original PR: #8

[copy original PR body]"
```

This works because `feat/branch-a` is now `main` + PR #7 + PR #8's squash commit. The new PR's diff is exactly PR #8's content.

Squash-merge the new PR. Main now has the content.

## Prevention (better than recovery)

When creating a stack of PRs that all need to land in main, **set base to `main` for all of them** and accept that GitHub will show overlapping diffs. Use draft status to gate merges in dependency order. Only mark PR N+1 ready after PR N merges to main.

The "base = previous branch" stacking pattern is appropriate when:
- you want GitHub to show only the delta diff in the second PR
- you intend to merge the entire stack as one squash via the bottom PR
- you understand that squash-merging mid-stack lands on the stacked base, not main

## Don't

- Don't `git revert` the stacked merge — it makes the history confusing and the content still needs to land on main eventually.
- Don't force-push to main to "fix" it.
- Don't delete the stacked branches until you've confirmed the re-merge PRs landed on main.
- Don't cherry-pick the squash commit by hand. You lose authorship attribution and `gh pr` linkage.

## Pitfalls encountered

### Stale downstream stack diffs
If subsequent PRs in the stack (e.g. PR #9, PR #10) had `feat/branch-a` or `feat/branch-b` as their base, they may now show stale diffs in the GitHub UI after re-merging via the new PR. The fix is to retarget their base to main (`gh pr edit N --base main`) and push a rebase, or to follow the same recovery pattern for each level of the stack.

### `gh pr create --body "..."` eats backticks and angle brackets
Bash interprets ``` ` ```, `$`, `<`, `>`, and parentheses inside double-quoted strings before they reach gh. The PR body arrives mangled — markdown code spans collapse, backticks vanish, route paths like `/api/demo/<slug>` become broken glob patterns, and you get errors like `bash: /demo/test-restaurant: No such file or directory` printed before the PR even creates.

**Always one of these:**
- `gh pr create ... --body-file /tmp/pr-body.md` (write the body to a file first)
- Or fix it after creation via the API:
  ```bash
  gh api repos/OWNER/REPO/pulls/N -X PATCH -f body="$(cat /tmp/pr-body.md)"
  ```
  Note: this also works inside Python via `subprocess.run([...], capture_output=True)` which handles escaping correctly without bash in the middle.

### Draft PRs cannot be merged
If the PR was opened with `--draft`, `gh pr merge N --squash` returns `GraphQL: Pull Request is still a draft`. Mark it ready first:
```bash
gh pr ready N --repo OWNER/REPO
gh pr merge N --repo OWNER/REPO --squash --delete-branch=false
```

### Worktree collision on `git checkout main`
Recovery requires checking out main locally to verify content arrived. If a parallel worktree already holds main:
```
fatal: 'main' is already used by worktree at '/path/to/other/worktree'
```
Run `git worktree list`, decide if the parallel worktree is disposable, and either:
- `git worktree remove /path/to/other/worktree` (if it's a temporary scratch worktree like `_demo_pr_worktrees/main-merge`)
- Or `cd /path/to/other/worktree && git pull origin main` and verify there

## Mandatory live verification

Git history is necessary but not sufficient. The re-merge succeeded only when the runtime sees the new content. After every re-merge to main:

1. Spot-check a file from the original PR exists on main:
   ```bash
   git show origin/main:path/to/new/file | head -5
   ```

2. If the project uses Docker volume mounts (most dev stacks do), the running container may still see code from a stale worktree branch. See related skill `carabineros-start-local-stack` section 8 for the volume-mount drift recovery — that pattern generalizes to any Docker dev stack with bind-mounted source.

3. Hit the new endpoints/routes with curl and confirm 200 status + expected content:
   ```bash
   curl -s -o /tmp/check.json -w '%{http_code}\n' http://localhost:8080/api/<new-route>
   curl -s -o /dev/null -w '%{http_code}\n' http://localhost:8080/<new-page>
   ```

4. Don't say "PR landed" until the runtime verification passes. The completion-gate rule from gstack applies here too.

## Real-world example

In CarabinerOS (April 2026), a 4-slice stack was created:
- PR #7 `feat/demo-route-isolation` -> main
- PR #8 `feat/demo-backend-fixture-and-api` -> `feat/demo-route-isolation`
- PR #9 `feat/demo-auth-and-tutorial` -> `feat/demo-backend-fixture-and-api`
- PR #10 `feat/demo-workspace-and-sandbox-actions` -> `feat/demo-auth-and-tutorial`

After PR #7 merged to main, PR #8 was squash-merged. State afterward:
- `gh pr view 8` -> `MERGED`
- `origin/main` did NOT contain the fixture-api content
- `origin/feat/demo-route-isolation` had a new squash commit `c8a881c5` with all of PR #8's content

Recovery: opened PR #11 `feat/demo-route-isolation -> main` titled `feat: add fixture-backed demo api skeleton (re-merge to main)`. Squash-merged it. Main now had the content as commit `39f5ba69`.
