---
name: hermes-gateway-troubleshooting
description: "Diagnose and fix Hermes gateway issues — Telegram not sending/receiving, cron jobs not firing, or both. The gateway process bundles messaging platforms + cron scheduler, so one failure kills everything."
tags: [hermes, gateway, telegram, cron, messaging, troubleshooting]
triggers:
  - "Telegram not working"
  - "cron job didn't fire"
  - "no messages"
  - "research didn't run"
  - "gateway down"
---

# Hermes Gateway Troubleshooting

## Key Insight

The Hermes gateway is a SINGLE process that runs:
- **All messaging platforms** (Telegram, Discord, Slack, WhatsApp, Signal, etc.)
- **The cron scheduler** (cron ticker)
- **Webhook listener**

If the gateway stops, ALL of these stop silently. Cron jobs report their last status as "ok" even though they simply didn't run. This is the #1 cause of "Telegram isn't working" AND "my cron job didn't fire."

## Diagnostic Steps

### 1. Check if gateway is running

```bash
ps aux | grep hermes | grep -v grep
```

Look for a gateway process. If only the CLI agent is running, the gateway is down.

### 2. Check gateway logs

```bash
# When did it stop?
tail -30 ~/.hermes/logs/gateway.log

# What errors occurred?
tail -20 ~/.hermes/logs/gateway.error.log
```

**Common causes of gateway death:**
- Telegram polling timeouts (network blip → reconnect loop → crash)
- Auth errors (401 invalid x-api-key — Anthropic key expired/rotated)
- Codex fallback not configured (warning, not fatal)
- Manual stop that was never restarted
- macOS sleep/wake cycle killing the launchd service

### 3. Check launchd KeepAlive config (ROOT CAUSE of repeated deaths)

The gateway is managed by `~/Library/LaunchAgents/ai.hermes.gateway.plist`. The upstream default ships with a **broken** KeepAlive:

```xml
<!-- BAD — only restarts on crash, NOT on clean shutdown -->
<key>KeepAlive</key>
<dict>
    <key>SuccessfulExit</key>
    <false/>
</dict>
```

The gateway catches SIGTERM and exits cleanly (exit 0), so launchd thinks "it's fine" and never restarts. The `--replace` flag, auto-updates (`git pull`), and `hermes gateway restart` all do clean shutdowns.

**Fix: unconditional KeepAlive + throttle:**

```xml
<key>KeepAlive</key>
<true/>

<key>ThrottleInterval</key>
<integer>10</integer>
```

After editing the plist:
```bash
launchctl unload ~/Library/LaunchAgents/ai.hermes.gateway.plist
launchctl load ~/Library/LaunchAgents/ai.hermes.gateway.plist
```

**WARNING**: `hermes gateway start` may overwrite the plist and revert this fix. Re-check after any Hermes update.

### 4. Restart the gateway

```bash
hermes gateway start
```

This is a launchd service on macOS. The command updates the service definition and starts it.

### 4. Verify everything reconnected

```bash
# Wait a few seconds, then check
sleep 5 && tail -20 ~/.hermes/logs/gateway.log
```

Look for these success indicators:
- `✓ telegram connected` (or other platform)
- `Cron ticker started (interval=60s)`
- Any pending cron jobs will fire immediately if overdue

### 5. Verify cron jobs are firing

```bash
# List all cron jobs
hermes cron list
```

Or from within a Hermes session:
```
cronjob(action="list")
```

Check `last_run_at` vs `next_run_at`. If `last_run_at` is stale (yesterday or older), the gateway was down during the scheduled time.

## Common Error Patterns

### Telegram TimedOut loop
```
telegram.error.TimedOut: Timed out
[Telegram] Telegram polling reconnect failed: Timed out
[Telegram] Telegram network error (attempt N/10)
```
**Cause**: Network interruption. Gateway retries 10 times with backoff, may crash after exhausting retries.
**Fix**: Restart gateway. If persistent, check network/VPN.

### 401 invalid x-api-key
```
Error code: 401 - authentication_error - invalid x-api-key
```
**Cause**: Anthropic API key expired or was rotated.
**Fix**: Update key in `~/.hermes/.env` (ANTHROPIC_API_KEY), then restart gateway.

### Fallback provider not configured
```
WARNING: openai-codex requested but no Codex OAuth token found
Fallback to openai-codex failed: provider not configured
```
**Cause**: Codex fallback is referenced but not set up.
**Fix**: Run `hermes model` to configure, or ignore if primary provider works.

## Telegram command menu only shows 3 commands (start/help/status) instead of the full set

**Symptom:** The Telegram command hint menu only shows `/start`, `/help`, `/status` — not the full ~100 Hermes slash commands. User reports the menu "looks like the Claude bot."

**Root cause:** Telegram bots register command lists at multiple scopes with precedence order:
1. `chat` (specific chat_id) — highest
2. `chat_administrators` / `chat_member`
3. `all_private_chats`
4. `all_group_chats` / `all_chat_administrators`
5. default — lowest

A more specific scope shadows a less specific one. An older Hermes version, a different gateway instance, or another process can write a minimal 3-command list to `all_private_chats`, which then shadows the full 100 commands registered at the default scope. The override persists until explicitly deleted.

**Diagnose:**
```bash
TOKEN=$(grep -E "^TELEGRAM_BOT_TOKEN" ~/.hermes/.env | head -1 | cut -d= -f2 | tr -d ' "')
# Default scope (should have the full set)
curl -s "https://api.telegram.org/bot${TOKEN}/getMyCommands" | python3 -c "import sys,json; print(len(json.load(sys.stdin)['result']),'commands')"
# Private chats scope (this is what shows in DMs)
curl -s -G "https://api.telegram.org/bot${TOKEN}/getMyCommands" --data-urlencode 'scope={"type":"all_private_chats"}' | python3 -c "import sys,json; print(len(json.load(sys.stdin)['result']),'commands')"
```

If default returns 100 but `all_private_chats` returns 3, you've found the stomper. Confirm the 3 are start/help/status.

**Fix (two layers):**
```bash
# 1. Delete the override so private chats fall back to default
curl -s -G "https://api.telegram.org/bot${TOKEN}/deleteMyCommands" --data-urlencode 'scope={"type":"all_private_chats"}'

# 2. Belt-and-suspenders: also hard-set the full list on all_private_chats so any new stomp is visible immediately
# Fetch the default list first
curl -s "https://api.telegram.org/bot${TOKEN}/getMyCommands" > /tmp/cmds.json
python3 <<'PY'
import json, subprocess, os
cmds = json.load(open('/tmp/cmds.json'))['result']
token = os.environ['TOKEN']
subprocess.run(['curl','-s','-G',
  f'https://api.telegram.org/bot{token}/setMyCommands',
  '--data-urlencode', f'commands={json.dumps(cmds)}',
  '--data-urlencode', 'scope={"type":"all_private_chats"}'])
PY
```

**Find the stomper (don't skip this — the fix alone may not hold):**
```bash
# Who is holding this bot token?
ps aux | grep -v grep | grep -iE "(telegram|gateway|bot)"

# Is there another .env file with the same token?
TOKEN_ID=$(echo "$TOKEN" | cut -d: -f1)
grep -rIl "$TOKEN_ID" ~ 2>/dev/null | grep -iE "(\.env|config\.(json|yaml|toml))" | head
```

Common stompers:
- Stale old Hermes gateway version that never cleared its override after an upgrade (the `Service definition is stale relative to the current Hermes install` warning is a strong hint)
- A second local process (e.g., Claude Code telegram plugin) accidentally sharing the same bot token
- A manual `setMyCommands` call someone made via BotFather or curl and forgot about

**Rule out Claude Code plugin first:** check `~/.claude/channels/telegram/.env` — if it contains the same `TELEGRAM_BOT_TOKEN` as `~/.hermes/.env`, they're fighting over one bot. Fix by giving Claude Code its own bot token from @BotFather. If the tokens are already different, Claude Code is NOT the stomper regardless of how suspicious the plugin process looks.

**Verify fix holds:** re-query `all_private_chats` scope 1-2 minutes later. If the count dropped back to 3, a process is actively stomping and you need to kill it or disable its command-registration step. If it's still 100, the override was stale leftover and the fix is permanent.

## Pitfalls

1. **Cron jobs show "ok" even when gateway was down** — the "ok" is from the LAST successful run, not proof it ran today. Always check `last_run_at` timestamp.

2. **Gateway logs vs agent error logs** — gateway.log shows platform connections and cron scheduling. gateway.error.log shows runtime errors during agent execution. Check both.

3. **The gateway is NOT the CLI session** — running `hermes --yolo` in a terminal is just the CLI agent. The gateway is a separate background service managed by launchd.

4. **File output from cron jobs** — even if the cron "ran," the agent inside may not have saved files correctly. Always verify the actual output files exist, not just that the job status is "ok."

5. **Telegram fallback IPs** — if you see `Telegram fallback IPs active: 149.154.167.220`, it means the primary Telegram API endpoint is unreachable and it's using a backup. This usually resolves itself but can indicate DNS/network issues.

6. **Auto-update restart loop** — `hermes gateway start --replace` does a `git pull` before starting. Each pull stops the running gateway (clean exit 0). If the last stop in a cycle has no matching restart, the gateway stays dead. Check `git -C ~/.hermes/hermes-agent reflog --date=iso | head -10` to see if auto-updates correlate with gateway deaths.

7. **launchd plist gets overwritten** — `hermes gateway start` regenerates the plist. Any manual KeepAlive fix will be reverted. After running that command, re-verify: `grep -A3 KeepAlive ~/Library/LaunchAgents/ai.hermes.gateway.plist`

8. **Confirmed auto-update death pattern (Apr 2026)** — On Apr 3 the gateway stopped/restarted 7 times in one day due to auto-updates and manual restarts. The final stop at 7:34 PM had no matching restart, leaving it dead for 14+ hours. The git reflog timestamps (`git -C ~/.hermes/hermes-agent reflog --date=iso`) correlated exactly with each gateway death. This is the primary reliability issue — not crashes, but clean shutdowns that launchd won't recover from without the KeepAlive fix.
