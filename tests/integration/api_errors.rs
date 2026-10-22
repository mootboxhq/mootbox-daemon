use crate::common::{CREDENTIAL, authenticated_route, fixture};
use mootbox_daemon::api::{ApiError, MootboxClient};
use mootbox_daemon::contracts::{ErrorErrorCategory, UpsertConnectionRequest};
use reqwest::Url;
use serde_json::json;
use uuid::Uuid;
use wiremock::{MockServer, ResponseTemplate};

const HTTP_UPGRADE_REQUIRED: u16 = 426;
const HTTP_CONFLICT: u16 = 409;
const HTTP_BAD_GATEWAY: u16 = 502;

async fn put_connection_answered_with(response: ResponseTemplate) -> Result<(), ApiError> {
    let server = MockServer::start().await;
    authenticated_route("PUT", "/runner/connections")
        .respond_with(response)
        .mount(&server)
        .await;
    let client = MootboxClient::authenticated(Url::parse(&server.uri()).expect("URL"), CREDENTIAL.to_string());
    let request = UpsertConnectionRequest {
        repository_id: Uuid::new_v4(),
        project_id: Uuid::new_v4(),
        local_key: "0".repeat(64).try_into().expect("local key"),
        branch: "main".try_into().expect("branch"),
        head_sha: "0".repeat(40).try_into().expect("head"),
        dirty: false,
    };
    client.upsert_connection(&request).await.map(|_| ())
}

#[tokio::test]
async fn upgrade_required_surfaces_the_server_text() {
    let body = fixture("error.upgrade-required.json");
    let server_text = body["error"]["message"].as_str().expect("message").to_string();

    let error = put_connection_answered_with(ResponseTemplate::new(HTTP_UPGRADE_REQUIRED).set_body_json(body))
        .await
        .expect_err("upgrade required");

    assert!(matches!(&error, ApiError::UpgradeRequired { message } if *message == server_text));
    let shown = error.to_string();
    assert!(shown.contains("requires a newer mootbox"), "{shown}");
    assert!(shown.contains(&server_text), "{shown}");
}

#[tokio::test]
async fn mootbox_error_bodies_keep_status_category_and_message() {
    let body = fixture("error.lease-lost.json");

    let error = put_connection_answered_with(ResponseTemplate::new(HTTP_CONFLICT).set_body_json(body))
        .await
        .expect_err("lease lost");

    let ApiError::Mootbox {
        status,
        category,
        message,
    } = error
    else {
        panic!("expected a mootbox error, got {error:?}");
    };
    assert_eq!(status, HTTP_CONFLICT);
    assert_eq!(category, ErrorErrorCategory::LeaseLost);
    assert_eq!(message, "run generation 1 is no longer current");
}

#[tokio::test]
async fn unreadable_error_bodies_report_the_http_status() {
    let error =
        put_connection_answered_with(ResponseTemplate::new(HTTP_BAD_GATEWAY).set_body_string("<html>bad gateway"))
            .await
            .expect_err("bad gateway");

    assert!(matches!(
        error,
        ApiError::UnreadableError {
            status: HTTP_BAD_GATEWAY
        }
    ));
}

#[tokio::test]
async fn responses_with_unknown_fields_are_accepted() {
    let mut body = json!({ "connection_id": Uuid::new_v4() });
    body["added_by_a_newer_server"] = json!({ "anything": true });

    let outcome = put_connection_answered_with(ResponseTemplate::new(200).set_body_json(body)).await;

    assert!(outcome.is_ok(), "{outcome:?}");
}

#[tokio::test]
async fn success_bodies_that_break_the_contract_are_reported() {
    let outcome =
        put_connection_answered_with(ResponseTemplate::new(200).set_body_json(json!({ "connection_id": "backend" })))
            .await;

    assert!(
        matches!(outcome, Err(ApiError::UnexpectedBody { status: 200, .. })),
        "{outcome:?}"
    );
}
