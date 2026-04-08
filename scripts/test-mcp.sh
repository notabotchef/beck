#!/usr/bin/env bash
#
# Integration test for `beck mcp`. Spawns the server, pipes JSON-RPC 2.0
# messages on stdin, checks stdout for the expected responses. Does not
# require any MCP client library.
#
# Usage: scripts/test-mcp.sh [path/to/beck]
#        Defaults to ./target/release/beck
#
# Exit 0 on all checks passing, non-zero on any failure.

set -euo pipefail

BECK="${1:-./target/release/beck}"
if [ ! -x "$BECK" ]; then
  echo "beck binary not found or not executable: $BECK" >&2
  echo "run: cargo build --release" >&2
  exit 1
fi

# Sanity check: a database must exist. If not, run sync first.
if [ ! -f "$HOME/Library/Application Support/beck/skills.db" ] && \
   [ ! -f "$HOME/.local/share/beck/skills.db" ]; then
  echo "no beck database found. running sync first..." >&2
  "$BECK" sync >&2
fi

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

REQUESTS="$TMP/requests.jsonl"
OUTPUT="$TMP/output.txt"

# MCP JSON-RPC 2.0 requests. The server reads one JSON object per line.
cat > "$REQUESTS" <<'EOF'
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"beck-test","version":"0.1.0"}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":2,"method":"tools/list"}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"skills_query","arguments":{"query":"whisper","top":2}}}
{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"skills_load","arguments":{"name":"whisper"}}}
EOF

# Spawn beck mcp with the request file as stdin. Close stdin after
# sending everything so the server exits cleanly.
"$BECK" mcp < "$REQUESTS" > "$OUTPUT" 2> "$TMP/stderr" || {
  echo "beck mcp exited with non-zero status" >&2
  cat "$TMP/stderr" >&2
  exit 1
}

echo "=== mcp stdout ==="
cat "$OUTPUT"
echo "=== end stdout ==="
echo

# ---- Assertions ----
FAILED=0

check() {
  local label="$1"
  local pattern="$2"
  if grep -q -- "$pattern" "$OUTPUT"; then
    echo "PASS  $label"
  else
    echo "FAIL  $label  (expected pattern: $pattern)" >&2
    FAILED=$((FAILED + 1))
  fi
}

check "initialize response has serverInfo.name=beck"           '"name":"beck"'
check "initialize response has protocolVersion"                '"protocolVersion"'
check "tools/list returns skills_query tool"                   '"skills_query"'
check "tools/list returns skills_load tool"                    '"skills_load"'
check "skills_query result contains whisper match"             'whisper'
check "skills_load result contains the whisper body"           'Whisper'

if [ "$FAILED" -gt 0 ]; then
  echo
  echo "FAILED: $FAILED checks" >&2
  exit 1
fi

echo
echo "ALL CHECKS PASSED"
