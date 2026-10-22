use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use uuid::Uuid;

use crate::private_files;

const SESSIONS_DIRECTORY: &str = "sessions";

pub struct SessionFile {
    path: PathBuf,
}

impl SessionFile {
    pub fn new(home: &Path, session_key: Uuid) -> Self {
        Self {
            path: home.join(SESSIONS_DIRECTORY).join(format!("{session_key}.json")),
        }
    }

    pub fn load(&self, is_required: bool) -> Result<Option<String>> {
        match std::fs::read(&self.path) {
            Ok(bytes) => Ok(Some(
                serde_json::from_slice(&bytes).context("local session mapping is corrupt; explicitly start fresh")?,
            )),
            Err(error) if error.kind() == ErrorKind::NotFound && !is_required => Ok(None),
            Err(error) => Err(error).context("local provider session is missing; explicitly start fresh in Mootbox"),
        }
    }

    pub fn save(&self, session_id: &str) -> Result<()> {
        private_files::create_private_directory(self.path.parent().context("session directory missing")?)?;
        let temporary = private_files::temporary_path(&self.path);
        private_files::write_owner_only(&temporary, &serde_json::to_vec(session_id)?)?;
        std::fs::rename(temporary, &self.path).context("could not persist local provider session")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_saved_session_loads_back_and_a_missing_one_fails_only_when_required() {
        let home = tempfile::tempdir().expect("temp dir");
        let file = SessionFile::new(home.path(), Uuid::new_v4());

        assert_eq!(file.load(false).expect("optional"), None);
        assert!(file.load(true).is_err());
        file.save("session-1").expect("saves");
        assert_eq!(file.load(true).expect("loads").as_deref(), Some("session-1"));
    }
}
