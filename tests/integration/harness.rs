use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::Duration;

use chrono::{TimeDelta, Utc};
use mootbox_daemon::config::DaemonConfig;
use mootbox_daemon::daemon::{self, DaemonOptions, DaemonTiming};
use mootbox_daemon::journal::Journal;
use mootbox_daemon::log_file;
use mootbox_daemon::providers::{ProviderOptions, SelectedProvider};
use mootbox_daemon::run::RunTiming;
use nix::errno::Errno;
use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;
use serde_json::{Value, json};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;
use wiremock::matchers::{body_partial_json, method, path, path_regex};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

use crate::common::{connect_checkout, fixture, paired_home};
pub use crate::fake_provider::{FakeProvider, Invocation};

pub const CLAIMING_PAUSED: &str = "claiming paused until Mootbox accepts this daemon's runner protocol";

const JOURNAL_FILE: &str = "journal.json";
const WAIT_TIMEOUT: Duration = Duration::from_secs(15);
const POLL_INTERVAL: Duration = Duration::from_millis(25);
const IDLE_CLAIM_DELAY: Duration = Duration::from_millis(50);
const LEASE_LENGTH: TimeDelta = TimeDelta::hours(1);
const FIRST_PRIORITY: u8 = 1;
const SECOND_PRIORITY: u8 = 2;
const FALLBACK_PRIORITY: u8 = 10;

pub struct TestDaemon {
    shutdown: CancellationToken,
    handle: JoinHandle<()>,
}

impl TestDaemon {
    pub fn start(options: DaemonOptions) -> Self {
        let shutdown = options.shutdown.clone();
        Self {
            shutdown,
            handle: tokio::spawn(daemon::run_daemon(options)),
        }
    }

    pub async fn stop(self) {
        self.request_stop();
        self.finished().await;
    }

    pub fn request_stop(&self) {
        self.shutdown.cancel();
    }

    pub async fn finished(self) {
        tokio::time::timeout(WAIT_TIMEOUT, self.handle)
            .await
            .expect("daemon stops in time")
            .expect("daemon task");
    }
}

pub fn daemon_options(config: DaemonConfig, home: &Path, provider: &FakeProvider) -> DaemonOptions {
    DaemonOptions {
        slots: config.slots,
        selected_provider: SelectedProvider {
            kind: config.provider,
            program: provider.program.clone(),
        },
        config,
        home: home.to_path_buf(),
        provider_options: ProviderOptions::default(),
        journal: Journal::open(home).expect("opens the journal"),
        timing: DaemonTiming {
            claim_wait: Duration::from_millis(100),
            backoff_base: Duration::from_millis(20),
            backoff_max: Duration::from_millis(100),
            reconcile_wait: Duration::from_millis(50),
            protocol_recheck: Duration::from_millis(100),
            shutdown_grace: Duration::from_secs(10),
        },
        run_timing: RunTiming {
            heartbeat_interval: Duration::from_millis(100),
            lease_safety: Duration::from_secs(5),
            wall_time: Duration::from_secs(60),
            cancel_grace: Duration::from_millis(300),
        },
        shutdown: CancellationToken::new(),
    }
}

fn future_lease() -> String {
    (Utc::now() + LEASE_LENGTH).to_rfc3339()
}

pub fn execute_claim(run_id: Uuid, repository_id: Uuid) -> Value {
    let branch = format!("mootbox/{run_id}");
    let mut body = fixture("claim-response.claimed-execute.json");
    let run = &mut body["run"];
    run["run_id"] = json!(run_id);
    run["repository_id"] = json!(repository_id);
    run["connection_id"] = json!(repository_id);
    run["manifest"]["session_key"] = json!(run_id);
    run["branch"] = json!(branch);
    run["lease_expires_at"] = json!(future_lease());
    run["manifest"]["run"]["id"] = json!(run_id);
    run["manifest"]["run"]["branch"] = json!(branch);
    run["manifest"]["target"]["repository_id"] = json!(repository_id);
    body
}

pub fn heartbeat_answer(is_cancel_requested: bool) -> Value {
    let mut body = fixture("heartbeat-response.json");
    body["lease_expires_at"] = json!(future_lease());
    body["cancel_requested"] = json!(is_cancel_requested);
    body
}

pub async fn mount_claim_once(server: &MockServer, response: ResponseTemplate) {
    Mock::given(method("POST"))
        .and(path("/runner/claim"))
        .respond_with(response)
        .up_to_n_times(1)
        .with_priority(SECOND_PRIORITY)
        .mount(server)
        .await;
}

pub async fn mount_claim_while_active(server: &MockServer, active_run_id: Uuid, response: Value) {
    Mock::given(method("POST"))
        .and(path("/runner/claim"))
        .and(body_partial_json(json!({ "active_run_ids": [active_run_id] })))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .up_to_n_times(1)
        .with_priority(FIRST_PRIORITY)
        .mount(server)
        .await;
}

pub async fn answer_later_heartbeats(server: &MockServer, response: ResponseTemplate) {
    Mock::given(method("POST"))
        .and(path_regex("^/runner/runs/[^/]+/heartbeat$"))
        .respond_with(response)
        .with_priority(FIRST_PRIORITY)
        .mount(server)
        .await;
}

pub async fn mount_mootbox(server: &MockServer, heartbeat: ResponseTemplate) {
    Mock::given(method("POST"))
        .and(path("/runner/claim"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(fixture("claim-response.idle.json"))
                .set_delay(IDLE_CLAIM_DELAY),
        )
        .with_priority(FALLBACK_PRIORITY)
        .mount(server)
        .await;
    Mock::given(method("POST"))
        .and(path_regex("^/runner/runs/[^/]+/heartbeat$"))
        .respond_with(heartbeat)
        .mount(server)
        .await;
    Mock::given(method("POST"))
        .and(path_regex("^/runner/runs/[^/]+/events$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("run-event-response.json")))
        .mount(server)
        .await;
    Mock::given(method("POST"))
        .and(path_regex("^/runner/runs/[^/]+/complete$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("complete-run-response.json")))
        .mount(server)
        .await;
}

pub fn run_request_index(requests: &[Request], run_id: Uuid, action: &str) -> Option<usize> {
    let route = format!("/runner/runs/{run_id}/{action}");
    requests.iter().position(|request| request.url.path() == route)
}

pub fn body_of(request: &Request) -> Value {
    serde_json::from_slice(&request.body).expect("JSON body")
}

pub async fn requests_to(server: &MockServer, route: &str) -> Vec<Value> {
    server
        .received_requests()
        .await
        .expect("recording")
        .iter()
        .filter(|request| request.url.path() == route)
        .map(body_of)
        .collect()
}

pub async fn completion_of(server: &MockServer, run_id: Uuid) -> Value {
    let route = format!("/runner/runs/{run_id}/complete");
    wait_until(|| async { requests_to(server, &route).await.into_iter().next() }).await
}

pub async fn wait_for_claims(server: &MockServer, count: usize) {
    wait_until(|| async { (requests_to(server, "/runner/claim").await.len() >= count).then_some(()) }).await;
}

pub fn daemon_log(home: &Path) -> String {
    fs::read_to_string(log_file::logs_directory(home).join(log_file::LOG_FILE)).unwrap_or_default()
}

pub async fn run_until_logged(mut command: Command, home: &Path, needle: &str) -> (Output, String) {
    let child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("mootbox starts");
    let pid = Pid::from_raw(i32::try_from(child.id()).expect("pid fits"));
    wait_until(|| async { daemon_log(home).contains(needle).then_some(()) }).await;
    kill(pid, Signal::SIGTERM).expect("signals mootbox");
    let output = tokio::task::spawn_blocking(move || child.wait_with_output().expect("mootbox exits"))
        .await
        .expect("mootbox finishes");
    (output, daemon_log(home))
}

pub async fn wait_for_exit(pid: Pid) {
    wait_until(|| async { (kill(pid, None) == Err(Errno::ESRCH)).then_some(()) }).await;
}

pub async fn wait_until<T, F, Probe>(probe: Probe) -> T
where
    Probe: Fn() -> F,
    F: Future<Output = Option<T>>,
{
    let deadline = tokio::time::Instant::now() + WAIT_TIMEOUT;
    loop {
        if let Some(value) = probe().await {
            return value;
        }
        assert!(tokio::time::Instant::now() < deadline, "condition not reached in time");
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

pub fn git(directory: &Path, arguments: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(directory)
        .args(["-c", "user.name=test", "-c", "user.email=test@example.com"])
        .args(arguments)
        .output()
        .expect("git runs");
    assert!(output.status.success(), "git {arguments:?} failed: {output:?}");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

pub fn committed_repository() -> (tempfile::TempDir, PathBuf) {
    let directory = tempfile::tempdir().expect("temp dir");
    let path = directory.path().canonicalize().expect("canonical path");
    git(&path, &["init", "--quiet", "-b", "main"]);
    fs::write(path.join("README.md"), "# checkout\n").expect("writes");
    git(&path, &["add", "README.md"]);
    git(&path, &["commit", "--quiet", "-m", "init"]);
    (directory, path)
}

pub struct Bound {
    _home: tempfile::TempDir,
    _repository: tempfile::TempDir,
    pub home: PathBuf,
    pub repository: PathBuf,
    pub repository_id: Uuid,
    pub config: DaemonConfig,
}

pub fn bound_home(server: &MockServer) -> Bound {
    let (home, mut config) = paired_home(&server.uri());
    let (repository_directory, repository) = committed_repository();
    let repository_id = Uuid::new_v4();
    connect_checkout(&mut config, repository_id, repository.clone());
    config.slots = 1;
    Bound {
        home: home.path().to_path_buf(),
        _home: home,
        _repository: repository_directory,
        repository,
        repository_id,
        config,
    }
}

pub fn claimed(body: Value) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(body)
}

pub fn healthy_heartbeats() -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(heartbeat_answer(false))
}

pub fn journal_on_disk(home: &Path) -> Vec<Value> {
    fs::read_to_string(home.join(JOURNAL_FILE))
        .map(|text| serde_json::from_str(&text).expect("journal is JSON"))
        .unwrap_or_default()
}

pub async fn wait_for_empty_journal(home: &Path) {
    wait_until(|| async { journal_on_disk(home).is_empty().then_some(()) }).await;
}
