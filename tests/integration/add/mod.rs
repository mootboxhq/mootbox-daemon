use std::collections::{BTreeSet, VecDeque};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::Mutex;
use std::time::Duration;

use anyhow::{Result, bail};
use mootbox_daemon::commands::Console;
use mootbox_daemon::commands::add::{
    self, AddOptions, BACKGROUND_HINT, CONNECT_ANOTHER_WORKSPACE, DETACHED_HEAD, NEW_PROJECT, NO_AUTHORITY, NO_COMMITS,
    NO_MOOTBOX_URL, NO_REMOTE, NOT_IN_REPOSITORY, PROJECT_HINT, WHICH_PROJECT, WHICH_WORKSPACE,
};
use mootbox_daemon::config::{
    self, Connection, DaemonConfig, HOME_OVERRIDE_VARIABLE, PROVIDER_COMMAND_VARIABLE, WorkspaceProfile,
};
use mootbox_daemon::service::{SERVICE_DIRECTORY_VARIABLE, SERVICE_MANAGER_VARIABLE};
use reqwest::Url;
use serde_json::{Value, json};
use uuid::Uuid;
use wiremock::matchers::{body_json, body_partial_json};
use wiremock::{MockServer, ResponseTemplate};

use crate::common::{
    CREDENTIAL, PAIRED_WORKSPACE_NAME, authenticated_route, fixture, fixture_uuid, paired_home, runner_route,
};
use crate::harness::{self, CLAIMING_PAUSED, requests_to, run_until_logged};

mod checkouts;
mod cli;
mod projects;
mod workspaces;

const DEVICE_NAME: &str = "maya-macbook";
const ORIGIN: &str = "git@github.com:acme/checkout.git";
const PAIRING_RESPONSE: &str = "start-pairing-response.json";
const PAIRED_RESPONSE: &str = "exchange-pairing-response.paired.json";
const MATCH_ROUTE: &str = "/runner/repositories/match";
const REGISTER_ROUTE: &str = "/runner/repositories";
const CONNECTIONS_ROUTE: &str = "/runner/connections";
const HTTP_FORBIDDEN: u16 = 403;
const HTTP_UPGRADE_REQUIRED: u16 = 426;
const FAST_POLL: Duration = Duration::from_millis(10);
const FAKE_PROVIDER: &str = "tests/fixtures/fake-claude.sh";
pub const ACCEPTING_MANAGER: &str = "true";
const REFUSING_MANAGER: &str = "false";
pub const SERVICE_FOLDER: &str = "services";

pub struct Folder {
    _parent: tempfile::TempDir,
    pub path: PathBuf,
}

#[derive(Default)]
struct ScriptedConsole {
    choices: Mutex<VecDeque<usize>>,
    answers: Mutex<VecDeque<String>>,
    questions: Mutex<Vec<(String, Vec<String>)>>,
    links: Mutex<Vec<Url>>,
}

impl ScriptedConsole {
    fn choosing(choices: &[usize]) -> Self {
        Self {
            choices: Mutex::new(choices.iter().copied().collect()),
            ..Self::default()
        }
    }

    fn questions(&self) -> Vec<(String, Vec<String>)> {
        self.questions.lock().expect("questions").clone()
    }

    fn links(&self) -> Vec<Url> {
        self.links.lock().expect("links").clone()
    }
}

impl Console for ScriptedConsole {
    fn choose(&self, question: &str, choices: &[String]) -> Result<usize> {
        self.questions
            .lock()
            .expect("questions")
            .push((question.to_string(), choices.to_vec()));
        match self.choices.lock().expect("choices").pop_front() {
            Some(choice) => Ok(choice),
            None => bail!("unexpected prompt: {question}"),
        }
    }

    fn ask(&self, question: &str, suggestion: &str) -> Result<String> {
        self.questions
            .lock()
            .expect("questions")
            .push((question.to_string(), vec![suggestion.to_string()]));
        match self.answers.lock().expect("answers").pop_front() {
            Some(answer) => Ok(answer),
            None => bail!("unexpected question: {question}"),
        }
    }

    fn open_link(&self, link: &Url) {
        self.links.lock().expect("links").push(link.clone());
    }

    fn show(&self, _line: &str) {}

    fn warn(&self, _line: &str) {}
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

fn plain_folder(name: &str) -> Folder {
    let parent = tempfile::tempdir().expect("temp dir");
    let path = parent.path().canonicalize().expect("canonical path").join(name);
    std::fs::create_dir(&path).expect("creates folder");
    Folder { _parent: parent, path }
}

fn repository_without_commits(name: &str) -> Folder {
    let folder = plain_folder(name);
    git(&folder.path, &["init", "--quiet", "-b", "main"]);
    folder
}

fn repository_without_remote(name: &str) -> Folder {
    let folder = repository_without_commits(name);
    harness::git(&folder.path, &["commit", "--quiet", "--allow-empty", "-m", "init"]);
    folder
}

pub fn checkout_repository(name: &str, remote: &str) -> Folder {
    let folder = repository_without_remote(name);
    git(&folder.path, &["remote", "add", "origin", remote]);
    folder
}

fn options(server: &MockServer, directory: &Path) -> AddOptions {
    AddOptions {
        workspace: None,
        mootbox_url: Some(Url::parse(&server.uri()).expect("URL")),
        project: None,
        new_workspace: false,
        device_name: DEVICE_NAME.to_string(),
        directory: directory.to_path_buf(),
        poll_interval: FAST_POLL,
    }
}

fn named(id: Uuid, name: &str) -> Value {
    json!({ "id": id, "name": name })
}

pub fn repo_match(repository_id: Uuid, project_id: Uuid, project_name: &str) -> Value {
    json!({
        "repository_id": repository_id,
        "repository_name": "checkout",
        "remote_name": "origin",
        "project": named(project_id, project_name),
        "team": named(Uuid::new_v4(), "Frontend"),
    })
}

pub async fn mount_match(server: &MockServer, result: Value) {
    runner_route("POST", MATCH_ROUTE)
        .respond_with(ResponseTemplate::new(200).set_body_json(result))
        .mount(server)
        .await;
}

pub async fn mount_connection(server: &MockServer, connection_id: Uuid) {
    runner_route("PUT", CONNECTIONS_ROUTE)
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "connection_id": connection_id })))
        .mount(server)
        .await;
}

async fn mount_pairing(server: &MockServer) {
    runner_route("POST", "/runner/pairings")
        .and(body_json(fixture("start-pairing-request.json")))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture(PAIRING_RESPONSE)))
        .expect(1)
        .mount(server)
        .await;
    let exchange = format!(
        "/runner/pairings/{}/exchange",
        fixture_uuid(PAIRING_RESPONSE, "/pairing_id")
    );
    runner_route("POST", &exchange)
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture(PAIRED_RESPONSE)))
        .mount(server)
        .await;
}

fn paired_profile(server: &MockServer) -> WorkspaceProfile {
    WorkspaceProfile {
        mootbox_url: Url::parse(&server.uri()).expect("URL").to_string(),
        workspace_id: fixture_uuid(PAIRED_RESPONSE, "/workspace_id"),
        runner_id: fixture_uuid(PAIRED_RESPONSE, "/runner_id"),
        credential: fixture(PAIRED_RESPONSE)["credential"]
            .as_str()
            .expect("credential")
            .to_string(),
        workspace_name: fixture(PAIRED_RESPONSE)["workspace_name"].as_str().map(str::to_string),
    }
}

fn sha256_hex(path: &Path) -> String {
    let output = Command::new("shasum")
        .args(["-a", "256"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            child
                .stdin
                .take()
                .expect("stdin")
                .write_all(path.to_str().expect("UTF-8").as_bytes())?;
            child.wait_with_output()
        })
        .expect("shasum runs");
    String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .next()
        .expect("digest")
        .to_string()
}

fn saved(home: &Path) -> DaemonConfig {
    config::load(home).expect("config")
}

fn connection_of(config: &DaemonConfig, connection_id: Uuid) -> &Connection {
    config.connections.get(&connection_id).expect("connection saved")
}

pub fn mootbox_command(home: &Path, directory: &Path, args: &[&str]) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mootbox"));
    command
        .current_dir(directory)
        .env(HOME_OVERRIDE_VARIABLE, home)
        .env(
            PROVIDER_COMMAND_VARIABLE,
            Path::new(env!("CARGO_MANIFEST_DIR")).join(FAKE_PROVIDER),
        )
        .env(SERVICE_DIRECTORY_VARIABLE, home.join(SERVICE_FOLDER))
        .env(SERVICE_MANAGER_VARIABLE, REFUSING_MANAGER)
        .args(args);
    command
}

pub async fn output_of(mut command: Command) -> Output {
    tokio::task::spawn_blocking(move || command.output().expect("mootbox runs"))
        .await
        .expect("mootbox finishes")
}

async fn mount_online_routes(server: &MockServer, project_id: Uuid) {
    mount_match(
        server,
        json!({ "kind": "one", "match": repo_match(Uuid::new_v4(), project_id, "Web App") }),
    )
    .await;
    mount_connection(server, Uuid::new_v4()).await;
    let mut health = fixture("health-response.json");
    health["server_time"] = json!(chrono::Utc::now().to_rfc3339());
    runner_route("GET", "/runner/health")
        .respond_with(ResponseTemplate::new(200).set_body_json(health))
        .mount(server)
        .await;
    runner_route("POST", "/runner/claim")
        .respond_with(
            ResponseTemplate::new(HTTP_UPGRADE_REQUIRED).set_body_json(fixture("error.upgrade-required.json")),
        )
        .mount(server)
        .await;
}

fn expected_report(server: &MockServer) -> String {
    let device = nix::unistd::gethostname()
        .expect("hostname")
        .into_string()
        .expect("UTF-8");
    format!(
        "✓ Signed in to {} (workspace {PAIRED_WORKSPACE_NAME})\n✓ Project Frontend / Web App\n✓ Repo checkout\n\
         ✓ Device {device}\n✓ Claude Code ready\n",
        server.address()
    )
}
