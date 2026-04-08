---
name: carabineros-start-local-stack
description: Canonical runbook for starting the CarabinerOS local dev stack. Follow every time Esteban asks to spin up CarabinerOS, start the engine, bring up the stack, or validate a slice live.
version: 1.0.0
author: Esteban + Hermes
---

# Start CarabinerOS (local dev)

Working dir: `/Users/estebannunez/Projects/carabiner-os`

Use this runbook every time. Do not improvise. Do not substitute commands.

## 0. Preconditions
- Docker Desktop must be running. Check with `docker info`. If it errors, stop and report "Docker not running". Do NOT try to start Docker yourself.
- A dirty `engine/agent-zero` submodule is fine. Compose mounts host volumes.

## 1. Check what's already up
Run in parallel:
```
docker ps --filter name=carabiner --format '{{.Names}}\t{{.Status}}'
lsof -i :8080 -i :5050 -i :5432 2>/dev/null | grep LISTEN
curl -sf http://localhost:8080/ -o /dev/null && echo "stack:UP" || echo "stack:DOWN"
```

Decision tree:
- all 4 containers healthy + `stack:UP` -> nothing to do, report status, exit
- partial/stale containers -> go to step 2
- nothing running -> go to step 2

## 2. Bring the stack up
```
docker compose -f docker-compose.dev.yml up -d
```

Expected container order and names:
- `carabiner-os-postgres-1` (Postgres 16, host :5432)
- `carabiner-os-agent-zero-1` (Flask + Socket.IO, host :5050)
- `carabiner-os-frontend-1` (Next.js 16, internal :3000)
- `carabiner-os-nginx-1` (routes everything, host :8080)

agent-zero can take up to 90s to report healthy. Do not panic-restart.

## 3. Verify health
```
docker ps --filter name=carabiner --format '{{.Names}}\t{{.Status}}'
curl -sf http://localhost:8080/api/health && echo
curl -sf http://localhost:8080/ -o /dev/null && echo "frontend:UP" || echo "frontend:DOWN"
```

If anything is wrong, tail logs:
```
docker logs --tail 100 carabiner-os-agent-zero-1
docker logs --tail 100 carabiner-os-frontend-1
```

## 4. What lives where
- http://localhost:8080/               -> CarabinerOS frontend
- http://localhost:8080/api/*          -> Carabiner REST + A0 API via nginx
- http://localhost:8080/socket.io/     -> realtime events
- http://localhost:8080/a0/            -> Agent Zero native WebUI
- postgresql://carabiner:carabiner@localhost:5432/carabiner -> DB

## 5. Stray :3000 pnpm dev
May exist from a previous session. Harmless. Has no backend.
Only kill if user explicitly asks:
```
lsof -i :3000 -t | xargs -r kill
```

## 6. Reporting back (mandatory)
When stack is up, report:
- container statuses (one line each)
- URL: http://localhost:8080
- any warnings from logs (migrations, env vars)

Do not say "done" without this evidence. Completion gate applies.

## 7. Known failure modes
- port already allocated on 8080 / 5432 -> report conflict via `lsof -i :<port>`, do not kill without user approval
- agent-zero unhealthy after 2 min -> almost always DB migration or missing env var, read logs, do not recreate container
- MCP "command not found" -> usr/settings.json path issue, reference CLAUDE.md .venv note
- submodule dirty warnings -> ignore unless user asks
- agent-zero hangs forever after `Preload completed.` with no Flask "Serving" log -> volume-mount drift (see section 8)

## 8. Volume-mount branch drift (CRITICAL)

`docker-compose.dev.yml` mounts `./carabiner:/cos/carabiner` as a live volume. Whatever branch the root worktree (`/Users/estebannunez/Projects/carabiner-os`) is checked out to, THAT is the Python code the running agent-zero container sees.

**Symptom of drift:**
- merged PR #N to main but new endpoints return 502 or empty
- agent-zero logs show `Preload completed.` then nothing — no `* Serving Flask app` line
- `docker exec ... curl http://localhost:80/api/health` returns `000` (Flask never bound port)
- duplicate Flask blueprint registration error possible (same blueprint name from two slice files)

**Root cause:**
The root worktree is on a feature branch (e.g. a stacked slice branch), so `./carabiner` contains code from a LATER slice than what's merged on `main`. The container loads BOTH the old slice's `flask_blueprint.py` (with its blueprint registration) AND the newly-built image's `ui_server.py` registration. Two blueprints with the same name = silent Flask init hang after preload.

**Fix (in this exact order):**
1. Confirm branch drift:
   ```
   git -C /Users/estebannunez/Projects/carabiner-os branch --show-current
   git -C /Users/estebannunez/Projects/carabiner-os log --oneline -3 origin/main
   ```
   If the root worktree branch != `main` and you just merged a backend PR to main, you have drift.

2. Check for parallel `main` worktrees (only one worktree can hold a branch):
   ```
   git worktree list
   ```
   If another worktree holds `main`, remove it (assuming it's disposable):
   ```
   git worktree remove <path>
   ```

3. Stash any local state in root worktree, then checkout main:
   ```
   git stash push -u -m "stack-align-main-$(date +%s)"
   git checkout main
   git pull origin main
   ```

4. Restart agent-zero ONLY (no rebuild needed if image already includes the merged change, no `down -v`):
   ```
   docker restart carabiner-os-agent-zero-1
   ```

5. Wait up to 90s for healthy, polling:
   ```
   for i in 1 2 3 4 5 6 7 8 9 10 11 12; do
     STATUS=$(docker inspect --format '{{.State.Health.Status}}' carabiner-os-agent-zero-1)
     CODE=$(curl -s -o /dev/null -w '%{http_code}' http://localhost:8080/api/health)
     echo "attempt $i: container=$STATUS http=$CODE"
     [ "$STATUS" = "healthy" ] && [ "$CODE" = "200" ] && break
     sleep 10
   done
   ```

6. Verify the new endpoints from the merged PR with `curl -s -o /tmp/x.json -w '%{http_code}'` and inspect content.

7. After validation, you can `git stash pop` if the original branch state matters.

## 9. When to rebuild agent-zero (vs. just restart)

**Rebuild required** (`docker compose -f docker-compose.dev.yml build agent-zero` then `docker compose ... up -d --no-deps agent-zero`):
- Code in `engine/agent-zero/` changed (NOT volume-mounted, baked into image)
- `Dockerfile.agent-zero` changed
- New Python deps added to the image install step

**Restart only** (`docker restart carabiner-os-agent-zero-1`):
- Code in `carabiner/` changed (volume-mounted, picked up on process restart)
- Code in `usr/` changed (volume-mounted)
- Branch checkout drift fixed (section 8)

A rebuild takes ~50s on Apple Silicon for an incremental layer change. A restart takes ~20s. Don't rebuild when restart is enough.

## Do not
- Do not run `docker compose down -v`
- Do not restart agent-zero mid-conversation if user is actively testing
- Do not treat merging branches as part of "starting" the stack
- Do not replace this runbook with a shortcut
