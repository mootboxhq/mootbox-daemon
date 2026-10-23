# Mootbox daemon

The Mootbox daemon connects a person's local checkouts and locally authenticated Claude Code and Codex CLIs to the Mootbox shared-chat service. It claims only work initiated by that person in a chat they can access, runs it in an isolated worktree, and publishes the result. A teammate's message or mention never starts another person's assistant.

The shared-chat MVP implementation is complete. The Rust formatting, Clippy and test checks pass, and the real Claude/Codex smoke completed two turns per provider and resumed after daemon restart. That smoke used two simulated users and separate daemon homes on one physical machine. Separate-machine/provider-account testing and provider terms review remain open; see [the main repository handover](../agent-mootbox/docs/product/HANDOVER.md) and [status checklist](../agent-mootbox/docs/product/PLAN.md).

## Build and connect

Requires the Rust toolchain pinned in `rust-toolchain.toml`.

```sh
cargo install --path .
mootbox --help
```

In a checkout registered to a Mootbox project, connect the local checkout and pair the device when prompted:

```sh
cd ~/code/checkout
mootbox add --mootbox-url https://mootbox.example.com
```

Use the Mootbox server URL configured for your environment. The person running the daemon must install and authenticate supported CLIs using their own provider flows. Mootbox does not collect provider credentials. The daemon stores its configuration under `~/.mootbox`; set `MOOTBOX_DAEMON_HOME` to use a separate profile/home.

Useful commands:

```sh
mootbox status
mootbox doctor
mootbox start
mootbox service install
mootbox service uninstall
```

The daemon runs in the foreground with `mootbox start`, or through the platform service manager after installation. Use `mootbox --help` for the current command surface and flags.

## Execution and sessions

The daemon claims work for the local user's own connected checkout and selected provider. Runs use isolated worktrees and are serialized for a given owner/chat, so Claude and Codex do not write to the same checkout at once. Cancellation stops the local child process; removed participants and cancelled runs cannot publish late results.

Provider session IDs stay on the local machine. Mootbox sends an opaque session key; the daemon stores its mapping to the provider session in owner-only files and resumes it after restart. If that local session is missing or cannot resume, the run reports the failure. The user can explicitly start a fresh session from the shared chat history.

Mootbox's runner API and this daemon use **protocol 7** (`x-mootbox-runner-protocol: 7`). Update the server and daemon together. Contracts are vendored in `contracts/runner/` and their generated Rust types are in `src/contracts.rs`.

## Code map

| Change | Where |
| --- | --- |
| A CLI flag or subcommand | `src/cli/mod.rs` (clap definitions, dispatch, printing) |
| What a command does | `src/commands/<name>.rs` (`add/` is split into checkout, workspace and placement) |
| A coding-agent CLI (Claude Code, Codex) | `src/providers/<name>/` implementing `providers::Provider` |
| Version and sign-in probes | `src/providers/probe.rs` |
| Claim loop, workspaces, reconcile, config reload | `src/daemon/` |
| One claimed run: lease, supervision, prompt, completion | `src/run/` |
| `mootbox doctor` checks | `src/diagnostics.rs` |
| `~/.mootbox/daemon.toml` | `src/config.rs` |
| Runner HTTP client | `src/api.rs` (wire types are generated in `src/contracts.rs`) |

Commands never print: they return `commands::Output`, or talk through `commands::Console` when they must prompt or report progress mid-flow. Only `src/cli/` writes to the terminal.

## Adding a provider

1. Create `src/providers/<name>/mod.rs` with a unit struct implementing `providers::Provider`: name, label, default command, minimum version, sign-in probe arguments and reader, launch plan (argv, environment, owned temp files) and the output fold into `ProviderResult`. Copy `src/providers/codex/` as the template.
2. Add `pub mod <name>;` and one entry to `providers::ALL` in `src/providers/mod.rs`. The claim request, `mootbox doctor`, `mootbox start` readiness, `--provider` parsing and help text all read that list.
3. Add a fake CLI under `tests/fixtures/` and an integration test like `tests/integration/codex.rs`.
4. The runner protocol must also list the provider name in its `provider` enums (`contracts/`, regenerated into `src/contracts.rs`). That is a coordinated protocol change with the Mootbox server; the unit test `every_registered_provider_has_a_runner_protocol_name` fails until it is done.

## Checks and real smoke

```sh
scripts/check.sh
```

This runs formatting, the no-comments check, the generated-contracts drift check, Clippy with warnings denied, and the Rust unit, contract-fixture and integration tests.

Enable the pre-commit hook (format, no-comments and Clippy checks) once per clone with `git config core.hooksPath .githooks`.

The real-provider smoke is driven by the main repository:

```sh
scripts/e2e.sh ../agent-mootbox --real
```

It builds the daemon and runs `scripts/mvp-smoke.mjs` in the main repository against disposable repositories and in-memory Mootbox state. It requires locally authenticated Claude and Codex CLIs and may consume provider usage. Its current evidence covers two simulated users on one physical machine; it is not a separate-machine or separate-provider-account test.

## Distribution

```sh
scripts/dist.sh <out-dir> [target-triple...]
```

The script builds release binaries and writes SHA-256 files. The Mootbox server can serve these through its installer endpoints when configured with `MOOTBOX_DAEMON_DIST_DIR`. Separate-machine distribution testing and applicable provider terms review remain open before distribution.

## Contract refresh

After the main repository's contract changes are committed, sync and regenerate the Rust types:

```sh
scripts/sync-contracts.sh ../agent-mootbox
cargo run -p gen-contracts
```

`contracts/SOURCE` records the source revision and protocol version.
