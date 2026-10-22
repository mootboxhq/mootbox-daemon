use std::fs::{self, DirBuilder, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

pub const OWNER_ONLY_FILE: u32 = 0o600;
pub const OWNER_ONLY_DIRECTORY: u32 = 0o700;

const TEMPORARY_SUFFIX: &str = ".tmp";

pub fn create_private_directory(path: &Path) -> Result<()> {
    DirBuilder::new()
        .recursive(true)
        .mode(OWNER_ONLY_DIRECTORY)
        .create(path)
        .with_context(|| format!("cannot create {}", path.display()))
}

pub fn replace_owner_only(path: &Path, contents: &[u8]) -> Result<()> {
    let temporary = temporary_path(path);
    write_owner_only(&temporary, contents)?;
    fs::rename(&temporary, path).with_context(|| format!("cannot replace {}", path.display()))
}

pub fn write_owner_only(path: &Path, contents: &[u8]) -> Result<()> {
    let _ = fs::remove_file(path);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(OWNER_ONLY_FILE)
        .open(path)
        .with_context(|| format!("cannot create {}", path.display()))?;
    file.write_all(contents)?;
    file.sync_all()?;
    Ok(())
}

pub fn temporary_path(path: &Path) -> PathBuf {
    let mut temporary = path.as_os_str().to_owned();
    temporary.push(TEMPORARY_SUFFIX);
    PathBuf::from(temporary)
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    fn mode_of(path: &Path) -> u32 {
        fs::metadata(path).expect("path exists").permissions().mode() & 0o777
    }

    #[test]
    fn replacing_goes_through_a_temporary_and_leaves_an_owner_only_file() {
        let directory = tempfile::tempdir().expect("temp dir");
        let path = directory.path().join("state.json");
        fs::write(temporary_path(&path), "stale partial write").expect("writes");
        fs::write(&path, "old").expect("writes");

        replace_owner_only(&path, b"new").expect("replaces");

        assert!(!temporary_path(&path).exists());
        assert_eq!(fs::read_to_string(&path).expect("reads"), "new");
        assert_eq!(mode_of(&path), OWNER_ONLY_FILE);
    }

    #[test]
    fn private_directories_are_owner_only() {
        let parent = tempfile::tempdir().expect("temp dir");
        let directory = parent.path().join("a").join("b");

        create_private_directory(&directory).expect("creates");

        assert_eq!(mode_of(&directory), OWNER_ONLY_DIRECTORY);
    }
}
