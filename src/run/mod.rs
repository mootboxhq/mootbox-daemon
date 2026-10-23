use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::api::MootboxClient;
use crate::config::{self, DaemonConfig, WorkspaceProfile};
use crate::contracts::ClaimResponseClaimedRun;
use crate::git::{self, WorktreeRequest};
use crate::journal::Journal;
use crate::private_files;
use crate::providers::{ProviderKind, ProviderOptions, SelectedProvider};

pub mod changes;
mod lease;
pub mod outcome;
pub mod process;
pub mod prompt;
mod report;
mod session;
mod supervise;

use lease::{JournalRecord, Lease, LeaseKeeper};
use session::SessionFile;
use supervise::ProviderRun;

pub const DEFAULT_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(15);
pub const DEFAULT_LEASE_SAFETY: Duration = Duration::from_secs(15);
pub const DEFAULT_WALL_TIME: Duration = wall_time_of(config::DEFAULT_WALL_TIME_MINUTES);
pub const DEFAULT_CANCEL_GRACE: Duration = Duration::from_secs(10);

const SECONDS_PER_MINUTE: u64 = 60;
const RUNS_DIRECTORY: &str = "runs";
const WORKTREES_DIRECTORY: &str = "worktrees";
const MANIFEST_FILE: &str = "manifest.json";
const REPOSITORY_NOT_BOUND: &str = "repository_not_bound_locally";
const WORKTREE_FAILED: &str = "worktree_failed";
const MANIFEST_FAILED: &str = "manifest_failed";

#[derive(Clone, Copy, Debug)]
pub struct RunTiming {
    pub heartbeat_interval: Duration,
    pub lease_safety: Duration,
    pub wall_time: Duration,
    pub cancel_grace: Duration,
}

pub const fn wall_time_of(minutes: u32) -> Duration {
    Duration::from_secs(minutes as u64 * SECONDS_PER_MINUTE)
}

impl Default for RunTiming {
    fn default() -> Self {
        Self {
            heartbeat_interval: DEFAULT_HEARTBEAT_INTERVAL,
            lease_safety: DEFAULT_LEASE_SAFETY,
            wall_time: DEFAULT_WALL_TIME,
            cancel_grace: DEFAULT_CANCEL_GRACE,
        }
    }
}

pub struct RunContext {
    pub client: MootboxClient,
    pub selected_provider: SelectedProvider,
    pub provider_options: ProviderOptions,
    pub profile: WorkspaceProfile,
    pub config: DaemonConfig,
    pub home: PathBuf,
    pub timing: RunTiming,
    pub journal: Arc<Journal>,
    pub stop_runs: CancellationToken,
}

pub async fn execute_claimed(context: &RunContext, run: ClaimResponseClaimedRun) {
    info!(run_id = %run.run_id, chat_id = %run.chat_id, branch = %run.branch, "claimed run");
    let lease = match Lease::new(run.run_id, run.generation, run.lease_expires_at) {
        Ok(lease) => lease,
        Err(error) => return tracing::warn!(%error, "skipping a claimed run"),
    };
    let Some(connection) = context.config.connections.get(&run.connection_id).filter(|connection| {
        connection.workspace_id == context.profile.workspace_id && connection.repository_id == run.repository_id
    }) else {
        return report::complete_failed(context, &lease, &anyhow!(REPOSITORY_NOT_BOUND)).await;
    };
    let provider: ProviderKind = match run.provider.to_string().parse() {
        Ok(provider) => provider,
        Err(error) => return report::complete_failed(context, &lease, &anyhow!(error)).await,
    };
    let session_file = SessionFile::new(&context.home, run.manifest.session_key);
    let local_session = match session_file.load(run.manifest.resume) {
        Ok(session) => session,
        Err(error) => return report::complete_failed(context, &lease, &error).await,
    };
    let worktree = match claimed_workspace(context, &lease, &run, &connection.path).await {
        Ok(worktree) => worktree,
        Err(failure) => return report::complete_failed(context, &lease, &failure).await,
    };
    let prompt = match prompt::execute_prompt(&run.manifest) {
        Ok(prompt) => prompt,
        Err(error) => return report::complete_failed(context, &lease, &anyhow!(error)).await,
    };
    let start = changes::snapshot_start(&worktree).await;
    if let Some(start) = &start {
        report::remember_start(context, lease.run_id, start).await;
    }
    let mut keeper = LeaseKeeper::new(context, &lease);
    let provider_run = ProviderRun {
        provider,
        working_directory: &worktree,
        prompt,
        run_token: &run.run_token,
        resume_session_id: local_session.as_deref(),
    };
    let supervised = match supervise::supervise(&mut keeper, provider_run).await {
        Ok(supervised) => supervised,
        Err(error) => return report::complete(context, &lease, report::spawn_failure(&lease, &error)).await,
    };
    if let Some(session_id) = supervised.session_id()
        && let Err(error) = session_file.save(session_id)
    {
        return report::complete_failed(context, &lease, &error).await;
    }
    if supervised.has_lost_lease() {
        return report::forget_lost_lease(context, &lease, supervised.stop_reason).await;
    }
    let checkpoint = git::capture_checkpoint(&worktree, &run.manifest.target.base_ref)
        .await
        .ok();
    if let Some(checkpoint) = &checkpoint {
        report::record_checkpoint(context, &lease, checkpoint).await;
    }
    let file_changes = match &start {
        Some(start) => changes::capture(&worktree, start).await,
        None => None,
    };
    let worktree_result = report::WorktreeResult {
        checkpoint,
        changes: file_changes,
    };
    let request = report::completion_request(&lease, &supervised, worktree_result);
    report::complete(context, &lease, request).await;
}

async fn claimed_workspace(
    context: &RunContext,
    lease: &Lease,
    run: &ClaimResponseClaimedRun,
    repository: &Path,
) -> Result<PathBuf> {
    let worktrees = context.home.join(WORKTREES_DIRECTORY);
    let request = WorktreeRequest {
        repository,
        worktrees: &worktrees,
        branch: &run.branch,
        base_ref: &run.manifest.target.base_ref,
    };
    let worktree = git::worktree_path(&request).context(WORKTREE_FAILED)?;
    let record = JournalRecord {
        workspace_id: context.profile.workspace_id,
        branch: &run.branch,
        worktree: &worktree,
    };
    report::remember(context, lease.journal_entry(record)).await;
    git::open_branch_worktree(&request).await.context(WORKTREE_FAILED)?;
    write_manifest(&context.home, run).context(MANIFEST_FAILED)?;
    Ok(worktree)
}

fn write_manifest(home: &Path, run: &ClaimResponseClaimedRun) -> Result<()> {
    let directory = home.join(RUNS_DIRECTORY).join(run.run_id.to_string());
    private_files::create_private_directory(&directory)?;
    private_files::write_owner_only(
        &directory.join(MANIFEST_FILE),
        &serde_json::to_vec_pretty(&run.manifest)?,
    )
}
