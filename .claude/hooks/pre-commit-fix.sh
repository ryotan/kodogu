#!/bin/bash
# git commit 前に自動修正 + 検証
COMMAND=$(jq -r '.tool_input.command // empty')
if echo "$COMMAND" | grep -q "git commit"; then
  mise run cargo:fix 2>/dev/null
  if ! mise run cargo:lint 2>&1; then
    echo "lint still failing after auto-fix" >&2
    exit 2
  fi
fi
exit 0
