use std::fs::{File, OpenOptions, TryLockError};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

use anyhow::{Context, Result, bail};

use crate::private_files;

const LOCK_FILE: &str = "daemon.lock";

#[derive(Debug)]
pub struct DaemonLock {
    _file: File,
}

pub fn acquire(home: &Path) -> Result<DaemonLock> {
    match try_acquire(home)? {
        Some(lock) => Ok(lock),
        None => bail!(
            "another mootbox daemon is already running for {}; stop it first",
            home.display()
        ),
    }
}

pub fn is_held(home: &Path) -> Result<bool> {
    Ok(try_acquire(home)?.is_none())
}

fn try_acquire(home: &Path) -> Result<Option<DaemonLock>> {
    private_files::create_private_directory(home)?;
    let path = home.join(LOCK_FILE);
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(false);
    #[cfg(unix)]
    options.mode(private_files::OWNER_ONLY_FILE);
    let file = options
        .open(&path)
        .with_context(|| format!("cannot open {}", path.display()))?;
    match file.try_lock() {
        Ok(()) => Ok(Some(DaemonLock { _file: file })),
        Err(TryLockError::WouldBlock) => Ok(None),
        Err(TryLockError::Error(error)) => Err(error).with_context(|| format!("cannot lock {}", path.display())),
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;

    const SETTLE_TIMEOUT: Duration = Duration::from_secs(2);
    const SETTLE_POLL: Duration = Duration::from_millis(10);

    fn settled<T>(probe: impl Fn() -> Option<T>) -> T {
        let deadline = Instant::now() + SETTLE_TIMEOUT;
        loop {
            if let Some(value) = probe() {
                return value;
            }
            assert!(Instant::now() < deadline, "the lock did not settle");
            std::thread::sleep(SETTLE_POLL);
        }
    }

    fn acquired(home: &Path) -> DaemonLock {
        settled(|| acquire(home).ok())
    }

    fn released(home: &Path) {
        settled(|| (!is_held(home).expect("checks")).then_some(()));
    }

    #[test]
    fn a_second_lock_on_the_same_home_is_refused_until_the_first_is_released() {
        let home = tempfile::tempdir().expect("temp dir");
        let first = acquired(home.path());

        let refused = acquire(home.path()).expect_err("second lock");

        assert!(refused.to_string().contains("already running"), "{refused}");
        drop(first);
        acquired(home.path());
    }

    #[test]
    fn the_lock_is_held_only_while_a_daemon_keeps_it() {
        let home = tempfile::tempdir().expect("temp dir");
        released(home.path());

        let running = acquired(home.path());

        assert!(is_held(home.path()).expect("checks"));
        drop(running);
        released(home.path());
    }
}
