---
name: a0-tool-and-extension-wiring
description: "Wire new capabilities into Agent Zero — tools (LLM-callable), extensions (event hooks), and system prompts. Covers the registration gotchas that cause tools to silently not appear."
tags: [agent-zero, tools, extensions, prompts, carabiner-os]
triggers:
  - "A0 doesn't use the tool"
  - "A0 isn't creating cards"
  - "tool not showing up in A0"
  - "add a tool to agent zero"
  - "extend agent zero"
  - "agent zero tool registration"
---

# Agent Zero Tool & Extension Wiring

How to add capabilities to Agent Zero correctly. There are THREE mechanisms, each for a different purpose.

## The Three Mechanisms

### 1. Tools (LLM-callable)

Tools are things A0 can choose to invoke during reasoning. They require **TWO files**:

```
engine/agent-zero/tools/<name>.py         # Python implementation
engine/agent-zero/prompts/agent.system.tool.<name>.md  # Prompt that tells A0 it exists
```

**CRITICAL**: If the prompt file is missing, A0 will NEVER know the tool exists. The Python file alone does nothing. A0 discovers tools by scanning `prompts/agent.system.tool.*.md` at startup.

**Tool Python pattern** (inherits from `helpers.tool.Tool`):
```python
from helpers.tool import Tool, Response

class MyTool(Tool):
    async def execute(self, **kwargs):
        arg1 = self.args.get("arg1", "")
        # ... do work ...
        return Response(message="Done", break_loop=False)
```

**Tool prompt pattern** (`prompts/agent.system.tool.my_tool.md`):
```markdown
### my_tool:
Description of what the tool does.

#### Arguments:
* "arg1" (string): Description
* "arg2" (Optional, string): Description

#### Usage examples:
##### 1: Example scenario
```json
{
    "tool_name": "my_tool",
    "tool_args": {
        "arg1": "value"
    }
}
```
```

**Existing tools** (reference): response, notify_user, call_subordinate, search_engine, document_query, scheduler, skills_tool, wait, a2a_chat, vision_load

### 2. Extensions (event hooks)

Extensions react to events — they're NOT callable by A0's reasoning. They fire automatically when specific events occur.

**Extension directories** (each is a hook point):
```
extensions/python/webui_ws_event/       # WebSocket events from frontend
extensions/python/webui_ws_connect/     # Client connects
extensions/python/webui_ws_disconnect/  # Client disconnects
extensions/python/tool_execute_before/  # Before any tool runs
extensions/python/tool_execute_after/   # After any tool runs
extensions/python/agent_loop_start/     # Agent loop begins
extensions/python/agent_loop_end/       # Agent loop ends
```

**Extension pattern** (inherits from `helpers.extension.Extension`):
```python
from helpers.extension import Extension

class MyExtension(Extension):
    async def execute(self, instance=None, sid="", event_type="", 
                      data=None, response_data=None, **kwargs):
        if event_type != "my_event":
            return
        # ... handle event ...
        if response_data is not None:
            response_data["key"] = "value"
```

Files are auto-discovered and executed in alphabetical order (`_10_` runs before `_20_`).

**For CarabinerOS**: Put extensions in `usr/extensions/python/<hook>/` to avoid modifying the A0 submodule.

### 3. System Prompts (context injection)

System prompts inject text into A0's context at startup. They guide behavior but don't create callable tools.

```
usr/extensions/python/system_prompt/_my_context.md
```

Any `.md` file in that directory is appended to A0's system prompt. Use for:
- Decision frameworks ("when to emit cards")
- Domain knowledge ("restaurant terminology")  
- Behavioral rules ("one card per vendor")

## Common Mistake: Building a Tool That A0 Can't See

**Symptom**: You create `python/tools/my_tool.py`, tests pass, but A0 never uses it.

**Root cause**: A0's tool discovery only looks at:
1. `engine/agent-zero/tools/*.py` for the Python implementation
2. `engine/agent-zero/prompts/agent.system.tool.*.md` for the prompt

Files outside these directories are invisible to A0's reasoning loop.

**Fix**: Either:
- Put the tool IN the A0 submodule (requires submodule commit)
- Use an existing tool (like `notify_user`) and guide A0 via system prompt
- Register the tool as a plugin tool (via `usr/plugins/<name>/tools/`)

## The notify_user → Action Card Bridge

`notify_user` is A0's built-in notification tool. It ALSO emits `action_card` Socket.IO events when the `detail` field contains valid JSON with card structure fields.

**This means**: To create action cards, A0 doesn't need a new tool — just teach it (via system prompt) to use `notify_user` with structured JSON in `detail`.

```json
{
    "tool_name": "notify_user",
    "tool_args": {
        "title": "Card headline",
        "message": "Card detail text",
        "type": "warning",
        "detail": "{\"module\":\"orders\",\"action\":\"create\",\"stats\":[...],\"changes\":[...],\"actions\":[...]}"
    }
}
```

The `_emit_action_card()` method in `notify_user.py` parses the JSON and broadcasts an `action_card` Socket.IO event.

**CRITICAL — Envelope Wrapping**: `send_data()` in `ws_manager.py` wraps ALL payloads in an envelope: `{handlerId, eventId, correlationId, ts, data: <your_payload>}`. So if you call `send_data("action_card", {"card": card})`, the frontend receives `{data: {card: {...}}, handlerId: ..., ...}` — NOT `{card: {...}}` directly. Frontend listeners must unwrap: `const envelope = payload.data ?? payload; const card = envelope.card ?? envelope;`

**chat_context_id linking**: The system prompt extension `_25_restaurant_context.py` dynamically injects the current `AgentContext.id` into A0's prompt so it can pass `--chat-context <id>` on CLI write commands. This links DB records (orders, recipes, etc.) back to the conversation that created them. Access pattern: `self.agent.context.id` from any tool or extension.

## Plugin Tools (usr/plugins/)

Plugins can register tools via:
```
usr/plugins/<name>/tools/<tool_name>.py
usr/plugins/<name>/prompts/agent.system.tool.<tool_name>.md
```

These are discovered by A0's plugin scanner at startup. Same two-file requirement.

## Verification Checklist

After adding a tool or extension:

1. **Rebuild container**: `docker compose -f docker-compose.dev.yml up -d --build agent-zero`
2. **Check files in container**: `docker exec <container> ls /a0/tools/` and `docker exec <container> ls /a0/prompts/ | grep tool`
3. **Check logs**: `docker logs <container> 2>&1 | grep -i "tool\|error\|import"`
4. **Test A0 actually sees it**: Ask A0 to list its tools or use the specific tool

## Pitfalls

1. **ALLOWED_ORIGINS blocks WebSocket streaming** — `docker-compose.dev.yml` may hardcode `ALLOWED_ORIGINS` to a specific IP (e.g., from an iPad session). If WebSocket state_push events aren't arriving (chat doesn't stream, cards don't appear), check: `docker exec <container> env | grep ALLOWED`. Fix by adding `*://localhost,*://localhost:*,*://127.0.0.1:*` to the compose env. This overrides the broader list in `usr/.env`.

2. **Submodule changes need separate commits** — `git add engine/agent-zero/file.py` fails with "is in submodule". Must `cd engine/agent-zero && git add && git commit` first, then `cd ../.. && git add engine/agent-zero`.

2. **Python path inside container** — The container's `/a0/` directory maps to the repo root. `python/` at the repo root becomes `/a0/python/` inside Docker. But A0's tool loader only checks `/a0/tools/`.

3. **Extension naming order matters** — `_10_` fires before `_20_`. If your extension depends on state from another, order the prefixes.

4. **System prompts are markdown** — They're injected as-is into the context. Keep them concise; every token counts against the context window.

5. **docker-compose env overrides .env** — Environment variables in `docker-compose.dev.yml` `environment:` block take precedence over values in `usr/.env`. If you update `.env` but the container still uses the old value, check the compose file.

6. **Container rebuild doesn't always pick up submodule changes** — After committing inside a submodule, you may need `docker compose build --no-cache agent-zero` to force a full rebuild. The build cache may reuse layers that predate the submodule update.
