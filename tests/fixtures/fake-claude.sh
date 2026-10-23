#!/bin/sh
here="$(dirname "$0")"
scenario="success"
capture=""
if [ -f "$here/scenario" ]; then
  scenario="$(cat "$here/scenario")"
  capture="$here"
fi

if [ "$1" = "--version" ]; then
  echo "${FAKE_CLAUDE_VERSION:-2.1.296} (Claude Code)"
  exit 0
fi

if [ "$1" = "auth" ] && [ "$2" = "status" ]; then
  echo "{\"loggedIn\":${FAKE_CLAUDE_LOGGED_IN:-true},\"authMethod\":\"claude.ai\"}"
  exit 0
fi

session_id="session-1"
resumed=""
mcp_config=""
previous=""
for argument in "$@"; do
  if [ "$previous" = "--resume" ]; then resumed="$argument"; session_id="$argument"; fi
  if [ "$previous" = "--mcp-config" ]; then mcp_config="$argument"; fi
  previous="$argument"
done

if [ -n "$capture" ]; then
  pwd -P > "$capture/$$.cwd"
  printf '%s\n' "$@" > "$capture/$$.args"
  cat > "$capture/$$.prompt"
  if [ -n "$mcp_config" ]; then
    printf '%s\n' "$mcp_config" > "$capture/$$.mcp-path"
    ls -ln "$mcp_config" | cut -c1-10 > "$capture/$$.mcp-mode"
    cat "$mcp_config" > "$capture/$$.mcp-config"
  fi
  echo "$$" >> "$capture/invocations"
else
  cat > /dev/null
fi

if [ "$scenario" = "resume-missing" ] && [ -n "$resumed" ]; then
  echo "No conversation found with session ID: $resumed" >&2
  exit 1
fi

printf '{"type":"system","subtype":"init","session_id":"%s"}\n' "$session_id"

case "$scenario" in
  edit)
    printf 'from the run\n' > run-notes.txt
    ;;
  crash)
    echo "fatal: the fake provider crashed" >&2
    exit 3
    ;;
  gated)
    (while [ ! -e "$capture/release" ]; do sleep 0.02; done) &
    echo "$!" > "$capture/$$.grandchild"
    wait "$!"
    ;;
  hang)
    sleep 600 &
    echo "$!" > "$capture/$$.grandchild"
    wait "$!"
    ;;
  stubborn)
    trap '' INT TERM
    sleep 600 &
    echo "$!" > "$capture/$$.grandchild"
    wait "$!"
    ;;
esac

printf '{"type":"result","subtype":"success","is_error":false,"result":"done","session_id":"%s","num_turns":3,"total_cost_usd":0.01}\n' "$session_id"
