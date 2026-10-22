use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::private_files;

pub const LOGS_DIRECTORY: &str = "logs";
pub const LOG_FILE: &str = "daemon.log";
pub const MAX_LOG_BYTES: u64 = 10 * 1024 * 1024;
pub const KEPT_ROTATED_FILES: usize = 3;

pub struct RotatingFile {
    path: PathBuf,
    max_bytes: u64,
    kept_rotated_files: usize,
    file: File,
    size: u64,
}

pub fn logs_directory(home: &Path) -> PathBuf {
    home.join(LOGS_DIRECTORY)
}

pub fn open_daemon_log(home: &Path) -> Result<RotatingFile> {
    let directory = logs_directory(home);
    private_files::create_private_directory(&directory)?;
    let path = directory.join(LOG_FILE);
    RotatingFile::open(path.clone(), MAX_LOG_BYTES, KEPT_ROTATED_FILES)
        .with_context(|| format!("cannot open {}", path.display()))
}

pub fn rotated_path(path: &Path, index: usize) -> PathBuf {
    let mut rotated = path.as_os_str().to_owned();
    rotated.push(format!(".{index}"));
    PathBuf::from(rotated)
}

impl RotatingFile {
    pub fn open(path: PathBuf, max_bytes: u64, kept_rotated_files: usize) -> io::Result<Self> {
        let file = open_for_append(&path)?;
        let size = file.metadata()?.len();
        Ok(Self {
            path,
            max_bytes,
            kept_rotated_files,
            file,
            size,
        })
    }

    fn rotate(&mut self) -> io::Result<()> {
        for index in (1..self.kept_rotated_files).rev() {
            rename_if_present(&rotated_path(&self.path, index), &rotated_path(&self.path, index + 1))?;
        }
        fs::rename(&self.path, rotated_path(&self.path, 1))?;
        self.file = open_for_append(&self.path)?;
        self.size = 0;
        Ok(())
    }

    fn is_full_before(&self, incoming: usize) -> bool {
        self.size > 0 && self.size + incoming as u64 > self.max_bytes
    }
}

impl Write for RotatingFile {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.is_full_before(bytes.len()) {
            self.rotate()?;
        }
        let written = self.file.write(bytes)?;
        self.size += written as u64;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

fn open_for_append(path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .create(true)
        .append(true)
        .mode(private_files::OWNER_ONLY_FILE)
        .open(path)
}

fn rename_if_present(from: &Path, to: &Path) -> io::Result<()> {
    match fs::rename(from, to) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        result => result,
    }
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    const LINE: &[u8] = b"0123456789\n";
    const LIMIT: u64 = 33;
    const KEPT: usize = 2;

    fn read(path: &Path) -> String {
        fs::read_to_string(path).unwrap_or_default()
    }

    #[test]
    fn the_log_rolls_over_once_the_next_write_would_pass_the_limit() {
        let directory = tempfile::tempdir().expect("temp dir");
        let path = directory.path().join(LOG_FILE);
        let mut log = RotatingFile::open(path.clone(), LIMIT, KEPT).expect("opens");

        for _ in 0..3 {
            log.write_all(LINE).expect("writes");
        }
        assert_eq!(read(&path).len(), 33);
        assert!(!rotated_path(&path, 1).exists());

        log.write_all(LINE).expect("writes");

        assert_eq!(read(&path).len(), LINE.len());
        assert_eq!(read(&rotated_path(&path, 1)).len(), 33);
    }

    #[test]
    fn only_the_kept_number_of_rotated_files_survive_with_the_newest_first() {
        let directory = tempfile::tempdir().expect("temp dir");
        let path = directory.path().join(LOG_FILE);
        let mut log = RotatingFile::open(path.clone(), LIMIT, KEPT).expect("opens");

        for line in 0..13 {
            writeln!(log, "line {line:04}").expect("writes");
        }

        assert_eq!(read(&path), "line 0012\n");
        assert_eq!(read(&rotated_path(&path, 1)), "line 0009\nline 0010\nline 0011\n");
        assert_eq!(read(&rotated_path(&path, 2)), "line 0006\nline 0007\nline 0008\n");
        assert!(!rotated_path(&path, 3).exists());
    }

    #[test]
    fn reopening_counts_what_the_file_already_holds() {
        let directory = tempfile::tempdir().expect("temp dir");
        let path = directory.path().join(LOG_FILE);
        fs::write(&path, [b'x'; 30]).expect("writes");
        let mut log = RotatingFile::open(path.clone(), LIMIT, KEPT).expect("opens");

        log.write_all(LINE).expect("writes");

        assert_eq!(read(&path).len(), LINE.len());
        assert_eq!(read(&rotated_path(&path, 1)).len(), 30);
    }

    #[test]
    fn the_daemon_log_is_owner_only_in_the_logs_directory() {
        let home = tempfile::tempdir().expect("temp dir");
        let mut log = open_daemon_log(home.path()).expect("opens");
        log.write_all(LINE).expect("writes");

        let path = logs_directory(home.path()).join(LOG_FILE);
        let mode = fs::metadata(path).expect("exists").permissions().mode() & 0o777;
        assert_eq!(mode, private_files::OWNER_ONLY_FILE);
    }
}
