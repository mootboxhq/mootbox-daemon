use std::path::Path;

use anyhow::Result;
use tracing::warn;

use crate::contracts::{
    CompleteRunRequestChanges, CompleteRunRequestChangesCommitsItem, CompleteRunRequestChangesFilesItem,
    CompleteRunRequestChangesFilesItemPatch, CompleteRunRequestChangesFilesItemStatus,
};
use crate::git::{self, ChangeStatus, ChangedFile, Commit, WorktreeSnapshot};

const MAX_CHANGED_FILES: usize = 300;
const MAX_FILE_PATCH_BYTES: usize = 65_536;
const MAX_TOTAL_PATCH_BYTES: usize = 1_048_576;
const MAX_COMMITS: usize = 50;
const MAX_SUBJECT_LENGTH: usize = 200;
const LINE_END: u8 = b'\n';

struct PatchBudget {
    remaining: usize,
}

struct BoundedPatch {
    text: Option<String>,
    is_truncated: bool,
}

pub async fn snapshot_start(worktree: &Path) -> Option<WorktreeSnapshot> {
    match git::snapshot_worktree(worktree).await {
        Ok(snapshot) => Some(snapshot),
        Err(error) => {
            warn!(
                error = format!("{error:#}"),
                "could not snapshot the worktree; the run reports no file changes"
            );
            None
        }
    }
}

pub async fn capture(worktree: &Path, start: &WorktreeSnapshot) -> Option<CompleteRunRequestChanges> {
    match collect(worktree, start).await {
        Ok(changes) => Some(changes),
        Err(error) => {
            warn!(error = format!("{error:#}"), "could not collect the run's file changes");
            None
        }
    }
}

async fn collect(worktree: &Path, start: &WorktreeSnapshot) -> Result<CompleteRunRequestChanges> {
    let end = git::snapshot_worktree(worktree).await?;
    let files = git::changed_files(worktree, &start.tree, &end.tree).await?;
    let commits = git::commits_since(worktree, &start.head, MAX_COMMITS).await?;
    Ok(reported_changes(&files, &commits))
}

fn reported_changes(files: &[ChangedFile], commits: &[Commit]) -> CompleteRunRequestChanges {
    let mut budget = PatchBudget {
        remaining: MAX_TOTAL_PATCH_BYTES,
    };
    let reportable: Vec<_> = files
        .iter()
        .take(MAX_CHANGED_FILES)
        .filter_map(|file| reported_file(file, &mut budget))
        .collect();
    CompleteRunRequestChanges {
        files_truncated: reportable.len() < files.len(),
        files: reportable,
        commits: commits.iter().filter_map(reported_commit).collect(),
    }
}

fn reported_file(file: &ChangedFile, budget: &mut PatchBudget) -> Option<CompleteRunRequestChangesFilesItem> {
    let path = file.path.as_str().try_into().ok()?;
    let previous_path = match file.previous_path.as_deref() {
        Some(previous) => Some(previous.try_into().ok()?),
        None => None,
    };
    let bounded_patch = if file.is_binary() {
        BoundedPatch {
            text: None,
            is_truncated: false,
        }
    } else {
        budget.take(&file.patch)
    };
    Some(CompleteRunRequestChangesFilesItem {
        path,
        previous_path,
        status: reported_status(file.status),
        additions: file.line_counts.and_then(|counts| i64::try_from(counts.additions).ok()),
        deletions: file.line_counts.and_then(|counts| i64::try_from(counts.deletions).ok()),
        binary: file.is_binary(),
        patch: bounded_patch
            .text
            .and_then(|text| CompleteRunRequestChangesFilesItemPatch::try_from(text).ok()),
        patch_truncated: bounded_patch.is_truncated,
    })
}

fn reported_status(status: ChangeStatus) -> CompleteRunRequestChangesFilesItemStatus {
    match status {
        ChangeStatus::Added => CompleteRunRequestChangesFilesItemStatus::Added,
        ChangeStatus::Modified => CompleteRunRequestChangesFilesItemStatus::Modified,
        ChangeStatus::Deleted => CompleteRunRequestChangesFilesItemStatus::Deleted,
        ChangeStatus::Renamed => CompleteRunRequestChangesFilesItemStatus::Renamed,
    }
}

fn reported_commit(commit: &Commit) -> Option<CompleteRunRequestChangesCommitsItem> {
    let subject: String = commit.subject.chars().take(MAX_SUBJECT_LENGTH).collect();
    Some(CompleteRunRequestChangesCommitsItem {
        sha: commit.sha.as_str().try_into().ok()?,
        subject: subject.try_into().ok()?,
    })
}

impl PatchBudget {
    fn take(&mut self, patch: &[u8]) -> BoundedPatch {
        let bounded = truncated_at_line(patch, MAX_FILE_PATCH_BYTES);
        let is_truncated = bounded.len() < patch.len();
        if bounded.len() > self.remaining {
            return BoundedPatch {
                text: None,
                is_truncated: true,
            };
        }
        self.remaining -= bounded.len();
        BoundedPatch {
            text: Some(String::from_utf8_lossy(bounded).into_owned()),
            is_truncated,
        }
    }
}

fn truncated_at_line(patch: &[u8], limit: usize) -> &[u8] {
    if patch.len() <= limit {
        return patch;
    }
    let window = &patch[..limit];
    let end = window
        .iter()
        .rposition(|byte| *byte == LINE_END)
        .map_or(0, |position| position + 1);
    &patch[..end]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::LineCounts;

    const SHA: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";

    fn text_file(name: &str, patch: Vec<u8>) -> ChangedFile {
        ChangedFile {
            path: name.to_string(),
            previous_path: None,
            status: ChangeStatus::Modified,
            line_counts: Some(LineCounts {
                additions: 1,
                deletions: 0,
            }),
            patch,
        }
    }

    fn patch_of_lines(line: &str, bytes: usize) -> Vec<u8> {
        line.repeat(bytes / line.len()).into_bytes()
    }

    #[test]
    fn files_beyond_the_cap_are_dropped_and_flagged() {
        let files: Vec<_> = (0..=MAX_CHANGED_FILES)
            .map(|index| text_file(&format!("file-{index}.txt"), b"+x\n".to_vec()))
            .collect();

        let changes = reported_changes(&files, &[]);

        assert_eq!(changes.files.len(), MAX_CHANGED_FILES);
        assert!(changes.files_truncated);
    }

    #[test]
    fn an_oversized_patch_is_cut_on_a_line_boundary() {
        let line = "+0123456789abcdef\n";
        let changes = reported_changes(
            &[text_file("big.txt", patch_of_lines(line, 2 * MAX_FILE_PATCH_BYTES))],
            &[],
        );

        let file = &changes.files[0];
        let patch = file.patch.as_deref().expect("kept a prefix");
        assert!(file.patch_truncated);
        assert!(patch.len() <= MAX_FILE_PATCH_BYTES);
        assert!(patch.ends_with(line));
        assert!(!changes.files_truncated);
    }

    #[test]
    fn patches_past_the_total_budget_become_null_and_truncated() {
        let patch = patch_of_lines("+line\n", MAX_FILE_PATCH_BYTES - 6);
        let files: Vec<_> = (0..20)
            .map(|index| text_file(&format!("f{index}"), patch.clone()))
            .collect();

        let changes = reported_changes(&files, &[]);

        let kept = changes.files.iter().filter(|file| file.patch.is_some()).count();
        let total: usize = changes
            .files
            .iter()
            .filter_map(|file| file.patch.as_ref().map(|patch| patch.len()))
            .sum();
        assert!(total <= MAX_TOTAL_PATCH_BYTES);
        assert_eq!(kept, MAX_TOTAL_PATCH_BYTES / patch.len());
        assert!(
            changes
                .files
                .iter()
                .filter(|file| file.patch.is_none())
                .all(|file| file.patch_truncated)
        );
        assert_eq!(changes.files.len(), 20);
    }

    #[test]
    fn binary_files_report_no_counts_and_no_patch() {
        let binary = ChangedFile {
            line_counts: None,
            ..text_file("logo.png", b"Binary files differ\n".to_vec())
        };

        let changes = reported_changes(&[binary], &[]);

        let file = &changes.files[0];
        assert!(file.binary);
        assert_eq!((file.additions, file.deletions), (None, None));
        assert!(file.patch.is_none());
        assert!(!file.patch_truncated);
    }

    #[test]
    fn long_subjects_are_shortened_and_paths_over_the_limit_are_dropped() {
        let commit = Commit {
            sha: SHA.to_string(),
            subject: "s".repeat(MAX_SUBJECT_LENGTH + 10),
        };
        let long_path = text_file(&"p".repeat(1_025), b"+x\n".to_vec());

        let changes = reported_changes(&[long_path], &[commit]);

        assert_eq!(changes.commits[0].subject.chars().count(), MAX_SUBJECT_LENGTH);
        assert!(changes.files.is_empty());
        assert!(changes.files_truncated);
    }
}
