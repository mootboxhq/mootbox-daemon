use std::collections::VecDeque;
use std::io;
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::process::{ExitStatus, Stdio};
use std::time::Duration;

use nix::errno::Errno;
use nix::sys::signal::{Signal, killpg};
use nix::unistd::Pid;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::task::JoinHandle;
use tokio::time::Instant;

use crate::providers::{ProviderKind, ProviderResult};

const STDERR_TAIL_LINES: usize = 50;
const GROUP_POLL_INTERVAL: Duration = Duration::from_millis(20);
const OUTPUT_DRAIN_TIMEOUT: Duration = Duration::from_secs(2);
const STOP_SIGNALS: [Signal; 2] = [Signal::SIGINT, Signal::SIGTERM];

pub struct ProviderLaunch<'a> {
    pub provider: ProviderKind,
    pub program: &'a Path,
    pub working_directory: &'a Path,
    pub arguments: Vec<String>,
    pub prompt: String,
    pub environment: Vec<(String, String)>,
}

#[derive(Debug)]
pub struct ProviderReport {
    pub status: ExitStatus,
    pub result: Option<ProviderResult>,
    pub stderr_tail: Vec<String>,
}

impl ProviderReport {
    pub fn is_success(&self) -> bool {
        self.status.success() && !self.result.as_ref().is_some_and(|result| result.is_error)
    }

    pub fn signal_name(&self) -> Option<&'static str> {
        let number = self.status.signal()?;
        Signal::try_from(number).ok().map(Signal::as_str)
    }
}

pub struct RunningProvider {
    child: Child,
    group: Pid,
    result: JoinHandle<Option<ProviderResult>>,
    stderr_tail: JoinHandle<Vec<String>>,
}

impl RunningProvider {
    pub fn spawn(launch: ProviderLaunch<'_>) -> io::Result<Self> {
        let mut command = Command::new(launch.program);
        command
            .args(&launch.arguments)
            .envs(launch.environment)
            .current_dir(launch.working_directory)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0)
            .kill_on_drop(true);
        let mut child = command.spawn()?;
        let pid = child
            .id()
            .and_then(|id| i32::try_from(id).ok())
            .ok_or_else(|| io::Error::other("the provider exited before its process group could be recorded"))?;
        let mut stdin = child.stdin.take().ok_or_else(|| unpiped("stdin"))?;
        let prompt = launch.prompt;
        tokio::spawn(async move {
            let _ = stdin.write_all(prompt.as_bytes()).await;
        });
        let stdout = child.stdout.take().ok_or_else(|| unpiped("stdout"))?;
        let stderr = child.stderr.take().ok_or_else(|| unpiped("stderr"))?;
        Ok(Self {
            child,
            group: Pid::from_raw(pid),
            result: tokio::spawn(read_result(stdout, launch.provider)),
            stderr_tail: tokio::spawn(read_tail(stderr)),
        })
    }

    pub fn pid(&self) -> i32 {
        self.group.as_raw()
    }

    pub async fn wait(&mut self) -> io::Result<ExitStatus> {
        self.child.wait().await
    }

    pub async fn terminate(&mut self, grace: Duration) {
        for signal in STOP_SIGNALS {
            self.signal_group(signal);
            if self.group_ends_within(grace).await {
                return;
            }
        }
        self.signal_group(Signal::SIGKILL);
    }

    pub async fn finish(mut self) -> io::Result<ProviderReport> {
        let status = self.child.wait().await?;
        self.signal_group(Signal::SIGKILL);
        Ok(ProviderReport {
            status,
            result: drained(self.result).await.flatten(),
            stderr_tail: drained(self.stderr_tail).await.unwrap_or_default(),
        })
    }

    fn signal_group(&self, signal: Signal) {
        let _ = killpg(self.group, signal);
    }

    async fn group_ends_within(&mut self, grace: Duration) -> bool {
        let deadline = Instant::now() + grace;
        loop {
            if self.has_group_ended() {
                return true;
            }
            if Instant::now() >= deadline {
                return false;
            }
            tokio::time::sleep(GROUP_POLL_INTERVAL).await;
        }
    }

    fn has_group_ended(&mut self) -> bool {
        let has_leader_exited = matches!(self.child.try_wait(), Ok(Some(_)));
        has_leader_exited && killpg(self.group, None) == Err(Errno::ESRCH)
    }
}

fn unpiped(stream: &str) -> io::Error {
    io::Error::other(format!("the provider {stream} is not piped"))
}

async fn read_result(stdout: impl AsyncRead + Unpin, provider: ProviderKind) -> Option<ProviderResult> {
    let mut lines = BufReader::new(stdout).split(b'\n');
    let mut result = ProviderResult::default();
    let mut seen = false;
    while let Ok(Some(line)) = lines.next_segment().await {
        seen |= provider.fold_output(&mut result, &line);
    }
    seen.then_some(result)
}

async fn read_tail(stderr: impl AsyncRead + Unpin) -> Vec<String> {
    let mut lines = BufReader::new(stderr).split(b'\n');
    let mut tail = VecDeque::with_capacity(STDERR_TAIL_LINES);
    while let Ok(Some(line)) = lines.next_segment().await {
        if tail.len() == STDERR_TAIL_LINES {
            tail.pop_front();
        }
        tail.push_back(String::from_utf8_lossy(&line).into_owned());
    }
    tail.into()
}

async fn drained<T>(reader: JoinHandle<T>) -> Option<T> {
    tokio::time::timeout(OUTPUT_DRAIN_TIMEOUT, reader).await.ok()?.ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn report_of(provider: &str, output: &str) -> ProviderReport {
        let directory = tempfile::tempdir().expect("temp dir");
        let launch = ProviderLaunch {
            provider: provider.parse().expect("known provider"),
            program: Path::new("/bin/sh"),
            working_directory: directory.path(),
            arguments: vec!["-c".to_string(), format!("echo '{output}'")],
            prompt: String::new(),
            environment: Vec::new(),
        };
        RunningProvider::spawn(launch)
            .expect("spawns")
            .finish()
            .await
            .expect("finishes")
    }

    #[tokio::test]
    async fn the_output_is_read_the_way_the_launched_provider_writes_it() {
        let codex = report_of("codex", r#"{"type":"thread.started","thread_id":"t-1"}"#).await;
        let claude = report_of("claude-code", r#"{"type":"result","session_id":"s-1"}"#).await;

        assert_eq!(
            codex.result.and_then(|result| result.session_id).as_deref(),
            Some("t-1")
        );
        assert_eq!(
            claude.result.and_then(|result| result.session_id).as_deref(),
            Some("s-1")
        );
    }

    #[tokio::test]
    async fn output_the_provider_does_not_recognise_yields_no_result() {
        let report = report_of("claude-code", r#"{"type":"thread.started","thread_id":"t-1"}"#).await;

        assert!(report.status.success());
        assert_eq!(report.result, None);
    }
}
