use std::collections::BTreeSet;
use std::num::NonZeroU64;

use serde_json::Map;
use tracing::{info, warn};
use uuid::Uuid;

use crate::api::ApiError;
use crate::contracts::{ClaimResponseRunsItem, CompleteRunRequest, CompleteRunRequestOutcome};
use crate::journal::JournalEntry;
use crate::run::RunContext;

const RUNNER_RESTARTED: &str = "runner_restarted";
const RESTARTED_BEFORE_CLAIM: &str = "runner_restarted_before_claim_was_received";

pub async fn reconcile_journal(context: &RunContext, active_run_ids: &BTreeSet<Uuid>) -> Result<(), ApiError> {
    for entry in context.journal.entries().await {
        if active_run_ids.contains(&entry.run_id) || !entry.belongs_to(context.profile.workspace_id) {
            continue;
        }
        complete_restarted(context, &entry).await?;
        if let Err(error) = context.journal.remove(entry.run_id).await {
            warn!(run_id = %entry.run_id, error = format!("{error:#}"), "could not drop the run from the journal");
        }
    }
    Ok(())
}

pub async fn fail_unreceived_claim(context: &RunContext, run: &ClaimResponseRunsItem) {
    let request = failed(run.generation, RESTARTED_BEFORE_CLAIM);
    if let Err(error) = context.client.complete(run.run_id, &request).await {
        warn!(run_id = %run.run_id, %error, "could not reconcile an unjournaled run");
    }
}

async fn complete_restarted(context: &RunContext, entry: &JournalEntry) -> Result<(), ApiError> {
    let request = failed(entry.generation, RUNNER_RESTARTED);
    match context.client.complete(entry.run_id, &request).await {
        Ok(response) => info!(run_id = %entry.run_id, state = ?response.state, "reconciled unfinished run"),
        Err(error) if error.is_rejection() => {
            info!(run_id = %entry.run_id, %error, "run was already reconciled by the server");
        }
        Err(error) => return Err(error),
    }
    Ok(())
}

fn failed(generation: NonZeroU64, error: &str) -> CompleteRunRequest {
    CompleteRunRequest {
        changes: None,
        error: error.try_into().ok(),
        estimated_cost_usd: None,
        exit: Map::new(),
        generation,
        outcome: CompleteRunRequestOutcome::Failed,
        effective_model: None,
        final_answer: None,
    }
}
