use std::io::IsTerminal;
use std::path::Path;
use std::sync::Mutex;

use anyhow::Result;

use crate::log_file;

pub fn init(home: &Path) -> Result<()> {
    let logs = tracing_subscriber::fmt();
    if std::io::stderr().is_terminal() {
        logs.with_writer(std::io::stderr).init();
        return Ok(());
    }
    let daemon_log = log_file::open_daemon_log(home)?;
    logs.json().with_writer(Mutex::new(daemon_log)).init();
    Ok(())
}
