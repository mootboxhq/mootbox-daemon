use std::path::Path;

use chrono::{DateTime, TimeDelta, Utc};

use crate::api::{ApiError, MootboxClient, RUNNER_PROTOCOL_VERSION, ServerHealth};
use crate::config::{self, Connection, DaemonConfig, WorkspaceProfile};
use crate::git;
use crate::providers::{SelectedProvider, probe};
use crate::service::ServiceHost;

pub const MAX_CLOCK_SKEW: TimeDelta = TimeDelta::seconds(30);

const NO_WORKSPACE: &str = "no workspace is paired with this machine yet";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Ok,
    Warn,
    Fail,
}

#[derive(Debug)]
pub struct Check {
    pub verdict: Verdict,
    pub subject: String,
    pub detail: String,
}

impl Check {
    fn new(verdict: Verdict, subject: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            verdict,
            subject: subject.into(),
            detail: detail.into(),
        }
    }

    fn ok(subject: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::new(Verdict::Ok, subject, detail)
    }

    fn warn(subject: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::new(Verdict::Warn, subject, detail)
    }

    fn fail(subject: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::new(Verdict::Fail, subject, detail)
    }

    pub fn is_failure(&self) -> bool {
        self.verdict == Verdict::Fail
    }

    pub fn line(&self) -> String {
        let label = match self.verdict {
            Verdict::Ok => "ok",
            Verdict::Warn => "warn",
            Verdict::Fail => "fail",
        };
        format!("{label:<4} {}: {}", self.subject, self.detail)
    }
}

pub async fn diagnose(home: &Path, service: Option<&ServiceHost>) -> Vec<Check> {
    let mut checks = diagnose_connection(home).await;
    checks.push(service_check(service));
    checks
}

pub async fn diagnose_connection(home: &Path) -> Vec<Check> {
    let config = config::load(home);
    let selected = match &config {
        Ok(config) => config.selected_provider(),
        Err(_) => DaemonConfig::default().selected_provider(),
    };
    let mut checks = provider_checks(&selected).await;
    checks.push(git_check().await);
    match config {
        Err(error) => checks.push(Check::fail("config", format!("{error:#}"))),
        Ok(config) => checks.extend(configured_checks(&config).await),
    }
    checks
}

async fn configured_checks(config: &DaemonConfig) -> Vec<Check> {
    if config.workspaces.is_empty() {
        return vec![Check::fail("config", NO_WORKSPACE)];
    }
    let mut checks = vec![Check::ok("config", paired_summary(config))];
    for connection in config.connections.values() {
        checks.push(repository_check(connection).await);
    }
    for profile in &config.workspaces {
        checks.extend(server_checks(profile).await);
    }
    checks
}

fn paired_summary(config: &DaemonConfig) -> String {
    config
        .workspaces
        .iter()
        .map(|profile| {
            format!(
                "paired as runner {} in workspace {} with {}",
                profile.runner_id, profile.workspace_id, profile.mootbox_url
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}

async fn provider_checks(selected: &SelectedProvider) -> Vec<Check> {
    let provider = selected.kind;
    let subject = provider.name();
    match probe::supported_version(provider, &selected.program).await {
        Ok(found) => vec![
            Check::ok(subject, format!("{} {found}", provider.label())),
            sign_in_check(selected).await,
        ],
        Err(error @ probe::VersionError::Unreadable { .. }) => vec![Check::fail(subject, error.to_string())],
        Err(error) => vec![Check::fail(subject, error.to_string()), sign_in_check(selected).await],
    }
}

async fn sign_in_check(selected: &SelectedProvider) -> Check {
    let subject = format!("{} sign-in", selected.kind.name());
    let program = selected.program.display();
    match probe::is_signed_in(selected.kind, &selected.program).await {
        Some(true) => Check::ok(subject, "signed in"),
        Some(false) => Check::fail(subject, format!("not signed in; run `{program}`")),
        None => Check::warn(subject, format!("cannot read the sign-in state of {program}")),
    }
}

async fn git_check() -> Check {
    match git::installed_version().await {
        Ok(version) => Check::ok("git", version),
        Err(error) => Check::fail("git", format!("{error:#}")),
    }
}

async fn repository_check(connection: &Connection) -> Check {
    let subject = format!("repository {}", connection.repository_id);
    let path = connection.path.as_path();
    if !path.exists() {
        return Check::fail(subject, format!("{} does not exist", path.display()));
    }
    if !git::is_repository(path).await {
        return Check::fail(subject, format!("{} is not a git repository", path.display()));
    }
    let Some(remote) = connection.remote.as_deref() else {
        return Check::ok(subject, path.display().to_string());
    };
    if !git::has_remote(path, remote).await {
        return Check::fail(
            subject,
            format!("remote {remote:?} is not configured in {}", path.display()),
        );
    }
    Check::ok(subject, format!("{} (remote: {remote})", path.display()))
}

async fn server_checks(profile: &WorkspaceProfile) -> Vec<Check> {
    let client = match MootboxClient::for_profile(profile) {
        Ok(client) => client,
        Err(error) => return vec![Check::fail("server", format!("{error:#}"))],
    };
    match client.health().await {
        Ok(ServerHealth::Reported(health)) => vec![
            Check::ok(
                "server",
                format!("{} answered; credential accepted", profile.mootbox_url),
            ),
            protocol_accepted(),
            clock_check(Some(health.server_time), Utc::now()),
        ],
        Ok(ServerHealth::RouteMissing { server_date }) => vec![
            Check::warn(
                "server",
                format!(
                    "{} answered without GET /runner/health (an older Mootbox); credential not checked",
                    profile.mootbox_url
                ),
            ),
            protocol_accepted(),
            clock_check(server_date, Utc::now()),
        ],
        Err(ApiError::UpgradeRequired { message }) => vec![
            Check::ok("server", format!("{} answered", profile.mootbox_url)),
            Check::fail(
                "protocol",
                format!("runner protocol {RUNNER_PROTOCOL_VERSION} refused: {message}"),
            ),
        ],
        Err(error) => vec![Check::fail("server", error.to_string())],
    }
}

fn protocol_accepted() -> Check {
    Check::ok(
        "protocol",
        format!("runner protocol {RUNNER_PROTOCOL_VERSION} accepted"),
    )
}

fn clock_check(server_date: Option<DateTime<Utc>>, local_date: DateTime<Utc>) -> Check {
    let Some(server_date) = server_date else {
        return Check::warn("clock", "the server sent no time and no readable Date header");
    };
    let skew = local_date - server_date;
    let seconds = skew.num_seconds().abs();
    if skew.abs() <= MAX_CLOCK_SKEW {
        return Check::ok("clock", format!("within {seconds}s of the server"));
    }
    let direction = if skew > TimeDelta::zero() { "ahead of" } else { "behind" };
    Check::warn(
        "clock",
        format!("this machine is {seconds}s {direction} the server; leases may expire early or late"),
    )
}

fn service_check(service: Option<&ServiceHost>) -> Check {
    let Some(service) = service else {
        return Check::warn("service", "no supported service manager on this platform");
    };
    let state = service.state();
    if !state.installed {
        return Check::warn("service", "not installed; run mootbox service install");
    }
    if !state.loaded {
        return Check::warn(
            "service",
            format!(
                "{} is not loaded; run mootbox service install",
                service.file_path().display()
            ),
        );
    }
    if !state.running {
        return Check::warn(
            "service",
            "installed and loaded, not running; see logs/service.err.log in the daemon home",
        );
    }
    Check::ok("service", "installed and loaded, running")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-10-17T12:00:00Z")
            .expect("valid")
            .with_timezone(&Utc)
    }

    #[test]
    fn a_clock_within_thirty_seconds_of_the_server_is_fine() {
        let check = clock_check(Some(now() - TimeDelta::seconds(30)), now());

        assert_eq!(check.verdict, Verdict::Ok);
        assert_eq!(check.detail, "within 30s of the server");
    }

    #[test]
    fn a_clock_more_than_thirty_seconds_off_warns_with_the_direction() {
        let ahead = clock_check(Some(now() - TimeDelta::seconds(31)), now());
        let behind = clock_check(Some(now() + TimeDelta::seconds(90)), now());

        assert_eq!(ahead.verdict, Verdict::Warn);
        assert!(ahead.detail.contains("31s ahead of the server"), "{}", ahead.detail);
        assert_eq!(behind.verdict, Verdict::Warn);
        assert!(behind.detail.contains("90s behind the server"), "{}", behind.detail);
    }

    #[test]
    fn a_missing_server_date_warns() {
        assert_eq!(clock_check(None, now()).verdict, Verdict::Warn);
    }

    #[test]
    fn lines_pad_the_verdict_to_one_column() {
        assert_eq!(
            Check::ok("git", "git version 2.50.0").line(),
            "ok   git: git version 2.50.0"
        );
        assert_eq!(Check::fail("config", "x").line(), "fail config: x");
    }
}
