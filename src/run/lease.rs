use std::num::NonZeroU64;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::{DateTime, TimeDelta, Utc};
use tokio::time::{Interval, MissedTickBehavior};
use tracing::{info, warn};
use uuid::Uuid;

use super::RunContext;
use crate::contracts::{CompleteRunRequestOutcome, HeartbeatRequest, HeartbeatRequestState};
use crate::journal::JournalEntry;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StopReason {
    CancelRequested,
    LeaseLost,
    LeaseUnrenewable,
    WallTimeExceeded,
    RunnerShutdown,
}

impl StopReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CancelRequested => "cancel_requested",
            Self::LeaseLost => "lease_lost",
            Self::LeaseUnrenewable => "lease_unrenewable",
            Self::WallTimeExceeded => "wall_time_exceeded",
            Self::RunnerShutdown => "runner_shutdown",
        }
    }

    pub fn is_lease_ending(self) -> bool {
        matches!(self, Self::LeaseLost | Self::LeaseUnrenewable)
    }

    pub fn outcome(self) -> CompleteRunRequestOutcome {
        match self {
            Self::CancelRequested => CompleteRunRequestOutcome::Cancelled,
            _ => CompleteRunRequestOutcome::Failed,
        }
    }
}

pub struct Lease {
    pub run_id: Uuid,
    pub generation: NonZeroU64,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Copy)]
pub struct JournalRecord<'a> {
    pub workspace_id: Uuid,
    pub branch: &'a str,
    pub worktree: &'a Path,
}

impl Lease {
    pub fn new(run_id: Uuid, generation: i64, expires_at: DateTime<Utc>) -> Result<Self> {
        let generation = u64::try_from(generation)
            .ok()
            .and_then(NonZeroU64::new)
            .with_context(|| format!("run {run_id} has an invalid generation {generation}"))?;
        Ok(Self {
            run_id,
            generation,
            expires_at,
        })
    }

    pub fn journal_entry(&self, record: JournalRecord<'_>) -> JournalEntry {
        JournalEntry {
            run_id: self.run_id,
            generation: self.generation,
            branch: record.branch.to_string(),
            worktree: record.worktree.to_path_buf(),
            started_at: Utc::now(),
            workspace_id: Some(record.workspace_id),
        }
    }
}

pub struct LeaseKeeper<'a> {
    pub context: &'a RunContext,
    pub lease: &'a Lease,
    renew_by: DateTime<Utc>,
    heartbeats: Interval,
    pending_state: Option<HeartbeatRequestState>,
    is_reassessment_logged: bool,
}

impl<'a> LeaseKeeper<'a> {
    pub fn new(context: &'a RunContext, lease: &'a Lease) -> Self {
        let mut heartbeats = tokio::time::interval(context.timing.heartbeat_interval);
        heartbeats.set_missed_tick_behavior(MissedTickBehavior::Delay);
        Self {
            context,
            lease,
            renew_by: lease.expires_at - safety_margin(context),
            heartbeats,
            pending_state: None,
            is_reassessment_logged: false,
        }
    }

    pub fn announce_executing(&mut self) {
        self.pending_state = Some(HeartbeatRequestState::Executing);
        self.heartbeats.reset_immediately();
    }

    pub async fn heartbeat(&mut self) -> Option<StopReason> {
        self.heartbeats.tick().await;
        let state = self.pending_state.take();
        self.renew(state).await
    }

    async fn renew(&mut self, state: Option<HeartbeatRequestState>) -> Option<StopReason> {
        let request = HeartbeatRequest {
            generation: self.lease.generation,
            state,
        };
        match self.context.client.heartbeat(self.lease.run_id, &request).await {
            Ok(response) if response.cancel_requested => return Some(StopReason::CancelRequested),
            Ok(response) => {
                self.renew_by = response.lease_expires_at - safety_margin(self.context);
                self.log_reassessment(response.reassessment_requested);
            }
            Err(error) if error.is_lease_lost() => return Some(StopReason::LeaseLost),
            Err(error) => warn!(run_id = %self.lease.run_id, %error, "heartbeat failed"),
        }
        (Utc::now() > self.renew_by).then_some(StopReason::LeaseUnrenewable)
    }

    fn log_reassessment(&mut self, is_requested: bool) {
        if !is_requested || self.is_reassessment_logged {
            return;
        }
        self.is_reassessment_logged = true;
        info!(run_id = %self.lease.run_id, "an input changed; the provider sees the stale marker in its context");
    }
}

fn safety_margin(context: &RunContext) -> TimeDelta {
    TimeDelta::from_std(context.timing.lease_safety).unwrap_or(TimeDelta::MAX)
}
