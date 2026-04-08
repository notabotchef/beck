#!/usr/bin/env bash
#
# claude-tokens.sh - report token usage for a Claude Code session.
#
# Usage:
#   claude-tokens                   # most recent session across all projects
#   claude-tokens first             # show only the first assistant turn
#                                   # (this is the answer to
#                                   # "how many tokens got injected
#                                   # right after my first message")
#   claude-tokens summary           # totals for the current session
#   claude-tokens <session-id>      # specific session by id
#
# Reads directly from Claude Code's local session transcripts at
# ~/.claude/projects/*/*.jsonl. No network, no API calls.

set -euo pipefail

mode="${1:-summary}"

find_latest_jsonl() {
  local latest
  latest=$(find "$HOME/.claude/projects" -name '*.jsonl' -type f 2>/dev/null \
    | xargs -I{} stat -f '%m %N' {} 2>/dev/null \
    | sort -rn \
    | head -1 \
    | awk '{$1=""; print substr($0,2)}')
  [ -n "$latest" ] && echo "$latest"
}

find_by_id() {
  local id="$1"
  find "$HOME/.claude/projects" -name "${id}.jsonl" -type f 2>/dev/null | head -1
}

# Dispatch
case "$mode" in
  first)
    file=$(find_latest_jsonl)
    ;;
  summary)
    file=$(find_latest_jsonl)
    ;;
  *)
    # Assume it's a session id (uuid-shaped)
    file=$(find_by_id "$mode")
    mode="summary"
    ;;
esac

if [ -z "${file:-}" ] || [ ! -f "$file" ]; then
  echo "could not find session jsonl" >&2
  exit 1
fi

echo "session file: $file"
echo

python3 - "$file" "$mode" <<'PY'
import json, sys, os

path, mode = sys.argv[1], sys.argv[2]

turns = []
with open(path) as f:
    for line in f:
        try:
            j = json.loads(line)
            msg = j.get('message') or {}
            usage = msg.get('usage') or {}
            if usage:
                turns.append({
                    'input': usage.get('input_tokens', 0),
                    'output': usage.get('output_tokens', 0),
                    'cache_creation': usage.get('cache_creation_input_tokens', 0),
                    'cache_read': usage.get('cache_read_input_tokens', 0),
                    'ts': j.get('timestamp', ''),
                })
        except Exception:
            pass

if not turns:
    print("no usage data in this session yet")
    print("(send at least one message and wait for a response, then try again)")
    sys.exit(0)

if mode == 'first':
    first = turns[0]
    print("=== first assistant turn ===")
    print("this is what got 'injected' before your first reply:")
    print()
    print(f"  input tokens          {first['input']:>12,}   (your own message)")
    print(f"  cache creation        {first['cache_creation']:>12,}   (initial system prompt, written to cache)")
    print(f"  cache read            {first['cache_read']:>12,}   (re-read from previous cache, if any)")
    print(f"  output tokens         {first['output']:>12,}   (the assistant reply)")
    print()
    static_prompt = first['cache_creation'] + first['cache_read']
    print(f"  STATIC SYSTEM PROMPT  {static_prompt:>12,} tokens")
    print("  ^ this is the 'tax' you pay before saying anything. tools + skills + CLAUDE.md + memory.")
    sys.exit(0)

# summary mode
total_input = sum(t['input'] for t in turns)
total_output = sum(t['output'] for t in turns)
total_cc = sum(t['cache_creation'] for t in turns)
total_cr = sum(t['cache_read'] for t in turns)
total = total_input + total_output + total_cc + total_cr

print(f"=== session summary ({len(turns)} assistant turns) ===")
print()
print(f"  input tokens          {total_input:>14,}")
print(f"  output tokens         {total_output:>14,}")
print(f"  cache creation        {total_cc:>14,}")
print(f"  cache read            {total_cr:>14,}")
print(f"  TOTAL                 {total:>14,}")
print()

# first turn
first_cc = turns[0]['cache_creation']
first_cr = turns[0]['cache_read']
print(f"  first-turn static     {first_cc + first_cr:>14,}  (what hit the model before turn 1 replied)")

# last turn
last = turns[-1]
print(f"  latest-turn static    {last['cache_read'] + last['cache_creation']:>14,}  (current per-turn cached context)")

# approximate Opus 4.6 cost at standard rates:
# input $15/M, output $75/M, cache_creation $18.75/M, cache_read $1.50/M
cost = (total_input*15 + total_output*75 + total_cc*18.75 + total_cr*1.50) / 1_000_000
print()
print(f"  approx cost (Opus std rates): ${cost:.2f}")
print(f"  (actual cost depends on your plan and model. this is a reference estimate.)")
PY
