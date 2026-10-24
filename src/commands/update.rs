use std::fs::{self, Permissions};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use tempfile::TempPath;

use super::Output;
use super::service::{self, ServiceEnvironment};
use crate::release::{self, ARCHIVED_EXECUTABLE, CHECKSUMS_ASSET, ReleaseSource, Version};
use crate::service::ServiceHost;

const STAGED_PREFIX: &str = ".mootbox-update-";
const EXECUTABLE_MODE: u32 = 0o755;
const VERSION_FLAG: &str = "--version";
const TAR_PROGRAM: &str = "tar";

struct InstalledService {
    host: ServiceHost,
    environment: ServiceEnvironment,
}

pub async fn check() -> Result<Output> {
    let current = release::current_version()?;
    let latest = ReleaseSource::of_this_process()?.latest_version().await?;
    if latest <= current {
        return Ok(up_to_date(current));
    }
    Ok(Output::lines(vec![format!(
        "mootbox {latest} is available (this is {current}); run mootbox update"
    )]))
}

pub async fn apply(home: &Path) -> Result<Output> {
    let asset = release::host_asset()?;
    let current = release::current_version()?;
    let source = ReleaseSource::of_this_process()?;
    let latest = source.latest_version().await?;
    if latest <= current {
        return Ok(up_to_date(current));
    }

    let checksums = String::from_utf8(source.download(latest, CHECKSUMS_ASSET).await?)
        .with_context(|| format!("{CHECKSUMS_ASSET} of mootbox {latest} is not text"))?;
    let archive = source.download(latest, &asset).await?;
    release::verify_checksum(&checksums, &asset, &archive)?;

    let service = installed_service()?;
    let executable = current_executable()?;
    let binary = unpack_executable(folder_of(&executable)?, &asset, &archive)?;
    replace_executable(&executable, &binary, latest)?;

    let mut output = Output::lines(vec![format!("updated mootbox {current} → {latest}")]);
    if let Some(service) = service {
        restart(home, service, &mut output);
    }
    Ok(output)
}

fn up_to_date(current: Version) -> Output {
    Output::lines(vec![format!("mootbox {current} is up to date")])
}

fn current_executable() -> Result<PathBuf> {
    std::env::current_exe()
        .and_then(|path| path.canonicalize())
        .context("cannot find the mootbox executable")
}

fn installed_service() -> Result<Option<InstalledService>> {
    let Ok(host) = ServiceHost::of_this_machine() else {
        return Ok(None);
    };
    if !host.file_path().exists() {
        return Ok(None);
    }
    Ok(Some(InstalledService {
        host,
        environment: ServiceEnvironment::of_this_process()?,
    }))
}

fn restart(home: &Path, service: InstalledService, output: &mut Output) {
    match service::reload(home, &service.host, service.environment) {
        Ok(()) => output.lines.push("restarted the background service".to_string()),
        Err(error) => output.warnings.extend([
            format!("warning: the background service did not restart: {error:#}"),
            "restart it with: mootbox service install".to_string(),
        ]),
    }
}

fn folder_of(executable: &Path) -> Result<&Path> {
    executable
        .parent()
        .with_context(|| format!("{} has no parent folder", executable.display()))
}

fn unpack_executable(directory: &Path, asset: &str, archive: &[u8]) -> Result<Vec<u8>> {
    let unpacked = tempfile::Builder::new()
        .prefix(STAGED_PREFIX)
        .tempdir_in(directory)
        .with_context(|| format!("cannot write to {}", directory.display()))?;
    let archive_path = unpacked.path().join(asset);
    fs::write(&archive_path, archive).with_context(|| format!("cannot write {}", archive_path.display()))?;
    extract_executable(&archive_path, unpacked.path(), asset)?;
    read_extracted_executable(&unpacked.path().join(ARCHIVED_EXECUTABLE), asset)
}

fn extract_executable(archive_path: &Path, destination: &Path, asset: &str) -> Result<()> {
    let ran = Command::new(TAR_PROGRAM)
        .arg("-xzf")
        .arg(archive_path)
        .arg("-C")
        .arg(destination)
        .arg(ARCHIVED_EXECUTABLE)
        .output()
        .with_context(|| format!("cannot run {TAR_PROGRAM} to unpack {asset}"))?;
    if !ran.status.success() {
        bail!(
            "cannot unpack {ARCHIVED_EXECUTABLE} from {asset}: {}; kept the current one",
            String::from_utf8_lossy(&ran.stderr).trim()
        );
    }
    Ok(())
}

fn read_extracted_executable(extracted: &Path, asset: &str) -> Result<Vec<u8>> {
    let is_regular_file = fs::symlink_metadata(extracted).is_ok_and(|metadata| metadata.is_file());
    if !is_regular_file {
        bail!("{asset} has no {ARCHIVED_EXECUTABLE} executable; kept the current one");
    }
    fs::read(extracted).with_context(|| format!("cannot read {}", extracted.display()))
}

fn replace_executable(executable: &Path, binary: &[u8], version: Version) -> Result<()> {
    let staged = stage(folder_of(executable)?, binary)?;
    confirm_version(&staged, version)?;
    staged
        .persist(executable)
        .with_context(|| format!("cannot replace {}", executable.display()))
}

fn stage(directory: &Path, binary: &[u8]) -> Result<TempPath> {
    let mut staged = tempfile::Builder::new()
        .prefix(STAGED_PREFIX)
        .tempfile_in(directory)
        .with_context(|| format!("cannot write to {}", directory.display()))?;
    staged
        .write_all(binary)
        .and_then(|()| staged.as_file().sync_all())
        .and_then(|()| {
            staged
                .as_file()
                .set_permissions(Permissions::from_mode(EXECUTABLE_MODE))
        })
        .with_context(|| format!("cannot write {}", staged.path().display()))?;
    Ok(staged.into_temp_path())
}

fn confirm_version(staged: &Path, version: Version) -> Result<()> {
    let ran = Command::new(staged)
        .arg(VERSION_FLAG)
        .output()
        .with_context(|| format!("cannot run the downloaded mootbox {version}"))?;
    let printed = String::from_utf8_lossy(&ran.stdout);
    let expected = format!("mootbox {version}");
    if !ran.status.success() || printed.trim() != expected {
        bail!(
            "the downloaded mootbox printed {:?} instead of {expected:?}; kept the current one",
            printed.trim()
        );
    }
    Ok(())
}
