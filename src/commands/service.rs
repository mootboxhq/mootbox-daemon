use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use super::Output;
use crate::config::{self, HOME_OVERRIDE_VARIABLE};
use crate::log_file;
use crate::service::{self, LoadOutcome, ServiceDefinition, ServiceHost, ServiceState};

const SEARCH_PATH_VARIABLE: &str = "PATH";

#[derive(Clone, Copy, Debug)]
pub enum ServiceAction {
    Install,
    Uninstall,
    Status,
}

pub struct ServiceEnvironment {
    pub executable: PathBuf,
    pub search_path: String,
    pub home_override: Option<PathBuf>,
}

impl ServiceEnvironment {
    pub fn of_this_process() -> Result<Self> {
        let current = std::env::current_exe().context("cannot find the mootbox executable")?;
        let search_path = std::env::var(SEARCH_PATH_VARIABLE).unwrap_or_default();
        Ok(Self {
            executable: service::stable_executable(&current, &search_path),
            search_path,
            home_override: std::env::var_os(HOME_OVERRIDE_VARIABLE).map(PathBuf::from),
        })
    }
}

pub fn run(home: &Path, action: ServiceAction) -> Result<Output> {
    let host = ServiceHost::of_this_machine()?;
    match action {
        ServiceAction::Install => install(home, &host, ServiceEnvironment::of_this_process()?),
        ServiceAction::Uninstall => uninstall(&host),
        ServiceAction::Status => Ok(Output::lines(summary(&host, host.state()))),
    }
}

pub fn install(home: &Path, host: &ServiceHost, environment: ServiceEnvironment) -> Result<Output> {
    let outcome = host.install(&definition(home, environment)?)?;
    let mut output = Output::lines(vec![format!("wrote {}", host.file_path().display())]);
    if let LoadOutcome::Failed { reason } = outcome {
        output.warnings = vec![
            format!("warning: the service is written but not running yet: {reason}"),
            format!("start it with: {}", host.manual_load_command()),
        ];
        return Ok(output);
    }
    output.lines.push(format!(
        "mootbox now starts at login and restarts after a crash; logs in {}",
        log_file::logs_directory(home).display()
    ));
    Ok(output)
}

pub fn run_in_background(home: &Path) -> Result<()> {
    reload(
        home,
        &ServiceHost::of_this_machine()?,
        ServiceEnvironment::of_this_process()?,
    )
}

pub fn reload(home: &Path, host: &ServiceHost, environment: ServiceEnvironment) -> Result<()> {
    match host.install(&definition(home, environment)?)? {
        LoadOutcome::Loaded => Ok(()),
        LoadOutcome::Failed { reason } => bail!(reason),
    }
}

pub fn uninstall(host: &ServiceHost) -> Result<Output> {
    let path = host.file_path();
    let was_installed = path.exists();
    host.uninstall()?;
    if !was_installed {
        return Ok(Output::lines(vec![format!(
            "no mootbox service is installed at {}",
            path.display()
        )]));
    }
    Ok(Output::lines(vec![format!("removed {}", path.display())]))
}

pub fn status_line(state: ServiceState) -> String {
    let shown = match (state.installed, state.running) {
        (false, _) => "not installed",
        (true, true) => "installed, running",
        (true, false) => "installed, not running",
    };
    format!("service: {shown}")
}

fn summary(host: &ServiceHost, state: ServiceState) -> Vec<String> {
    vec![
        format!("service: {}", host.file_path().display()),
        format!("installed: {}", yes_or_no(state.installed)),
        format!("loaded: {}", yes_or_no(state.loaded)),
        format!("running: {}", yes_or_no(state.running)),
    ]
}

fn definition(home: &Path, environment: ServiceEnvironment) -> Result<ServiceDefinition> {
    let config = config::load(home)?;
    Ok(ServiceDefinition {
        executable: environment.executable,
        home: home.to_path_buf(),
        home_override: environment.home_override,
        search_path: environment.search_path,
        shutdown_grace_seconds: config.shutdown_grace_seconds,
    })
}

fn yes_or_no(answer: bool) -> &'static str {
    if answer { "yes" } else { "no" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_service_status_line_says_whether_it_is_installed_and_running() {
        let state = |installed, running| ServiceState {
            installed,
            loaded: running,
            running,
        };

        assert_eq!(status_line(state(false, false)), "service: not installed");
        assert_eq!(status_line(state(true, false)), "service: installed, not running");
        assert_eq!(status_line(state(true, true)), "service: installed, running");
    }
}
