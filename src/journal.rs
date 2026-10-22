use std::fs;
use std::io::ErrorKind;
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::warn;
use uuid::Uuid;

use crate::private_files;

const JOURNAL_FILE: &str = "journal.json";
const CORRUPT_SUFFIX: &str = ".corrupt-";
const CORRUPT_TIMESTAMP_FORMAT: &str = "%Y%m%dT%H%M%S%.3fZ";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalEntry {
    pub run_id: Uuid,
    pub generation: NonZeroU64,
    pub branch: String,
    pub worktree: PathBuf,
    pub started_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<Uuid>,
}

impl JournalEntry {
    pub fn belongs_to(&self, workspace_id: Uuid) -> bool {
        self.workspace_id.is_none_or(|owner| owner == workspace_id)
    }
}

pub struct Journal {
    path: PathBuf,
    entries: Mutex<Vec<JournalEntry>>,
}

impl Journal {
    pub fn open(home: &Path) -> Result<Self> {
        private_files::create_private_directory(home)?;
        let path = home.join(JOURNAL_FILE);
        let entries = load_entries(&path)?;
        Ok(Self {
            path,
            entries: Mutex::new(entries),
        })
    }

    pub async fn entries(&self) -> Vec<JournalEntry> {
        self.entries.lock().await.clone()
    }

    pub async fn entry(&self, run_id: Uuid) -> Option<JournalEntry> {
        self.entries
            .lock()
            .await
            .iter()
            .find(|entry| entry.run_id == run_id)
            .cloned()
    }

    pub async fn record(&self, entry: JournalEntry) -> Result<()> {
        self.rewrite(|current| {
            let mut changed = without(current, entry.run_id);
            changed.push(entry);
            changed
        })
        .await
    }

    pub async fn remove(&self, run_id: Uuid) -> Result<()> {
        self.rewrite(|current| without(current, run_id)).await
    }

    async fn rewrite(&self, change: impl FnOnce(&[JournalEntry]) -> Vec<JournalEntry>) -> Result<()> {
        let mut entries = self.entries.lock().await;
        let changed = change(&entries);
        if changed == *entries {
            return Ok(());
        }
        self.write(&changed)?;
        *entries = changed;
        Ok(())
    }

    fn write(&self, entries: &[JournalEntry]) -> Result<()> {
        private_files::replace_owner_only(&self.path, &serde_json::to_vec_pretty(entries)?)
    }
}

fn without(entries: &[JournalEntry], run_id: Uuid) -> Vec<JournalEntry> {
    entries.iter().filter(|entry| entry.run_id != run_id).cloned().collect()
}

fn load_entries(path: &Path) -> Result<Vec<JournalEntry>> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error).with_context(|| format!("cannot read {}", path.display())),
    };
    match serde_json::from_str(&text) {
        Ok(entries) => Ok(entries),
        Err(error) => {
            let aside = move_aside(path)?;
            warn!(%error, moved_to = %aside.display(), "the run journal was unreadable; starting with an empty journal");
            Ok(Vec::new())
        }
    }
}

fn move_aside(path: &Path) -> Result<PathBuf> {
    let mut aside = path.as_os_str().to_owned();
    aside.push(CORRUPT_SUFFIX);
    aside.push(Utc::now().format(CORRUPT_TIMESTAMP_FORMAT).to_string());
    let aside = PathBuf::from(aside);
    fs::rename(path, &aside).with_context(|| format!("cannot move {} aside", path.display()))?;
    Ok(aside)
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;
    use std::sync::Arc;

    use tokio::task::JoinSet;

    use super::*;

    const CONCURRENT_RUNS: usize = 12;

    fn entry_for(run_id: Uuid) -> JournalEntry {
        JournalEntry {
            run_id,
            generation: NonZeroU64::MIN,
            branch: String::new(),
            worktree: PathBuf::from("/tmp/worktree"),
            started_at: Utc::now(),
            workspace_id: None,
        }
    }

    fn files_in(home: &Path) -> Vec<String> {
        let mut names: Vec<_> = fs::read_dir(home)
            .expect("home")
            .filter_map(Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn concurrent_records_and_removes_keep_every_entry() {
        let home = tempfile::tempdir().expect("temp dir");
        let journal = Arc::new(Journal::open(home.path()).expect("opens"));
        let run_ids: Vec<_> = (0..CONCURRENT_RUNS).map(|_| Uuid::new_v4()).collect();
        let mut writes = JoinSet::new();
        for run_id in run_ids.clone() {
            let journal = Arc::clone(&journal);
            writes.spawn(async move { journal.record(entry_for(run_id)).await });
        }
        writes
            .join_all()
            .await
            .into_iter()
            .collect::<Result<()>>()
            .expect("records");

        let removed = run_ids[0];
        let added = Uuid::new_v4();
        let (removing, adding) = tokio::join!(journal.remove(removed), journal.record(entry_for(added)));
        removing.expect("removes");
        adding.expect("records");

        let reopened: Vec<_> = Journal::open(home.path())
            .expect("reopens")
            .entries()
            .await
            .into_iter()
            .map(|entry| entry.run_id)
            .collect();
        assert_eq!(reopened.len(), CONCURRENT_RUNS);
        assert!(!reopened.contains(&removed));
        assert!(reopened.contains(&added));
        assert!(run_ids[1..].iter().all(|run_id| reopened.contains(run_id)));
    }

    #[tokio::test]
    async fn the_journal_file_is_owner_only_and_leaves_no_temporary_behind() {
        let home = tempfile::tempdir().expect("temp dir");
        let journal = Journal::open(home.path()).expect("opens");

        journal.record(entry_for(Uuid::new_v4())).await.expect("records");

        let mode = fs::metadata(home.path().join(JOURNAL_FILE))
            .expect("journal")
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
        assert_eq!(files_in(home.path()), [JOURNAL_FILE]);
    }

    #[tokio::test]
    async fn a_missing_journal_opens_empty() {
        let home = tempfile::tempdir().expect("temp dir");

        let journal = Journal::open(home.path()).expect("opens");

        assert_eq!(journal.entries().await, Vec::new());
        assert_eq!(files_in(home.path()), Vec::<String>::new());
    }

    #[tokio::test]
    async fn a_corrupt_journal_is_moved_aside_and_opens_empty() {
        let home = tempfile::tempdir().expect("temp dir");
        fs::write(home.path().join(JOURNAL_FILE), "{ not a journal").expect("writes");

        let journal = Journal::open(home.path()).expect("opens");

        assert_eq!(journal.entries().await, Vec::new());
        let files = files_in(home.path());
        assert_eq!(files.len(), 1, "{files:?}");
        assert!(files[0].starts_with("journal.json.corrupt-"), "{files:?}");
        let aside = fs::read_to_string(home.path().join(&files[0])).expect("kept");
        assert_eq!(aside, "{ not a journal");
    }
}
