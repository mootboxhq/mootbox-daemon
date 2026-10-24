use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use mootbox_daemon::commands::service::{self as service_command, ServiceEnvironment};
use mootbox_daemon::config::{self, HOME_OVERRIDE_VARIABLE};
use mootbox_daemon::service::{
    LoadOutcome, SERVICE_DIRECTORY_VARIABLE, SERVICE_MANAGER_VARIABLE, ServiceDefinition, ServiceHost, ServiceKind,
    ServiceState,
};

use crate::common::{FakeServiceManager, paired_home};
use crate::service_files::definition;

const USER_ID: u32 = 501;

struct FakeManager(FakeServiceManager);

impl FakeManager {
    fn new() -> Self {
        Self(FakeServiceManager::printing(""))
    }

    fn failing_on(command: &str) -> Self {
        Self(FakeServiceManager::failing_on(command))
    }

    fn printing(output: &str) -> Self {
        Self(FakeServiceManager::printing(output))
    }

    fn busy_once_on(command: &str) -> Self {
        Self(FakeServiceManager::busy_once_on(command))
    }

    fn calls(&self) -> Vec<String> {
        self.0.calls()
    }

    fn host(&self, kind: ServiceKind, directory: &Path) -> ServiceHost {
        ServiceHost {
            kind,
            directory: directory.to_path_buf(),
            manager: self.0.program.clone(),
            user_id: USER_ID,
        }
    }
}

fn environment() -> ServiceEnvironment {
    ServiceEnvironment {
        executable: PathBuf::from("/opt/mootbox/bin/mootbox"),
        search_path: "/usr/local/bin:/usr/bin:/bin".to_string(),
        home_override: None,
    }
}

#[test]
fn launchd_install_writes_the_plist_and_bootstraps_it_into_the_gui_domain() {
    let (home, _) = paired_home("http://localhost:3000");
    let agents = tempfile::tempdir().expect("temp dir");
    let manager = FakeManager::new();
    let host = manager.host(ServiceKind::Launchd, &agents.path().join("LaunchAgents"));

    service_command::install(home.path(), &host, environment()).expect("installs");

    let plist_path = agents.path().join("LaunchAgents/dev.mootbox.daemon.plist");
    let plist = fs::read_to_string(&plist_path).expect("plist written");
    assert!(
        plist.contains("<string>/usr/local/bin:/usr/bin:/bin</string>"),
        "{plist}"
    );
    assert!(plist.contains("<integer>75</integer>"), "{plist}");
    assert!(home.path().join("logs").is_dir());
    assert_eq!(
        manager.calls(),
        [
            "bootout gui/501/dev.mootbox.daemon".to_string(),
            format!("bootstrap gui/501 {}", plist_path.display()),
        ]
    );
}

#[test]
fn systemd_install_writes_the_unit_then_reloads_enables_and_restarts_it() {
    let (home, mut config) = paired_home("http://localhost:3000");
    config.shutdown_grace_seconds = 10;
    config::save(home.path(), &config).expect("saves");
    let units = tempfile::tempdir().expect("temp dir");
    let manager = FakeManager::new();
    let host = manager.host(ServiceKind::Systemd, units.path());

    service_command::install(home.path(), &host, environment()).expect("installs");

    let unit = fs::read_to_string(units.path().join("mootbox-daemon.service")).expect("unit written");
    assert!(unit.contains("TimeoutStopSec=55\n"), "{unit}");
    assert_eq!(
        manager.calls(),
        [
            "--user daemon-reload",
            "--user enable mootbox-daemon.service",
            "--user restart mootbox-daemon.service",
        ]
    );
}

#[test]
fn reinstalling_overwrites_the_same_file_with_the_current_config() {
    let (home, mut config) = paired_home("http://localhost:3000");
    let agents = tempfile::tempdir().expect("temp dir");
    let manager = FakeManager::new();
    let host = manager.host(ServiceKind::Launchd, agents.path());
    service_command::install(home.path(), &host, environment()).expect("installs");
    let first = fs::read_to_string(host.file_path()).expect("plist");
    service_command::install(home.path(), &host, environment()).expect("reinstalls");
    assert_eq!(fs::read_to_string(host.file_path()).expect("plist"), first);

    config.shutdown_grace_seconds = 5;
    config::save(home.path(), &config).expect("saves");
    service_command::install(home.path(), &host, environment()).expect("reinstalls");

    let entries = fs::read_dir(agents.path()).expect("dir").count();
    let plist = fs::read_to_string(host.file_path()).expect("plist");
    assert_eq!(entries, 1);
    assert!(plist.contains("<integer>50</integer>"), "{plist}");
    assert_eq!(manager.calls().len(), 6);
}

#[test]
fn install_refuses_before_pairing_without_touching_the_service_manager() {
    let home = tempfile::tempdir().expect("temp dir");
    let agents = tempfile::tempdir().expect("temp dir");
    let manager = FakeManager::new();
    let host = manager.host(ServiceKind::Launchd, agents.path());

    let error = service_command::install(home.path(), &host, environment()).expect_err("not paired");

    assert!(
        error
            .to_string()
            .contains("run mootbox login, then run mootbox inside your repo folder"),
        "{error}"
    );
    assert!(!host.file_path().exists());
    assert_eq!(manager.calls(), Vec::<String>::new());
}

#[test]
fn a_refused_bootstrap_keeps_the_written_plist_and_reports_the_reason() {
    let (home, _) = paired_home("http://localhost:3000");
    let agents = tempfile::tempdir().expect("temp dir");
    let manager = FakeManager::failing_on("bootstrap");
    let host = manager.host(ServiceKind::Launchd, agents.path());
    let definition = ServiceDefinition {
        home: home.path().to_path_buf(),
        ..definition(None, 30)
    };

    let outcome = host.install(&definition).expect("file written");

    let LoadOutcome::Failed { reason } = outcome else {
        panic!("expected a failed load, got {outcome:?}");
    };
    assert!(reason.contains("bootstrap refused"), "{reason}");
    assert!(host.file_path().exists());
    assert_eq!(
        host.manual_load_command(),
        format!("launchctl bootstrap gui/501 {}", host.file_path().display())
    );
}

#[test]
fn a_bootstrap_refused_while_launchd_tears_down_the_old_job_is_retried() {
    let (home, _) = paired_home("http://localhost:3000");
    let agents = tempfile::tempdir().expect("temp dir");
    let manager = FakeManager::busy_once_on("bootstrap");
    let host = manager.host(ServiceKind::Launchd, agents.path());

    service_command::reload(home.path(), &host, environment()).expect("restarts after the retry");

    let bootstrap = format!("bootstrap gui/501 {}", host.file_path().display());
    assert_eq!(
        manager.calls(),
        [
            "bootout gui/501/dev.mootbox.daemon".to_string(),
            bootstrap.clone(),
            bootstrap,
        ]
    );
}

#[test]
fn uninstall_unloads_and_removes_the_file() {
    let (home, _) = paired_home("http://localhost:3000");
    let agents = tempfile::tempdir().expect("temp dir");
    let manager = FakeManager::new();
    let host = manager.host(ServiceKind::Launchd, agents.path());
    service_command::install(home.path(), &host, environment()).expect("installs");

    service_command::uninstall(&host).expect("uninstalls");

    assert!(!host.file_path().exists());
    assert_eq!(
        manager.calls().last().map(String::as_str),
        Some("bootout gui/501/dev.mootbox.daemon")
    );
    service_command::uninstall(&host).expect("uninstalling twice is fine");
}

#[test]
fn systemd_uninstall_disables_removes_and_reloads() {
    let units = tempfile::tempdir().expect("temp dir");
    let manager = FakeManager::new();
    let host = manager.host(ServiceKind::Systemd, units.path());
    fs::write(host.file_path(), "[Unit]\n").expect("writes");

    service_command::uninstall(&host).expect("uninstalls");

    assert!(!host.file_path().exists());
    assert_eq!(
        manager.calls(),
        ["--user disable --now mootbox-daemon.service", "--user daemon-reload"]
    );
}

#[test]
fn launchd_state_reads_loaded_and_running_from_launchctl_print() {
    let agents = tempfile::tempdir().expect("temp dir");
    let running = FakeManager::printing("gui/501/dev.mootbox.daemon = {\n\tstate = running\n\tpid = 42\n}");
    let host = running.host(ServiceKind::Launchd, agents.path());
    fs::write(host.file_path(), "plist").expect("writes");

    assert_eq!(
        host.state(),
        ServiceState {
            installed: true,
            loaded: true,
            running: true
        }
    );
    assert_eq!(running.calls(), ["print gui/501/dev.mootbox.daemon"]);

    let unknown = FakeManager::failing_on("print");
    assert_eq!(
        unknown.host(ServiceKind::Launchd, agents.path()).state(),
        ServiceState {
            installed: true,
            loaded: false,
            running: false
        }
    );
}

#[test]
fn systemd_state_reads_enabled_and_active() {
    let units = tempfile::tempdir().expect("temp dir");
    let manager = FakeManager::failing_on("is-active");

    let state = manager.host(ServiceKind::Systemd, units.path()).state();

    assert_eq!(
        state,
        ServiceState {
            installed: false,
            loaded: true,
            running: false
        }
    );
}

#[test]
fn the_service_command_uses_the_injected_directory_and_manager() {
    let (home, _) = paired_home("http://localhost:3000");
    let services = tempfile::tempdir().expect("temp dir");
    let manager = FakeManager::printing("state = running");
    let mootbox = |action: &str| {
        Command::new(env!("CARGO_BIN_EXE_mootbox"))
            .env(HOME_OVERRIDE_VARIABLE, home.path())
            .env(SERVICE_DIRECTORY_VARIABLE, services.path())
            .env(SERVICE_MANAGER_VARIABLE, &manager.0.program)
            .args(["service", action])
            .output()
            .expect("mootbox runs")
    };

    let installed = mootbox("install");
    let status = mootbox("status");

    assert!(installed.status.success(), "{installed:?}");
    let host = manager.host(ServiceKind::of_this_machine().expect("supported"), services.path());
    let written = fs::read_to_string(host.file_path()).expect("service file written");
    assert!(written.contains(env!("CARGO_BIN_EXE_mootbox")), "{written}");
    assert!(written.contains(&home.path().display().to_string()), "{written}");
    let printed = String::from_utf8_lossy(&status.stdout);
    assert!(
        printed.contains("installed: yes\nloaded: yes\nrunning: yes"),
        "{printed}"
    );
    assert!(mootbox("uninstall").status.success());
    assert!(!host.file_path().exists());
}
