#!/usr/bin/env bash
# PostToolUse hook: run tests for the file that was just edited/written.
# Receives the tool input as JSON on stdin.

f=$(jq -r '.tool_input.file_path // ""')
ext="${f##*.}"

case "$ext" in
  rs)
    if [[ "$f" == */web_server/* ]]; then
      pkg="web_server"
    elif [[ "$f" == */api_gen/* ]]; then
      pkg="api_gen"
    else
      exit 0
    fi

    mod=$(basename "$f" .rs)
    if [[ "$mod" == "lib" || "$mod" == "main" || "$mod" == "mod" ]]; then
      cargo test -p "$pkg" 2>&1 | tail -40
    else
      cargo test -p "$pkg" "$mod" 2>&1 | tail -40
    fi
    ;;

  ts|js|svelte)
    cd webapp && npm test 2>&1 | tail -40
    ;;
esac
