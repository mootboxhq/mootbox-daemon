use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use mootbox_daemon::config::{HOME_OVERRIDE_VARIABLE, RELEASES_URL_VARIABLE};
use mootbox_daemon::release;
use mootbox_daemon::service::{SERVICE_DIRECTORY_VARIABLE, SERVICE_MANAGER_VARIABLE};
use sha2::{Digest, Sha256};
use tempfile::TempDir;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::common::{FakeServiceManager, paired_home};

const NEWER_VERSION: &str = "999.0.0";
const HTTP_FOUND: u16 = 302;
const REFUSING_MANAGER: &str = "false";
const SERVICE_FOLDER: &str = "services";
const EXECUTABLE_NAME: &str = "mootbox";

struct Installed {
    folder: TempDir,
    original: Vec<u8>,
}

impl Installed {
    fn copy_of_this_build() -> Self {
        let folder = tempfile::tempdir().expect("temp dir");
        let executable = folder.path().join(EXECUTABLE_NAME);
        fs::copy(env!("CARGO_BIN_EXE_mootbox"), &executable).expect("copies mootbox");
        let original = fs::read(&executable).expect("reads mootbox");
        Self { folder, original }
    }

    fn executable(&self) -> PathBuf {
        self.folder.path().join(EXECUTABLE_NAME)
    }

    fn contents(&self) -> Vec<u8> {
        fs::read(self.executable()).expect("reads mootbox")
    }

    fn entries(&self) -> Vec<String> {
        fs::read_dir(self.folder.path())
            .expect("lists the folder")
            .map(|entry| entry.expect("entry").file_name().to_string_lossy().into_owned())
            .collect()
    }

    fn run(&self, home: &Path, server: &MockServer, args: &[&str]) -> Output {
        self.run_with_manager(home, server, Path::new(REFUSING_MANAGER), args)
    }

    fn run_with_manager(&self, home: &Path, server: &MockServer, manager: &Path, args: &[&str]) -> Output {
        Command::new(self.executable())
            .env(HOME_OVERRIDE_VARIABLE, home)
            .env(RELEASES_URL_VARIABLE, server.uri())
            .env(SERVICE_DIRECTORY_VARIABLE, home.join(SERVICE_FOLDER))
            .env(SERVICE_MANAGER_VARIABLE, manager)
            .args(args)
            .output()
            .expect("mootbox runs")
    }
}

fn fake_binary(prints: &str) -> Vec<u8> {
    format!("#!/bin/sh\necho 'mootbox {prints}'\n").into_bytes()
}

fn sha256_hex(contents: &[u8]) -> String {
    Sha256::digest(contents)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .concat()
}

async fn serve_latest(server: &MockServer, version: &str) {
    Mock::given(method("GET"))
        .and(path("/releases/latest"))
        .respond_with(
            ResponseTemplate::new(HTTP_FOUND)
                .insert_header("location", format!("{}/releases/tag/v{version}", server.uri()).as_str()),
        )
        .mount(server)
        .await;
}

async fn serve_asset(server: &MockServer, version: &str, asset: &str, contents: Vec<u8>) {
    Mock::given(method("GET"))
        .and(path(format!("/releases/download/v{version}/{asset}")))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(contents))
        .mount(server)
        .await;
}

async fn serve_release(server: &MockServer, binary: &[u8], listed_digest: &str) {
    let asset = release::host_asset().expect("supported host");
    serve_latest(server, NEWER_VERSION).await;
    let checksums = format!("{}  mootbox-other\n{listed_digest}  {asset}\n", "0".repeat(64));
    serve_asset(server, NEWER_VERSION, "SHA256SUMS", checksums.into_bytes()).await;
    serve_asset(server, NEWER_VERSION, &asset, binary.to_vec()).await;
}

fn stdout_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[tokio::test]
async fn update_reports_up_to_date_when_the_latest_release_is_this_version() {
    let server = MockServer::start().await;
    serve_latest(&server, env!("CARGO_PKG_VERSION")).await;
    let home = tempfile::tempdir().expect("temp dir");
    let installed = Installed::copy_of_this_build();

    let output = installed.run(home.path(), &server, &["update"]);

    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        stdout_of(&output),
        format!("mootbox {} is up to date\n", env!("CARGO_PKG_VERSION"))
    );
    assert_eq!(installed.contents(), installed.original);
}

#[tokio::test]
async fn update_replaces_the_running_executable_with_the_verified_release() {
    let server = MockServer::start().await;
    let binary = fake_binary(NEWER_VERSION);
    serve_release(&server, &binary, &sha256_hex(&binary)).await;
    let home = tempfile::tempdir().expect("temp dir");
    let installed = Installed::copy_of_this_build();

    let output = installed.run(home.path(), &server, &["update"]);

    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        stdout_of(&output),
        format!("updated mootbox {} → {NEWER_VERSION}\n", env!("CARGO_PKG_VERSION"))
    );
    assert_eq!(installed.contents(), binary);
    assert_eq!(installed.entries(), [EXECUTABLE_NAME]);
    let mode = fs::metadata(installed.executable())
        .expect("metadata")
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o755);
}

#[tokio::test]
async fn update_refuses_a_release_whose_checksum_does_not_match() {
    let server = MockServer::start().await;
    let binary = fake_binary(NEWER_VERSION);
    serve_release(&server, &binary, &"f".repeat(64)).await;
    let home = tempfile::tempdir().expect("temp dir");
    let installed = Installed::copy_of_this_build();

    let output = installed.run(home.path(), &server, &["update"]);

    assert!(!output.status.success(), "{output:?}");
    assert!(stderr_of(&output).contains("does not match SHA256SUMS"), "{output:?}");
    assert_eq!(installed.contents(), installed.original);
    assert_eq!(installed.entries(), [EXECUTABLE_NAME]);
}

#[tokio::test]
async fn update_refuses_a_release_that_reports_another_version() {
    let server = MockServer::start().await;
    let binary = fake_binary("0.0.1");
    serve_release(&server, &binary, &sha256_hex(&binary)).await;
    let home = tempfile::tempdir().expect("temp dir");
    let installed = Installed::copy_of_this_build();

    let output = installed.run(home.path(), &server, &["update"]);

    assert!(!output.status.success(), "{output:?}");
    assert!(stderr_of(&output).contains("kept the current one"), "{output:?}");
    assert_eq!(installed.contents(), installed.original);
    assert_eq!(installed.entries(), [EXECUTABLE_NAME]);
}

#[tokio::test]
async fn update_check_reports_a_newer_release_without_downloading_it() {
    let server = MockServer::start().await;
    let binary = fake_binary(NEWER_VERSION);
    serve_release(&server, &binary, &sha256_hex(&binary)).await;
    let home = tempfile::tempdir().expect("temp dir");
    let installed = Installed::copy_of_this_build();

    let output = installed.run(home.path(), &server, &["update", "--check"]);

    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        stdout_of(&output),
        format!(
            "mootbox {NEWER_VERSION} is available (this is {}); run mootbox update\n",
            env!("CARGO_PKG_VERSION")
        )
    );
    let requested: Vec<String> = server
        .received_requests()
        .await
        .expect("recording")
        .iter()
        .map(|request| request.url.path().to_string())
        .collect();
    assert_eq!(requested, ["/releases/latest"]);
    assert_eq!(installed.contents(), installed.original);
}

#[tokio::test]
async fn update_restarts_an_installed_background_service_on_the_new_binary() {
    let server = MockServer::start().await;
    let binary = fake_binary(NEWER_VERSION);
    serve_release(&server, &binary, &sha256_hex(&binary)).await;
    let (home, _) = paired_home("http://localhost:3000");
    let services = home.path().join(SERVICE_FOLDER);
    fs::create_dir_all(&services).expect("service folder");
    fs::write(services.join("dev.mootbox.daemon.plist"), "").expect("installed launchd service");
    fs::write(services.join("mootbox-daemon.service"), "").expect("installed systemd service");
    let manager = FakeServiceManager::printing("");
    let installed = Installed::copy_of_this_build();

    let output = installed.run_with_manager(home.path(), &server, &manager.program, &["update"]);

    assert!(output.status.success(), "{output:?}");
    assert!(
        stdout_of(&output).ends_with("restarted the background service\n"),
        "{output:?}"
    );
    let calls = manager.calls();
    assert!(
        calls
            .iter()
            .any(|call| call.starts_with("bootstrap ") || call.ends_with("restart mootbox-daemon.service")),
        "{calls:?}"
    );
    let executable = installed.executable().display().to_string();
    let rewritten = fs::read_dir(&services)
        .expect("lists services")
        .map(|entry| fs::read_to_string(entry.expect("entry").path()).expect("reads service"))
        .collect::<String>();
    assert!(rewritten.contains(&executable), "{rewritten}");
}
