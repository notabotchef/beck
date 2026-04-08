---
name: ctl-check-the-logs
description: "User shortcut /ctl — read Docker logs, extract errors, delegate debugging to autonomous background agent on a separate branch. Keeps main context window clean."
tags: [ctl, debug, docker, logs, delegate, autonomous-agent]
triggers:
  - "/ctl"
  - "check the logs"
  - "what went wrong"
  - "debug the last error"
---

# /ctl — Check The Logs

When the user types `/ctl`, immediately:

1. **Read Docker logs** — don't ask, just do it
2. **Extract the error** — find the root cause
3. **Delegate the fix** to a background agent on its own branch
4. **Report back** with a short summary

The user wants their context window CLEAN. Never debug inline. Always delegate.

## Step-by-Step

### 1. Pull Logs (silent, fast)

```bash
docker logs carabiner-os-agent-zero-1 2>&1 > /tmp/a0_logs.txt
# Filter noise
grep -v "DeprecationWarning\|pathspec\|pattern_factory" /tmp/a0_logs.txt | \
  grep -i "error\|traceback\|exception\|fail\|ValueError\|TypeError" | tail -30
```

Also check frontend if relevant:
```bash
docker logs carabiner-os-frontend-1 2>&1 | tail -20
```

### 2. Extract & Summarize the Error

Read enough context around the error to understand root cause. Look for:
- Python tracebacks (ValueError, TypeError, ImportError)
- Tool execution failures ("Tool request must have a tool_name")
- WebSocket disconnects (origin validation, CSRF issues)
- CLI errors ("No such option", command not found)
- Frontend runtime errors (TypeError in React hooks)

### 3. Delegate to Background Agent

Spawn a `delegate_task` with:
- **Goal**: Clear description of the bug + what to fix
- **Context**: The relevant error text, file paths, architectural notes
- **Toolsets**: `["terminal", "file"]`
- **Branch rule**: Agent MUST create a branch like `fix/descriptive-name`
- **Autonomy**: Agent should use rune:research or last30days if needed
- **Verification**: Agent must verify the fix compiles/passes tests
- **Report**: Agent reports what it found, what it changed, branch name

Template:
```
delegate_task(
  goal="Debug and fix: <error summary>. Create branch fix/<name>, make the fix, verify with build/tests, commit.",
  context="<error text>\n<relevant file paths>\n<architectural notes>",
  toolsets=["terminal", "file"]
)
```

### 4. Report Back (short)

Use this format:
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  /ctl REPORT
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

ROOT CAUSE: <one line>
FIX: <what the agent changed>
Branch: <branch name>
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

Then merge the branch to main and rebuild if needed.

## Pitfalls

1. **Don't paste full logs into the main conversation** — summarize, delegate, report. The whole point is keeping context clean.

2. **Check BOTH docker-compose env vars AND usr/.env** — compose overrides .env and this has caused bugs before (ALLOWED_ORIGINS).

3. **A0 submodule files can't be committed from the parent** — if the fix is inside `engine/agent-zero/`, commit inside the submodule first, then update the submodule reference in the parent.

4. **Frontend fixes need `pnpm build` verification** — TypeScript errors won't show in dev mode but will break production builds.

5. **LLM tool-call formatting errors are NOT fixable in app code** — if A0's subordinate produces malformed JSON for a tool call, that's an LLM reliability issue. Just retry the prompt.

## When NOT to Use /ctl

- Simple config issues you can fix in 1 line (just fix it)
- Issues that need user input/decision (ask first, then delegate)
- Multiple unrelated errors (triage first, delegate each separately)
