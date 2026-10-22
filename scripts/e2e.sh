#!/usr/bin/env bash
set -euo pipefail
if [ "$#" -ne 2 ] || [ "$2" != "--real" ]; then
  echo "usage: scripts/e2e.sh <path-to-agent-mootbox> --real" >&2
  exit 2
fi
daemon_root="$(cd "$(dirname "$0")/.." && pwd)"
mootbox_root="$(cd "$1" && pwd)"
cargo build --manifest-path "$daemon_root/Cargo.toml" --bin mootbox
MOOTBOX_DAEMON_BINARY="$daemon_root/target/debug/mootbox" node "$mootbox_root/scripts/mvp-smoke.mjs" --real
