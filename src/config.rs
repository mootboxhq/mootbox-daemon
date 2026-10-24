use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::private_files;
use crate::providers::{ProviderKind, SelectedProvider};

pub const HOME_OVERRIDE_VARIABLE: &str = "MOOTBOX_DAEMON_HOME";
pub const PROVIDER_COMMAND_VARIABLE: &str = "MOOTBOX_DAEMON_PROVIDER_COMMAND";
pub const RELEASES_URL_VARIABLE: &str = "MOOTBOX_RELEASES_URL";
pub const DEFAULT_RELEASES_URL: &str = "https://github.com/mootboxhq/mootbox-daemon";
pub const DEFAULT_SLOTS: u32 = 2;
pub const DEFAULT_WALL_TIME_MINUTES: u32 = 30;
pub const DEFAULT_SHUTDOWN_GRACE_SECONDS: u64 = 30;
pub const DEFAULT_MOOTBOX_URL: &str = "https://mootbox.zbib.dev";
pub const MOOTBOX_URL_VARIABLE: &str = "MOOTBOX_URL";

pub const NOT_CONNECTED: &str =
    "this machine is not connected to Mootbox; run mootbox login, then run mootbox inside your repo folder";

const CONFIG_FILE: &str = "daemon.toml";
const HOME_DIRECTORY: &str = ".mootbox";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceProfile {
    pub mootbox_url: String,
    pub workspace_id: Uuid,
    pub runner_id: Uuid,
    pub credential: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CliSession {
    pub mootbox_url: String,
    pub token: String,
    pub email: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Connection {
    pub workspace_id: Uuid,
    pub repository_id: Uuid,
    #[serde(default)]
    pub project_ids: BTreeSet<Uuid>,
    pub path: PathBuf,
    #[serde(default, alias = "push_remote", skip_serializing_if = "Option::is_none")]
    pub remote: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DaemonConfig {
    #[serde(default)]
    pub workspaces: Vec<WorkspaceProfile>,
    #[serde(default)]
    pub connections: BTreeMap<Uuid, Connection>,
    #[serde(default)]
    pub provider: ProviderKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default = "default_slots")]
    pub slots: u32,
    #[serde(default = "default_wall_time_minutes")]
    pub wall_time_minutes: u32,
    #[serde(default = "default_shutdown_grace_seconds")]
    pub shutdown_grace_seconds: u64,
    #[serde(default)]
    pub allowed_commands: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_command: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_budget_usd: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session: Option<CliSession>,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            workspaces: Vec::new(),
            connections: BTreeMap::new(),
            provider: ProviderKind::default(),
            model: None,
            slots: DEFAULT_SLOTS,
            wall_time_minutes: DEFAULT_WALL_TIME_MINUTES,
            shutdown_grace_seconds: DEFAULT_SHUTDOWN_GRACE_SECONDS,
            allowed_commands: Vec::new(),
            provider_command: None,
            max_budget_usd: None,
            session: None,
        }
    }
}

impl WorkspaceProfile {
    pub fn display_name(&self) -> String {
        self.workspace_name
            .clone()
            .unwrap_or_else(|| self.workspace_id.to_string())
    }

    fn is_same_workspace(&self, other: &WorkspaceProfile) -> bool {
        self.mootbox_url == other.mootbox_url && self.workspace_id == other.workspace_id
    }
}

impl DaemonConfig {
    pub fn upsert_workspace(&mut self, profile: WorkspaceProfile) {
        match self
            .workspaces
            .iter_mut()
            .find(|saved| saved.is_same_workspace(&profile))
        {
            Some(saved) => *saved = profile,
            None => self.workspaces.push(profile),
        }
    }

    pub fn upsert_connection(&mut self, id: Uuid, connection: Connection) {
        self.connections.insert(id, connection);
    }

    pub fn selected_provider(&self) -> SelectedProvider {
        let program = std::env::var_os(PROVIDER_COMMAND_VARIABLE)
            .map(PathBuf::from)
            .or_else(|| self.provider_command.clone())
            .unwrap_or_else(|| PathBuf::from(self.provider.default_command()));
        SelectedProvider {
            kind: self.provider,
            program,
        }
    }
}

fn default_slots() -> u32 {
    DEFAULT_SLOTS
}

fn default_wall_time_minutes() -> u32 {
    DEFAULT_WALL_TIME_MINUTES
}

fn default_shutdown_grace_seconds() -> u64 {
    DEFAULT_SHUTDOWN_GRACE_SECONDS
}

pub fn daemon_home() -> Result<PathBuf> {
    if let Some(home) = std::env::var_os(HOME_OVERRIDE_VARIABLE) {
        return Ok(PathBuf::from(home));
    }
    let base = directories::BaseDirs::new().context("cannot find the home directory")?;
    Ok(base.home_dir().join(HOME_DIRECTORY))
}

pub fn config_path(home: &Path) -> PathBuf {
    home.join(CONFIG_FILE)
}

pub fn load(home: &Path) -> Result<DaemonConfig> {
    let path = config_path(home);
    if !path.exists() {
        bail!("{NOT_CONNECTED} (looked in {})", path.display());
    }
    let text = fs::read_to_string(&path).with_context(|| format!("cannot read {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("{} is not a valid daemon config", path.display()))
}

pub fn load_or_start_fresh(home: &Path) -> Result<DaemonConfig> {
    if !config_path(home).exists() {
        return Ok(DaemonConfig::default());
    }
    load(home)
}

pub fn save(home: &Path, config: &DaemonConfig) -> Result<()> {
    private_files::create_private_directory(home)?;
    private_files::replace_owner_only(&config_path(home), toml::to_string(config)?.as_bytes())
}

#[cfg(test)]
mod tests {
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    use super::*;
    #[cfg(unix)]
    use crate::private_files::{OWNER_ONLY_DIRECTORY, OWNER_ONLY_FILE};

    const MOOTBOX_URL: &str = "http://localhost:3000/";

    #[cfg(unix)]
    fn mode_of(path: &Path) -> u32 {
        fs::metadata(path).expect("path exists").permissions().mode() & 0o777
    }

    fn profile(workspace_id: Uuid, credential: &str) -> WorkspaceProfile {
        WorkspaceProfile {
            mootbox_url: MOOTBOX_URL.to_string(),
            workspace_id,
            runner_id: Uuid::new_v4(),
            credential: credential.to_string(),
            workspace_name: Some("Acme".to_string()),
        }
    }

    fn sample_config() -> DaemonConfig {
        let workspace_id = Uuid::new_v4();
        let mut config = DaemonConfig::default();
        config.upsert_workspace(profile(workspace_id, "mootbox_runner_secret"));
        config.upsert_connection(
            Uuid::new_v4(),
            Connection {
                workspace_id,
                repository_id: Uuid::new_v4(),
                project_ids: BTreeSet::from([Uuid::new_v4(), Uuid::new_v4()]),
                path: PathBuf::from("/work/checkout"),
                remote: Some("origin".to_string()),
            },
        );
        config.allowed_commands = vec!["Bash(pnpm test:*)".to_string()];
        config.provider = "codex".parse().expect("known provider");
        config.model = Some("opus".to_string());
        config.max_budget_usd = Some(2.5);
        config.session = Some(CliSession {
            mootbox_url: MOOTBOX_URL.to_string(),
            token: "mootbox_cli_secret".to_string(),
            email: "maya@example.com".to_string(),
        });
        config
    }

    #[test]
    fn saved_config_loads_back_unchanged() {
        let home = tempfile::tempdir().expect("temp dir");
        let config = sample_config();

        save(home.path(), &config).expect("saves");

        assert_eq!(load(home.path()).expect("loads"), config);
    }

    #[test]
    fn an_empty_config_loads_with_defaults() {
        let home = tempfile::tempdir().expect("temp dir");
        fs::write(config_path(home.path()), "").expect("writes");

        let loaded = load(home.path()).expect("loads");

        assert_eq!(loaded, DaemonConfig::default());
        assert_eq!(loaded.slots, DEFAULT_SLOTS);
        assert_eq!(loaded.wall_time_minutes, DEFAULT_WALL_TIME_MINUTES);
        assert_eq!(loaded.shutdown_grace_seconds, DEFAULT_SHUTDOWN_GRACE_SECONDS);
    }

    #[test]
    fn a_saved_push_remote_loads_as_the_remote() {
        let home = tempfile::tempdir().expect("temp dir");
        let connection_id = Uuid::new_v4();
        let text = format!(
            "[connections.{connection_id}]\nworkspace_id = \"{}\"\nrepository_id = \"{}\"\n\
             path = \"/work/checkout\"\npush_remote = \"upstream\"\n",
            Uuid::new_v4(),
            Uuid::new_v4()
        );
        fs::write(config_path(home.path()), text).expect("writes");

        let loaded = load(home.path()).expect("loads");

        assert_eq!(loaded.connections[&connection_id].remote.as_deref(), Some("upstream"));
    }

    #[test]
    fn unknown_fields_and_providers_are_refused() {
        let home = tempfile::tempdir().expect("temp dir");
        for (text, named) in [
            ("slots = 2\nagents = [\"backend\"]\n", "agents"),
            ("provider = \"gemini\"\n", "unknown provider"),
        ] {
            fs::write(config_path(home.path()), text).expect("writes");

            let error = load(home.path()).expect_err("refused");

            assert!(format!("{error:#}").contains(named), "{error:#}");
        }
    }

    #[test]
    fn upsert_second_workspace_keeps_first() {
        let mut config = DaemonConfig::default();
        let first = profile(Uuid::new_v4(), "first");
        let second = profile(Uuid::new_v4(), "second");

        config.upsert_workspace(first.clone());
        config.upsert_workspace(second.clone());

        assert_eq!(config.workspaces, [first, second]);
    }

    #[test]
    fn upsert_same_workspace_replaces_credential_only() {
        let mut config = DaemonConfig::default();
        let workspace_id = Uuid::new_v4();
        let other = profile(Uuid::new_v4(), "other");
        config.upsert_workspace(profile(workspace_id, "old"));
        config.upsert_workspace(other.clone());
        let renewed = profile(workspace_id, "new");

        config.upsert_workspace(renewed.clone());

        assert_eq!(config.workspaces, [renewed, other]);
    }

    #[test]
    fn the_same_workspace_on_another_server_is_a_separate_profile() {
        let mut config = DaemonConfig::default();
        let local = profile(Uuid::new_v4(), "local");
        let hosted = WorkspaceProfile {
            mootbox_url: "https://mootbox.example.com/".to_string(),
            ..local.clone()
        };

        config.upsert_workspace(local.clone());
        config.upsert_workspace(hosted.clone());

        assert_eq!(config.workspaces, [local, hosted]);
    }

    #[cfg(unix)]
    #[test]
    fn saved_config_is_owner_only_inside_an_owner_only_home() {
        let parent = tempfile::tempdir().expect("temp dir");
        let home = parent.path().join("mootbox");

        save(&home, &sample_config()).expect("saves");

        assert_eq!(mode_of(&config_path(&home)), OWNER_ONLY_FILE);
        assert_eq!(mode_of(&home), OWNER_ONLY_DIRECTORY);
    }

    #[test]
    fn a_profile_saved_without_a_workspace_name_still_loads_and_shows_its_id() {
        let home = tempfile::tempdir().expect("temp dir");
        let workspace_id = Uuid::new_v4();
        let unnamed = WorkspaceProfile {
            workspace_name: None,
            ..profile(workspace_id, "mootbox_runner_secret")
        };
        let mut config = DaemonConfig::default();
        config.upsert_workspace(unnamed.clone());

        save(home.path(), &config).expect("saves");

        assert!(
            !fs::read_to_string(config_path(home.path()))
                .expect("reads")
                .contains("workspace_name")
        );
        assert_eq!(unnamed.display_name(), workspace_id.to_string());
        assert_eq!(load(home.path()).expect("loads").workspaces, [unnamed]);
    }

    #[test]
    fn a_missing_config_starts_fresh_and_explains_how_to_connect_when_required() {
        let home = tempfile::tempdir().expect("temp dir");

        let error = load(home.path()).expect_err("not paired");

        assert_eq!(
            load_or_start_fresh(home.path()).expect("fresh"),
            DaemonConfig::default()
        );
        assert!(
            error
                .to_string()
                .contains("run mootbox login, then run mootbox inside your repo folder"),
            "{error}"
        );
    }
}
