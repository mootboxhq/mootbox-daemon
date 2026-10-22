use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::time::Duration;

use serde_json::{Value, json};
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::common::fixture;
use crate::harness::{
    FakeProvider, TestDaemon, answer_later_heartbeats, bound_home, claimed, completion_of, daemon_options,
    execute_claim, git, healthy_heartbeats, heartbeat_answer, journal_on_disk, mount_claim_once,
    mount_claim_while_active, mount_mootbox, requests_to, run_request_index, wait_for_claims, wait_for_exit,
    wait_until,
};

const STOP_HEARTBEATS: usize = 3;
const HTTP_CONFLICT: u16 = 409;
const HTTP_BAD_GATEWAY: u16 = 502;
const HTTP_UPGRADE_REQUIRED: u16 = 426;
const HEALTH_ROUTE: &str = "/runner/health";

#[tokio::test]
async fn two_slots_run_two_runs_at_once_and_report_the_active_run() {
    let server = MockServer::start().await;
    let mut bound = bound_home(&server);
    bound.config.slots = 2;
    let (first, second) = (Uuid::new_v4(), Uuid::new_v4());
    mount_claim_once(&server, claimed(execute_claim(first, bound.repository_id))).await;
    mount_claim_while_active(&server, first, execute_claim(second, bound.repository_id)).await;
    mount_mootbox(&server, healthy_heartbeats()).await;
    let provider = FakeProvider::claude("gated");
    let daemon = TestDaemon::start(daemon_options(bound.config.clone(), &bound.home, &provider));
    let second_heartbeats = format!("/runner/runs/{second}/heartbeat");
    wait_until(|| async { (!requests_to(&server, &second_heartbeats).await.is_empty()).then_some(()) }).await;
    provider.release();

    let first_completion = completion_of(&server, first).await;
    let second_completion = completion_of(&server, second).await;
    daemon.stop().await;

    let requests = server.received_requests().await.expect("recording");
    let second_started = run_request_index(&requests, second, "heartbeat").expect("second heartbeat");
    let first_finished = run_request_index(&requests, first, "complete").expect("first completion");
    assert!(
        second_started < first_finished,
        "the second run started only after the first finished"
    );
    for (run_id, completion) in [(first, &first_completion), (second, &second_completion)] {
        assert_eq!(completion["outcome"], "succeeded", "{completion}");
        assert!(completion.get("provider_session_id").is_none());
        assert_eq!(completion["estimated_cost_usd"], 0.01);
        assert_eq!(completion["exit"]["code"], 0);
        assert_eq!(completion["exit"]["checkpoint"]["branch"], format!("mootbox/{run_id}"));
        let manifest = bound.home.join("runs").join(run_id.to_string()).join("manifest.json");
        assert_eq!(
            fs::metadata(&manifest).expect("manifest").permissions().mode() & 0o777,
            0o600
        );
        let worktree = bound.home.join("worktrees").join(format!("mootbox-{run_id}"));
        assert_eq!(
            git(&worktree, &["rev-parse", "--abbrev-ref", "HEAD"]),
            format!("mootbox/{run_id}")
        );
    }
    let first_heartbeat = requests_to(&server, &format!("/runner/runs/{first}/heartbeat")).await;
    assert_eq!(first_heartbeat[0], json!({ "generation": 1, "state": "executing" }));
    let checkpoint = requests_to(&server, &format!("/runner/runs/{first}/events")).await;
    assert_eq!(checkpoint[0]["type"], "checkpoint");
    assert!(
        checkpoint[0]["payload"]
            .get("agent_may_push_with_local_credentials")
            .is_none()
    );
    assert_eq!(git(&bound.repository, &["rev-parse", "--abbrev-ref", "HEAD"]), "main");
    assert!(
        provider.invocations()[0]
            .prompt
            .contains("Owner's instruction: @Claude inspect the users endpoint")
    );
}

#[tokio::test]
async fn a_requested_cancel_kills_the_whole_process_group() {
    let server = MockServer::start().await;
    let bound = bound_home(&server);
    let run_id = Uuid::new_v4();
    mount_claim_once(&server, claimed(execute_claim(run_id, bound.repository_id))).await;
    mount_mootbox(&server, healthy_heartbeats()).await;
    let provider = FakeProvider::claude("hang");
    let daemon = TestDaemon::start(daemon_options(bound.config.clone(), &bound.home, &provider));
    let grandchild = provider.grandchild().await;

    answer_later_heartbeats(
        &server,
        ResponseTemplate::new(200).set_body_json(heartbeat_answer(true)),
    )
    .await;
    let completion = completion_of(&server, run_id).await;
    daemon.stop().await;

    assert_eq!(completion["outcome"], "cancelled", "{completion}");
    assert_eq!(completion["error"], "cancel_requested");
    wait_for_exit(grandchild).await;
}

#[tokio::test]
async fn heartbeats_continue_until_a_stubborn_process_group_is_gone() {
    let server = MockServer::start().await;
    let bound = bound_home(&server);
    let run_id = Uuid::new_v4();
    let heartbeat_route = format!("/runner/runs/{run_id}/heartbeat");
    mount_claim_once(&server, claimed(execute_claim(run_id, bound.repository_id))).await;
    mount_mootbox(&server, healthy_heartbeats()).await;
    let provider = FakeProvider::claude("stubborn");
    let daemon = TestDaemon::start(daemon_options(bound.config.clone(), &bound.home, &provider));
    let grandchild = provider.grandchild().await;

    let before_cancel = requests_to(&server, &heartbeat_route).await.len();
    answer_later_heartbeats(
        &server,
        ResponseTemplate::new(200).set_body_json(heartbeat_answer(true)),
    )
    .await;
    let completion = completion_of(&server, run_id).await;
    daemon.stop().await;

    assert_eq!(completion["outcome"], "cancelled", "{completion}");
    wait_for_exit(grandchild).await;
    let during_stop = requests_to(&server, &heartbeat_route).await.len() - before_cancel;
    assert!(
        during_stop >= STOP_HEARTBEATS,
        "only {during_stop} heartbeats while stopping"
    );
}

#[tokio::test]
async fn a_lost_lease_stops_the_run_without_completing_it() {
    let server = MockServer::start().await;
    let bound = bound_home(&server);
    let run_id = Uuid::new_v4();
    mount_claim_once(&server, claimed(execute_claim(run_id, bound.repository_id))).await;
    mount_mootbox(&server, healthy_heartbeats()).await;
    let provider = FakeProvider::claude("hang");
    let daemon = TestDaemon::start(daemon_options(bound.config.clone(), &bound.home, &provider));
    let grandchild = provider.grandchild().await;

    let lease_lost = ResponseTemplate::new(HTTP_CONFLICT).set_body_json(fixture("error.lease-lost.json"));
    answer_later_heartbeats(&server, lease_lost).await;
    wait_for_claims(&server, 2).await;
    daemon.stop().await;

    wait_for_exit(grandchild).await;
    assert_eq!(
        requests_to(&server, &format!("/runner/runs/{run_id}/complete")).await,
        Vec::<Value>::new()
    );
    assert_eq!(journal_on_disk(&bound.home), Vec::<Value>::new());
}

#[tokio::test]
async fn wall_time_stops_the_run_as_failed() {
    let server = MockServer::start().await;
    let bound = bound_home(&server);
    let run_id = Uuid::new_v4();
    mount_claim_once(&server, claimed(execute_claim(run_id, bound.repository_id))).await;
    mount_mootbox(&server, healthy_heartbeats()).await;
    let provider = FakeProvider::claude("hang");
    let mut options = daemon_options(bound.config.clone(), &bound.home, &provider);
    options.run_timing.wall_time = Duration::from_millis(300);
    let daemon = TestDaemon::start(options);

    let completion = completion_of(&server, run_id).await;
    daemon.stop().await;

    assert_eq!(completion["outcome"], "failed", "{completion}");
    assert_eq!(completion["error"], "wall_time_exceeded");
}

#[tokio::test]
async fn a_crashed_provider_fails_the_run_with_its_exit_code() {
    let server = MockServer::start().await;
    let bound = bound_home(&server);
    let run_id = Uuid::new_v4();
    mount_claim_once(&server, claimed(execute_claim(run_id, bound.repository_id))).await;
    mount_mootbox(&server, healthy_heartbeats()).await;
    let provider = FakeProvider::claude("crash");
    let daemon = TestDaemon::start(daemon_options(bound.config.clone(), &bound.home, &provider));

    let completion = completion_of(&server, run_id).await;
    daemon.stop().await;

    assert_eq!(completion["outcome"], "failed", "{completion}");
    assert_eq!(completion["error"], "process_failed");
    assert_eq!(completion["exit"]["code"], 3);
    assert!(completion.get("provider_session_id").is_none());
}

#[tokio::test]
async fn a_run_for_an_unbound_repository_fails_without_starting_the_provider() {
    let server = MockServer::start().await;
    let bound = bound_home(&server);
    let run_id = Uuid::new_v4();
    mount_claim_once(&server, claimed(execute_claim(run_id, Uuid::new_v4()))).await;
    mount_mootbox(&server, healthy_heartbeats()).await;
    let provider = FakeProvider::claude("success");
    let daemon = TestDaemon::start(daemon_options(bound.config.clone(), &bound.home, &provider));

    let completion = completion_of(&server, run_id).await;
    daemon.stop().await;

    assert_eq!(completion["error"], "repository_not_bound_locally");
    assert!(provider.invocations().is_empty());
}

#[tokio::test]
async fn shutdown_lets_an_active_run_finish_within_the_grace_period() {
    let server = MockServer::start().await;
    let bound = bound_home(&server);
    let run_id = Uuid::new_v4();
    mount_claim_once(&server, claimed(execute_claim(run_id, bound.repository_id))).await;
    mount_mootbox(&server, healthy_heartbeats()).await;
    let provider = FakeProvider::claude("gated");
    let daemon = TestDaemon::start(daemon_options(bound.config.clone(), &bound.home, &provider));

    provider.grandchild().await;
    daemon.request_stop();
    provider.release();
    daemon.finished().await;

    let completion = completion_of(&server, run_id).await;
    assert_eq!(completion["outcome"], "succeeded", "{completion}");
}

#[tokio::test]
async fn shutdown_cancels_runs_still_active_after_the_grace_period() {
    let server = MockServer::start().await;
    let bound = bound_home(&server);
    let run_id = Uuid::new_v4();
    mount_claim_once(&server, claimed(execute_claim(run_id, bound.repository_id))).await;
    mount_mootbox(&server, healthy_heartbeats()).await;
    let provider = FakeProvider::claude("hang");
    let mut options = daemon_options(bound.config.clone(), &bound.home, &provider);
    options.timing.shutdown_grace = Duration::from_millis(200);
    let daemon = TestDaemon::start(options);

    let grandchild = provider.grandchild().await;
    daemon.stop().await;

    let completion = completion_of(&server, run_id).await;
    assert_eq!(completion["outcome"], "failed", "{completion}");
    assert_eq!(completion["error"], "runner_shutdown");
    wait_for_exit(grandchild).await;
}

#[tokio::test]
async fn claim_failures_back_off_and_the_slot_keeps_claiming() {
    let server = MockServer::start().await;
    let bound = bound_home(&server);
    let run_id = Uuid::new_v4();
    mount_claim_once(&server, ResponseTemplate::new(HTTP_BAD_GATEWAY)).await;
    mount_claim_while_active(&server, Uuid::nil(), json!({})).await;
    mount_mootbox(&server, healthy_heartbeats()).await;
    let provider = FakeProvider::claude("success");
    let daemon = TestDaemon::start(daemon_options(bound.config.clone(), &bound.home, &provider));
    wait_for_claims(&server, 2).await;
    mount_claim_once(&server, claimed(execute_claim(run_id, bound.repository_id))).await;

    let completion = completion_of(&server, run_id).await;
    daemon.stop().await;

    assert_eq!(completion["outcome"], "succeeded", "{completion}");
}

#[tokio::test]
async fn reconcile_required_is_logged_and_claiming_continues() {
    let server = MockServer::start().await;
    let bound = bound_home(&server);
    let reconcile = fixture("claim-response.reconcile-required.json");
    mount_claim_once(&server, claimed(reconcile)).await;
    mount_mootbox(&server, healthy_heartbeats()).await;
    let provider = FakeProvider::claude("success");
    let daemon = TestDaemon::start(daemon_options(bound.config.clone(), &bound.home, &provider));

    wait_for_claims(&server, 3).await;

    daemon.stop().await;
}

#[tokio::test]
async fn upgrade_required_pauses_claiming_until_the_health_check_is_accepted() {
    let server = MockServer::start().await;
    let mut bound = bound_home(&server);
    bound.config.slots = 2;
    let upgrade = fixture("error.upgrade-required.json");
    for _ in 0..2 {
        mount_claim_once(
            &server,
            ResponseTemplate::new(HTTP_UPGRADE_REQUIRED).set_body_json(upgrade.clone()),
        )
        .await;
    }
    Mock::given(method("GET"))
        .and(path(HEALTH_ROUTE))
        .respond_with(ResponseTemplate::new(HTTP_UPGRADE_REQUIRED).set_body_json(upgrade))
        .up_to_n_times(1)
        .with_priority(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(HEALTH_ROUTE))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("health-response.json")))
        .mount(&server)
        .await;
    mount_mootbox(&server, healthy_heartbeats()).await;
    let provider = FakeProvider::claude("success");
    let daemon = TestDaemon::start(daemon_options(bound.config.clone(), &bound.home, &provider));

    wait_for_claims(&server, 4).await;
    daemon.stop().await;

    let requests = server.received_requests().await.expect("recording");
    let paths: Vec<&str> = requests.iter().map(|request| request.url.path()).collect();
    let health_checks: Vec<usize> = (0..paths.len()).filter(|&index| paths[index] == HEALTH_ROUTE).collect();
    assert_eq!(health_checks.len(), 2, "{paths:?}");
    let claims_before_acceptance = paths[..health_checks[1]]
        .iter()
        .filter(|&&route| route == "/runner/claim")
        .count();
    assert_eq!(claims_before_acceptance, 2, "{paths:?}");
}
