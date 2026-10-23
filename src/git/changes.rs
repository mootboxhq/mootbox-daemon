use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use super::{LOCAL_HEAD, git, git_command, git_output};

const INDEX_ENVIRONMENT: &str = "GIT_INDEX_FILE";
const SNAPSHOT_INDEX_FILE: &str = "index";
const RECORD_SEPARATOR: char = '\0';
const FIELD_SEPARATOR: char = '\t';
const BINARY_COUNT: &str = "-";
const DIFF_HEADER: &[u8] = b"diff --git ";
const COMMIT_FORMAT: &str = "--format=%H%x00%s";
const DIFF_TREE_OPTIONS: [&str; 6] = ["diff-tree", "-r", "-M", "--no-color", "--no-ext-diff", "--no-textconv"];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorktreeSnapshot {
    pub head: String,
    pub tree: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChangeStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LineCounts {
    pub additions: u64,
    pub deletions: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChangedFile {
    pub path: String,
    pub previous_path: Option<String>,
    pub status: ChangeStatus,
    pub line_counts: Option<LineCounts>,
    pub patch: Vec<u8>,
}

impl ChangedFile {
    pub fn is_binary(&self) -> bool {
        self.line_counts.is_none()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Commit {
    pub sha: String,
    pub subject: String,
}

struct NamedChange {
    path: String,
    previous_path: Option<String>,
    status: ChangeStatus,
}

pub async fn snapshot_worktree(worktree: &Path) -> Result<WorktreeSnapshot> {
    let head = git(worktree, ["rev-parse", LOCAL_HEAD]).await?;
    let tree = snapshot_tree(worktree).await?;
    Ok(WorktreeSnapshot { head, tree })
}

pub async fn changed_files(worktree: &Path, from_tree: &str, to_tree: &str) -> Result<Vec<ChangedFile>> {
    let names = diff_tree(worktree, "--name-status", [from_tree, to_tree]).await?;
    let counts = diff_tree(worktree, "--numstat", [from_tree, to_tree]).await?;
    let patch = diff_tree(worktree, "--patch", [from_tree, to_tree]).await?;
    let names = parse_name_status(&String::from_utf8_lossy(&names))?;
    let counts = parse_numstat(&String::from_utf8_lossy(&counts))?;
    let patches = split_patches(&patch);
    if names.len() != counts.len() || names.len() != patches.len() {
        bail!("git diff-tree listed a different number of files per format");
    }
    Ok(names
        .into_iter()
        .zip(counts)
        .zip(patches)
        .map(|((name, line_counts), patch)| ChangedFile {
            path: name.path,
            previous_path: name.previous_path,
            status: name.status,
            line_counts,
            patch: patch.to_vec(),
        })
        .collect())
}

pub async fn commits_since(worktree: &Path, start_head: &str, limit: usize) -> Result<Vec<Commit>> {
    let range = format!("{start_head}..{LOCAL_HEAD}");
    let max_count = format!("--max-count={limit}");
    let listing = git(worktree, ["log", COMMIT_FORMAT, max_count.as_str(), range.as_str()]).await?;
    Ok(listing
        .lines()
        .filter_map(|line| line.split_once(RECORD_SEPARATOR))
        .map(|(sha, subject)| Commit {
            sha: sha.to_string(),
            subject: subject.to_string(),
        })
        .collect())
}

async fn snapshot_tree(worktree: &Path) -> Result<String> {
    let directory = tempfile::tempdir().context("cannot create a snapshot index directory")?;
    let index = directory.path().join(SNAPSHOT_INDEX_FILE);
    git_with_index(worktree, &index, &["read-tree", LOCAL_HEAD]).await?;
    git_with_index(worktree, &index, &["add", "--all"]).await?;
    git_with_index(worktree, &index, &["write-tree"]).await
}

async fn git_with_index(worktree: &Path, index: &Path, arguments: &[&str]) -> Result<String> {
    let mut command = git_command(worktree, arguments);
    command.env(INDEX_ENVIRONMENT, index);
    let stdout = git_output(command, worktree).await?;
    Ok(String::from_utf8_lossy(&stdout).trim().to_string())
}

async fn diff_tree(worktree: &Path, format: &str, trees: [&str; 2]) -> Result<Vec<u8>> {
    let command = git_command(
        worktree,
        DIFF_TREE_OPTIONS.into_iter().chain(["-z", format]).chain(trees),
    );
    git_output(command, worktree).await
}

fn parse_name_status(listing: &str) -> Result<Vec<NamedChange>> {
    let mut fields = listing.split(RECORD_SEPARATOR).filter(|field| !field.is_empty());
    let mut changes = Vec::new();
    while let Some(code) = fields.next() {
        let status = status_of(code);
        let first = fields.next().context("git diff-tree --name-status ended early")?;
        let change = if status == ChangeStatus::Renamed {
            let path = fields.next().context("git diff-tree --name-status ended early")?;
            NamedChange {
                path: path.to_string(),
                previous_path: Some(first.to_string()),
                status,
            }
        } else {
            NamedChange {
                path: first.to_string(),
                previous_path: None,
                status,
            }
        };
        changes.push(change);
    }
    Ok(changes)
}

fn status_of(code: &str) -> ChangeStatus {
    match code.chars().next() {
        Some('A' | 'C') => ChangeStatus::Added,
        Some('D') => ChangeStatus::Deleted,
        Some('R') => ChangeStatus::Renamed,
        _ => ChangeStatus::Modified,
    }
}

fn parse_numstat(listing: &str) -> Result<Vec<Option<LineCounts>>> {
    let mut fields = listing.split(RECORD_SEPARATOR).filter(|field| !field.is_empty());
    let mut counts = Vec::new();
    while let Some(record) = fields.next() {
        let mut columns = record.splitn(3, FIELD_SEPARATOR);
        let additions = columns.next().context("git diff-tree --numstat ended early")?;
        let deletions = columns.next().context("git diff-tree --numstat ended early")?;
        if columns.next().is_none_or(str::is_empty) {
            fields.next().context("git diff-tree --numstat ended early")?;
            fields.next().context("git diff-tree --numstat ended early")?;
        }
        counts.push(line_counts(additions, deletions)?);
    }
    Ok(counts)
}

fn line_counts(additions: &str, deletions: &str) -> Result<Option<LineCounts>> {
    if additions == BINARY_COUNT || deletions == BINARY_COUNT {
        return Ok(None);
    }
    Ok(Some(LineCounts {
        additions: additions.parse().context("git printed a non-number line count")?,
        deletions: deletions.parse().context("git printed a non-number line count")?,
    }))
}

fn split_patches(patch: &[u8]) -> Vec<&[u8]> {
    let mut starts: Vec<usize> = line_starts(patch)
        .filter(|start| patch[*start..].starts_with(DIFF_HEADER))
        .collect();
    starts.push(patch.len());
    starts.windows(2).map(|bounds| &patch[bounds[0]..bounds[1]]).collect()
}

fn line_starts(text: &[u8]) -> impl Iterator<Item = usize> + '_ {
    std::iter::once(0).chain(
        text.iter()
            .enumerate()
            .filter(|(_, byte)| **byte == b'\n')
            .map(|(position, _)| position + 1)
            .filter(move |start| *start < text.len()),
    )
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn run_git(directory: &Path, arguments: &[&str]) {
        let status = std::process::Command::new("git")
            .arg("-C")
            .arg(directory)
            .args(["-c", "user.name=Moot", "-c", "user.email=moot@example.com"])
            .args(arguments)
            .status()
            .expect("git runs");
        assert!(status.success(), "git {arguments:?} failed");
    }

    fn write(root: &Path, path: &str, contents: &str) {
        std::fs::write(root.join(path), contents).expect("writes");
    }

    fn committed_repository() -> (tempfile::TempDir, PathBuf) {
        let directory = tempfile::tempdir().expect("temp dir");
        let root = directory.path().canonicalize().expect("canonical path");
        run_git(&root, &["init", "--quiet"]);
        write(&root, "hello.txt", "one\ntwo\nthree\nfour\nfive\n");
        write(&root, "notes.txt", "draft\n");
        run_git(&root, &["add", "--all"]);
        run_git(&root, &["commit", "--quiet", "-m", "start"]);
        (directory, root)
    }

    async fn changes_since(root: &Path, start: &WorktreeSnapshot) -> Vec<ChangedFile> {
        let end = snapshot_worktree(root).await.expect("snapshots");
        changed_files(root, &start.tree, &end.tree).await.expect("diffs")
    }

    fn paths(files: &[ChangedFile]) -> Vec<&str> {
        files.iter().map(|file| file.path.as_str()).collect()
    }

    #[tokio::test]
    async fn uncommitted_work_from_before_the_start_is_not_in_the_diff() {
        let (_directory, root) = committed_repository();
        write(&root, "notes.txt", "earlier run\n");
        let start = snapshot_worktree(&root).await.expect("snapshots");

        write(&root, "hello.txt", "one\nTWO\nthree\nfour\nfive\n");
        let files = changes_since(&root, &start).await;

        assert_eq!(paths(&files), ["hello.txt"]);
        assert_eq!(files[0].status, ChangeStatus::Modified);
        assert_eq!(
            files[0].line_counts,
            Some(LineCounts {
                additions: 1,
                deletions: 1
            })
        );
        let patch = String::from_utf8(files[0].patch.clone()).expect("UTF-8 patch");
        assert!(patch.starts_with("diff --git a/hello.txt b/hello.txt\n"), "{patch}");
        assert!(patch.contains("-two\n+TWO\n"), "{patch}");
    }

    #[tokio::test]
    async fn snapshotting_leaves_the_real_index_untouched() {
        let (_directory, root) = committed_repository();
        write(&root, "fresh.txt", "new\n");

        snapshot_worktree(&root).await.expect("snapshots");

        let status = git(&root, ["status", "--porcelain"]).await.expect("status");
        assert_eq!(status, "?? fresh.txt");
    }

    #[tokio::test]
    async fn an_untracked_new_file_is_in_the_diff() {
        let (_directory, root) = committed_repository();
        let start = snapshot_worktree(&root).await.expect("snapshots");

        write(&root, "added.txt", "a\nb\n");
        let files = changes_since(&root, &start).await;

        assert_eq!(paths(&files), ["added.txt"]);
        assert_eq!(files[0].status, ChangeStatus::Added);
        assert_eq!(
            files[0].line_counts,
            Some(LineCounts {
                additions: 2,
                deletions: 0
            })
        );
    }

    #[tokio::test]
    async fn a_renamed_file_reports_its_previous_path() {
        let (_directory, root) = committed_repository();
        let start = snapshot_worktree(&root).await.expect("snapshots");

        std::fs::rename(root.join("hello.txt"), root.join("greeting.txt")).expect("renames");
        std::fs::remove_file(root.join("notes.txt")).expect("removes");
        let files = changes_since(&root, &start).await;

        assert_eq!(paths(&files), ["greeting.txt", "notes.txt"]);
        assert_eq!(files[0].status, ChangeStatus::Renamed);
        assert_eq!(files[0].previous_path.as_deref(), Some("hello.txt"));
        assert_eq!(files[1].status, ChangeStatus::Deleted);
    }

    #[tokio::test]
    async fn a_binary_file_has_no_line_counts() {
        let (_directory, root) = committed_repository();
        let start = snapshot_worktree(&root).await.expect("snapshots");

        std::fs::write(root.join("logo.bin"), [0_u8, 159, 146, 150, 0, 1]).expect("writes");
        let files = changes_since(&root, &start).await;

        assert_eq!(paths(&files), ["logo.bin"]);
        assert!(files[0].is_binary());
    }

    #[tokio::test]
    async fn commits_since_the_start_are_listed_newest_first() {
        let (_directory, root) = committed_repository();
        let start = snapshot_worktree(&root).await.expect("snapshots");
        write(&root, "hello.txt", "changed\n");
        run_git(&root, &["commit", "--quiet", "--all", "-m", "first change"]);
        write(&root, "notes.txt", "changed\n");
        run_git(&root, &["commit", "--quiet", "--all", "-m", "second change"]);

        let commits = commits_since(&root, &start.head, 50).await.expect("lists");
        let limited = commits_since(&root, &start.head, 1).await.expect("lists");

        let subjects: Vec<_> = commits.iter().map(|commit| commit.subject.as_str()).collect();
        assert_eq!(subjects, ["second change", "first change"]);
        assert_eq!(commits[0].sha.len(), 40);
        assert_eq!(limited.len(), 1);
    }

    #[test]
    fn numstat_records_of_renames_consume_both_paths() {
        let counts = parse_numstat("1\t2\t\0old.txt\0new.txt\0-\t-\tlogo.bin\0").expect("parses");

        assert_eq!(
            counts,
            [
                Some(LineCounts {
                    additions: 1,
                    deletions: 2
                }),
                None
            ]
        );
    }
}
