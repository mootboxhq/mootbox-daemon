#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, SecondsFormat};
use mootbox_daemon::api::{RUNNER_PROTOCOL_HEADER, RUNNER_PROTOCOL_VERSION};
use mootbox_daemon::contracts::*;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

fn contracts_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("contracts")
}

fn read_json(path: &Path) -> Value {
    let text = fs::read_to_string(path).unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    serde_json::from_str(&text).expect("contract file is JSON")
}

fn read_fixture(name: &str) -> Value {
    read_json(&contracts_dir().join("runner/fixtures").join(name))
}

fn round_trip<T: Serialize + DeserializeOwned>(fixture: &Value) -> Value {
    let typed: T = serde_json::from_value(fixture.clone()).expect("fixture deserializes into its type");
    serde_json::to_value(typed).expect("type serializes")
}

fn timestamps_as_instants(value: Value) -> Value {
    match value {
        Value::String(text) => match DateTime::parse_from_rfc3339(&text) {
            Ok(instant) => Value::String(instant.to_utc().to_rfc3339_opts(SecondsFormat::Millis, true)),
            Err(_) => Value::String(text),
        },
        Value::Array(items) => Value::Array(items.into_iter().map(timestamps_as_instants).collect()),
        Value::Object(fields) => Value::Object(
            fields
                .into_iter()
                .map(|(key, field)| (key, timestamps_as_instants(field)))
                .collect(),
        ),
        other => other,
    }
}

fn assert_round_trips_with_timestamps_compared_as_instants<T: Serialize + DeserializeOwned>(fixture_name: &str) {
    let fixture = read_fixture(fixture_name);
    let reserialized = round_trip::<T>(&fixture);
    assert_eq!(
        timestamps_as_instants(reserialized),
        timestamps_as_instants(fixture),
        "{fixture_name}"
    );
}

macro_rules! round_trip_tests {
    ($($test_name:ident: $fixture:literal => $contract:ty,)*) => {
        const ROUND_TRIPPED_FIXTURES: &[&str] = &[$($fixture,)* MINIMAL_CLAIM_REQUEST];

        mod round_trips_with_timestamps_compared_as_instants {
            use super::*;
            $(
                #[test]
                fn $test_name() {
                    assert_round_trips_with_timestamps_compared_as_instants::<$contract>($fixture);
                }
            )*
        }
    };
}

const MINIMAL_CLAIM_REQUEST: &str = "claim-request.minimal.json";

round_trip_tests! {
    claim_pairing_code_request: "claim-pairing-code-request.json" => ClaimPairingCodeRequest,
    claim_pairing_code_response: "claim-pairing-code-response.json" => ClaimPairingCodeResponse,
    claim_request: "claim-request.json" => ClaimRequest,
    claim_request_with_device: "claim-request.with-device.json" => ClaimRequest,
    claim_response_claimed_execute: "claim-response.claimed-execute.json" => ClaimResponse,
    claim_response_claimed_execute_with_context: "claim-response.claimed-execute.with-context.json" => ClaimResponse,
    claim_response_idle: "claim-response.idle.json" => ClaimResponse,
    claim_response_reconcile_required: "claim-response.reconcile-required.json" => ClaimResponse,
    complete_run_request: "complete-run-request.json" => CompleteRunRequest,
    complete_run_request_with_changes: "complete-run-request.with-changes.json" => CompleteRunRequest,
    complete_run_response: "complete-run-response.json" => CompleteRunResponse,
    error_cli_session_invalid: "error.cli-session-invalid.json" => Error,
    error_lease_lost: "error.lease-lost.json" => Error,
    error_rate_limited: "error.rate-limited.json" => Error,
    error_repository_not_found: "error.repository-not-found.json" => Error,
    error_upgrade_required: "error.upgrade-required.json" => Error,
    exchange_cli_login_request: "exchange-cli-login-request.json" => ExchangeCliLoginRequest,
    exchange_cli_login_response_pending: "exchange-cli-login-response.pending.json" => ExchangeCliLoginResponse,
    exchange_cli_login_response_signed_in: "exchange-cli-login-response.signed-in.json" => ExchangeCliLoginResponse,
    exchange_pairing_request: "exchange-pairing-request.json" => ExchangePairingRequest,
    exchange_pairing_response_paired: "exchange-pairing-response.paired.json" => ExchangePairingResponse,
    exchange_pairing_response_pending: "exchange-pairing-response.pending.json" => ExchangePairingResponse,
    exchange_repository_link_response_linked: "exchange-repository-link-response.linked.json" => ExchangeRepositoryLinkResponse,
    exchange_repository_link_response_pending: "exchange-repository-link-response.pending.json" => ExchangeRepositoryLinkResponse,
    health_response: "health-response.json" => HealthResponse,
    heartbeat_request: "heartbeat-request.json" => HeartbeatRequest,
    heartbeat_response: "heartbeat-response.json" => HeartbeatResponse,
    heartbeat_response_cancel_requested: "heartbeat-response.cancel-requested.json" => HeartbeatResponse,
    match_repositories_request: "match-repositories-request.json" => MatchRepositoriesRequest,
    match_repositories_response: "match-repositories-response.json" => MatchRepositoriesResponse,
    match_repositories_response_none: "match-repositories-response.none.json" => MatchRepositoriesResponse,
    register_repository_request: "register-repository-request.json" => RegisterRepositoryRequest,
    register_repository_request_new_project: "register-repository-request.new-project.json" => RegisterRepositoryRequest,
    register_repository_response: "register-repository-response.json" => RegisterRepositoryResponse,
    run_event_request: "run-event-request.json" => RunEventRequest,
    run_event_response: "run-event-response.json" => RunEventResponse,
    start_cli_login_request: "start-cli-login-request.json" => StartCliLoginRequest,
    start_cli_login_response: "start-cli-login-response.json" => StartCliLoginResponse,
    start_pairing_request: "start-pairing-request.json" => StartPairingRequest,
    start_pairing_response: "start-pairing-response.json" => StartPairingResponse,
    start_repository_link_request: "start-repository-link-request.json" => StartRepositoryLinkRequest,
    start_repository_link_response_choose: "start-repository-link-response.choose.json" => StartRepositoryLinkResponse,
    start_repository_link_response_linked: "start-repository-link-response.linked.json" => StartRepositoryLinkResponse,
    start_repository_link_response_linked_with_runner: "start-repository-link-response.linked-with-runner.json" => StartRepositoryLinkResponse,
    upsert_connection_request: "upsert-connection-request.json" => UpsertConnectionRequest,
    upsert_connection_response: "upsert-connection-response.json" => UpsertConnectionResponse,
}

#[test]
fn minimal_claim_request_round_trips_with_schema_defaults_filled_in() {
    let fixture = read_fixture(MINIMAL_CLAIM_REQUEST);
    let schema_properties = read_json(&contracts_dir().join("runner/schema/claim-request.json"))["properties"].clone();
    let mut expected = fixture.clone();
    for field in ["capacity", "wait_ms"] {
        expected[field] = schema_properties[field]["default"].clone();
    }

    assert_eq!(round_trip::<ClaimRequest>(&fixture), expected);
}

#[test]
fn every_vendored_fixture_has_a_round_trip_test() {
    let vendored: BTreeSet<String> = fs::read_dir(contracts_dir().join("runner/fixtures"))
        .expect("fixtures are vendored")
        .map(|entry| {
            entry
                .expect("fixture entry")
                .file_name()
                .into_string()
                .expect("UTF-8 name")
        })
        .collect();
    let tested: BTreeSet<String> = ROUND_TRIPPED_FIXTURES.iter().map(ToString::to_string).collect();

    assert_eq!(vendored, tested);
}

#[test]
fn contracts_source_protocol_version_matches_the_client() {
    let source = fs::read_to_string(contracts_dir().join("SOURCE")).expect("contracts/SOURCE exists");
    let protocol_version = source
        .lines()
        .find_map(|line| line.strip_prefix("protocol_version="))
        .expect("SOURCE records protocol_version");

    assert_eq!(protocol_version, RUNNER_PROTOCOL_VERSION.to_string());
}

#[test]
fn vendored_index_protocol_header_and_version_match_the_client() {
    let index = read_json(&contracts_dir().join("runner/schema/index.json"));

    assert_eq!(index["protocol_header"], RUNNER_PROTOCOL_HEADER);
    assert_eq!(index["protocol_version"], RUNNER_PROTOCOL_VERSION);
}
