# mootbox-daemon — agent guide

The Rust daemon that runs on each person's machine. It pairs with a Mootbox server, claims work, drives the user's **locally authenticated** Claude Code or Codex CLI in an isolated git worktree, and publishes results back. It never holds provider credentials and never pushes branches.

## Relationship to the server

The daemon is a separate repository from the `agent-mootbox` TypeScript monorepo and talks to it over the **runner protocol**.

- Source of truth for the contract: `agent-mootbox/packages/contracts/schema/runner` and `fixtures/runner`.
- Vendored copy in this repo: `contracts/runner/`, with `contracts/SOURCE` recording the commit and protocol version it came from.
- Re-sync after changing the contract upstream: `scripts/sync-contracts.sh <path-to-agent-mootbox>`.
- `cargo run -p gen-contracts -- --check` runs in `scripts/check.sh` and fails on drift. Keep both sides in the same commit.
- Sending a stale `x-mootbox-runner-protocol` gets `426 upgrade_required`; the message tells the user to upgrade the daemon.

## Commands

```bash
scripts/check.sh          # fmt + comment ban + contract drift + clippy -D warnings + tests — run before claiming done
scripts/e2e.sh <path-to-agent-mootbox-repo> --real   # end-to-end against a built mootbox binary with real CLIs
scripts/dist.sh <out-dir>   # distribution build
scripts/sync-contracts.sh <path-to-agent-mootbox-repo>
cargo test --workspace    # 233 tests
```

Integration tests live in `tests/integration/`. They use fakes for the providers, so they are fast and hermetic; `e2e.sh` is the only thing that launches the real CLIs.

## Layout

```
src/commands/     one file per CLI command: add, allow, doctor, pair, service, start, status
src/providers/    the provider trait plus one folder per CLI (claude_code, codex)
src/run/          run execution, worktrees, session resume
src/daemon/       service loop, claim/poll, lease handling
contracts/        vendored runner schemas and fixtures (do not hand-edit; re-sync)
gen-contracts/    drift checker
```

Adding a provider means one folder under `src/providers/`, one entry in `src/providers/mod.rs::ALL`, and a fake plus integration tests. Providers are a registry, not a match statement.

## Rules

- **No comments.** `scripts/check-no-comments.sh` fails the build on any comment.
- **Clippy is denied at workspace level** (`clippy::all = deny`, `-D warnings` in `scripts/check.sh`), `unsafe_code` is forbidden and `unused_must_use` is denied. Unwrap/expect are allowed only in tests (`clippy.toml`).
- `rustfmt` with `max_width = 120`.
- Provider sessions are **local and owner-only**: the daemon persists the key-to-provider-session mapping in owner-only files and resumes from it after restart. The server stores an opaque session key, never a provider session ID.
- Worktrees are isolated per owner/chat/checkout. Do not reintroduce branch pushes — that flow was deliberately removed.
- User prompts arrive as bounded context; a prompt golden test guards the shape of the shared-chat prompt.

## Local config

`~/.mootbox/daemon.toml`. Unknown keys are rejected, so a renamed field is a hard startup failure — when you rename config, check the user's existing file. `mootbox doctor` reports sign-in, repository connection, server reachability, accepted protocol version and service state.