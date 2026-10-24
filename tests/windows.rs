#![cfg(windows)]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use mootbox_daemon::providers::{ProviderKind, probe};
use mootbox_daemon::run::process::{ProviderLaunch, RunningProvider};
use mootbox_daemon::service::{ServiceDefinition, ServiceHost, ServiceKind};
use mootbox_daemon::{config, lock, platform, private_files, release};
use sha2::{Digest, Sha256};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn command_shim(directory: &Path) -> PathBuf {
    let program = directory.join("fake provider.cmd");
    fs::write(&program, "@echo off\r\nif \"%1\"==\"--version\" (echo codex-cli 0.120.0 & exit /b 0)\r\necho {\"type\":\"thread.started\",\"thread_id\":\"windows-session\"}\r\n").expect("writes shim");
    program
}

#[test]
fn windows_release_uses_the_native_executable_and_zip() {
    assert_eq!(platform::EXECUTABLE_NAME, "mootbox.exe");
    assert_eq!(release::host_asset().expect("asset"), "mootbox-windows-x64.zip");
    let output = Command::new(env!("CARGO_BIN_EXE_mootbox"))
        .arg("--version")
        .output()
        .expect("runs native binary");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        concat!("mootbox ", env!("CARGO_PKG_VERSION"))
    );
}

#[tokio::test]
async fn npm_style_cmd_shims_are_discovered_and_can_run_provider_sessions() {
    let directory = tempfile::Builder::new()
        .prefix("mootbox with spaces ")
        .tempdir()
        .expect("temp dir");
    let program = command_shim(directory.path());
    let extensionless = program.with_extension("");
    assert_eq!(platform::provider_program(&extensionless), program);
    assert_eq!(
        probe::installation(&extensionless).await,
        probe::Installation::Installed(Some(probe::Version(0, 120, 0)))
    );
    let launch = ProviderLaunch {
        provider: "codex".parse::<ProviderKind>().expect("provider"),
        program: &extensionless,
        working_directory: directory.path(),
        arguments: vec!["--model".to_string(), "a model with spaces".to_string()],
        prompt: "hello from Windows".to_string(),
        environment: Vec::new(),
    };
    let report = RunningProvider::spawn(launch)
        .expect("spawns shim in a job object")
        .finish()
        .await
        .expect("finishes");
    assert!(report.status.success(), "{:?}", report.stderr_tail);
    assert_eq!(
        report.result.and_then(|result| result.session_id).as_deref(),
        Some("windows-session")
    );
}

#[tokio::test]
async fn cancellation_terminates_the_provider_and_its_descendants() {
    let directory = tempfile::tempdir().expect("temp dir");
    let ready = directory.path().join("child.txt");
    let launch = ProviderLaunch {
        provider: "codex".parse().expect("provider"),
        program: Path::new("powershell.exe"),
        working_directory: directory.path(),
        arguments: vec!["-NoProfile".to_string(), "-NonInteractive".to_string(), "-Command".to_string(), "$child = Start-Process powershell.exe -ArgumentList '-NoProfile -Command Start-Sleep -Seconds 120' -PassThru; $child.Id | Set-Content -LiteralPath $env:MOOTBOX_TEST_READY; Start-Sleep -Seconds 120".to_string()],
        prompt: String::new(),
        environment: vec![("MOOTBOX_TEST_READY".to_string(), ready.display().to_string())],
    };
    let mut running = RunningProvider::spawn(launch).expect("spawns");
    tokio::time::timeout(Duration::from_secs(30), async {
        while !ready.exists() {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .expect("child starts");
    let child = fs::read_to_string(&ready).expect("child pid").trim().to_string();
    running.terminate(Duration::from_millis(50)).await;
    let report = tokio::time::timeout(Duration::from_secs(10), running.finish())
        .await
        .expect("bounded shutdown")
        .expect("finishes");
    assert!(!report.status.success());
    let status =
        platform::powershell("if (Get-Process -Id $env:MOOTBOX_TEST_PID -ErrorAction SilentlyContinue) { exit 1 }")
            .env("MOOTBOX_TEST_PID", child)
            .status()
            .expect("checks process");
    assert!(status.success(), "a provider descendant survived cancellation");
}

#[test]
fn private_state_is_replaced_and_its_acl_only_grants_the_current_user() {
    let directory = tempfile::tempdir().expect("temp dir");
    let home = directory.path().join("private state");
    private_files::create_private_directory(&home).expect("private home");
    let file = home.join("secret.json");
    private_files::replace_owner_only(&file, b"old").expect("initial state");
    private_files::replace_owner_only(&file, b"new").expect("replaces state");
    assert_eq!(fs::read(&file).expect("reads"), b"new");
    let output = platform::powershell("$sid = [System.Security.Principal.WindowsIdentity]::GetCurrent().User; $acl = Get-Acl -LiteralPath $env:MOOTBOX_TEST_FILE; if (-not $acl.AreAccessRulesProtected) { exit 1 }; foreach ($rule in $acl.Access) { if ($rule.IdentityReference.Translate([System.Security.Principal.SecurityIdentifier]) -ne $sid) { exit 2 } }")
        .env("MOOTBOX_TEST_FILE", file).output().expect("inspects ACL");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let first = lock::acquire(&home).expect("first lock");
    assert!(lock::acquire(&home).is_err());
    drop(first);
    lock::acquire(&home).expect("released lock");
}

#[test]
fn scheduled_task_runs_as_the_interactive_user_and_preserves_paths_with_spaces() {
    let host = ServiceHost {
        kind: ServiceKind::TaskScheduler,
        directory: PathBuf::from(r"C:\Users\Some Person\Mootbox"),
        manager: PathBuf::from("schtasks.exe"),
        user_sid: "S-1-5-21-123".to_string(),
    };
    let xml = host.render(&ServiceDefinition {
        executable: PathBuf::from(r"C:\Users\Some Person\mootbox.exe"),
        home: PathBuf::from(r"C:\Users\Some Person\.mootbox"),
        home_override: None,
        search_path: r"C:\Program Files\Git\cmd;C:\Users\Some Person\AppData\Roaming\npm".to_string(),
        shutdown_grace_seconds: 30,
    });
    assert!(xml.contains("<LogonType>InteractiveToken</LogonType>"));
    assert!(xml.contains("<RunLevel>LeastPrivilege</RunLevel>"));
    assert!(xml.contains("-EncodedCommand "));
    assert!(!xml.contains("ExecutionPolicy"));
}

#[tokio::test]
async fn update_verifies_a_zip_and_replaces_a_running_native_executable() {
    let directory = tempfile::tempdir().expect("temp dir");
    let staging = directory.path().join("staging");
    fs::create_dir(&staging).expect("staging");
    let source = directory.path().join("replacement.rs");
    fs::write(&source, "fn main() { println!(\"mootbox 999.0.0\"); }").expect("source");
    let replacement = staging.join("mootbox.exe");
    assert!(
        Command::new("rustc")
            .arg(&source)
            .arg("-o")
            .arg(&replacement)
            .status()
            .expect("compiles replacement")
            .success()
    );
    let archive = directory.path().join("release.zip");
    assert!(platform::powershell("$ErrorActionPreference = 'Stop'; Compress-Archive -LiteralPath $env:MOOTBOX_TEST_EXE -DestinationPath $env:MOOTBOX_TEST_ARCHIVE")
        .env("MOOTBOX_TEST_EXE", &replacement).env("MOOTBOX_TEST_ARCHIVE", &archive).status().expect("creates zip").success());
    let bytes = fs::read(&archive).expect("zip");
    let digest: String = Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    let sums = format!("{digest}  mootbox-windows-x64.zip\n");
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/releases/latest"))
        .respond_with(
            ResponseTemplate::new(302).insert_header("location", format!("{}/releases/tag/v999.0.0", server.uri())),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/releases/download/v999.0.0/SHA256SUMS"))
        .respond_with(ResponseTemplate::new(200).set_body_string(sums))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/releases/download/v999.0.0/mootbox-windows-x64.zip"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(bytes))
        .mount(&server)
        .await;
    let installed = directory.path().join("mootbox.exe");
    fs::copy(env!("CARGO_BIN_EXE_mootbox"), &installed).expect("installs original");
    let binary = installed.clone();
    let home = directory.path().join("home");
    let uri = server.uri();
    let outcome = tokio::task::spawn_blocking(move || {
        Command::new(binary)
            .arg("update")
            .env(config::HOME_OVERRIDE_VARIABLE, &home)
            .env(config::RELEASES_URL_VARIABLE, uri)
            .env("MOOTBOX_DAEMON_SERVICE_DIR", home.join("services"))
            .output()
            .expect("updates")
    })
    .await
    .expect("joins");
    assert!(outcome.status.success(), "{}", String::from_utf8_lossy(&outcome.stderr));
    let version = Command::new(&installed)
        .arg("--version")
        .output()
        .expect("runs updated executable");
    assert_eq!(String::from_utf8_lossy(&version.stdout).trim(), "mootbox 999.0.0");
}

#[test]
fn the_powershell_installer_parses_in_windows_powershell() {
    let installer = Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/install.ps1");
    let output = platform::powershell("$errors = $null; $tokens = $null; [System.Management.Automation.Language.Parser]::ParseFile($env:MOOTBOX_TEST_INSTALLER, [ref]$tokens, [ref]$errors) | Out-Null; if ($errors.Count -gt 0) { $errors | Out-String | Write-Error; exit 1 }")
        .env("MOOTBOX_TEST_INSTALLER", installer).output().expect("parses installer");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
}

#[tokio::test]
async fn git_for_windows_reports_and_connects_a_local_checkout() {
    let directory = tempfile::tempdir().expect("temp dir");
    let repository = directory.path().join("repo with spaces");
    fs::create_dir(&repository).expect("repository folder");
    for arguments in [
        vec!["init", "-b", "main"],
        vec!["config", "user.name", "Mootbox test"],
        vec!["config", "user.email", "test@example.com"],
        vec!["config", "core.autocrlf", "false"],
        vec!["remote", "add", "origin", "https://github.com/example/windows.git"],
    ] {
        assert!(
            Command::new("git")
                .args(arguments)
                .current_dir(&repository)
                .status()
                .expect("git")
                .success()
        );
    }
    fs::write(repository.join("README.md"), "windows\n").expect("readme");
    for arguments in [vec!["add", "."], vec!["commit", "-m", "initial"]] {
        assert!(
            Command::new("git")
                .args(arguments)
                .current_dir(&repository)
                .status()
                .expect("git")
                .success()
        );
    }
    let state = mootbox_daemon::git::checkout_state(&repository)
        .await
        .expect("checkout state");
    assert_eq!(state.branch.as_deref(), Some("main"));
    assert!(!state.is_dirty);
    let remotes = mootbox_daemon::git::remotes(&repository).await.expect("remote");
    assert_eq!(remotes[0].url, "https://github.com/example/windows.git");
    let target = directory.path().join("worktrees");
    let worktree = mootbox_daemon::git::open_branch_worktree(&mootbox_daemon::git::WorktreeRequest {
        repository: &repository,
        worktrees: &target,
        branch: "mootbox/windows",
        base_ref: "HEAD",
    })
    .await
    .expect("creates a Windows worktree");
    assert!(worktree.join("README.md").is_file());
}
