use std::path::Path;

use anyhow::Result;
use tracing::{info, warn};

use super::lease::{Lease, LeaseKeeper, StopReason};
use super::outcome::{self, Attempt, ErrorCategory};
use super::process::{ProviderLaunch, ProviderReport, RunningProvider};
use crate::contracts::CompleteRunRequestOutcome;
use crate::providers::{self, LaunchPlan, LaunchRequest, ProviderKind};

const FRESH_SESSION: &str = "fresh";
const STDERR_LINES_LOGGED: usize = 10;

pub struct ProviderRun<'a> {
    pub provider: ProviderKind,
    pub working_directory: &'a Path,
    pub prompt: String,
    pub run_token: &'a str,
    pub resume_session_id: Option<&'a str>,
}

pub struct Supervised {
    pub report: ProviderReport,
    pub stop_reason: Option<StopReason>,
    pub error_category: Option<ErrorCategory>,
}

impl Supervised {
    pub fn has_lost_lease(&self) -> bool {
        self.stop_reason.is_some_and(StopReason::is_lease_ending)
    }

    pub fn session_id(&self) -> Option<&str> {
        self.report.result.as_ref()?.session_id.as_deref()
    }

    pub fn outcome(&self) -> CompleteRunRequestOutcome {
        match self.stop_reason {
            Some(reason) => reason.outcome(),
            None if self.report.is_success() && self.error_category.is_none() => CompleteRunRequestOutcome::Succeeded,
            None => CompleteRunRequestOutcome::Failed,
        }
    }

    pub fn error(&self) -> Option<&'static str> {
        if let Some(reason) = self.stop_reason {
            return Some(reason.as_str());
        }
        self.error_category.map(ErrorCategory::as_str)
    }
}

pub async fn supervise(keeper: &mut LeaseKeeper<'_>, run: ProviderRun<'_>) -> Result<Supervised> {
    let context = keeper.context;
    let run_id = keeper.lease.run_id;
    let program = context.selected_provider.program_for(run.provider);
    let mootbox_mcp_url = providers::mootbox_mcp_url(&context.profile.mootbox_url)?;
    let LaunchPlan {
        arguments,
        environment,
        owned_directory,
    } = run.provider.launch(&LaunchRequest {
        mootbox_mcp_url: &mootbox_mcp_url,
        run_token: run.run_token,
        allowed_commands: &context.config.allowed_commands,
        resume_session_id: run.resume_session_id,
        options: &context.provider_options,
    })?;
    let launch = ProviderLaunch {
        provider: run.provider,
        program: &program,
        working_directory: run.working_directory,
        arguments,
        prompt: run.prompt,
        environment,
    };
    let mut provider = RunningProvider::spawn(launch)?;
    let directory = run.working_directory.display().to_string();
    let session = run.resume_session_id.unwrap_or(FRESH_SESSION);
    info!(%run_id, pid = provider.pid(), %directory, session, "started provider");
    keeper.announce_executing();
    let activity = provider
        .activity
        .take()
        .ok_or_else(|| anyhow::anyhow!("activity stream unavailable"))?;
    let (report, stop_reason) = {
        let reporter = super::progress::upload(context, keeper.lease, activity);
        tokio::pin!(reporter);
        let work = async {
            let stop_reason = watch(keeper, &mut provider).await;
            if let Some(reason) = stop_reason {
                info!(%run_id, reason = reason.as_str(), "stopping run");
                keep_lease_while(provider.terminate(context.timing.cancel_grace), keeper).await;
            }
            let report = provider.finish().await?;
            Ok::<_, std::io::Error>((report, stop_reason))
        };
        tokio::pin!(work);
        tokio::select! {
            result = &mut work => {
                let _ = tokio::time::timeout(std::time::Duration::from_secs(2), &mut reporter).await;
                result?
            }
            () = &mut reporter => work.await?,
        }
    };
    drop(owned_directory);
    log_failed_provider(keeper.lease, &report);
    let attempt = Attempt {
        is_resumed: run.resume_session_id.is_some(),
        is_stop_requested: stop_reason.is_some(),
    };
    Ok(Supervised {
        error_category: outcome::classify(&report, attempt),
        report,
        stop_reason,
    })
}

async fn watch(keeper: &mut LeaseKeeper<'_>, provider: &mut RunningProvider) -> Option<StopReason> {
    let context = keeper.context;
    let wall_time = tokio::time::sleep(context.timing.wall_time);
    tokio::pin!(wall_time);
    loop {
        tokio::select! {
            _ = provider.wait() => return None,
            () = &mut wall_time => return Some(StopReason::WallTimeExceeded),
            () = context.stop_runs.cancelled() => return Some(StopReason::RunnerShutdown),
            reason = keeper.heartbeat() => {
                if reason.is_some() {
                    return reason;
                }
            }
        }
    }
}

async fn keep_lease_while(work: impl Future<Output = ()>, keeper: &mut LeaseKeeper<'_>) {
    tokio::pin!(work);
    loop {
        tokio::select! {
            () = &mut work => return,
            _ = keeper.heartbeat() => {}
        }
    }
}

fn log_failed_provider(lease: &Lease, report: &ProviderReport) {
    if report.is_success() {
        return;
    }
    let skipped = report.stderr_tail.len().saturating_sub(STDERR_LINES_LOGGED);
    let stderr = report.stderr_tail[skipped..].join("\n");
    warn!(run_id = %lease.run_id, status = %report.status, %stderr, "provider failed");
}
