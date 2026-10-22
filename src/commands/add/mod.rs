use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Result, anyhow, bail};
use reqwest::Url;
use uuid::Uuid;

use super::Console;
use super::service;
use super::start::{self, StartOptions};
use crate::api::{ApiError, MootboxClient};
use crate::config::{self, DaemonConfig};
use crate::diagnostics::{self, Check};
use crate::lock;
use crate::providers::{ProviderKind, probe};

mod checkout;
mod placement;
mod workspace;

use checkout::LocalCheckout;
use placement::Placement;

pub const NOT_IN_REPOSITORY: &str = "Run this inside your repo folder.";
pub const NO_REMOTE: &str =
    "This repository has no remote. Add one (git remote add origin …) and run mootbox add again.";
pub const DETACHED_HEAD: &str = "This checkout is not on a branch. Check out a branch and run mootbox add again.";
pub const NO_COMMITS: &str = "This repository has no commits yet. Make a first commit and run mootbox add again.";
pub const NO_AUTHORITY: &str =
    "No project you can access uses this repository. Ask your team lead to register it in Mootbox.";
pub const NO_MOOTBOX_URL: &str = "Pass --mootbox-url <url> to say which Mootbox server to connect to.";
pub const CONNECT_ANOTHER_WORKSPACE: &str = "Connect another workspace…";
pub const NEW_PROJECT: &str = "New project…";
pub const PROJECT_HINT: &str = "Run mootbox add in a terminal to choose, or pass --project <id>.";
pub const ALREADY_RUNNING: &str =
    "mootbox is already running on this machine; it picks this checkout up without a restart.";
pub const BACKGROUND_HINT: &str = "See it with mootbox status; stop it with mootbox service uninstall.";
pub const WHICH_WORKSPACE: &str = "Which workspace should this repository connect to?";
pub const WHICH_PROJECT: &str = "This repository is in several projects. Which one is this checkout for?";
pub const WHICH_TEAM: &str = "Which team should own this repository?";
pub const WHICH_REMOTE: &str = "Which remote identifies this repository in Mootbox?";
pub const NEW_PROJECT_NAME: &str = "Project name";

const CHECK_MARK: &str = "✓";
const MISSING_MARK: &str = "✗";
const ONLINE_MARK: &str = "●";
const ONLINE_SUBJECT: &str = "This computer is online";
const STOP_HINT: &str = "Ctrl+C to stop";

pub struct AddOptions {
    pub mootbox_url: Option<Url>,
    pub project: Option<Uuid>,
    pub new_workspace: bool,
    pub device_name: String,
    pub directory: PathBuf,
    pub poll_interval: Duration,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Serving {
    Background,
    Foreground,
}

pub struct AddRequest {
    pub options: AddOptions,
    pub start: StartOptions,
    pub serving: Serving,
}

#[derive(Debug)]
pub struct AddReport {
    pub mootbox_host: String,
    pub workspace_id: Uuid,
    pub workspace_name: String,
    pub team_name: String,
    pub project_name: String,
    pub repository_name: String,
    pub device_name: String,
    pub provider: ProviderKind,
    pub is_provider_ready: bool,
    pub is_daemon_running: bool,
}

struct Adding<'a> {
    options: &'a AddOptions,
    console: &'a dyn Console,
    mootbox_url: Url,
}

impl AddReport {
    pub fn lines(&self) -> Vec<String> {
        let label = self.provider.label();
        let readiness = if self.is_provider_ready {
            format!("{CHECK_MARK} {label} ready")
        } else {
            format!("{MISSING_MARK} {label} not ready; run mootbox doctor")
        };
        vec![
            format!(
                "{CHECK_MARK} Signed in to {} (workspace {})",
                self.mootbox_host, self.workspace_name
            ),
            format!("{CHECK_MARK} Project {} / {}", self.team_name, self.project_name),
            format!("{CHECK_MARK} Repo {}", self.repository_name),
            format!("{CHECK_MARK} Device {}", self.device_name),
            readiness,
        ]
    }

    fn show(&self, console: &dyn Console) {
        self.lines().iter().for_each(|line| console.show(line));
    }
}

pub async fn run(home: &Path, request: AddRequest, console: &dyn Console) -> Result<()> {
    let report = connect(home, request.options, console).await?;
    if report.is_daemon_running {
        report.show(console);
        console.show(ALREADY_RUNNING);
        return Ok(());
    }
    require_passing_checks(home, console).await?;
    report.show(console);
    match request.serving {
        Serving::Foreground => serve_in_foreground(home, request.start, console).await,
        Serving::Background => serve_in_background(home, request.start, console).await,
    }
}

pub async fn connect(home: &Path, options: AddOptions, console: &dyn Console) -> Result<AddReport> {
    let checkout = LocalCheckout::of(&options.directory, console).await?;
    let saved = config::load_or_start_fresh(home)?;
    let adding = Adding {
        mootbox_url: workspace::chosen_mootbox_url(options.mootbox_url.as_ref(), &saved)?,
        options: &options,
        console,
    };
    let mut config = saved.clone();
    let profile = adding.chosen_profile(&config, &checkout.root).await?;
    if !config.workspaces.contains(&profile) {
        config.upsert_workspace(profile.clone());
        config::save(home, &config)?;
    }
    let client = MootboxClient::for_profile(&profile)?;
    let remembered = placement::remembered_projects(&config, profile.workspace_id, &checkout.root);
    let placement = adding.placement(&client, &checkout, &remembered).await?;
    let connection_id = placement::connect_checkout(&client, &checkout, &placement).await?;
    record_connection(
        &mut config,
        connection_id,
        checkout.connection(profile.workspace_id, &placement),
    );
    if config != saved {
        config::save(home, &config)?;
    }
    let Placement {
        team_name,
        project_name,
        repository_name,
        ..
    } = placement;
    Ok(AddReport {
        mootbox_host: host_of(&adding.mootbox_url),
        workspace_id: profile.workspace_id,
        workspace_name: profile.display_name(),
        team_name,
        project_name,
        repository_name,
        device_name: options.device_name.clone(),
        provider: config.provider,
        is_provider_ready: is_provider_ready(&config).await,
        is_daemon_running: lock::is_held(home)?,
    })
}

pub fn online_line() -> String {
    format!("{ONLINE_MARK} {ONLINE_SUBJECT} — {STOP_HINT}")
}

pub fn background_online_line() -> String {
    format!("{ONLINE_MARK} {ONLINE_SUBJECT} in the background")
}

async fn require_passing_checks(home: &Path, console: &dyn Console) -> Result<()> {
    let failures: Vec<Check> = diagnostics::diagnose_connection(home)
        .await
        .into_iter()
        .filter(Check::is_failure)
        .collect();
    if failures.is_empty() {
        return Ok(());
    }
    for check in &failures {
        console.warn(&check.line());
    }
    bail!(
        "fix the {} failing check(s) above, then run mootbox add again",
        failures.len()
    )
}

async fn serve_in_background(home: &Path, start: StartOptions, console: &dyn Console) -> Result<()> {
    start::save_options(home, &start)?;
    if let Err(error) = service::run_in_background(home) {
        console.warn(&format!("warning: the background service did not start: {error:#}"));
        console.warn("serving in this terminal instead");
        return serve_in_foreground(home, start, console).await;
    }
    console.show(&background_online_line());
    console.show(BACKGROUND_HINT);
    Ok(())
}

async fn serve_in_foreground(home: &Path, start: StartOptions, console: &dyn Console) -> Result<()> {
    let runner = start::go_online(home, start).await?;
    console.show(&online_line());
    runner.serve().await;
    Ok(())
}

fn record_connection(config: &mut DaemonConfig, connection_id: Uuid, mut connection: config::Connection) {
    if let Some(saved) = config.connections.get(&connection_id) {
        connection.project_ids.extend(saved.project_ids.iter().copied());
    }
    config.upsert_connection(connection_id, connection);
}

async fn is_provider_ready(config: &DaemonConfig) -> bool {
    let selected = config.selected_provider();
    probe::supported_version(selected.kind, &selected.program).await.is_ok()
}

fn host_of(mootbox_url: &Url) -> String {
    let host = mootbox_url.host_str().unwrap_or_default();
    match mootbox_url.port() {
        Some(port) => format!("{host}:{port}"),
        None => host.to_string(),
    }
}

fn server_message(error: impl Into<anyhow::Error>) -> anyhow::Error {
    match error.into().downcast::<ApiError>() {
        Ok(ApiError::Mootbox { message, .. }) => anyhow!(message),
        Ok(other) => other.into(),
        Err(other) => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_online_lines_say_this_computer_is_online() {
        assert_eq!(online_line(), "● This computer is online — Ctrl+C to stop");
        assert_eq!(background_online_line(), "● This computer is online in the background");
    }

    #[test]
    fn the_mootbox_host_keeps_an_explicit_port() {
        assert_eq!(
            host_of(&Url::parse("http://127.0.0.1:3000/").expect("URL")),
            "127.0.0.1:3000"
        );
        assert_eq!(
            host_of(&Url::parse("https://mootbox.example.com").expect("URL")),
            "mootbox.example.com"
        );
    }

    #[test]
    fn mootbox_refusals_show_only_the_server_text() {
        let refusal = ApiError::Mootbox {
            status: 403,
            category: crate::contracts::ErrorErrorCategory::PermissionDenied,
            message: "pairing expired".to_string(),
        };

        assert_eq!(server_message(refusal).to_string(), "pairing expired");
        assert_eq!(server_message(anyhow!("other")).to_string(), "other");
    }
}
