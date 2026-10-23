use std::collections::BTreeSet;
use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
use uuid::Uuid;

use super::{DaemonTiming, locked, reconcile};
use crate::api::ApiError;
use crate::contracts::{ClaimRequest, ClaimRequestDevice, ClaimRequestProvidersItem, ClaimResponse};
use crate::device::{self, DeviceReporter, SignIn};
use crate::run::{self, RunContext};

const SLOT_CAPACITY: i64 = 1;

pub struct Slots {
    context: Mutex<Arc<RunContext>>,
    pub timing: DaemonTiming,
    pub stop_claiming: CancellationToken,
    active_run_ids: Mutex<BTreeSet<Uuid>>,
    protocol_wait: tokio::sync::Mutex<()>,
    protocol_acceptances: AtomicU64,
    device_reporter: Mutex<DeviceReporter>,
}

impl Slots {
    pub fn new(context: Arc<RunContext>, timing: DaemonTiming, stop_claiming: CancellationToken) -> Self {
        Self {
            context: Mutex::new(context),
            timing,
            stop_claiming,
            active_run_ids: Mutex::new(BTreeSet::new()),
            protocol_wait: tokio::sync::Mutex::new(()),
            protocol_acceptances: AtomicU64::new(0),
            device_reporter: Mutex::new(DeviceReporter::default()),
        }
    }

    pub fn context(&self) -> Arc<RunContext> {
        Arc::clone(&locked(&self.context))
    }

    pub fn replace_context(&self, context: Arc<RunContext>) {
        *locked(&self.context) = context;
    }

    pub async fn claim_request(&self) -> ClaimRequest {
        let context = self.context();
        let sign_ins = device::check_sign_ins(|kind| context.selected_provider.program_for(kind)).await;
        ClaimRequest {
            active_run_ids: self.active_runs().into_iter().collect(),
            providers: signed_in_providers(&sign_ins),
            capacity: SLOT_CAPACITY,
            wait_ms: i64::try_from(self.timing.claim_wait.as_millis()).unwrap_or(i64::MAX),
            device: self.due_device_report(&sign_ins).await,
        }
    }

    async fn due_device_report(&self, sign_ins: &[SignIn]) -> Option<ClaimRequestDevice> {
        let now = tokio::time::Instant::now();
        let cached = locked(&self.device_reporter).fresh_inventory(now);
        let inventory = if let Some(inventory) = cached {
            inventory
        } else {
            let context = self.context();
            device::take_inventory(|kind| context.selected_provider.program_for(kind)).await
        };
        let report = locked(&self.device_reporter).due(now, inventory, sign_ins)?;
        match report.contract() {
            Ok(contract) => Some(contract),
            Err(error) => {
                warn!(error = %error, "this device's report does not fit the runner protocol; not sending it");
                None
            }
        }
    }

    pub fn resend_device_report(&self) {
        locked(&self.device_reporter).forget();
    }

    fn active_runs(&self) -> BTreeSet<Uuid> {
        locked(&self.active_run_ids).clone()
    }

    pub async fn handle(&self, response: ClaimResponse) {
        let context = self.context();
        match response {
            ClaimResponse::Idle => {}
            ClaimResponse::Claimed { run } => {
                self.track(run.run_id, run::execute_claimed(&context, *run)).await;
            }
            ClaimResponse::ReconcileRequired { run_ids, .. } if self.names_only_active_runs(&run_ids) => {}
            ClaimResponse::ReconcileRequired { run_ids, runs } => {
                info!(?run_ids, "server reports unfinished runs; reconciling");
                self.reconcile().await;
                for run in runs {
                    if !run_ids.contains(&run.run_id)
                        || self.active_runs().contains(&run.run_id)
                        || context.journal.entry(run.run_id).await.is_some()
                    {
                        continue;
                    }
                    reconcile::fail_unreceived_claim(&context, &run).await;
                }
                self.pause(self.timing.reconcile_wait).await;
            }
        }
    }

    fn names_only_active_runs(&self, run_ids: &[Uuid]) -> bool {
        let active = self.active_runs();
        !run_ids.is_empty() && run_ids.iter().all(|run_id| active.contains(run_id))
    }

    pub async fn reconcile(&self) {
        if let Err(error) = reconcile::reconcile_journal(&self.context(), &self.active_runs()).await {
            warn!(%error, "could not reconcile unfinished runs; retrying when Mootbox asks");
        }
    }

    async fn track(&self, run_id: Uuid, work: impl Future<Output = ()>) {
        locked(&self.active_run_ids).insert(run_id);
        work.await;
        locked(&self.active_run_ids).remove(&run_id);
    }

    pub async fn wait_until_protocol_accepted(&self, refusal: &ApiError) {
        let acceptances_seen = self.protocol_acceptances.load(Ordering::SeqCst);
        let _waiting = self.protocol_wait.lock().await;
        let is_settled = self.protocol_acceptances.load(Ordering::SeqCst) != acceptances_seen;
        if is_settled || self.stop_claiming.is_cancelled() {
            return;
        }
        error!(
            error = %refusal,
            recheck_in_s = self.timing.protocol_recheck.as_secs(),
            "claiming paused until Mootbox accepts this daemon's runner protocol"
        );
        loop {
            self.pause(self.timing.protocol_recheck).await;
            if self.stop_claiming.is_cancelled() {
                return;
            }
            match self.context().client.health().await {
                Ok(_) => {
                    info!("Mootbox accepts this daemon's runner protocol again; claiming resumed");
                    self.protocol_acceptances.fetch_add(1, Ordering::SeqCst);
                    return;
                }
                Err(ApiError::UpgradeRequired { .. }) => {}
                Err(failure) => warn!(error = %failure, "could not re-check the runner protocol"),
            }
        }
    }

    pub async fn pause(&self, duration: Duration) {
        tokio::select! {
            () = tokio::time::sleep(duration) => {}
            () = self.stop_claiming.cancelled() => {}
        }
    }
}

fn signed_in_providers(sign_ins: &[SignIn]) -> Vec<ClaimRequestProvidersItem> {
    sign_ins
        .iter()
        .filter(|sign_in| sign_in.signed_in == Some(true))
        .filter_map(|sign_in| sign_in.kind.contract().ok())
        .collect()
}
