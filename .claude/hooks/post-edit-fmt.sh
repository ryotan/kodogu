#!/bin/bash
# .rs ファイル編集後に cargo fmt を自動実行
FILE_PATH=$(jq -r '.tool_input.file_path // empty')
if [[ "$FILE_PATH" == *.rs ]]; then
  cargo fmt --quiet 2>/dev/null
fi
exit 0
