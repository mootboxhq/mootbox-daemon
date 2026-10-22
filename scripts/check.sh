#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
cargo fmt --all -- --check
scripts/check-no-comments.sh
cargo run -q -p gen-contracts -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
