use serde_json::{Map, Value, json};
use tracing::{info, warn};
use uuid::Uuid;

use super::RunContext;
use super::lease::{Lease, StopReason};
use super::outcome::ErrorCategory;
use super::supervise::Supervised;
use crate::contracts::{
    CompleteRunRequest, CompleteRunRequestError, CompleteRunRequestOutcome, RunEventRequest, RunEventRequestType,
};
use crate::git::Checkpoint;
use crate::journal::JournalEntry;

const PROCESS_FAILED: &str = "process_failed";
const MAX_ERROR_LENGTH: usize = 2000;
const MAX_FINAL_ANSWER_LENGTH: usize = 20_000;

pub fn completion_request(
    lease: &Lease,
    supervised: &Supervised,
    checkpoint: Option<&Checkpoint>,
) -> CompleteRunRequest {
    let report = &supervised.report;
    let result = report.result.as_ref();
    let exit = Map::from_iter([
        ("code".to_string(), json!(report.status.code())),
        ("signal".to_string(), json!(report.signal_name())),
        ("turns".to_string(), json!(result.and_then(|result| result.turns))),
        (
            "error_category".to_string(),
            json!(supervised.error_category.map(ErrorCategory::as_str)),
        ),
        ("checkpoint".to_string(), json!(checkpoint)),
    ]);
    CompleteRunRequest {
        error: supervised.error().and_then(error_text),
        estimated_cost_usd: result.and_then(|result| result.cost_usd),
        exit,
        generation: lease.generation,
        outcome: supervised.outcome(),
        final_answer: result
            .and_then(|result| result.text.as_deref())
            .filter(|text| !text.trim().is_empty())
            .map(|text| text.chars().take(MAX_FINAL_ANSWER_LENGTH).collect::<String>())
            .and_then(|text| text.try_into().ok()),
        effective_model: result
            .and_then(|result| result.effective_model.as_deref())
            .and_then(|model| model.try_into().ok()),
    }
}

pub fn spawn_failure(lease: &Lease, error: &anyhow::Error) -> CompleteRunRequest {
    let message = format!("{error:#}");
    warn!(run_id = %lease.run_id, error = %message, "could not run the provider");
    failed_request(
        lease,
        PROCESS_FAILED,
        Map::from_iter([("spawn_error".to_string(), json!(message))]),
    )
}

pub async fn complete_failed(context: &RunContext, lease: &Lease, failure: &anyhow::Error) {
    let message = format!("{failure:#}");
    warn!(run_id = %lease.run_id, error = %message, "run failed before the provider started");
    complete(context, lease, failed_request(lease, &message, Map::new())).await;
}

fn failed_request(lease: &Lease, error: &str, exit: Map<String, Value>) -> CompleteRunRequest {
    CompleteRunRequest {
        error: error_text(error),
        estimated_cost_usd: None,
        exit,
        generation: lease.generation,
        outcome: CompleteRunRequestOutcome::Failed,
        effective_model: None,
        final_answer: None,
    }
}

fn error_text(message: &str) -> Option<CompleteRunRequestError> {
    message
        .chars()
        .take(MAX_ERROR_LENGTH)
        .collect::<String>()
        .try_into()
        .ok()
}

pub async fn record_checkpoint(context: &RunContext, lease: &Lease, checkpoint: &Checkpoint) {
    let payload = json!(checkpoint).as_object().cloned().unwrap_or_default();
    let request = RunEventRequest {
        generation: lease.generation,
        payload,
        type_: RunEventRequestType::Checkpoint,
    };
    if let Err(error) = context.client.record_event(lease.run_id, &request).await {
        warn!(run_id = %lease.run_id, %error, "checkpoint upload failed");
    }
}

pub async fn complete(context: &RunContext, lease: &Lease, request: CompleteRunRequest) {
    match context.client.complete(lease.run_id, &request).await {
        Ok(response) => info!(
            run_id = %lease.run_id,
            state = ?response.state,
            "run finished"
        ),
        Err(error) if error.is_rejection() => warn!(run_id = %lease.run_id, %error, "could not report the run"),
        Err(error) => {
            return warn!(run_id = %lease.run_id, %error, "could not report the run; the journal keeps it to reconcile");
        }
    }
    forget(context, lease.run_id).await;
}

pub async fn remember(context: &RunContext, entry: JournalEntry) {
    let run_id = entry.run_id;
    if let Err(error) = context.journal.record(entry).await {
        warn!(%run_id, error = format!("{error:#}"), "could not record the run in the journal");
    }
}

pub async fn forget_lost_lease(context: &RunContext, lease: &Lease, stop_reason: Option<StopReason>) {
    let reason = stop_reason.map(StopReason::as_str);
    info!(run_id = %lease.run_id, ?reason, "run lost its lease; leaving reconciliation to the server");
    forget(context, lease.run_id).await;
}

async fn forget(context: &RunContext, run_id: Uuid) {
    if let Err(error) = context.journal.remove(run_id).await {
        warn!(%run_id, error = format!("{error:#}"), "could not drop the run from the journal");
    }
}
