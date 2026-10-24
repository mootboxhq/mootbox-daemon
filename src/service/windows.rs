use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use base64::Engine as _;

use crate::config::HOME_OVERRIDE_VARIABLE;
use crate::{log_file, platform, private_files};

pub const SERVICE_DIRECTORY_VARIABLE: &str = "MOOTBOX_DAEMON_SERVICE_DIR";
pub const SERVICE_MANAGER_VARIABLE: &str = "MOOTBOX_DAEMON_SERVICE_MANAGER";
const TASK_PREFIX: &str = "Mootbox Daemon";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServiceKind {
    TaskScheduler,
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
    pub user_sid: String,
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

impl ServiceHost {
    pub fn of_this_machine() -> Result<Self> {
        let base = directories::BaseDirs::new().context("cannot find the home directory")?;
        let identity = platform::powershell("[System.Security.Principal.WindowsIdentity]::GetCurrent().User.Value")
            .output()
            .context("cannot identify the current Windows account")?;
        let user_sid = String::from_utf8_lossy(&identity.stdout).trim().to_string();
        if !identity.status.success() || !user_sid.starts_with("S-1-") {
            bail!("cannot identify the current Windows account");
        }
        Ok(Self {
            kind: ServiceKind::TaskScheduler,
            directory: std::env::var_os(SERVICE_DIRECTORY_VARIABLE)
                .map_or_else(|| base.data_local_dir().join("Mootbox/service"), PathBuf::from),
            manager: std::env::var_os(SERVICE_MANAGER_VARIABLE)
                .map_or_else(|| PathBuf::from("schtasks.exe"), PathBuf::from),
            user_sid,
        })
    }

    fn task_name(&self) -> String {
        format!("{TASK_PREFIX}-{}", self.user_sid)
    }

    pub fn file_path(&self) -> PathBuf {
        self.directory.join("mootbox-task.xml")
    }

    pub fn render(&self, definition: &ServiceDefinition) -> String {
        let home = definition.home_override.as_deref().unwrap_or(&definition.home);
        let logs = log_file::logs_directory(&definition.home);
        let script = format!(
            "$env:PATH = {}; $env:{HOME_OVERRIDE_VARIABLE} = {}; & {} start 1>> {} 2>> {}; exit $LASTEXITCODE",
            ps_quote(&definition.search_path),
            ps_quote(&home.display().to_string()),
            ps_quote(&definition.executable.display().to_string()),
            ps_quote(&logs.join("service.out.log").display().to_string()),
            ps_quote(&logs.join("service.err.log").display().to_string()),
        );
        let bytes: Vec<u8> = script.encode_utf16().flat_map(u16::to_le_bytes).collect();
        let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<Task version="1.4" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo><Description>Mootbox per-user background daemon</Description></RegistrationInfo>
  <Triggers><LogonTrigger><Enabled>true</Enabled><UserId>{sid}</UserId></LogonTrigger></Triggers>
  <Principals><Principal id="User"><UserId>{sid}</UserId><LogonType>InteractiveToken</LogonType><RunLevel>LeastPrivilege</RunLevel></Principal></Principals>
  <Settings><MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy><DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries><StopIfGoingOnBatteries>false</StopIfGoingOnBatteries><ExecutionTimeLimit>PT0S</ExecutionTimeLimit><RestartOnFailure><Interval>PT1M</Interval><Count>999</Count></RestartOnFailure></Settings>
  <Actions Context="User"><Exec><Command>powershell.exe</Command><Arguments>-NoLogo -NoProfile -NonInteractive -WindowStyle Hidden -EncodedCommand {encoded}</Arguments></Exec></Actions>
</Task>
"#,
            sid = self.user_sid
        )
    }

    pub fn install(&self, definition: &ServiceDefinition) -> Result<LoadOutcome> {
        private_files::create_private_directory(&log_file::logs_directory(&definition.home))?;
        private_files::create_private_directory(&self.directory)?;
        private_files::write_owner_only(&self.file_path(), self.render(definition).as_bytes())?;
        let result = self
            .manage(&[
                "/Create",
                "/TN",
                &self.task_name(),
                "/XML",
                &self.file_path().display().to_string(),
                "/F",
            ])
            .and_then(|()| {
                let _ = self.manage(&["/End", "/TN", &self.task_name()]);
                self.manage(&["/Run", "/TN", &self.task_name()])
            });
        Ok(match result {
            Ok(()) => LoadOutcome::Loaded,
            Err(error) => LoadOutcome::Failed {
                reason: format!("{error:#}"),
            },
        })
    }

    pub fn uninstall(&self) -> Result<()> {
        let _ = self.manage(&["/End", "/TN", &self.task_name()]);
        let exists = self.manage(&["/Query", "/TN", &self.task_name()]).is_ok();
        if exists {
            self.manage(&["/Delete", "/TN", &self.task_name(), "/F"])?;
        }
        match fs::remove_file(self.file_path()) {
            Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error.into()),
            _ => Ok(()),
        }
    }

    pub fn state(&self) -> ServiceState {
        let installed = self.file_path().exists();
        let loaded = self.manage(&["/Query", "/TN", &self.task_name()]).is_ok();
        let running = loaded
            && platform::powershell(
                "$ErrorActionPreference = 'Stop'; [int](Get-ScheduledTask -TaskName $env:MOOTBOX_TASK_NAME).State",
            )
            .env("MOOTBOX_TASK_NAME", self.task_name())
            .output()
            .is_ok_and(|output| output.status.success() && String::from_utf8_lossy(&output.stdout).trim() == "4");
        ServiceState {
            installed,
            loaded,
            running,
        }
    }

    pub fn manual_load_command(&self) -> String {
        format!("schtasks.exe /Run /TN \"{}\"", self.task_name())
    }

    fn manage(&self, arguments: &[&str]) -> Result<()> {
        let output = Command::new(&self.manager)
            .args(arguments)
            .output()
            .with_context(|| format!("cannot run {}", self.manager.display()))?;
        if !output.status.success() {
            bail!(
                "Task Scheduler failed: {} {}",
                String::from_utf8_lossy(&output.stdout).trim(),
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
        Ok(())
    }
}

pub fn stable_executable(current: &Path, _search_path: &str) -> PathBuf {
    current.to_path_buf()
}

fn ps_quote(text: &str) -> String {
    format!("'{}'", text.replace('\'', "''"))
}
