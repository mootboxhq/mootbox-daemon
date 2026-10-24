use super::*;

#[tokio::test]
async fn a_rejected_credential_fails_the_server_check() {
    let server = MockServer::start().await;
    runner_route("GET", "/runner/health")
        .respond_with(ResponseTemplate::new(HTTP_UNAUTHORIZED).set_body_json(json!({
            "error": { "category": "permission_denied", "message": "runner credential is invalid or revoked", "retryable": false }
        })))
        .mount(&server)
        .await;
    let (machine, _) = Machine::paired(&server.uri());

    let report = machine.doctor().await;

    assert!(!report.is_success());
    assert!(
        report
            .line("server")
            .starts_with("fail server: Mootbox refused the request (permission_denied, HTTP 401)"),
        "{}",
        report.line("server")
    );
    assert!(!report.has_subject("protocol"));
    assert!(!report.has_subject("clock"));
}

#[tokio::test]
async fn a_server_that_requires_a_newer_protocol_fails_the_protocol_check() {
    let server = MockServer::start().await;
    let upgrade = fixture("error.upgrade-required.json");
    let server_text = upgrade["error"]["message"].as_str().expect("message").to_string();
    runner_route("GET", "/runner/health")
        .respond_with(ResponseTemplate::new(HTTP_UPGRADE_REQUIRED).set_body_json(upgrade))
        .mount(&server)
        .await;
    let (machine, _) = Machine::paired(&server.uri());

    let report = machine.doctor().await;

    assert!(!report.is_success());
    assert!(report.line("server").starts_with("ok   server: "));
    assert_eq!(
        report.line("protocol"),
        format!("fail protocol: runner protocol 9 refused: {server_text}; run mootbox update")
    );
}

#[tokio::test]
async fn an_unreachable_server_fails_the_server_check() {
    let (machine, _) = Machine::paired(UNREACHABLE_MOOTBOX);

    let report = machine.doctor().await;

    assert!(!report.is_success());
    assert!(
        report.line("server").starts_with("fail server: cannot reach Mootbox"),
        "{}",
        report.line("server")
    );
}

#[tokio::test]
async fn an_unpaired_machine_fails_the_config_check_and_skips_the_server() {
    let machine = Machine::unpaired();

    let report = machine.doctor().await;

    assert!(!report.is_success());
    assert!(
        report
            .line("config")
            .contains("run mootbox add --mootbox-url <url> inside your repo folder"),
        "{}",
        report.line("config")
    );
    assert!(report.line("claude-code").starts_with("ok "));
    assert!(!report.has_subject("server"));
}

#[tokio::test]
async fn a_missing_git_fails_the_git_check() {
    let server = accepting_server().await;
    let (mut machine, _) = Machine::paired(&server.uri());
    let empty = tempfile::tempdir().expect("temp dir");
    machine.search_path = Some(empty.path().to_path_buf());

    let report = machine.doctor().await;

    assert!(!report.is_success());
    assert!(
        report.line("git").starts_with("fail git: cannot run git"),
        "{}",
        report.line("git")
    );
    assert!(report.line("claude-code").starts_with("ok "));
}
