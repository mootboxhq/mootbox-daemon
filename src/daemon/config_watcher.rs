use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use tracing::warn;

use crate::config::{self, DaemonConfig};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FileStamp {
    modified: SystemTime,
    length: u64,
}

pub struct ConfigWatcher {
    home: PathBuf,
    stamp: Option<FileStamp>,
}

impl ConfigWatcher {
    pub fn new(home: &Path) -> Self {
        Self {
            home: home.to_path_buf(),
            stamp: stamp_of(&config::config_path(home)),
        }
    }

    pub fn changed(&mut self) -> Option<DaemonConfig> {
        let stamp = stamp_of(&config::config_path(&self.home));
        if stamp == self.stamp {
            return None;
        }
        self.stamp = stamp;
        match config::load(&self.home) {
            Ok(config) => Some(config),
            Err(error) => {
                warn!(
                    error = format!("{error:#}"),
                    "keeping the running config; the changed one is unreadable"
                );
                None
            }
        }
    }
}

fn stamp_of(path: &Path) -> Option<FileStamp> {
    let metadata = fs::metadata(path).ok()?;
    Some(FileStamp {
        modified: metadata.modified().ok()?,
        length: metadata.len(),
    })
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    const LATER: Duration = Duration::from_secs(5);

    fn touch_later(home: &Path) {
        let file = fs::File::options()
            .write(true)
            .open(config::config_path(home))
            .expect("opens");
        file.set_modified(SystemTime::now() + LATER).expect("sets mtime");
    }

    #[test]
    fn an_unchanged_config_is_not_reloaded() {
        let home = tempfile::tempdir().expect("temp dir");
        config::save(home.path(), &DaemonConfig::default()).expect("saves");
        let mut watcher = ConfigWatcher::new(home.path());

        assert_eq!(watcher.changed(), None);
    }

    #[test]
    fn a_saved_config_is_reloaded_once() {
        let home = tempfile::tempdir().expect("temp dir");
        config::save(home.path(), &DaemonConfig::default()).expect("saves");
        let mut watcher = ConfigWatcher::new(home.path());
        let changed = DaemonConfig {
            slots: 4,
            ..DaemonConfig::default()
        };

        config::save(home.path(), &changed).expect("saves");
        touch_later(home.path());

        assert_eq!(watcher.changed(), Some(changed));
        assert_eq!(watcher.changed(), None);
    }

    #[test]
    fn a_config_created_after_start_is_loaded() {
        let home = tempfile::tempdir().expect("temp dir");
        let mut watcher = ConfigWatcher::new(home.path());

        config::save(home.path(), &DaemonConfig::default()).expect("saves");

        assert_eq!(watcher.changed(), Some(DaemonConfig::default()));
    }

    #[test]
    fn an_unreadable_change_keeps_the_running_config() {
        let home = tempfile::tempdir().expect("temp dir");
        config::save(home.path(), &DaemonConfig::default()).expect("saves");
        let mut watcher = ConfigWatcher::new(home.path());

        fs::write(config::config_path(home.path()), "slots = \"many\"\n").expect("writes");
        touch_later(home.path());

        assert_eq!(watcher.changed(), None);
    }
}
