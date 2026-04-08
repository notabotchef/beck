---
name: carabiner-os-troubleshooting
description: Systematic troubleshooting for CarabinerOS multi-service Docker stack when frontend loads but APIs fail
version: 1.0.0
author: Hermes Agent
license: MIT
metadata:
  hermes:
    tags: [CarabinerOS, Docker, Troubleshooting, API, Frontend, Agent-Zero]
    related_skills: [docker-compose-troubleshooting, systematic-debugging]
---

# CarabinerOS Troubleshooting

When CarabinerOS frontend loads but side menu/database functionality fails, follow this systematic approach.

## Symptoms

- Frontend loads at localhost:3000 or localhost:8080
- Side menu doesn't work
- Database content not loading
- API endpoints return 404 or "endpoint not found"

## Root Cause Analysis Pattern

CarabinerOS uses Agent Zero's API dispatch system which requires handler registration during startup. Missing endpoints mean the startup migration didn't register all required handlers.

## Diagnostic Steps

### 1. Check Service Status

```bash
cd ~/Projects/carabiner-os
docker compose -f docker-compose.dev.yml ps
docker compose -f docker-compose.dev.yml logs agent-zero --tail 20
```

Look for: `CarabinerOS API: X endpoints registered`

### 2. Test API Endpoints

```bash
# Test basic health
curl -I http://localhost:5050/api/health

# Test workspace endpoints
curl -s http://localhost:3000/api/inventory | head -5

# Test chat endpoints (common failure point)
curl -s http://localhost:3000/api/chats
curl -s http://localhost:3000/api/csrf_token
```

### 3. Check Frontend Logs

```bash
cd ~/Projects/carabiner-os && docker compose -f docker-compose.dev.yml logs frontend --tail 20
```

Look for 404 errors on missing JS files or API routes.

## Common Issues & Solutions

### Missing Chat Endpoints

**Problem**: Frontend expects `/api/chats`, `/api/message`, `/api/csrf_token` but Agent Zero doesn't register them.

**Solution**: Add missing endpoints to startup registration:

1. Edit `usr/plugins/carabiner/extensions/python/startup_migration/_10_carabiner_init.py`
2. Add to `_CUSTOM_ROUTES`:
   ```python
   ("chats", "make_chats_handler"),
   ("message", "make_message_handler"), 
   ("message_async", "make_message_async_handler"),
   ("csrf_token", "make_csrf_token_handler"),
   ```

3. Create handler factories in `carabiner/api/_a0_handlers.py`:
   ```python
   def make_chats_handler() -> type:
       class ChatsHandler(ApiHandler):
           @classmethod
           def get_methods(cls): return ["GET", "POST", "DELETE"]
           @classmethod
           def requires_auth(cls): return False
           @classmethod
           def requires_csrf(cls): return False

           async def process(self, input: dict, request: Any) -> Any:
               from flask import Response
               from carabiner.chat_store import chat_store
               import json
               
               method = request.method
               if method == "GET":
                   contexts = [ctx.to_summary() for ctx in chat_store.all()]
                   return Response(json.dumps(contexts, default=str), content_type="application/json")
               # ... handle POST/DELETE
       return ChatsHandler
   ```

### FastAPI/Flask Compatibility Issues

**Problem**: Handler tries to import `fastapi` in Agent Zero's Flask environment.

**Solution**: Use Flask-only handlers. Replace FastAPI imports with Flask equivalents:
- `from flask import Response` instead of FastAPI responses
- `json.dumps()` instead of FastAPI's automatic serialization
- Flask request object instead of FastAPI Request

### Frontend Build Issues

**Problem**: Missing JS files (404s) or stale Next.js cache.

**Solution**: 
1. Kill existing dev server: `pkill -f "next dev"`
2. Clear cache: `rm -rf .next`
3. Restart with correct env: `A0_URL=http://localhost:5050 pnpm dev`

### Database Connection Failures

**Problem**: PostgreSQL not accessible or tables missing.

**Solution**:
```bash
# Check PostgreSQL health
docker compose -f docker-compose.dev.yml logs postgres --tail 10

# Check CarabinerOS database connection in logs
docker compose -f docker-compose.dev.yml logs agent-zero | grep -i "carabiner.*database"
```

Look for: `CarabinerOS database connected`

## Complete Fix Workflow

1. **Stop everything**:
   ```bash
   cd ~/Projects/carabiner-os
   docker compose -f docker-compose.dev.yml down
   pkill -f "next dev"
   ```

2. **Add missing endpoint registrations** (see above)

3. **Rebuild and restart**:
   ```bash
   docker compose -f docker-compose.dev.yml up -d
   cd frontend && A0_URL=http://localhost:5050 pnpm dev &
   ```

4. **Verify endpoints registered**:
   ```bash
   # Should show increased endpoint count
   docker compose -f docker-compose.dev.yml logs agent-zero | grep "endpoints registered"
   ```

5. **Test functionality**:
   ```bash
   curl -s http://localhost:3000/api/chats
   curl -s http://localhost:3000/api/inventory | head -3
   ```

## Network Access (iPad/External)

For iPad access on same WiFi:

1. **Get Mac IP**: `ifconfig en0 | grep "inet " | awk '{print $2}'`
2. **iPad URL**: `http://<MAC_IP>:3000`
3. **If blocked, configure firewall**:
   ```bash
   sudo /usr/libexec/ApplicationFirewall/socketfilterfw --add /opt/homebrew/bin/node
   sudo /usr/libexec/ApplicationFirewall/socketfilterfw --unblock /opt/homebrew/bin/node
   ```

## Debugging Tips

- **Agent Zero startup is slow**: Wait for "Preload completed" message
- **Environment variables**: Frontend needs `A0_URL=http://localhost:5050` 
- **Port conflicts**: Kill existing processes on 3000/5050
- **API routing**: Next.js rewrites in `next.config.ts` proxy `/api/*` to Agent Zero

## Success Indicators

- `CarabinerOS API: 18+ endpoints registered` in logs
- `curl http://localhost:3000/api/chats` returns `[]` (not 404)
- Inventory API returns Spanish restaurant data (Jamón Ibérico, etc.)
- Frontend loads with working sidebar navigation

## Module Sidechat / chat_context_id Issues

**Symptom**: Module page sidechat (orders, inventory, prep, etc.) creates a new orphan conversation instead of showing the chat thread that created the record. Or duplicate DB records appear when the agent creates items.

**Architecture**: Every workspace model inherits `ChatContextMixin` which adds `chat_context_id` (nullable string). The frontend `ModuleChat` component reads `order.chat_context_id` (or equivalent) and subscribes to that A0 conversation. If null, it creates a new chat — breaking the link.

**Root cause (sidechat disconnected)**: The agent didn't include `chat_context_id` when calling `db_mutate(action="create")`. Fix:
1. Ensure `usr/knowledge/main/database-schema.md` instructs the agent to ALWAYS include `chat_context_id` on create/update
2. The MCP server `_prepare_data` passes all fields through — no server changes needed
3. Backfill existing records: `docker compose -f docker-compose.dev.yml exec postgres psql -U carabiner -d carabiner -c "UPDATE workspace_orders SET chat_context_id = '<CHAT_ID>' WHERE chat_context_id IS NULL;"`

**Root cause (double entries)**: `_create()` in `repositories.py` does blind INSERT. Fix: add dedup guard to the specific `create_<module>` function — check for same key fields (e.g. vendor + chat_context_id) within last 60 seconds, return existing if found.

**Diagnostic query**:
```bash
docker compose -f docker-compose.dev.yml exec postgres psql -U carabiner -d carabiner -c \
  "SELECT id, vendor, chat_context_id, created_at FROM workspace_orders ORDER BY created_at DESC LIMIT 10;"
```

**Key files**:
- `carabiner/db/base.py` — ChatContextMixin definition
- `carabiner/db/repositories.py` — create_order (and other create_* functions)
- `frontend/src/components/module-chat.tsx` — reads chatContextId, subscribes to conversation
- `frontend/src/app/orders/components/order-detail-panel.tsx` — passes order.chat_context_id to ModuleChat
- `usr/knowledge/main/database-schema.md` — agent instructions for including chat_context_id

## Action Cards Pipeline Issues

**Symptom**: User sends message, A0 processes it, but no action card appears in the Tickets panel. Or card appears but content is empty / frontend crashes.

### Card Not Appearing At All

**Cause 1: System prompt references wrong tool**
The action cards system prompt (`usr/extensions/python/system_prompt/_action_cards.md`) must tell A0 to use `notify_user` — NOT a standalone `action_card` tool. A0 only has tools registered in `engine/agent-zero/tools/` with matching prompt files in `prompts/agent.system.tool.<name>.md`. The `action_card` tool was never registered as an A0 tool.

**Fix**: The prompt must show `notify_user` with structured JSON in the `detail` field:
```json
{"tool_name": "notify_user", "tool_args": {
    "title": "Headline", "message": "Context",
    "type": "warning",
    "detail": "{\"module\":\"orders\",\"stats\":[...],\"changes\":[...],\"actions\":[...]}"
}}
```

**Cause 2: A0 tool registration**
A0 tools need TWO files: `tools/<name>.py` + `prompts/agent.system.tool.<name>.md`. Having just the Python file is not enough.

### Card Appears But Content Empty / Frontend Crashes

**Symptom**: `TypeError: undefined is not an object (evaluating 'incoming.type')` in use-action-cards.ts

**Root cause**: A0's `send_data()` wraps payloads in an envelope: `{handlerId, eventId, ts, data: {card: {...}}}`. Frontend was reading `payload.card` but actual card was at `payload.data.card`.

**Fix**: Unwrap envelope in `handleActionCard`:
```typescript
const handleActionCard = (payload: Record<string, unknown>) => {
  const envelope = (payload as any)?.data ?? payload;
  const incoming = envelope?.card ?? envelope;
  if (!incoming?.type) return; // guard against malformed payloads
```

### WebSocket Streaming Not Working (state_push missing)

**Symptom**: A0 processes message, creates records, but chat text doesn't stream. User must refresh to see response.

**Root cause**: `ALLOWED_ORIGINS` in `docker-compose.dev.yml` may be stale (e.g., only allowing a LAN IP from a previous session). WebSocket origin validation rejects localhost connections, killing state_push delivery.

**Diagnostic**:
```bash
docker exec carabiner-os-agent-zero-1 env | grep ALLOWED_ORIGINS
```

**Fix**: Ensure docker-compose.dev.yml includes localhost:
```yaml
- ALLOWED_ORIGINS=*://localhost,*://localhost:*,*://127.0.0.1,*://127.0.0.1:*,*://0.0.0.0:*,*://10.0.0.39:*
```

**PITFALL**: `docker-compose.dev.yml` env vars OVERRIDE `usr/.env`. Always check both when debugging environment issues.

### LLM Produces Malformed Tool JSON

**Symptom**: `ValueError: Tool request must have a tool_name (type string) field` in A0 logs. The subordinate (A1/AGM) wrote malformed JSON for the `notify_user` call.

**Root cause**: LLM reliability issue — the model formatted the tool call incorrectly (e.g., `{"notify_user", ...}` instead of `{"tool_name": "notify_user", ...}`).

**Not fixable in app code** — this is an LLM generation error. The mitigation is:
1. A0's error recovery should still send the response even when notify_user fails
2. Retry the prompt — usually works on second attempt

### Key Files for Action Cards

- `engine/agent-zero/tools/notify_user.py` — emits action_card via `_emit_action_card()` → `send_data()`
- `usr/extensions/python/system_prompt/_action_cards.md` — tells A0 when/how to create cards
- `frontend/src/hooks/use-action-cards.ts` — Socket.IO listener, card state management
- `frontend/src/components/notification-panel.tsx` — Tickets panel UI
- `frontend/src/components/action-card.tsx` — compact card component
- `frontend/src/components/action-card-expanded.tsx` — expanded view with chat + buttons
- `frontend/src/components/shell.tsx` — mounts NotificationPanel, wires useActionCards
- `python/websocket_handlers/state_sync_handler/action_cards_handler.py` — handles card_message/commit/dismiss
- `engine/agent-zero/extensions/python/webui_ws_event/_20_action_cards.py` — wires card events into A0

## Prevention

- Always check endpoint count after CarabinerOS changes
- Test both workspace APIs (`/api/inventory`) and chat APIs (`/api/chats`) 
- Use `docker compose down && up` rather than restart for major changes
- Keep frontend and backend environment variables in sync