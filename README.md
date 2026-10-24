# Mootbox daemon

The Mootbox daemon connects a person's local checkouts and locally authenticated Claude Code and Codex CLIs to the Mootbox shared-chat service. It claims only work initiated by that person in a chat they can access, runs it in an isolated worktree, and publishes the result. A teammate's message or mention never starts another person's assistant.

The shared-chat MVP implementation is complete. The Rust formatting, Clippy and test checks pass, and the real Claude/Codex smoke completed two turns per provider and resumed after daemon restart. That smoke used two simulated users and separate daemon homes on one physical machine. Separate-machine/provider-account testing and provider terms review remain open; see [the main repository handover](../agent-mootbox/docs/product/HANDOVER.md) and [status checklist](../agent-mootbox/docs/product/PLAN.md).

## Install and connect

Install the latest release for macOS or Linux (x86_64 or aarch64), and update it later in place:

```sh
curl -fsSL https://mootbox.zbib.dev/install | sh
mootbox update
```

`mootbox update` downloads the latest [mootbox-daemon release](https://github.com/mootboxhq/mootbox-daemon/releases), verifies it against the release's `SHA256SUMS`, replaces the running executable and restarts the background service if one is installed. `mootbox update --check` only reports whether a newer release exists.

To build from source instead, with the Rust toolchain pinned in `rust-toolchain.toml`:

```sh
cargo install --path .
mootbox --help
```

In a checkout registered to a Mootbox project, connect the local checkout and pair the device when prompted:

```sh
cd ~/code/checkout
mootbox add --mootbox-url https://mootbox.example.com
```

To pair a device on its own, create a code with **Pair a device** on the Devices page and run `mootbox pair --mootbox-url https://mootbox.example.com --code ABCD-EFGH`. Without `--code`, `mootbox pair` asks for one, or prints a code to enter on the web when you leave it empty.

Use the Mootbox server URL configured for your environment. The person running the daemon must install and authenticate supported CLIs using their own provider flows. Mootbox does not collect provider credentials. The daemon stores its configuration under `~/.mootbox`; set `MOOTBOX_DAEMON_HOME` to use a separate profile/home.

Useful commands:

```sh
mootbox status
mootbox doctor
mootbox start
mootbox service install
mootbox service uninstall
mootbox update
```

The daemon runs in the foreground with `mootbox start`, or through the platform service manager after installation. Use `mootbox --help` for the current command surface and flags.

## Execution and sessions

The daemon claims work for the local user's own connected checkout and selected provider. Runs use isolated worktrees and are serialized for a given owner/chat, so Claude and Codex do not write to the same checkout at once. Cancellation stops the local child process; removed participants and cancelled runs cannot publish late results.

Provider session IDs stay on the local machine. Mootbox sends an opaque session key; the daemon stores its mapping to the provider session in owner-only files and resumes it after restart. If that local session is missing or cannot resume, the run reports the failure. The user can explicitly start a fresh session from the shared chat history.

Mootbox's runner API and this daemon use **protocol 9** (`x-mootbox-runner-protocol: 9`). Update the server and daemon together. Contracts are vendored in `contracts/runner/` and their generated Rust types are in `src/contracts.rs`.

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
| Release lookup and checksum verification for `mootbox update` | `src/release.rs` |
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

## Releases

Bump `version` in `Cargo.toml` and merge to `main`; CI publishes `v<version>` to [mootbox-daemon releases](https://github.com/mootboxhq/mootbox-daemon/releases). A push whose version already has a release publishes nothing.

Each release carries one archive per platform and a `SHA256SUMS` file in `sha256sum` format covering them:

| Platform | Asset | Built for |
| --- | --- | --- |
| macOS Apple silicon | `mootbox-macos-arm64.tar.gz` | `aarch64-apple-darwin` |
| macOS Intel | `mootbox-macos-x64.tar.gz` | `x86_64-apple-darwin` |
| Linux x86_64 and Windows via WSL2 | `mootbox-linux-x64.tar.gz` | `x86_64-unknown-linux-musl` |
| Linux ARM64 | `mootbox-linux-arm64.tar.gz` | `aarch64-unknown-linux-musl` |

Each archive holds `mootbox` (mode `0755`) and `README.md` at its top level, so `tar -xzf mootbox-<os>-<arch>.tar.gz` yields `mootbox` directly. Asset names carry no version, so the latest ones download from `https://github.com/mootboxhq/mootbox-daemon/releases/latest/download/<asset>`.

v0.1.0 shipped bare binaries named by target triple, so `mootbox update` on v0.1.0 cannot update itself to v0.1.1 or later. Reinstall it with `curl -fsSL https://mootbox.zbib.dev/install | sh`.

## Contract refresh

After the main repository's contract changes are committed, sync and regenerate the Rust types:

```sh
scripts/sync-contracts.sh ../agent-mootbox
cargo run -p gen-contracts
```

`contracts/SOURCE` records the source revision and protocol version.
