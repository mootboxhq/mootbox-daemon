use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use anyhow::{Context, Result, bail};
use serde::Serialize;
use tokio::process::Command;

use crate::private_files::create_private_directory;

mod changes;

pub use changes::{
    ChangeStatus, ChangedFile, Commit, LineCounts, WorktreeSnapshot, changed_files, commits_since, snapshot_worktree,
};

const FETCH_SUFFIX: &str = " (fetch)";
const REMOTE_SEPARATOR: char = '\t';
const LOCAL_HEAD: &str = "HEAD";
const PATH_SEPARATOR: char = '/';
const PATH_SEPARATOR_REPLACEMENT: &str = "-";
const RESERVED_DIRECTORY_NAMES: [&str; 3] = ["", ".", ".."];

pub struct WorktreeRequest<'a> {
    pub repository: &'a Path,
    pub worktrees: &'a Path,
    pub branch: &'a str,
    pub base_ref: &'a str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Remote {
    pub name: String,
    pub url: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckoutState {
    pub branch: Option<String>,
    pub head: Option<String>,
    pub is_dirty: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Checkpoint {
    pub branch: String,
    pub head: String,
    pub base_ref: String,
    pub commits_ahead: u64,
    pub uncommitted_files: usize,
}

pub fn directory_name(name: &str) -> Result<String> {
    let flattened = name.replace(PATH_SEPARATOR, PATH_SEPARATOR_REPLACEMENT);
    if RESERVED_DIRECTORY_NAMES.contains(&flattened.as_str()) {
        bail!("{name:?} cannot be used as a directory name");
    }
    Ok(flattened)
}

pub fn worktree_path(request: &WorktreeRequest<'_>) -> Result<PathBuf> {
    Ok(request.worktrees.join(directory_name(request.branch)?))
}

pub async fn open_branch_worktree(request: &WorktreeRequest<'_>) -> Result<PathBuf> {
    let path = worktree_path(request)?;
    if is_checked_out(&path, request.branch).await {
        return Ok(path);
    }
    create_private_directory(request.worktrees)?;
    let is_existing_branch = !git(request.repository, ["branch", "--list", request.branch])
        .await?
        .is_empty();
    let path_argument = path.as_os_str();
    if is_existing_branch {
        git(
            request.repository,
            [
                OsStr::new("worktree"),
                OsStr::new("add"),
                path_argument,
                OsStr::new(request.branch),
            ],
        )
        .await?;
        return Ok(path);
    }
    git(
        request.repository,
        [
            OsStr::new("worktree"),
            OsStr::new("add"),
            OsStr::new("-b"),
            OsStr::new(request.branch),
            path_argument,
            OsStr::new(request.base_ref),
        ],
    )
    .await?;
    Ok(path)
}

pub async fn capture_checkpoint(worktree: &Path, base_ref: &str) -> Result<Checkpoint> {
    let branch = git(worktree, ["rev-parse", "--abbrev-ref", LOCAL_HEAD]).await?;
    let head = git(worktree, ["rev-parse", LOCAL_HEAD]).await?;
    let commits_ahead = git(worktree, ["rev-list", "--count", &format!("{base_ref}..{LOCAL_HEAD}")])
        .await?
        .parse()
        .context("git rev-list --count printed a non-number")?;
    let status = git(worktree, ["status", "--porcelain"]).await?;
    Ok(Checkpoint {
        branch,
        head,
        base_ref: base_ref.to_string(),
        commits_ahead,
        uncommitted_files: status.lines().count(),
    })
}

pub async fn installed_version() -> Result<String> {
    git(Path::new("."), ["--version"]).await
}

pub async fn is_repository(path: &Path) -> bool {
    git(path, ["rev-parse", "--git-dir"]).await.is_ok()
}

pub async fn has_remote(repository: &Path, remote: &str) -> bool {
    git(repository, ["remote", "get-url", remote]).await.is_ok()
}

pub async fn top_level(path: &Path) -> Result<PathBuf> {
    git(path, ["rev-parse", "--show-toplevel"]).await.map(PathBuf::from)
}

pub async fn remotes(repository: &Path) -> Result<Vec<Remote>> {
    let listing = git(repository, ["remote", "-v"]).await?;
    Ok(fetch_remotes(&listing))
}

pub async fn remote_default_branch(repository: &Path, remote: &str) -> Option<String> {
    let remote_head = format!("refs/remotes/{remote}/HEAD");
    let branch = git(repository, ["symbolic-ref", "--short", &remote_head]).await.ok()?;
    let prefix = format!("{remote}/");
    Some(branch.strip_prefix(&prefix).unwrap_or(&branch).to_string())
}

pub async fn checkout_state(repository: &Path) -> Result<CheckoutState> {
    let status = git(repository, ["status", "--porcelain"]).await?;
    Ok(CheckoutState {
        branch: git(repository, ["symbolic-ref", "--short", "--quiet", LOCAL_HEAD])
            .await
            .ok(),
        head: git(repository, ["rev-parse", "--verify", "--quiet", LOCAL_HEAD])
            .await
            .ok(),
        is_dirty: !status.is_empty(),
    })
}

fn fetch_remotes(listing: &str) -> Vec<Remote> {
    listing
        .lines()
        .filter_map(|line| line.strip_suffix(FETCH_SUFFIX))
        .filter_map(|line| line.split_once(REMOTE_SEPARATOR))
        .map(|(name, url)| Remote {
            name: name.to_string(),
            url: url.to_string(),
        })
        .collect()
}

async fn is_checked_out(path: &Path, branch: &str) -> bool {
    git(path, ["rev-parse", "--abbrev-ref", LOCAL_HEAD])
        .await
        .is_ok_and(|current| current == branch)
}

async fn git<I, S>(directory: &Path, arguments: I) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let stdout = git_output(git_command(directory, arguments), directory).await?;
    Ok(String::from_utf8_lossy(&stdout).trim().to_string())
}

fn git_command<I, S>(directory: &Path, arguments: I) -> Command
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new("git");
    command.arg("-C").arg(directory).args(arguments).stdin(Stdio::null());
    command
}

async fn git_output(mut command: Command, directory: &Path) -> Result<Vec<u8>> {
    let output = command.output().await.context("cannot run git")?;
    if !output.status.success() {
        bail!(
            "git failed in {}: {}",
            directory.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(output.stdout)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_git(directory: &Path, arguments: &[&str]) {
        let status = std::process::Command::new("git")
            .arg("-C")
            .arg(directory)
            .args(arguments)
            .status()
            .expect("git runs");
        assert!(status.success(), "git {arguments:?} failed");
    }

    fn initialized_repository() -> (tempfile::TempDir, PathBuf) {
        let directory = tempfile::tempdir().expect("temp dir");
        let path = directory.path().canonicalize().expect("canonical path");
        run_git(&path, &["init", "--quiet"]);
        (directory, path)
    }

    #[tokio::test]
    async fn top_level_of_a_subdirectory_is_the_repository_root() {
        let (_directory, root) = initialized_repository();
        let nested = root.join("src/deep");
        std::fs::create_dir_all(&nested).expect("creates nested directory");

        assert_eq!(top_level(&nested).await.expect("inside a repository"), root);
    }

    #[tokio::test]
    async fn top_level_outside_a_repository_fails() {
        let plain = tempfile::tempdir().expect("temp dir");

        assert!(top_level(plain.path()).await.is_err());
    }

    #[tokio::test]
    async fn remotes_lists_each_fetch_remote_once() {
        let (_directory, root) = initialized_repository();
        assert_eq!(remotes(&root).await.expect("lists"), []);

        run_git(&root, &["remote", "add", "origin", "git@github.com:acme/checkout.git"]);
        run_git(
            &root,
            &["remote", "add", "upstream", "https://github.com/acme/checkout.git"],
        );

        assert_eq!(
            remotes(&root).await.expect("lists"),
            [
                Remote {
                    name: "origin".to_string(),
                    url: "git@github.com:acme/checkout.git".to_string(),
                },
                Remote {
                    name: "upstream".to_string(),
                    url: "https://github.com/acme/checkout.git".to_string(),
                },
            ]
        );
    }

    #[tokio::test]
    async fn remote_default_branch_drops_the_remote_prefix() {
        let (_directory, root) = initialized_repository();
        assert_eq!(remote_default_branch(&root, "origin").await, None);

        run_git(
            &root,
            &["symbolic-ref", "refs/remotes/origin/HEAD", "refs/remotes/origin/trunk"],
        );

        assert_eq!(remote_default_branch(&root, "origin").await.as_deref(), Some("trunk"));
    }

    #[tokio::test]
    async fn the_checkout_state_of_a_fresh_repository_has_a_branch_and_no_head() {
        let (_directory, root) = initialized_repository();
        run_git(&root, &["symbolic-ref", LOCAL_HEAD, "refs/heads/main"]);

        let state = checkout_state(&root).await.expect("reads");

        assert_eq!(state.branch.as_deref(), Some("main"));
        assert_eq!(state.head, None);
        assert!(!state.is_dirty);
    }

    #[tokio::test]
    async fn an_untracked_file_makes_the_checkout_dirty() {
        let (_directory, root) = initialized_repository();
        std::fs::write(root.join("notes.txt"), "draft").expect("writes");

        assert!(checkout_state(&root).await.expect("reads").is_dirty);
    }

    #[test]
    fn branch_names_flatten_into_one_directory_name() {
        assert_eq!(
            directory_name("mootbox/abc-checkout").expect("name"),
            "mootbox-abc-checkout"
        );
    }

    #[test]
    fn names_that_would_leave_the_parent_directory_are_refused() {
        for name in ["", ".", ".."] {
            assert!(directory_name(name).is_err(), "{name:?}");
        }
    }
}
