use std::time::Duration;

use crate::common::{CREDENTIAL, authenticated_route, fixture};
use mootbox_daemon::api::{MootboxClient, claim_timeout};
use mootbox_daemon::contracts::{ClaimResponse, HeartbeatRequest};
use reqwest::Url;
use serde::de::DeserializeOwned;
use uuid::Uuid;
use wiremock::matchers::{body_json, header_exists, header_regex};
use wiremock::{MockServer, ResponseTemplate};

const RUN_ID: &str = "5d1f5c1e-7f43-4c6a-9a51-0d6a1c1b2016";

fn request<T: DeserializeOwned>(name: &str) -> T {
    serde_json::from_value(fixture(name)).expect("request fixture")
}

fn run_id() -> Uuid {
    RUN_ID.parse().expect("UUID")
}

fn run_trace_pattern() -> String {
    format!("^00-{}-[0-9a-f]{{16}}-01$", run_id().simple())
}

async fn client_answering(route: &str, request_fixture: &str, response_fixture: &str) -> (MockServer, MootboxClient) {
    let server = MockServer::start().await;
    authenticated_route("POST", route)
        .and(body_json(fixture(request_fixture)))
        .and(header_exists("x-request-id"))
        .and(header_regex("traceparent", &run_trace_pattern()))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture(response_fixture)))
        .expect(1)
        .mount(&server)
        .await;
    let client = MootboxClient::authenticated(Url::parse(&server.uri()).expect("URL"), CREDENTIAL.to_string());
    (server, client)
}

#[tokio::test]
async fn claim_sends_the_slot_request_and_reads_a_claimed_run() {
    let server = MockServer::start().await;
    authenticated_route("POST", "/runner/claim")
        .and(body_json(fixture("claim-request.json")))
        .and(header_regex("traceparent", "^00-[0-9a-f]{32}-[0-9a-f]{16}-01$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("claim-response.claimed-execute.json")))
        .expect(1)
        .mount(&server)
        .await;
    let client = MootboxClient::authenticated(Url::parse(&server.uri()).expect("URL"), CREDENTIAL.to_string());

    let outcome = client.claim(&request("claim-request.json")).await.expect("claims");

    let ClaimResponse::Claimed { run } = outcome else {
        panic!("expected a claimed run, got {outcome:?}");
    };
    assert_eq!(run.provider.to_string(), "claude-code");
    assert!(!run.manifest.resume);
}

#[test]
fn the_claim_timeout_outlasts_the_server_wait() {
    assert!(claim_timeout(25_000) > Duration::from_secs(25));
}

#[tokio::test]
async fn heartbeat_is_traced_by_run_id() {
    let route = format!("/runner/runs/{RUN_ID}/heartbeat");
    let (_server, client) = client_answering(&route, "heartbeat-request.json", "heartbeat-response.json").await;
    let body: HeartbeatRequest = request("heartbeat-request.json");

    let response = client.heartbeat(run_id(), &body).await.expect("heartbeat");

    assert!(!response.cancel_requested);
}

#[tokio::test]
async fn run_events_are_recorded() {
    let route = format!("/runner/runs/{RUN_ID}/events");
    let (_server, client) = client_answering(&route, "run-event-request.json", "run-event-response.json").await;

    let response = client
        .record_event(run_id(), &request("run-event-request.json"))
        .await
        .expect("event");

    assert!(response.recorded);
}

#[tokio::test]
async fn completion_returns_the_run_state() {
    let route = format!("/runner/runs/{RUN_ID}/complete");
    let (_server, client) = client_answering(&route, "complete-run-request.json", "complete-run-response.json").await;

    let response = client
        .complete(run_id(), &request("complete-run-request.json"))
        .await
        .expect("completes");

    assert_eq!(response.state.to_string(), "succeeded");
}
