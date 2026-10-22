use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Result, bail};
use mootbox_daemon::api::{RUNNER_PROTOCOL_HEADER, RUNNER_PROTOCOL_VERSION};
use mootbox_daemon::commands::Console;
use mootbox_daemon::config::{self, Connection, DaemonConfig, WorkspaceProfile};
use reqwest::Url;
use serde_json::Value;
use uuid::Uuid;
use wiremock::MockBuilder;
use wiremock::matchers::{header, method, path};

pub const CREDENTIAL: &str = "mootbox_runner_test_credential";
pub const PAIRED_WORKSPACE_NAME: &str = "Acme Labs";

const FAKE_SERVICE_MANAGER: &str = "tests/fixtures/fake-service-manager.sh";
const CALLS_FILE: &str = "calls";
const OUTPUT_FILE: &str = "output";
const FAILING_FILE: &str = "failing";
const STOPPED_FILE: &str = "stopped";

pub fn fixture(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("contracts/runner/fixtures")
        .join(name);
    serde_json::from_str(&fs::read_to_string(path).expect("fixture exists")).expect("fixture is JSON")
}

pub fn fixture_uuid(name: &str, pointer: &str) -> Uuid {
    fixture(name)
        .pointer(pointer)
        .and_then(Value::as_str)
        .and_then(|text| text.parse().ok())
        .expect("fixture field is a UUID")
}

pub fn runner_route(http_method: &str, route: &str) -> MockBuilder {
    wiremock::Mock::given(method(http_method)).and(path(route)).and(header(
        RUNNER_PROTOCOL_HEADER,
        RUNNER_PROTOCOL_VERSION.to_string().as_str(),
    ))
}

pub fn authenticated_route(http_method: &str, route: &str) -> MockBuilder {
    runner_route(http_method, route).and(header("authorization", format!("Bearer {CREDENTIAL}").as_str()))
}

pub fn paired_config(mootbox_url: &str) -> DaemonConfig {
    let mut config = DaemonConfig::default();
    config.upsert_workspace(WorkspaceProfile {
        mootbox_url: mootbox_url.to_string(),
        workspace_id: Uuid::new_v4(),
        runner_id: Uuid::new_v4(),
        credential: CREDENTIAL.to_string(),
        workspace_name: Some(PAIRED_WORKSPACE_NAME.to_string()),
    });
    config
}

pub fn paired_home(mootbox_url: &str) -> (tempfile::TempDir, DaemonConfig) {
    let home = tempfile::tempdir().expect("temp dir");
    let config = paired_config(mootbox_url);
    config::save(home.path(), &config).expect("saves config");
    (home, config)
}

pub fn connect_checkout(config: &mut DaemonConfig, repository_id: Uuid, path: PathBuf) -> Uuid {
    let connection_id = repository_id;
    let connection = Connection {
        workspace_id: config.workspaces[0].workspace_id,
        repository_id,
        project_ids: BTreeSet::default(),
        path,
        remote: None,
    };
    config.upsert_connection(connection_id, connection);
    connection_id
}

pub fn checkout_of(config: &mut DaemonConfig, repository_id: Uuid) -> &mut Connection {
    config
        .connections
        .values_mut()
        .find(|connection| connection.repository_id == repository_id)
        .expect("the repository is connected")
}

pub fn git_repository() -> (tempfile::TempDir, PathBuf) {
    let directory = tempfile::tempdir().expect("temp dir");
    let status = Command::new("git")
        .args(["init", "--quiet"])
        .arg(directory.path())
        .status()
        .expect("git runs");
    assert!(status.success(), "git init failed");
    let path = directory.path().canonicalize().expect("canonical path");
    (directory, path)
}

pub struct SilentConsole;

impl Console for SilentConsole {
    fn choose(&self, question: &str, _choices: &[String]) -> Result<usize> {
        bail!("unexpected prompt: {question}")
    }

    fn ask(&self, question: &str, _suggestion: &str) -> Result<String> {
        bail!("unexpected question: {question}")
    }

    fn open_link(&self, _link: &Url) {}

    fn show(&self, _line: &str) {}

    fn warn(&self, _line: &str) {}
}

pub struct FakeServiceManager {
    directory: tempfile::TempDir,
    pub program: PathBuf,
}

impl FakeServiceManager {
    pub fn printing(output: &str) -> Self {
        let directory = tempfile::tempdir().expect("temp dir");
        let program = directory.path().join("manager");
        std::os::unix::fs::symlink(
            Path::new(env!("CARGO_MANIFEST_DIR")).join(FAKE_SERVICE_MANAGER),
            &program,
        )
        .expect("links the fake service manager");
        let manager = Self { directory, program };
        manager.write(OUTPUT_FILE, output);
        manager
    }

    pub fn failing_on(command: &str) -> Self {
        let manager = Self::printing("");
        manager.write(FAILING_FILE, command);
        manager
    }

    pub fn stop(&self, output: &str) {
        self.write(STOPPED_FILE, "");
        self.write(OUTPUT_FILE, output);
    }

    pub fn calls(&self) -> Vec<String> {
        fs::read_to_string(self.directory.path().join(CALLS_FILE))
            .unwrap_or_default()
            .lines()
            .map(str::to_string)
            .collect()
    }

    fn write(&self, name: &str, contents: &str) {
        fs::write(self.directory.path().join(name), contents).expect("configures the fake service manager");
    }
}
