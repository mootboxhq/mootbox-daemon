#!/usr/bin/env bash
set -euo pipefail

if [ $# -ne 1 ]; then
  echo "usage: scripts/sync-contracts.sh <path-to-agent-mootbox>" >&2
  exit 2
fi

source_repo="$(cd "$1" && pwd)"
target="$(cd "$(dirname "$0")/.." && pwd)/contracts"
schema_source="$source_repo/packages/contracts/schema/runner"
fixture_source="$source_repo/packages/contracts/fixtures/runner"

if [ ! -f "$schema_source/index.json" ]; then
  echo "no runner contracts at $schema_source" >&2
  exit 1
fi

commit="$(git -C "$source_repo" rev-parse HEAD)"
if [ -n "$(git -C "$source_repo" status --porcelain -- packages/contracts/schema/runner packages/contracts/fixtures/runner)" ]; then
  echo "warning: $source_repo has uncommitted runner contract changes; SOURCE records $commit" >&2
fi
protocol_version="$(sed -n 's/^ *"protocol_version": *\([0-9][0-9]*\).*/\1/p' "$schema_source/index.json")"

rm -rf "$target/runner"
mkdir -p "$target/runner/schema" "$target/runner/fixtures"
cp "$schema_source"/*.json "$target/runner/schema/"
cp "$fixture_source"/*.json "$target/runner/fixtures/"
printf 'commit=%s\nprotocol_version=%s\n' "$commit" "$protocol_version" > "$target/SOURCE"

echo "synced runner contracts from $source_repo@$commit (protocol $protocol_version)"
