use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use chrono::{DateTime, TimeDelta, Utc};
use mootbox_daemon::config::{self, DaemonConfig, HOME_OVERRIDE_VARIABLE, PROVIDER_COMMAND_VARIABLE};
use mootbox_daemon::service::{SERVICE_DIRECTORY_VARIABLE, SERVICE_MANAGER_VARIABLE, ServiceHost, ServiceKind};
use serde_json::json;
use uuid::Uuid;
use wiremock::{MockServer, ResponseTemplate};

use crate::common::{
    FakeServiceManager, authenticated_route, checkout_of, connect_checkout, fixture, git_repository, paired_home,
    runner_route,
};

mod failures;

const FAKE_PROVIDER: &str = "tests/fixtures/fake-claude.sh";
const HTTP_UNAUTHORIZED: u16 = 401;
const HTTP_NOT_FOUND: u16 = 404;
const HTTP_UPGRADE_REQUIRED: u16 = 426;
const UNREACHABLE_MOOTBOX: &str = "http://127.0.0.1:9";

struct Machine {
    home: tempfile::TempDir,
    services: tempfile::TempDir,
    manager: FakeServiceManager,
    claude_version: &'static str,
    claude_logged_in: &'static str,
    search_path: Option<PathBuf>,
}

impl Machine {
    fn paired(mootbox_url: &str) -> (Self, DaemonConfig) {
        let (home, config) = paired_home(mootbox_url);
        (Self::with_home(home), config)
    }

    fn unpaired() -> Self {
        Self::with_home(tempfile::tempdir().expect("temp dir"))
    }

    fn with_home(home: tempfile::TempDir) -> Self {
        let services = tempfile::tempdir().expect("temp dir");
        let manager = FakeServiceManager::printing("state = running");
        Self {
            home,
            services,
            manager,
            claude_version: "2.1.296",
            claude_logged_in: "true",
            search_path: None,
        }
    }

    fn install_service(&self) {
        let host = ServiceHost {
            kind: ServiceKind::of_this_machine().expect("supported platform"),
            directory: self.services.path().to_path_buf(),
            manager: self.manager.program.clone(),
            user_id: 0,
        };
        fs::write(host.file_path(), "service").expect("writes service file");
    }

    fn stop_service(&self) {
        self.manager.stop("state = waiting");
    }

    fn save(&self, config: &DaemonConfig) {
        config::save(self.home.path(), config).expect("saves config");
    }

    async fn doctor(&self) -> Report {
        let mut command = Command::new(env!("CARGO_BIN_EXE_mootbox"));
        command
            .env(HOME_OVERRIDE_VARIABLE, self.home.path())
            .env(
                PROVIDER_COMMAND_VARIABLE,
                Path::new(env!("CARGO_MANIFEST_DIR")).join(FAKE_PROVIDER),
            )
            .env("FAKE_CLAUDE_VERSION", self.claude_version)
            .env("FAKE_CLAUDE_LOGGED_IN", self.claude_logged_in)
            .env(SERVICE_DIRECTORY_VARIABLE, self.services.path())
            .env(SERVICE_MANAGER_VARIABLE, &self.manager.program)
            .arg("doctor");
        if let Some(search_path) = &self.search_path {
            command.env("PATH", search_path);
        }
        let output = tokio::task::spawn_blocking(move || command.output().expect("mootbox runs"))
            .await
            .expect("doctor finishes");
        Report(output)
    }
}

struct Report(Output);

impl Report {
    fn lines(&self) -> Vec<String> {
        String::from_utf8_lossy(&self.0.stdout)
            .lines()
            .map(str::to_string)
            .collect()
    }

    fn line(&self, subject: &str) -> String {
        let marker = format!(" {subject}: ");
        self.lines()
            .into_iter()
            .find(|line| line.contains(&marker))
            .unwrap_or_else(|| panic!("no {subject} line in {:?}", self.lines()))
    }

    fn has_subject(&self, subject: &str) -> bool {
        self.lines().iter().any(|line| line.contains(&format!(" {subject}: ")))
    }

    fn is_success(&self) -> bool {
        self.0.status.success()
    }

    fn stderr(&self) -> String {
        String::from_utf8_lossy(&self.0.stderr).into_owned()
    }
}

async fn accepting_server() -> MockServer {
    server_reporting_time(Utc::now()).await
}

async fn server_reporting_time(server_time: DateTime<Utc>) -> MockServer {
    let server = MockServer::start().await;
    let mut health = fixture("health-response.json");
    health["server_time"] = json!(server_time.to_rfc3339());
    authenticated_route("GET", "/runner/health")
        .respond_with(ResponseTemplate::new(200).set_body_json(health))
        .mount(&server)
        .await;
    server
}

fn git(directory: &Path, arguments: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(directory)
        .args(arguments)
        .status()
        .expect("git runs");
    assert!(status.success(), "git {arguments:?} failed");
}

fn bind(config: &mut DaemonConfig, path: PathBuf, remote: Option<&str>) -> Uuid {
    let repository_id = Uuid::new_v4();
    connect_checkout(config, repository_id, path);
    checkout_of(config, repository_id).remote = remote.map(str::to_string);
    repository_id
}

#[tokio::test]
async fn a_healthy_machine_passes_every_check_without_claiming_work() {
    let server = accepting_server().await;
    let (machine, mut config) = Machine::paired(&server.uri());
    let (_clone, path) = git_repository();
    git(&path, &["remote", "add", "origin", "https://example.com/checkout.git"]);
    let repository_id = bind(&mut config, path.clone(), Some("origin"));
    machine.save(&config);
    machine.install_service();

    let report = machine.doctor().await;

    assert!(report.is_success(), "{:?}\n{}", report.lines(), report.stderr());
    assert!(
        report.lines().iter().all(|line| line.starts_with("ok ")),
        "{:?}",
        report.lines()
    );
    assert_eq!(report.line("claude-code"), "ok   claude-code: Claude Code 2.1.296");
    assert_eq!(
        report.line("claude-code sign-in"),
        "ok   claude-code sign-in: signed in"
    );
    assert!(report.line("git").starts_with("ok   git: git version "));
    assert_eq!(
        report.line(&format!("repository {repository_id}")),
        format!("ok   repository {repository_id}: {} (remote: origin)", path.display())
    );
    assert_eq!(report.line("protocol"), "ok   protocol: runner protocol 9 accepted");
    assert!(report.line("clock").starts_with("ok   clock: within "));
    assert_eq!(report.line("service"), "ok   service: installed and loaded, running");
    assert_eq!(server.received_requests().await.expect("recording").len(), 1);
}

#[tokio::test]
async fn the_configured_provider_is_checked() {
    let server = accepting_server().await;
    let (machine, mut config) = Machine::paired(&server.uri());
    config.provider = "codex".parse().expect("known provider");
    machine.save(&config);

    let report = machine.doctor().await;

    assert!(report.has_subject("codex"), "{:?}", report.lines());
    assert!(!report.has_subject("claude-code"), "{:?}", report.lines());
}

#[tokio::test]
async fn warnings_alone_still_exit_zero() {
    let server = server_reporting_time(Utc::now() - TimeDelta::minutes(5)).await;
    let (mut machine, _) = Machine::paired(&server.uri());
    machine.claude_logged_in = "\"unknown\"}x";

    let report = machine.doctor().await;

    assert!(report.is_success(), "{:?}\n{}", report.lines(), report.stderr());
    assert!(
        report.line("clock").starts_with("warn clock: this machine is 30"),
        "{}",
        report.line("clock")
    );
    assert!(report.line("clock").contains("ahead of the server"));
    assert!(
        report
            .line("claude-code sign-in")
            .starts_with("warn claude-code sign-in: cannot read")
    );
    assert_eq!(
        report.line("service"),
        "warn service: not installed; run mootbox service install"
    );
}

#[tokio::test]
async fn a_server_without_the_health_route_warns_and_reads_the_clock_from_the_date_header() {
    let server = MockServer::start().await;
    let skewed = (Utc::now() + TimeDelta::minutes(2))
        .to_rfc2822()
        .replace("+0000", "GMT");
    runner_route("GET", "/runner/health")
        .respond_with(
            ResponseTemplate::new(HTTP_NOT_FOUND)
                .insert_header("date", skewed.as_str())
                .set_body_json(json!({
                    "error": { "category": "not_found", "message": "route GET /runner/health not found", "retryable": false }
                })),
        )
        .mount(&server)
        .await;
    let (machine, _) = Machine::paired(&server.uri());

    let report = machine.doctor().await;

    assert!(report.is_success(), "{:?}\n{}", report.lines(), report.stderr());
    assert!(
        report.line("server").starts_with("warn server: ") && report.line("server").contains("credential not checked"),
        "{}",
        report.line("server")
    );
    assert_eq!(report.line("protocol"), "ok   protocol: runner protocol 9 accepted");
    assert!(
        report.line("clock").contains("behind the server"),
        "{}",
        report.line("clock")
    );
}

#[tokio::test]
async fn a_loaded_service_that_is_not_running_warns() {
    let server = accepting_server().await;
    let (machine, _) = Machine::paired(&server.uri());
    machine.install_service();
    machine.stop_service();

    let report = machine.doctor().await;

    assert!(report.is_success(), "{:?}\n{}", report.lines(), report.stderr());
    assert!(
        report
            .line("service")
            .starts_with("warn service: installed and loaded, not running"),
        "{}",
        report.line("service")
    );
}

#[tokio::test]
async fn an_old_signed_out_claude_and_broken_bindings_fail_with_a_non_zero_exit() {
    let server = accepting_server().await;
    let (mut machine, mut config) = Machine::paired(&server.uri());
    machine.claude_version = "2.1.294";
    machine.claude_logged_in = "false";
    let missing = bind(&mut config, PathBuf::from("/nonexistent/checkout"), None);
    let plain_directory = tempfile::tempdir().expect("temp dir");
    let not_a_repository = bind(&mut config, plain_directory.path().to_path_buf(), None);
    let (_clone, path) = git_repository();
    let without_remote = bind(&mut config, path.clone(), Some("upstream"));
    machine.save(&config);

    let report = machine.doctor().await;

    assert!(!report.is_success());
    assert!(
        report.stderr().contains("doctor found 5 failing check(s)"),
        "{}",
        report.stderr()
    );
    assert_eq!(
        report.line("claude-code"),
        "fail claude-code: Claude Code 2.1.294 is too old; mootbox needs 2.1.295 or newer"
    );
    assert!(
        report
            .line("claude-code sign-in")
            .starts_with("fail claude-code sign-in: not signed in; run `")
    );
    assert_eq!(
        report.line(&format!("repository {missing}")),
        format!("fail repository {missing}: /nonexistent/checkout does not exist")
    );
    assert!(
        report
            .line(&format!("repository {not_a_repository}"))
            .ends_with("is not a git repository")
    );
    assert_eq!(
        report.line(&format!("repository {without_remote}")),
        format!(
            "fail repository {without_remote}: remote \"upstream\" is not configured in {}",
            path.display()
        )
    );
}
