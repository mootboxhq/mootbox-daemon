#!/bin/sh
here="$(dirname "$0")"
scenario="success"
capture=""
if [ -f "$here/scenario" ]; then
  scenario="$(cat "$here/scenario")"
  capture="$here"
fi

if [ "$1" = "--version" ]; then
  echo "codex-cli ${FAKE_CODEX_VERSION:-0.120.0}"
  exit 0
fi

if [ "$1" = "login" ] && [ "$2" = "status" ]; then
  if [ "${FAKE_CODEX_LOGGED_IN:-true}" = "true" ]; then
    echo "Logged in using ChatGPT"
    exit 0
  fi
  echo "Not logged in" >&2
  exit 1
fi

thread_id="thread-1"
if [ "$1" = "exec" ] && [ "$2" = "resume" ]; then
  thread_id="$3"
fi

if [ -n "$capture" ]; then
  pwd -P > "$capture/$$.cwd"
  printf '%s\n' "$@" > "$capture/$$.args"
  printf '%s' "${MOOTBOX_RUN_TOKEN:-}" > "$capture/$$.run-token"
  cat > "$capture/$$.prompt"
  echo "$$" >> "$capture/invocations"
else
  cat > /dev/null
fi

printf '{"type":"thread.started","thread_id":"%s"}\n' "$thread_id"
printf '{"type":"turn.started"}\n'

case "$scenario" in
  fail)
    printf '{"type":"turn.failed","error":{"message":"usage limit reached"}}\n'
    exit 1
    ;;
  mcp_blocked)
    printf '{"type":"item.completed","item":{"id":"item_0","type":"mcp_tool_call","server":"mootbox","tool":"mootbox_submit_result","arguments":{},"result":null,"error":{"message":"MCP tool call requires approval, but approval policy is never"},"status":"failed"}}\n'
    printf '{"type":"item.completed","item":{"id":"item_1","type":"agent_message","text":"MCP tool call requires approval, but approval policy is never"}}\n'
    printf '{"type":"turn.completed","usage":{"input_tokens":10,"cached_input_tokens":0,"output_tokens":5}}\n'
    exit 0
    ;;
esac

printf '{"type":"item.completed","item":{"id":"item_0","type":"reasoning","text":"thinking"}}\n'
printf '{"type":"item.completed","item":{"id":"item_1","type":"agent_message","text":"done in codex"}}\n'
printf '{"type":"turn.completed","usage":{"input_tokens":10,"cached_input_tokens":0,"output_tokens":5}}\n'
