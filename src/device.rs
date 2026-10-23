use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context as _, Result};
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::process::Command;
use tokio::time::Instant;

use crate::providers::probe::{self, Installation};
use crate::providers::{self, ProviderKind};

pub const DEVICE_REPORT_INTERVAL: Duration = Duration::from_mins(5);

const DAEMON_VERSION: &str = env!("CARGO_PKG_VERSION");
const OS_RELEASE_PATH: &str = "/etc/os-release";
const OS_RELEASE_VERSION_KEY: &str = "VERSION_ID=";
const MACOS_VERSION_PROGRAM: &str = "sw_vers";
const MACOS_VERSION_FLAG: &str = "-productVersion";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SignIn {
    pub kind: ProviderKind,
    pub signed_in: Option<bool>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Install {
    pub kind: ProviderKind,
    pub installation: Installation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inventory {
    pub os_version: Option<String>,
    pub installs: Vec<Install>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ProviderStatus {
    pub name: &'static str,
    pub installed: bool,
    pub version: Option<String>,
    pub signed_in: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DeviceReport {
    pub os: &'static str,
    pub os_version: Option<String>,
    pub daemon_version: &'static str,
    pub providers: Vec<ProviderStatus>,
}

impl DeviceReport {
    pub fn assemble(inventory: &Inventory, sign_ins: &[SignIn]) -> Self {
        let providers = inventory
            .installs
            .iter()
            .map(|install| provider_status(install, sign_ins))
            .collect();
        Self {
            os: os_label(std::env::consts::OS),
            os_version: inventory.os_version.clone(),
            daemon_version: DAEMON_VERSION,
            providers,
        }
    }

    pub fn contract<Contract: DeserializeOwned>(&self) -> Result<Contract> {
        serde_json::to_value(self)
            .and_then(serde_json::from_value)
            .context("the device report does not fit the runner protocol")
    }
}

fn provider_status(install: &Install, sign_ins: &[SignIn]) -> ProviderStatus {
    let signed_in = sign_ins
        .iter()
        .find(|sign_in| sign_in.kind == install.kind)
        .and_then(|sign_in| sign_in.signed_in);
    match install.installation {
        Installation::Missing => ProviderStatus {
            name: install.kind.name(),
            installed: false,
            version: None,
            signed_in: None,
        },
        Installation::Installed(version) => ProviderStatus {
            name: install.kind.name(),
            installed: true,
            version: version.map(|found| found.to_string()),
            signed_in,
        },
    }
}

pub fn os_label(os: &str) -> &'static str {
    match os {
        "macos" => "macOS",
        "linux" => "Linux",
        "windows" => "Windows",
        _ => "other",
    }
}

pub fn os_release_version(os_release: &str) -> Option<String> {
    os_release
        .lines()
        .find_map(|line| line.trim().strip_prefix(OS_RELEASE_VERSION_KEY))
        .map(|value| value.trim().trim_matches(['"', '\'']).to_string())
        .filter(|value| !value.is_empty())
}

async fn os_version() -> Option<String> {
    match std::env::consts::OS {
        "macos" => macos_version().await,
        "linux" => std::fs::read_to_string(OS_RELEASE_PATH)
            .ok()
            .and_then(|text| os_release_version(&text)),
        _ => None,
    }
}

async fn macos_version() -> Option<String> {
    let output = Command::new(MACOS_VERSION_PROGRAM)
        .arg(MACOS_VERSION_FLAG)
        .kill_on_drop(true)
        .output()
        .await
        .ok()?;
    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (output.status.success() && !version.is_empty()).then_some(version)
}

pub async fn take_inventory(program_for: impl Fn(ProviderKind) -> PathBuf) -> Inventory {
    let probes = providers::ALL.map(|kind| {
        let program = program_for(kind);
        tokio::spawn(async move { (kind, probe::installation(&program).await) })
    });
    let mut installs = Vec::new();
    for probe in probes {
        if let Ok((kind, installation)) = probe.await {
            installs.push(Install { kind, installation });
        }
    }
    Inventory {
        os_version: os_version().await,
        installs,
    }
}

pub async fn check_sign_ins(program_for: impl Fn(ProviderKind) -> PathBuf) -> Vec<SignIn> {
    let probes = providers::ALL.map(|kind| {
        let program = program_for(kind);
        tokio::spawn(async move { (kind, probe::is_signed_in(kind, &program).await) })
    });
    let mut sign_ins = Vec::new();
    for probe in probes {
        if let Ok((kind, signed_in)) = probe.await {
            sign_ins.push(SignIn { kind, signed_in });
        }
    }
    sign_ins
}

pub async fn report_device(program_for: impl Fn(ProviderKind) -> PathBuf + Copy) -> DeviceReport {
    let inventory = take_inventory(program_for).await;
    DeviceReport::assemble(&inventory, &check_sign_ins(program_for).await)
}

pub fn default_program(kind: ProviderKind) -> PathBuf {
    PathBuf::from(kind.default_command())
}

struct SentReport {
    inventoried_at: Instant,
    inventory: Inventory,
    report: DeviceReport,
}

#[derive(Default)]
pub struct DeviceReporter {
    last: Option<SentReport>,
}

impl DeviceReporter {
    pub fn fresh_inventory(&self, now: Instant) -> Option<Inventory> {
        self.last
            .as_ref()
            .filter(|last| now.duration_since(last.inventoried_at) < DEVICE_REPORT_INTERVAL)
            .map(|last| last.inventory.clone())
    }

    pub fn due(&mut self, now: Instant, inventory: Inventory, sign_ins: &[SignIn]) -> Option<DeviceReport> {
        let report = DeviceReport::assemble(&inventory, sign_ins);
        match &mut self.last {
            Some(last) if now.duration_since(last.inventoried_at) < DEVICE_REPORT_INTERVAL => {
                if last.report == report {
                    return None;
                }
                last.report = report.clone();
            }
            _ => {
                self.last = Some(SentReport {
                    inventoried_at: now,
                    inventory,
                    report: report.clone(),
                });
            }
        }
        Some(report)
    }

    pub fn forget(&mut self) {
        self.last = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::{ALL, Version};

    fn kind(name: &str) -> ProviderKind {
        name.parse().expect("known provider")
    }

    fn inventory() -> Inventory {
        Inventory {
            os_version: Some("15.1".to_string()),
            installs: vec![
                Install {
                    kind: kind("claude-code"),
                    installation: Installation::Installed(Some(Version(2, 1, 296))),
                },
                Install {
                    kind: kind("codex"),
                    installation: Installation::Missing,
                },
            ],
        }
    }

    fn signed_in(claude: Option<bool>) -> Vec<SignIn> {
        vec![
            SignIn {
                kind: kind("claude-code"),
                signed_in: claude,
            },
            SignIn {
                kind: kind("codex"),
                signed_in: Some(true),
            },
        ]
    }

    #[test]
    fn operating_systems_are_named_for_people() {
        assert_eq!(os_label("macos"), "macOS");
        assert_eq!(os_label("linux"), "Linux");
        assert_eq!(os_label("windows"), "Windows");
        assert_eq!(os_label("freebsd"), "other");
    }

    #[test]
    fn the_linux_version_is_read_from_os_release() {
        let os_release = "NAME=\"Ubuntu\"\nVERSION_ID=\"24.04\"\nID=ubuntu\n";

        assert_eq!(os_release_version(os_release), Some("24.04".to_string()));
        assert_eq!(os_release_version("VERSION_ID=12\n"), Some("12".to_string()));
        assert_eq!(os_release_version("NAME=Arch\n"), None);
        assert_eq!(os_release_version("VERSION_ID=\"\"\n"), None);
    }

    #[test]
    fn a_report_lists_versions_and_sign_in_and_hides_sign_in_for_missing_tools() {
        let report = DeviceReport::assemble(&inventory(), &signed_in(Some(true)));

        assert_eq!(report.os_version.as_deref(), Some("15.1"));
        assert_eq!(report.daemon_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(
            report.providers,
            [
                ProviderStatus {
                    name: "claude-code",
                    installed: true,
                    version: Some("2.1.296".to_string()),
                    signed_in: Some(true),
                },
                ProviderStatus {
                    name: "codex",
                    installed: false,
                    version: None,
                    signed_in: None,
                },
            ]
        );
    }

    #[test]
    fn a_report_converts_into_the_runner_contracts() {
        let report = DeviceReport::assemble(&inventory(), &signed_in(None));

        let claim: crate::contracts::ClaimRequestDevice = report.contract().expect("fits the claim");
        let pairing: crate::contracts::ClaimPairingCodeRequestDevice = report.contract().expect("fits the pairing");

        assert_eq!(serde_json::to_value(&claim).ok(), serde_json::to_value(&report).ok());
        assert_eq!(serde_json::to_value(&pairing).ok(), serde_json::to_value(&report).ok());
    }

    #[test]
    fn the_reporter_sends_first_then_only_on_change_or_after_the_interval() {
        let mut reporter = DeviceReporter::default();
        let start = Instant::now();

        assert!(reporter.fresh_inventory(start).is_none());
        assert!(reporter.due(start, inventory(), &signed_in(Some(true))).is_some());
        let soon = start + Duration::from_secs(25);
        let cached = reporter.fresh_inventory(soon).expect("inventory is reused");
        assert!(reporter.due(soon, cached.clone(), &signed_in(Some(true))).is_none());
        let signed_out = reporter
            .due(soon, cached, &signed_in(Some(false)))
            .expect("a sign-in change is sent");
        assert_eq!(signed_out.providers[0].signed_in, Some(false));

        let later = start + DEVICE_REPORT_INTERVAL;
        assert!(reporter.fresh_inventory(later).is_none());
        assert!(reporter.due(later, inventory(), &signed_in(Some(false))).is_some());
    }

    #[test]
    fn a_forgotten_report_is_sent_again() {
        let mut reporter = DeviceReporter::default();
        let now = Instant::now();
        assert!(reporter.due(now, inventory(), &signed_in(Some(true))).is_some());

        reporter.forget();

        assert!(reporter.fresh_inventory(now).is_none());
        assert!(reporter.due(now, inventory(), &signed_in(Some(true))).is_some());
    }

    #[tokio::test]
    async fn every_registered_provider_is_reported_even_when_missing() {
        let report = report_device(|_| PathBuf::from("/nonexistent/provider")).await;

        assert_eq!(report.providers.len(), ALL.len());
        assert!(
            report
                .providers
                .iter()
                .all(|status| !status.installed && status.signed_in.is_none())
        );
    }
}
