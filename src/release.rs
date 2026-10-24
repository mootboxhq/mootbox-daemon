use std::fmt;
use std::str::FromStr;
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use reqwest::header::LOCATION;
use reqwest::{Client, redirect};
use sha2::{Digest, Sha256};

use crate::config::{DEFAULT_RELEASES_URL, RELEASES_URL_VARIABLE};

pub const CHECKSUMS_ASSET: &str = "SHA256SUMS";
pub const ARCHIVED_EXECUTABLE: &str = crate::platform::EXECUTABLE_NAME;

const LATEST_TAG_MARKER: &str = "/releases/tag/v";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
const LOOKUP_TIMEOUT: Duration = Duration::from_secs(30);
const DOWNLOAD_TIMEOUT: Duration = Duration::from_mins(5);
const BINARY_MODE_MARKER: char = '*';
const ARCHIVE_EXTENSION: &str = ".tar.gz";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    major: u64,
    minor: u64,
    patch: u64,
}

impl FromStr for Version {
    type Err = anyhow::Error;

    fn from_str(text: &str) -> Result<Self> {
        let parts = text
            .split('.')
            .map(str::parse::<u64>)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| anyhow!("{text:?} is not a MAJOR.MINOR.PATCH version"))?;
        let [major, minor, patch] = parts[..] else {
            bail!("{text:?} is not a MAJOR.MINOR.PATCH version");
        };
        Ok(Self { major, minor, patch })
    }
}

impl fmt::Display for Version {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

pub struct ReleaseSource {
    base_url: String,
    lookup: Client,
    download: Client,
}

impl ReleaseSource {
    pub fn of_this_process() -> Result<Self> {
        let base_url = std::env::var(RELEASES_URL_VARIABLE).unwrap_or_else(|_| DEFAULT_RELEASES_URL.to_string());
        Self::at(&base_url)
    }

    pub fn at(base_url: &str) -> Result<Self> {
        let lookup = Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .redirect(redirect::Policy::none())
            .build()
            .context("cannot build the release lookup client")?;
        let download = Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .build()
            .context("cannot build the release download client")?;
        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            lookup,
            download,
        })
    }

    pub async fn latest_version(&self) -> Result<Version> {
        let url = format!("{}/releases/latest", self.base_url);
        let response = self
            .lookup
            .get(&url)
            .timeout(LOOKUP_TIMEOUT)
            .send()
            .await
            .with_context(|| format!("cannot reach {url}"))?;
        let location = response
            .headers()
            .get(LOCATION)
            .and_then(|value| value.to_str().ok())
            .with_context(|| format!("{url} answered HTTP {} without a release", response.status()))?;
        let (_, tag_version) = location
            .rsplit_once(LATEST_TAG_MARKER)
            .with_context(|| format!("{url} pointed at {location}, not a release tag"))?;
        tag_version.parse()
    }

    pub async fn download(&self, version: Version, asset: &str) -> Result<Vec<u8>> {
        let url = format!("{}/releases/download/v{version}/{asset}", self.base_url);
        let response = self
            .download
            .get(&url)
            .timeout(DOWNLOAD_TIMEOUT)
            .send()
            .await
            .and_then(reqwest::Response::error_for_status)
            .with_context(|| format!("cannot download {url}"))?;
        let bytes = response
            .bytes()
            .await
            .with_context(|| format!("cannot download {url}"))?;
        Ok(bytes.to_vec())
    }
}

pub fn current_version() -> Result<Version> {
    crate::device::DAEMON_VERSION.parse()
}

pub fn host_asset() -> Result<String> {
    asset_for(std::env::consts::OS, std::env::consts::ARCH)
}

fn asset_for(os: &str, arch: &str) -> Result<String> {
    let system = match os {
        "macos" => "macos",
        "linux" => "linux",
        "windows" => "windows",
        other => bail!("mootbox releases are built for macOS, Linux, and Windows only, not {other}"),
    };
    let architecture = match arch {
        "aarch64" => "arm64",
        "x86_64" => "x64",
        other => bail!("mootbox releases are built for x86_64 and aarch64 only, not {other}"),
    };
    let extension = if os == "windows" { ".zip" } else { ARCHIVE_EXTENSION };
    Ok(format!("mootbox-{system}-{architecture}{extension}"))
}

pub fn verify_checksum(checksums: &str, asset: &str, contents: &[u8]) -> Result<()> {
    let expected = expected_checksum(checksums, asset)?;
    let actual = sha256_hex(contents);
    if !expected.eq_ignore_ascii_case(&actual) {
        bail!("{asset} does not match {CHECKSUMS_ASSET}: expected {expected}, got {actual}");
    }
    Ok(())
}

fn expected_checksum<'a>(checksums: &'a str, asset: &str) -> Result<&'a str> {
    checksums
        .lines()
        .filter_map(|line| line.split_once(char::is_whitespace))
        .find(|(_, name)| name.trim_start().trim_start_matches(BINARY_MODE_MARKER) == asset)
        .map(|(digest, _)| digest)
        .with_context(|| format!("{CHECKSUMS_ASSET} has no line for {asset}"))
}

fn sha256_hex(contents: &[u8]) -> String {
    Sha256::digest(contents)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .concat()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EMPTY_SHA256: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

    #[test]
    fn versions_compare_numerically_by_major_minor_and_patch() {
        let version = |text: &str| text.parse::<Version>().expect("valid version");

        assert!(version("0.10.0") > version("0.9.9"));
        assert!(version("1.0.0") > version("0.99.99"));
        assert_eq!(version("0.1.0"), version("0.1.0"));
        assert_eq!(version("2.3.4").to_string(), "2.3.4");
    }

    #[test]
    fn versions_without_three_numeric_parts_are_refused() {
        assert!("1.2".parse::<Version>().is_err());
        assert!("1.2.3.4".parse::<Version>().is_err());
        assert!("1.2.x".parse::<Version>().is_err());
    }

    #[test]
    fn checksums_match_the_line_for_the_asset_in_text_or_binary_mode() {
        let checksums = format!("{}  mootbox-other\n{EMPTY_SHA256} *mootbox-wanted\n", "0".repeat(64));

        assert!(verify_checksum(&checksums, "mootbox-wanted", b"").is_ok());
        assert!(verify_checksum(&checksums, "mootbox-other", b"").is_err());
    }

    #[test]
    fn a_missing_checksum_line_is_refused() {
        let error = verify_checksum("", "mootbox-wanted", b"").expect_err("no line");

        assert!(error.to_string().contains("has no line for mootbox-wanted"), "{error}");
    }

    #[test]
    fn each_released_platform_maps_to_its_archive() {
        let asset = |os: &str, arch: &str| asset_for(os, arch).expect("released platform");

        assert_eq!(asset("macos", "aarch64"), "mootbox-macos-arm64.tar.gz");
        assert_eq!(asset("macos", "x86_64"), "mootbox-macos-x64.tar.gz");
        assert_eq!(asset("linux", "x86_64"), "mootbox-linux-x64.tar.gz");
        assert_eq!(asset("linux", "aarch64"), "mootbox-linux-arm64.tar.gz");
        assert_eq!(asset("windows", "x86_64"), "mootbox-windows-x64.zip");
    }

    #[test]
    fn unreleased_platforms_are_refused() {
        assert!(asset_for("freebsd", "x86_64").is_err());
        assert!(asset_for("linux", "riscv64").is_err());
    }

    #[test]
    fn the_host_asset_names_a_released_archive() {
        let asset = host_asset().expect("supported host");

        assert!(
            asset.starts_with("mootbox-macos-")
                || asset.starts_with("mootbox-linux-")
                || asset.starts_with("mootbox-windows-"),
            "{asset}"
        );
        assert!(
            asset.ends_with(".tar.gz")
                || std::path::Path::new(&asset)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("zip")),
            "{asset}"
        );
    }
}
