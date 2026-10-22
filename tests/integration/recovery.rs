use std::num::NonZeroU64;
use std::path::Path;
use std::time::Duration;

use chrono::Utc;
use mootbox_daemon::journal::{Journal, JournalEntry};
use serde_json::{Value, json};
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::common::fixture;
use crate::harness::{
    FakeProvider, TestDaemon, bound_home, claimed, completion_of, daemon_options, execute_claim, healthy_heartbeats,
    journal_on_disk, mount_claim_once, mount_claim_while_active, mount_mootbox, requests_to, run_request_index,
    wait_for_claims, wait_for_empty_journal, wait_until,
};

const BEFORE_EVERYTHING_PRIORITY: u8 = 1;
const UNREACHABLE_RECONCILE_WAIT: Duration = Duration::from_secs(600);
const SETTLE_TIME: Duration = Duration::from_millis(250);

fn orphan(run_id: Uuid, worktree: &Path) -> JournalEntry {
    JournalEntry {
        run_id,
        generation: NonZeroU64::MIN,
        branch: format!("mootbox/{run_id}"),
        worktree: worktree.to_path_buf(),
        started_at: Utc::now(),
        workspace_id: None,
    }
}

async fn seed(home: &Path, entry: JournalEntry) {
    Journal::open(home)
        .expect("opens")
        .record(entry)
        .await
        .expect("records");
}

fn restarted() -> Value {
    json!({ "generation": 1, "outcome": "failed", "error": "runner_restarted" })
}

#[tokio::test]
async fn an_unfinished_run_in_the_journal_is_completed_as_runner_restarted_before_claiming() {
    let server = MockServer::start().await;
    let bound = bound_home(&server);
    let run_id = Uuid::new_v4();
    seed(&bound.home, orphan(run_id, Path::new("/nonexistent"))).await;
    mount_mootbox(&server, healthy_heartbeats()).await;
    let provider = FakeProvider::claude("success");
    let daemon = TestDaemon::start(daemon_options(bound.config.clone(), &bound.home, &provider));

    let completion = completion_of(&server, run_id).await;
    wait_for_claims(&server, 2).await;
    daemon.stop().await;

    assert_eq!(completion, restarted());
    let requests = server.received_requests().await.expect("recording");
    let completed = run_request_index(&requests, run_id, "complete").expect("completion");
    let first_claim = requests
        .iter()
        .position(|request| request.url.path() == "/runner/claim");
    assert!(
        first_claim.is_some_and(|claim| completed < claim),
        "completed after claiming"
    );
    assert_eq!(journal_on_disk(&bound.home), Vec::<Value>::new());
    assert!(provider.invocations().is_empty());
}

#[tokio::test]
async fn reconcile_required_retries_a_run_the_startup_reconcile_could_not_report() {
    let server = MockServer::start().await;
    let bound = bound_home(&server);
    let run_id = Uuid::new_v4();
    seed(&bound.home, orphan(run_id, Path::new("/nonexistent"))).await;
    Mock::given(method("POST"))
        .and(path(format!("/runner/runs/{run_id}/complete")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .up_to_n_times(1)
        .with_priority(BEFORE_EVERYTHING_PRIORITY)
        .mount(&server)
        .await;
    mount_claim_once(&server, claimed(fixture("claim-response.reconcile-required.json"))).await;
    mount_mootbox(&server, healthy_heartbeats()).await;
    let provider = FakeProvider::claude("success");
    let daemon = TestDaemon::start(daemon_options(bound.config.clone(), &bound.home, &provider));

    wait_for_empty_journal(&bound.home).await;
    daemon.stop().await;

    let completions = requests_to(&server, &format!("/runner/runs/{run_id}/complete")).await;
    assert_eq!(completions, [restarted(), restarted()]);
}

#[tokio::test]
async fn reconcile_required_leaves_a_run_active_in_this_daemon_alone() {
    let server = MockServer::start().await;
    let mut bound = bound_home(&server);
    bound.config.slots = 2;
    let run_id = Uuid::new_v4();
    mount_claim_once(&server, claimed(execute_claim(run_id, bound.repository_id))).await;
    let mut reconcile = fixture("claim-response.reconcile-required.json");
    reconcile["run_ids"] = json!([run_id]);
    mount_claim_while_active(&server, run_id, reconcile).await;
    mount_mootbox(&server, healthy_heartbeats()).await;
    let provider = FakeProvider::claude("gated");
    let daemon = TestDaemon::start(daemon_options(bound.config.clone(), &bound.home, &provider));

    wait_until(|| async {
        let claims = requests_to(&server, "/runner/claim").await;
        claims
            .iter()
            .any(|claim| claim["active_run_ids"] == json!([run_id]))
            .then_some(())
    })
    .await;
    let during_run = journal_on_disk(&bound.home);
    provider.release();
    let completion = completion_of(&server, run_id).await;
    wait_for_empty_journal(&bound.home).await;
    daemon.stop().await;

    assert_eq!(during_run.len(), 1, "{during_run:?}");
    assert_eq!(during_run[0]["run_id"], json!(run_id));
    assert_eq!(during_run[0]["branch"], format!("mootbox/{run_id}"));
    assert_eq!(during_run[0]["generation"], 1);
    let worktree = bound.home.join("worktrees").join(format!("mootbox-{run_id}"));
    assert_eq!(during_run[0]["worktree"], json!(worktree));
    assert_eq!(completion["outcome"], "succeeded", "{completion}");
    let completions = requests_to(&server, &format!("/runner/runs/{run_id}/complete")).await;
    assert_eq!(completions.len(), 1);
}

#[tokio::test]
async fn reconcile_required_for_runs_active_in_this_daemon_claims_again_without_waiting() {
    let server = MockServer::start().await;
    let mut bound = bound_home(&server);
    bound.config.slots = 2;
    let run_id = Uuid::new_v4();
    mount_claim_once(&server, claimed(execute_claim(run_id, bound.repository_id))).await;
    let mut reconcile = fixture("claim-response.reconcile-required.json");
    reconcile["run_ids"] = json!([run_id]);
    mount_claim_while_active(&server, run_id, reconcile).await;
    mount_mootbox(&server, healthy_heartbeats()).await;
    let provider = FakeProvider::claude("gated");
    let mut options = daemon_options(bound.config.clone(), &bound.home, &provider);
    options.timing.reconcile_wait = UNREACHABLE_RECONCILE_WAIT;
    let daemon = TestDaemon::start(options);

    wait_until(|| async {
        let claims = requests_to(&server, "/runner/claim").await;
        let claims_during_run = claims
            .iter()
            .filter(|claim| claim["active_run_ids"] == json!([run_id]))
            .count();
        (claims_during_run >= 2).then_some(())
    })
    .await;
    provider.release();
    completion_of(&server, run_id).await;
    daemon.stop().await;
}

#[tokio::test]
async fn reconcile_required_without_run_ids_still_waits_before_claiming_again() {
    let server = MockServer::start().await;
    let bound = bound_home(&server);
    let mut reconcile = fixture("claim-response.reconcile-required.json");
    reconcile["run_ids"] = json!([]);
    mount_claim_once(&server, claimed(reconcile)).await;
    mount_mootbox(&server, healthy_heartbeats()).await;
    let provider = FakeProvider::claude("success");
    let mut options = daemon_options(bound.config.clone(), &bound.home, &provider);
    options.timing.reconcile_wait = UNREACHABLE_RECONCILE_WAIT;
    let daemon = TestDaemon::start(options);

    wait_for_claims(&server, 1).await;
    tokio::time::sleep(SETTLE_TIME).await;
    daemon.stop().await;

    assert_eq!(requests_to(&server, "/runner/claim").await.len(), 1);
}
