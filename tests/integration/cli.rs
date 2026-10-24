use std::path::Path;
use std::process::{Command, Output};

use crate::common::{CREDENTIAL, authenticated_route, checkout_of, connect_checkout, fixture, paired_home};
use crate::harness::{CLAIMING_PAUSED, run_until_logged};
use mootbox_daemon::config::{self, HOME_OVERRIDE_VARIABLE, PROVIDER_COMMAND_VARIABLE};
use mootbox_daemon::lock;
use mootbox_daemon::service::{SERVICE_DIRECTORY_VARIABLE, SERVICE_MANAGER_VARIABLE};
use wiremock::{MockServer, ResponseTemplate};

const HTTP_UPGRADE_REQUIRED: u16 = 426;
const FAKE_PROVIDER: &str = "tests/fixtures/fake-claude.sh";
const FAKE_VERSION_VARIABLE: &str = "FAKE_CLAUDE_VERSION";
const SUPPORTED_CLAUDE_VERSION: &str = "2.1.296";
const OLD_CLAUDE_VERSION: &str = "2.1.294";
const FAKE_LOGGED_IN_VARIABLE: &str = "FAKE_CLAUDE_LOGGED_IN";
const REFUSING_MANAGER: &str = "false";
const SERVICE_FOLDER: &str = "services";

fn mootbox(home: &Path, args: &[&str]) -> Output {
    mootbox_with_claude(home, args, SUPPORTED_CLAUDE_VERSION)
}

fn mootbox_with_claude(home: &Path, args: &[&str], claude_version: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mootbox"))
        .env(HOME_OVERRIDE_VARIABLE, home)
        .env(
            PROVIDER_COMMAND_VARIABLE,
            Path::new(env!("CARGO_MANIFEST_DIR")).join(FAKE_PROVIDER),
        )
        .env(FAKE_VERSION_VARIABLE, claude_version)
        .env(SERVICE_DIRECTORY_VARIABLE, home.join(SERVICE_FOLDER))
        .env(SERVICE_MANAGER_VARIABLE, REFUSING_MANAGER)
        .args(args)
        .output()
        .expect("mootbox runs")
}

async fn start_daemon(home: &Path) -> Output {
    let home = home.to_path_buf();
    tokio::task::spawn_blocking(move || mootbox(&home, &["start"]))
        .await
        .expect("mootbox start finishes")
}

async fn start_daemon_until_logged(home: &Path, needle: &str) -> (Output, String) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mootbox"));
    command
        .env(HOME_OVERRIDE_VARIABLE, home)
        .env(
            PROVIDER_COMMAND_VARIABLE,
            Path::new(env!("CARGO_MANIFEST_DIR")).join(FAKE_PROVIDER),
        )
        .env(FAKE_VERSION_VARIABLE, SUPPORTED_CLAUDE_VERSION)
        .arg("start");
    run_until_logged(command, home, needle).await
}

fn stderr_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn stdout_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn allow_saves_a_single_command_rule_once() {
    let (home, _) = paired_home("http://localhost:3000");

    for _ in 0..2 {
        assert!(mootbox(home.path(), &["allow", "Bash(pnpm test:*)"]).status.success());
    }

    let saved = config::load(home.path()).expect("config");
    assert_eq!(saved.allowed_commands, ["Bash(pnpm test:*)"]);
}

#[test]
fn allow_rejects_chained_commands_and_keeps_the_config() {
    let (home, before) = paired_home("http://localhost:3000");

    let output = mootbox(home.path(), &["allow", "Bash(pnpm test && rm -rf /)"]);

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("expected a single command rule"));
    assert_eq!(config::load(home.path()).expect("config"), before);
}

#[test]
fn allow_warns_when_a_rule_permits_pushing() {
    let (home, _) = paired_home("http://localhost:3000");

    let output = mootbox(home.path(), &["allow", "Bash(git push:*)"]);

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("permits pushing"));
}

#[test]
fn status_prints_the_config_summary_without_the_credential() {
    let (home, config) = paired_home("http://localhost:3000");

    let output = mootbox(home.path(), &["status"]);

    let printed = stdout_of(&output);
    assert!(output.status.success());
    assert!(
        printed.contains(&config.workspaces[0].runner_id.to_string()),
        "{printed}"
    );
    assert!(printed.contains("slots: 2"), "{printed}");
    assert!(printed.ends_with("service: not installed\n"), "{printed}");
    assert!(!printed.contains(CREDENTIAL), "{printed}");
}

#[test]
fn status_lists_each_connected_checkout_with_its_workspace() {
    let (home, mut config) = paired_home("http://localhost:3000");
    let (checkout, payments) = (uuid::Uuid::new_v4(), uuid::Uuid::new_v4());
    connect_checkout(&mut config, checkout, "/work/checkout".into());
    connect_checkout(&mut config, payments, "/work/payments".into());
    checkout_of(&mut config, checkout).remote = Some("origin".to_string());
    config::save(home.path(), &config).expect("saves");

    let output = mootbox(home.path(), &["status"]);

    let printed = stdout_of(&output);
    let workspace_id = config.workspaces[0].workspace_id;
    assert!(output.status.success());
    assert!(printed.contains("connections: 2\n"), "{printed}");
    assert!(
        printed.contains(&format!(
            "  {checkout} -> /work/checkout (remote: origin) (workspace {workspace_id})\n"
        )),
        "{printed}"
    );
    assert!(
        printed.contains(&format!("  {payments} -> /work/payments (workspace {workspace_id})\n")),
        "{printed}"
    );
}

#[test]
fn status_shows_the_provider() {
    let (home, mut config) = paired_home("http://localhost:3000");
    config.provider = "codex".parse().expect("known provider");
    config::save(home.path(), &config).expect("saves");

    let output = mootbox(home.path(), &["status"]);

    assert!(stdout_of(&output).contains("provider: codex\n"), "{output:?}");
}

#[test]
fn commands_before_connecting_explain_how_to_connect() {
    let home = tempfile::tempdir().expect("temp dir");

    let output = mootbox(home.path(), &["status"]);

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("run mootbox login, then run mootbox inside your repo folder")
    );
}

#[tokio::test]
async fn start_refuses_while_another_daemon_holds_the_home() {
    let server = MockServer::start().await;
    let (home, _) = paired_home(&server.uri());
    let _running = lock::acquire(home.path()).expect("first daemon lock");

    let output = start_daemon(home.path()).await;

    assert!(!output.status.success());
    assert!(
        stderr_of(&output).contains("another mootbox daemon is already running"),
        "{output:?}"
    );
    assert!(server.received_requests().await.expect("recording").is_empty());
}

#[tokio::test]
async fn start_pauses_claiming_on_upgrade_required() {
    let server = MockServer::start().await;
    let (home, _) = paired_home(&server.uri());
    let upgrade = fixture("error.upgrade-required.json");
    let server_text = upgrade["error"]["message"].as_str().expect("message").to_string();
    authenticated_route("POST", "/runner/claim")
        .respond_with(ResponseTemplate::new(HTTP_UPGRADE_REQUIRED).set_body_json(upgrade))
        .mount(&server)
        .await;

    let (output, logs) = start_daemon_until_logged(home.path(), CLAIMING_PAUSED).await;

    assert!(output.status.success(), "{output:?}");
    assert_eq!(logs.matches(CLAIMING_PAUSED).count(), 1, "{logs}");
    assert!(logs.contains("requires a newer mootbox"), "{logs}");
    assert!(logs.contains(&server_text), "{logs}");
}

#[test]
fn start_refuses_a_claude_code_older_than_the_verified_minimum() {
    let (home, _) = paired_home("http://127.0.0.1:9");

    let output = mootbox_with_claude(home.path(), &["start"], OLD_CLAUDE_VERSION);

    let shown = stderr_of(&output);
    assert!(!output.status.success());
    assert!(
        shown.contains("Claude Code 2.1.294 is too old; mootbox needs 2.1.295 or newer"),
        "{shown}"
    );
}

#[test]
fn start_refuses_when_claude_code_is_not_signed_in() {
    let (home, _) = paired_home("http://127.0.0.1:9");

    let output = Command::new(env!("CARGO_BIN_EXE_mootbox"))
        .env(HOME_OVERRIDE_VARIABLE, home.path())
        .env(
            PROVIDER_COMMAND_VARIABLE,
            Path::new(env!("CARGO_MANIFEST_DIR")).join(FAKE_PROVIDER),
        )
        .env(FAKE_LOGGED_IN_VARIABLE, "false")
        .args(["start"])
        .output()
        .expect("mootbox runs");

    let shown = stderr_of(&output);
    assert!(!output.status.success());
    assert!(
        shown.contains("Claude Code is not ready: authentication_required"),
        "{shown}"
    );
}
