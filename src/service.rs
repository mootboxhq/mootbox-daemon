use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::config::HOME_OVERRIDE_VARIABLE;
use crate::log_file;
use crate::private_files;

pub const LAUNCHD_LABEL: &str = "dev.mootbox.daemon";
pub const SYSTEMD_UNIT: &str = "mootbox-daemon.service";
pub const SERVICE_DIRECTORY_VARIABLE: &str = "MOOTBOX_DAEMON_SERVICE_DIR";
pub const SERVICE_MANAGER_VARIABLE: &str = "MOOTBOX_DAEMON_SERVICE_MANAGER";
pub const STOP_MARGIN_SECONDS: u64 = 45;
pub const RESTART_DELAY_SECONDS: u64 = 10;

const START_COMMAND: &str = "start";
const EXECUTABLE_NAME: &str = "mootbox";
const SEARCH_PATH_VARIABLE: &str = "PATH";
const SERVICE_STDOUT_FILE: &str = "service.out.log";
const SERVICE_STDERR_FILE: &str = "service.err.log";
const LAUNCHCTL: &str = "launchctl";
const SYSTEMCTL: &str = "systemctl";
const SYSTEMD_USER_FLAG: &str = "--user";
const LAUNCHD_RUNNING_STATE: &str = "state = running";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServiceKind {
    Launchd,
    Systemd,
}

pub struct ServiceDefinition {
    pub executable: PathBuf,
    pub home: PathBuf,
    pub home_override: Option<PathBuf>,
    pub search_path: String,
    pub shutdown_grace_seconds: u64,
}

pub struct ServiceHost {
    pub kind: ServiceKind,
    pub directory: PathBuf,
    pub manager: PathBuf,
    pub user_id: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ServiceState {
    pub installed: bool,
    pub loaded: bool,
    pub running: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LoadOutcome {
    Loaded,
    Failed { reason: String },
}

impl ServiceDefinition {
    fn stop_timeout_seconds(&self) -> u64 {
        self.shutdown_grace_seconds + STOP_MARGIN_SECONDS
    }

    fn environment(&self) -> Vec<(&'static str, String)> {
        let mut variables = vec![(SEARCH_PATH_VARIABLE, self.search_path.clone())];
        if let Some(home) = &self.home_override {
            variables.push((HOME_OVERRIDE_VARIABLE, home.display().to_string()));
        }
        variables
    }
}

impl ServiceKind {
    pub fn of_this_machine() -> Result<Self> {
        if cfg!(target_os = "macos") {
            return Ok(Self::Launchd);
        }
        if cfg!(target_os = "linux") {
            return Ok(Self::Systemd);
        }
        bail!("mootbox services are supported on macOS (launchd) and Linux (systemd) only")
    }

    fn default_directory(self, base: &directories::BaseDirs) -> PathBuf {
        match self {
            Self::Launchd => base.home_dir().join("Library/LaunchAgents"),
            Self::Systemd => base.config_dir().join("systemd/user"),
        }
    }

    fn default_manager(self) -> &'static str {
        match self {
            Self::Launchd => LAUNCHCTL,
            Self::Systemd => SYSTEMCTL,
        }
    }
}

impl ServiceHost {
    pub fn of_this_machine() -> Result<Self> {
        let kind = ServiceKind::of_this_machine()?;
        let base = directories::BaseDirs::new().context("cannot find the home directory")?;
        let directory =
            std::env::var_os(SERVICE_DIRECTORY_VARIABLE).map_or_else(|| kind.default_directory(&base), PathBuf::from);
        let manager = std::env::var_os(SERVICE_MANAGER_VARIABLE)
            .map_or_else(|| PathBuf::from(kind.default_manager()), PathBuf::from);
        Ok(Self {
            kind,
            directory,
            manager,
            user_id: nix::unistd::getuid().as_raw(),
        })
    }

    pub fn file_path(&self) -> PathBuf {
        match self.kind {
            ServiceKind::Launchd => self.directory.join(format!("{LAUNCHD_LABEL}.plist")),
            ServiceKind::Systemd => self.directory.join(SYSTEMD_UNIT),
        }
    }

    pub fn render(&self, definition: &ServiceDefinition) -> String {
        match self.kind {
            ServiceKind::Launchd => launchd_plist(definition),
            ServiceKind::Systemd => systemd_unit(definition),
        }
    }

    pub fn install(&self, definition: &ServiceDefinition) -> Result<LoadOutcome> {
        private_files::create_private_directory(&log_file::logs_directory(&definition.home))?;
        fs::create_dir_all(&self.directory).with_context(|| format!("cannot create {}", self.directory.display()))?;
        let path = self.file_path();
        fs::write(&path, self.render(definition)).with_context(|| format!("cannot write {}", path.display()))?;
        Ok(match self.load() {
            Ok(()) => LoadOutcome::Loaded,
            Err(error) => LoadOutcome::Failed {
                reason: format!("{error:#}"),
            },
        })
    }

    pub fn uninstall(&self) -> Result<()> {
        let _ = self.unload();
        let path = self.file_path();
        match fs::remove_file(&path) {
            Err(error) if error.kind() != io::ErrorKind::NotFound => {
                return Err(error).with_context(|| format!("cannot remove {}", path.display()));
            }
            _ => {}
        }
        if self.kind == ServiceKind::Systemd {
            let _ = self.manage(&[SYSTEMD_USER_FLAG, "daemon-reload"]);
        }
        Ok(())
    }

    pub fn state(&self) -> ServiceState {
        let installed = self.file_path().exists();
        match self.kind {
            ServiceKind::Launchd => {
                let printed = self.manage(&["print", &self.launchd_target()]);
                ServiceState {
                    installed,
                    loaded: printed.is_ok(),
                    running: printed.is_ok_and(|text| text.contains(LAUNCHD_RUNNING_STATE)),
                }
            }
            ServiceKind::Systemd => ServiceState {
                installed,
                loaded: self.manage(&[SYSTEMD_USER_FLAG, "is-enabled", SYSTEMD_UNIT]).is_ok(),
                running: self.manage(&[SYSTEMD_USER_FLAG, "is-active", SYSTEMD_UNIT]).is_ok(),
            },
        }
    }

    pub fn manual_load_command(&self) -> String {
        match self.kind {
            ServiceKind::Launchd => format!(
                "launchctl bootstrap {} {}",
                self.launchd_domain(),
                self.file_path().display()
            ),
            ServiceKind::Systemd => {
                format!("systemctl --user daemon-reload && systemctl --user enable --now {SYSTEMD_UNIT}")
            }
        }
    }

    fn load(&self) -> Result<()> {
        match self.kind {
            ServiceKind::Launchd => {
                let _ = self.unload();
                let path = self.file_path().display().to_string();
                self.manage(&["bootstrap", &self.launchd_domain(), &path]).map(drop)
            }
            ServiceKind::Systemd => {
                self.manage(&[SYSTEMD_USER_FLAG, "daemon-reload"])?;
                self.manage(&[SYSTEMD_USER_FLAG, "enable", SYSTEMD_UNIT])?;
                self.manage(&[SYSTEMD_USER_FLAG, "restart", SYSTEMD_UNIT]).map(drop)
            }
        }
    }

    fn unload(&self) -> Result<String> {
        match self.kind {
            ServiceKind::Launchd => self.manage(&["bootout", &self.launchd_target()]),
            ServiceKind::Systemd => self.manage(&[SYSTEMD_USER_FLAG, "disable", "--now", SYSTEMD_UNIT]),
        }
    }

    fn launchd_domain(&self) -> String {
        format!("gui/{}", self.user_id)
    }

    fn launchd_target(&self) -> String {
        format!("{}/{LAUNCHD_LABEL}", self.launchd_domain())
    }

    fn manage(&self, arguments: &[&str]) -> Result<String> {
        let output = Command::new(&self.manager)
            .args(arguments)
            .output()
            .with_context(|| format!("cannot run {}", self.manager.display()))?;
        if !output.status.success() {
            bail!(
                "`{} {}` failed: {}",
                self.manager.display(),
                arguments.join(" "),
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }
}

pub fn stable_executable(current: &Path, search_path: &str) -> PathBuf {
    let Ok(resolved_current) = current.canonicalize() else {
        return current.to_path_buf();
    };
    std::env::split_paths(search_path)
        .map(|directory| directory.join(EXECUTABLE_NAME))
        .find(|candidate| candidate.is_file())
        .filter(|candidate| {
            candidate
                .canonicalize()
                .is_ok_and(|resolved| resolved == resolved_current)
        })
        .unwrap_or_else(|| current.to_path_buf())
}

pub fn launchd_plist(definition: &ServiceDefinition) -> String {
    let logs = log_file::logs_directory(&definition.home);
    let environment = definition
        .environment()
        .iter()
        .map(|(name, value)| {
            format!(
                "      <key>{name}</key>\n      <string>{}</string>\n",
                xml_escaped(value)
            )
        })
        .collect::<Vec<_>>()
        .concat();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
  <dict>
    <key>Label</key>
    <string>{LAUNCHD_LABEL}</string>
    <key>ProgramArguments</key>
    <array>
      <string>{executable}</string>
      <string>{START_COMMAND}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <dict>
      <key>SuccessfulExit</key>
      <false/>
    </dict>
    <key>ThrottleInterval</key>
    <integer>{RESTART_DELAY_SECONDS}</integer>
    <key>ExitTimeOut</key>
    <integer>{stop_timeout}</integer>
    <key>StandardOutPath</key>
    <string>{stdout}</string>
    <key>StandardErrorPath</key>
    <string>{stderr}</string>
    <key>EnvironmentVariables</key>
    <dict>
{environment}    </dict>
  </dict>
</plist>
"#,
        executable = xml_escaped(&definition.executable.display().to_string()),
        stop_timeout = definition.stop_timeout_seconds(),
        stdout = xml_escaped(&logs.join(SERVICE_STDOUT_FILE).display().to_string()),
        stderr = xml_escaped(&logs.join(SERVICE_STDERR_FILE).display().to_string()),
    )
}

pub fn systemd_unit(definition: &ServiceDefinition) -> String {
    let environment = definition
        .environment()
        .iter()
        .map(|(name, value)| format!("Environment={}\n", systemd_quoted(&format!("{name}={value}"))))
        .collect::<Vec<_>>()
        .concat();
    format!(
        "[Unit]
Description=Mootbox daemon

[Service]
ExecStart={executable} {START_COMMAND}
Restart=on-failure
RestartSec={RESTART_DELAY_SECONDS}
KillMode=mixed
TimeoutStopSec={stop_timeout}
{environment}
[Install]
WantedBy=default.target
",
        executable = systemd_quoted(&definition.executable.display().to_string().replace('$', "$$")),
        stop_timeout = definition.stop_timeout_seconds(),
    )
}

fn xml_escaped(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn systemd_quoted(text: &str) -> String {
    let escaped = text.replace('\\', "\\\\").replace('"', "\\\"").replace('%', "%%");
    format!("\"{escaped}\"")
}
