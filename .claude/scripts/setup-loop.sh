#!/usr/bin/env bash
set -euo pipefail

STATE_FILE=".claude/loop-state.local.md"
PROMPT=""
PROMISE=""

# Parse arguments: everything before --promise is the prompt
while [[ $# -gt 0 ]]; do
  case $1 in
    --promise)
      shift
      PROMISE="$1"
      shift
      ;;
    *)
      PROMPT="$PROMPT $1"
      shift
      ;;
  esac
done

PROMPT="${PROMPT# }"  # trim leading space

if [[ -z "$PROMPT" ]]; then
  echo "Error: No prompt provided" >&2
  exit 1
fi

if [[ -z "$PROMISE" ]]; then
  echo "Error: No --promise provided" >&2
  exit 1
fi

# Write state file
cat > "$STATE_FILE" << EOF
---
active: true
promise: "$PROMISE"
---

# Loop Task

$PROMPT
EOF

# Output for Claude
echo "═══════════════════════════════════════════════════════════"
echo "LOOP STARTED"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "Task: $PROMPT"
echo ""
echo "Completion promise: $PROMISE"
echo ""
echo "To exit, output: <promise>$PROMISE</promise>"
echo "Only when the statement is TRUE."
echo "═══════════════════════════════════════════════════════════"
