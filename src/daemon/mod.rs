use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use anyhow::Result;
use tokio::sync::{Semaphore, mpsc};
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::api::{ApiError, MootboxClient};
use crate::config::{self, DaemonConfig, WorkspaceProfile};
use crate::journal::Journal;
use crate::providers::{ProviderOptions, SelectedProvider};
use crate::run::{RunContext, RunTiming};

mod config_watcher;
mod reconcile;
mod slots;

use config_watcher::ConfigWatcher;
use slots::Slots;

pub const DEFAULT_CLAIM_WAIT: Duration = Duration::from_secs(25);
pub const DEFAULT_BACKOFF_BASE: Duration = Duration::from_secs(1);
pub const DEFAULT_BACKOFF_MAX: Duration = Duration::from_secs(60);
pub const DEFAULT_RECONCILE_WAIT: Duration = Duration::from_secs(15);
pub const DEFAULT_PROTOCOL_RECHECK: Duration = Duration::from_mins(10);
pub const DEFAULT_SHUTDOWN_GRACE: Duration = Duration::from_secs(config::DEFAULT_SHUTDOWN_GRACE_SECONDS);

#[derive(Clone, Copy, Debug)]
pub struct DaemonTiming {
    pub claim_wait: Duration,
    pub backoff_base: Duration,
    pub backoff_max: Duration,
    pub reconcile_wait: Duration,
    pub protocol_recheck: Duration,
    pub shutdown_grace: Duration,
}

impl Default for DaemonTiming {
    fn default() -> Self {
        Self {
            claim_wait: DEFAULT_CLAIM_WAIT,
            backoff_base: DEFAULT_BACKOFF_BASE,
            backoff_max: DEFAULT_BACKOFF_MAX,
            reconcile_wait: DEFAULT_RECONCILE_WAIT,
            protocol_recheck: DEFAULT_PROTOCOL_RECHECK,
            shutdown_grace: DEFAULT_SHUTDOWN_GRACE,
        }
    }
}

pub struct DaemonOptions {
    pub config: DaemonConfig,
    pub home: PathBuf,
    pub selected_provider: SelectedProvider,
    pub provider_options: ProviderOptions,
    pub slots: u32,
    pub timing: DaemonTiming,
    pub run_timing: RunTiming,
    pub journal: Journal,
    pub shutdown: CancellationToken,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct WorkspaceKey {
    mootbox_url: String,
    workspace_id: Uuid,
}

impl From<&WorkspaceProfile> for WorkspaceKey {
    fn from(profile: &WorkspaceProfile) -> Self {
        Self {
            mootbox_url: profile.mootbox_url.clone(),
            workspace_id: profile.workspace_id,
        }
    }
}

struct RunSettings {
    selected_provider: SelectedProvider,
    provider_options: ProviderOptions,
    home: PathBuf,
    timing: RunTiming,
    journal: Arc<Journal>,
    stop_runs: CancellationToken,
}

impl RunSettings {
    fn context_for(&self, profile: &WorkspaceProfile, config: &DaemonConfig) -> Result<RunContext> {
        Ok(RunContext {
            client: MootboxClient::for_profile(profile)?,
            selected_provider: self.selected_provider.clone(),
            provider_options: self.provider_options.clone(),
            profile: profile.clone(),
            config: config.clone(),
            home: self.home.clone(),
            timing: self.timing,
            journal: Arc::clone(&self.journal),
            stop_runs: self.stop_runs.clone(),
        })
    }
}

struct Daemon {
    permits: Arc<Semaphore>,
    slot_count: u32,
    timing: DaemonTiming,
    run_settings: RunSettings,
    watcher: Mutex<ConfigWatcher>,
    workspaces: Mutex<BTreeMap<WorkspaceKey, Arc<Slots>>>,
    started: mpsc::UnboundedSender<Arc<Slots>>,
    shutdown: CancellationToken,
}

impl Daemon {
    fn refresh(&self) {
        let changed = locked(&self.watcher).changed();
        if let Some(config) = changed {
            info!(workspaces = config.workspaces.len(), "daemon config changed; reloading");
            self.apply(&config);
        }
    }

    fn apply(&self, config: &DaemonConfig) {
        let mut workspaces = locked(&self.workspaces);
        for profile in &config.workspaces {
            let context = match self.run_settings.context_for(profile, config) {
                Ok(context) => Arc::new(context),
                Err(failure) => {
                    warn!(workspace_id = %profile.workspace_id, error = format!("{failure:#}"), "cannot serve this workspace");
                    continue;
                }
            };
            match workspaces.get(&WorkspaceKey::from(profile)) {
                Some(slots) => slots.replace_context(context),
                None => self.start_workspace(&mut workspaces, profile, context),
            }
        }
        let wanted: BTreeSet<WorkspaceKey> = config.workspaces.iter().map(WorkspaceKey::from).collect();
        workspaces.retain(|key, slots| {
            let is_wanted = wanted.contains(key);
            if !is_wanted {
                info!(workspace_id = %key.workspace_id, "workspace removed; finishing its active runs");
                slots.stop_claiming.cancel();
            }
            is_wanted
        });
    }

    fn start_workspace(
        &self,
        workspaces: &mut BTreeMap<WorkspaceKey, Arc<Slots>>,
        profile: &WorkspaceProfile,
        context: Arc<RunContext>,
    ) {
        let slots = Arc::new(Slots::new(context, self.timing, self.shutdown.child_token()));
        workspaces.insert(WorkspaceKey::from(profile), Arc::clone(&slots));
        info!(workspace_id = %profile.workspace_id, runner_id = %profile.runner_id, "claiming for workspace");
        if self.started.send(slots).is_err() {
            warn!(workspace_id = %profile.workspace_id, "the daemon is stopping; not claiming for this workspace");
        }
    }
}

pub async fn run_daemon(options: DaemonOptions) {
    let stop_runs = CancellationToken::new();
    let (started, mut starting) = mpsc::unbounded_channel();
    let daemon = Arc::new(Daemon {
        permits: Arc::new(Semaphore::new(usize::try_from(options.slots).unwrap_or(usize::MAX))),
        slot_count: options.slots,
        timing: options.timing,
        run_settings: RunSettings {
            selected_provider: options.selected_provider,
            provider_options: options.provider_options,
            home: options.home.clone(),
            timing: options.run_timing,
            journal: Arc::new(options.journal),
            stop_runs: stop_runs.clone(),
        },
        watcher: Mutex::new(ConfigWatcher::new(&options.home)),
        workspaces: Mutex::new(BTreeMap::new()),
        started,
        shutdown: options.shutdown.clone(),
    });
    let grace = tokio::spawn(cancel_runs_after_grace(
        options.shutdown,
        stop_runs,
        options.timing.shutdown_grace,
    ));
    daemon.apply(&options.config);
    let mut running = JoinSet::new();
    loop {
        tokio::select! {
            biased;
            Some(slots) = starting.recv() => {
                running.spawn(serve_workspace(Arc::clone(&daemon), slots));
            }
            joined = running.join_next() => match joined {
                None => break,
                Some(Err(failure)) => error!(%failure, "a workspace's claim slots crashed"),
                Some(Ok(())) => {}
            },
        }
    }
    grace.abort();
}

fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

async fn serve_workspace(daemon: Arc<Daemon>, slots: Arc<Slots>) {
    slots.reconcile().await;
    let mut claiming = JoinSet::new();
    for _ in 0..daemon.slot_count {
        claiming.spawn(run_slot(Arc::clone(&daemon), Arc::clone(&slots)));
    }
    while let Some(joined) = claiming.join_next().await {
        if let Err(failure) = joined {
            error!(%failure, "a claim slot crashed");
        }
    }
}

async fn run_slot(daemon: Arc<Daemon>, slots: Arc<Slots>) {
    let mut failures = 0;
    while !slots.stop_claiming.is_cancelled() {
        let permit = tokio::select! {
            () = slots.stop_claiming.cancelled() => break,
            permit = Arc::clone(&daemon.permits).acquire_owned() => match permit {
                Ok(permit) => permit,
                Err(_) => break,
            },
        };
        let request = slots.claim_request().await;
        let client_context = slots.context();
        let claimed = tokio::select! {
            () = slots.stop_claiming.cancelled() => break,
            claimed = client_context.client.claim(&request) => claimed,
        };
        daemon.refresh();
        match claimed {
            Ok(response) => {
                failures = 0;
                slots.handle(response).await;
            }
            Err(refusal @ ApiError::UpgradeRequired { .. }) => {
                failures = 0;
                drop(permit);
                slots.resend_device_report();
                slots.wait_until_protocol_accepted(&refusal).await;
            }
            Err(failure) => {
                failures += 1;
                drop(permit);
                slots.resend_device_report();
                let wait = backoff_with_jitter(failures, &slots.timing);
                warn!(error = %failure, retry_in_ms = wait.as_millis(), "claim failed");
                slots.pause(wait).await;
            }
        }
    }
}

async fn cancel_runs_after_grace(stop_claiming: CancellationToken, stop_runs: CancellationToken, grace: Duration) {
    stop_claiming.cancelled().await;
    info!(
        grace_ms = grace.as_millis(),
        "no longer claiming; waiting for active runs"
    );
    tokio::time::sleep(grace).await;
    stop_runs.cancel();
}

pub fn backoff_with_jitter(attempt: u32, timing: &DaemonTiming) -> Duration {
    let growth = 2_u32.saturating_pow(attempt.saturating_sub(1));
    let ceiling = timing.backoff_base.saturating_mul(growth).min(timing.backoff_max);
    ceiling / 2 + ceiling.mul_f64(fastrand::f64() / 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_doubles_up_to_the_maximum_and_keeps_at_least_half() {
        let timing = DaemonTiming::default();
        for (attempt, ceiling) in [(1, 1), (2, 2), (3, 4), (7, 60), (40, 60)] {
            let ceiling = Duration::from_secs(ceiling);

            let wait = backoff_with_jitter(attempt, &timing);

            assert!(wait >= ceiling / 2 && wait <= ceiling, "attempt {attempt}: {wait:?}");
        }
    }
}
