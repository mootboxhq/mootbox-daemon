use std::os::unix::fs::PermissionsExt;
use std::time::Duration;

use crate::common::{SilentConsole, fixture, fixture_uuid, runner_route};
use mootbox_daemon::commands::pair::{self, PairOptions};
use mootbox_daemon::config::{self, WorkspaceProfile};
use reqwest::Url;
use serde_json::json;
use wiremock::matchers::{body_json, body_partial_json};
use wiremock::{MockServer, ResponseTemplate};

const FAST_POLL: Duration = Duration::from_millis(10);
const HTTP_FORBIDDEN: u16 = 403;
const HTTP_UPGRADE_REQUIRED: u16 = 426;
const PAIRING_RESPONSE: &str = "start-pairing-response.json";
const DEVICE_NAME: &str = "maya-macbook";
const CLAIMED_CODE: &str = "claim-pairing-code-response.json";
const HTTP_NOT_FOUND: u16 = 404;

async fn mount_started_pairing(server: &MockServer) -> String {
    runner_route("POST", "/runner/pairings")
        .and(body_json(fixture("start-pairing-request.json")))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture(PAIRING_RESPONSE)))
        .expect(1)
        .mount(server)
        .await;
    format!(
        "/runner/pairings/{}/exchange",
        fixture_uuid(PAIRING_RESPONSE, "/pairing_id")
    )
}

fn pair_options(server: &MockServer) -> PairOptions {
    PairOptions {
        mootbox_url: Url::parse(&server.uri()).expect("URL"),
        device_name: DEVICE_NAME.to_string(),
        code: None,
        poll_interval: FAST_POLL,
    }
}

#[tokio::test]
async fn pairing_with_a_web_code_claims_it_once_and_saves_the_credential() {
    let server = MockServer::start().await;
    runner_route("POST", "/runner/pairings/claim-code")
        .and(body_partial_json(
            json!({ "user_code": "ABCD-EFGH", "name": DEVICE_NAME }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture(CLAIMED_CODE)))
        .expect(1)
        .mount(&server)
        .await;
    runner_route("POST", "/runner/pairings")
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(&server)
        .await;
    let home = tempfile::tempdir().expect("temp dir");
    let options = PairOptions {
        code: Some("ABCD-EFGH".to_string()),
        ..pair_options(&server)
    };

    pair::run(home.path(), &options, &SilentConsole).await.expect("pairs");

    let claimed = server.received_requests().await.expect("recorded")[0]
        .body_json::<serde_json::Value>()
        .expect("JSON body");
    let device = &claimed["device"];
    assert_eq!(device["daemon_version"], env!("CARGO_PKG_VERSION"));
    assert!(["macOS", "Linux", "Windows", "other"].contains(&device["os"].as_str().expect("os")));
    assert_eq!(device["providers"].as_array().expect("providers").len(), 2);
    let saved = config::load(home.path()).expect("config saved");
    let expected = WorkspaceProfile {
        mootbox_url: Url::parse(&server.uri()).expect("URL").to_string(),
        workspace_id: fixture_uuid(CLAIMED_CODE, "/workspace_id"),
        runner_id: fixture_uuid(CLAIMED_CODE, "/runner_id"),
        credential: fixture(CLAIMED_CODE)["credential"]
            .as_str()
            .expect("credential")
            .to_string(),
        workspace_name: fixture(CLAIMED_CODE)["workspace_name"].as_str().map(str::to_string),
    };
    assert_eq!(saved.workspaces, [expected]);
}

#[tokio::test]
async fn an_unclaimable_web_code_saves_nothing() {
    let server = MockServer::start().await;
    let unclaimable =
        json!({ "error": { "category": "not_found", "message": "pairing code is invalid, expired, or already used" } });
    runner_route("POST", "/runner/pairings/claim-code")
        .respond_with(ResponseTemplate::new(HTTP_NOT_FOUND).set_body_json(unclaimable))
        .expect(1)
        .mount(&server)
        .await;
    let home = tempfile::tempdir().expect("temp dir");
    let options = PairOptions {
        code: Some("ZZZZ-ZZZZ".to_string()),
        ..pair_options(&server)
    };

    let error = pair::run(home.path(), &options, &SilentConsole)
        .await
        .expect_err("unclaimable");

    assert!(
        error.to_string().contains("invalid, expired, or already used"),
        "{error}"
    );
    assert!(!config::config_path(home.path()).exists());
}

#[tokio::test]
async fn pairing_polls_until_paired_and_saves_the_credential() {
    let server = MockServer::start().await;
    let exchange_route = mount_started_pairing(&server).await;
    runner_route("POST", &exchange_route)
        .and(body_json(fixture("exchange-pairing-request.json")))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("exchange-pairing-response.pending.json")))
        .up_to_n_times(2)
        .with_priority(1)
        .expect(2)
        .mount(&server)
        .await;
    runner_route("POST", &exchange_route)
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("exchange-pairing-response.paired.json")))
        .expect(1)
        .mount(&server)
        .await;
    let home = tempfile::tempdir().expect("temp dir");

    pair::run(home.path(), &pair_options(&server), &SilentConsole)
        .await
        .expect("pairs");

    let paired = "exchange-pairing-response.paired.json";
    let saved = config::load(home.path()).expect("config saved");
    let expected = WorkspaceProfile {
        mootbox_url: Url::parse(&server.uri()).expect("URL").to_string(),
        workspace_id: fixture_uuid(paired, "/workspace_id"),
        runner_id: fixture_uuid(paired, "/runner_id"),
        credential: fixture(paired)["credential"].as_str().expect("credential").to_string(),
        workspace_name: fixture(paired)["workspace_name"].as_str().map(str::to_string),
    };
    assert_eq!(saved.workspaces, [expected]);
    assert!(saved.connections.is_empty());
    let mode = std::fs::metadata(config::config_path(home.path()))
        .expect("file")
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o600);
}

#[tokio::test]
async fn expired_pairing_stops_polling_and_saves_nothing() {
    let server = MockServer::start().await;
    let exchange_route = mount_started_pairing(&server).await;
    let expired = json!({ "error": { "category": "permission_denied", "message": "pairing expired" } });
    runner_route("POST", &exchange_route)
        .respond_with(ResponseTemplate::new(HTTP_FORBIDDEN).set_body_json(expired))
        .expect(1)
        .mount(&server)
        .await;
    let home = tempfile::tempdir().expect("temp dir");

    let error = pair::run(home.path(), &pair_options(&server), &SilentConsole)
        .await
        .expect_err("expired");

    assert!(error.to_string().contains("pairing expired"), "{error}");
    assert!(!config::config_path(home.path()).exists());
}

#[tokio::test]
async fn pairing_against_a_newer_server_explains_the_upgrade() {
    let server = MockServer::start().await;
    runner_route("POST", "/runner/pairings")
        .respond_with(
            ResponseTemplate::new(HTTP_UPGRADE_REQUIRED).set_body_json(fixture("error.upgrade-required.json")),
        )
        .mount(&server)
        .await;
    let home = tempfile::tempdir().expect("temp dir");

    let error = pair::run(home.path(), &pair_options(&server), &SilentConsole)
        .await
        .expect_err("upgrade");

    let shown = error.to_string();
    assert!(shown.contains("requires a newer mootbox"), "{shown}");
    assert!(
        shown.contains("upgrade mootbox (the Mootbox daemon) and restart it"),
        "{shown}"
    );
}
