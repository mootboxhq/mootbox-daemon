use std::fmt;
use std::path::Path;
use std::process::Output;
use std::sync::LazyLock;
use std::time::Duration;

use regress::Regex;
use tokio::process::Command;

use super::ProviderKind;

const PROBE_TIMEOUT: Duration = Duration::from_secs(15);
const VERSION_FLAG: &str = "--version";

static VERSION_PATTERN: LazyLock<Option<Regex>> = LazyLock::new(|| Regex::new(r"(\d+)\.(\d+)\.(\d+)").ok());

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version(pub u64, pub u64, pub u64);

#[derive(Debug, thiserror::Error)]
pub enum VersionError {
    #[error("cannot read the {label} version from `{program} --version`; install {label} {minimum} or newer")]
    Unreadable {
        label: &'static str,
        program: String,
        minimum: Version,
    },
    #[error("{label} {found} is too old; mootbox needs {minimum} or newer")]
    TooOld {
        label: &'static str,
        found: Version,
        minimum: Version,
    },
}

impl Version {
    pub fn parse(text: &str) -> Option<Self> {
        let found = VERSION_PATTERN.as_ref()?.find(text)?;
        let part = |group: usize| found.group(group).and_then(|range| text[range].parse().ok());
        Some(Self(part(1)?, part(2)?, part(3)?))
    }
}

impl fmt::Display for Version {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}.{}", self.0, self.1, self.2)
    }
}

pub async fn supported_version(provider: ProviderKind, program: &Path) -> Result<Version, VersionError> {
    let label = provider.label();
    let minimum = provider.minimum_version();
    let Some(found) = installed_version(program).await else {
        return Err(VersionError::Unreadable {
            label,
            program: program.display().to_string(),
            minimum,
        });
    };
    if found < minimum {
        return Err(VersionError::TooOld { label, found, minimum });
    }
    Ok(found)
}

pub async fn is_signed_in(provider: ProviderKind, program: &Path) -> Option<bool> {
    let output = run_probe(program, provider.sign_in_arguments()).await?;
    provider.read_sign_in(&output)
}

async fn installed_version(program: &Path) -> Option<Version> {
    let output = run_probe(program, &[VERSION_FLAG]).await?;
    if !output.status.success() {
        return None;
    }
    Version::parse(&String::from_utf8_lossy(&output.stdout))
}

async fn run_probe(program: &Path, arguments: &[&str]) -> Option<Output> {
    let probe = Command::new(program).args(arguments).kill_on_drop(true).output();
    tokio::time::timeout(PROBE_TIMEOUT, probe).await.ok()?.ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::ALL;

    #[test]
    fn versions_are_read_from_the_version_line() {
        assert_eq!(Version::parse("2.1.296 (Claude Code)"), Some(Version(2, 1, 296)));
        assert_eq!(Version::parse("codex-cli 0.120.0"), Some(Version(0, 120, 0)));
        assert_eq!(Version::parse("claude 10.0.3"), Some(Version(10, 0, 3)));
        assert_eq!(Version::parse("Claude Code"), None);
        assert_eq!(Version::parse("2.1"), None);
    }

    #[test]
    fn versions_compare_numerically() {
        let minimum = Version(2, 1, 295);
        for (version, is_supported) in [
            (Version(2, 1, 294), false),
            (Version(2, 1, 295), true),
            (Version(2, 1, 1000), true),
            (Version(2, 2, 0), true),
            (Version(10, 0, 0), true),
            (Version(1, 9, 999), false),
        ] {
            assert_eq!(version >= minimum, is_supported, "{version}");
        }
    }

    #[tokio::test]
    async fn a_missing_executable_has_no_version_and_an_unknown_sign_in_state() {
        for kind in ALL {
            let missing = Path::new("/nonexistent/provider");

            let error = supported_version(kind, missing).await.expect_err("no version");

            assert!(matches!(error, VersionError::Unreadable { .. }), "{error}");
            assert_eq!(is_signed_in(kind, missing).await, None);
        }
    }
}
