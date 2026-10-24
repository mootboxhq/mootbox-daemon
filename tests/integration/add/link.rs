use mootbox_daemon::commands::add::link::{self, LinkOptions, LinkReport};
use mootbox_daemon::commands::login::{self, LoginOptions};
use mootbox_daemon::config::CliSession;
use wiremock::matchers::header;

use super::*;

const LOGIN_DEVICE_NAME: &str = "maya-laptop";
const LOGIN_RESPONSE: &str = "start-cli-login-response.json";
const SIGNED_IN_RESPONSE: &str = "exchange-cli-login-response.signed-in.json";
const LINKED_RESPONSE: &str = "start-repository-link-response.linked.json";
const LINKED_WITH_RUNNER_RESPONSE: &str = "start-repository-link-response.linked-with-runner.json";
const CHOOSE_RESPONSE: &str = "start-repository-link-response.choose.json";
const EXCHANGED_LINK_RESPONSE: &str = "exchange-repository-link-response.linked.json";
const LINKS_ROUTE: &str = "/runner/links";
const SAVED_TOKEN: &str = "mootbox_cli_saved_session";
const SIGNED_IN_EMAIL: &str = "maya@example.com";
const HTTP_UNAUTHORIZED: u16 = 401;

fn mootbox_url(server: &MockServer) -> Url {
    Url::parse(&server.uri()).expect("URL")
}

fn login_options(server: &MockServer) -> LoginOptions {
    LoginOptions {
        mootbox_url: Some(mootbox_url(server)),
        device_name: LOGIN_DEVICE_NAME.to_string(),
        poll_interval: FAST_POLL,
    }
}

fn link_options(server: &MockServer, directory: &Path) -> LinkOptions {
    LinkOptions {
        login: login_options(server),
        directory: directory.to_path_buf(),
    }
}

fn fixture_text(name: &str, pointer: &str) -> String {
    fixture(name)
        .pointer(pointer)
        .and_then(Value::as_str)
        .expect("fixture field is text")
        .to_string()
}

fn signed_in_token() -> String {
    fixture_text(SIGNED_IN_RESPONSE, "/session_token")
}

fn fake_provider_config() -> DaemonConfig {
    DaemonConfig {
        provider_command: Some(Path::new(env!("CARGO_MANIFEST_DIR")).join(FAKE_PROVIDER)),
        ..DaemonConfig::default()
    }
}

fn home_with(config: &DaemonConfig) -> tempfile::TempDir {
    let home = tempfile::tempdir().expect("temp dir");
    config::save(home.path(), config).expect("saves config");
    home
}

fn saved_session(server: &MockServer) -> CliSession {
    CliSession {
        mootbox_url: mootbox_url(server).to_string(),
        token: SAVED_TOKEN.to_string(),
        email: SIGNED_IN_EMAIL.to_string(),
    }
}

fn signed_in_config(server: &MockServer) -> DaemonConfig {
    DaemonConfig {
        session: Some(saved_session(server)),
        ..fake_provider_config()
    }
}

fn minted_profile(server: &MockServer) -> WorkspaceProfile {
    WorkspaceProfile {
        mootbox_url: mootbox_url(server).to_string(),
        workspace_id: fixture_uuid(LINKED_WITH_RUNNER_RESPONSE, "/runner/workspace_id"),
        runner_id: fixture_uuid(LINKED_WITH_RUNNER_RESPONSE, "/runner/runner_id"),
        credential: fixture_text(LINKED_WITH_RUNNER_RESPONSE, "/runner/credential"),
        workspace_name: Some(fixture_text(LINKED_WITH_RUNNER_RESPONSE, "/runner/workspace_name")),
    }
}

fn expected_lines() -> Vec<String> {
    vec![
        "Linked to “Checkout” in Acme.".to_string(),
        String::new(),
        format!("Start chatting: {}", fixture_text(LINKED_RESPONSE, "/chat_url")),
    ]
}

fn linked_connection(folder: &Folder, workspace_id: Uuid) -> Connection {
    Connection {
        workspace_id,
        repository_id: fixture_uuid(LINKED_RESPONSE, "/repository_id"),
        project_ids: BTreeSet::from([fixture_uuid(LINKED_RESPONSE, "/project/id")]),
        path: folder.path.clone(),
        remote: Some("origin".to_string()),
    }
}

async fn mount_login(server: &MockServer) {
    runner_route("POST", "/runner/logins")
        .and(body_json(fixture("start-cli-login-request.json")))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture(LOGIN_RESPONSE)))
        .expect(1)
        .mount(server)
        .await;
    let exchange = format!("/runner/logins/{}/exchange", fixture_uuid(LOGIN_RESPONSE, "/login_id"));
    runner_route("POST", &exchange)
        .and(body_json(fixture("exchange-cli-login-request.json")))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture("exchange-cli-login-response.pending.json")))
        .up_to_n_times(1)
        .mount(server)
        .await;
    runner_route("POST", &exchange)
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture(SIGNED_IN_RESPONSE)))
        .mount(server)
        .await;
}

async fn mount_link_start(server: &MockServer, token: &str, response: &str) {
    runner_route("POST", LINKS_ROUTE)
        .and(header("authorization", format!("Bearer {token}").as_str()))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture(response)))
        .expect(1)
        .mount(server)
        .await;
}

async fn mount_minted_connection(server: &MockServer, connection_id: Uuid) {
    let credential = fixture_text(LINKED_WITH_RUNNER_RESPONSE, "/runner/credential");
    runner_route("PUT", CONNECTIONS_ROUTE)
        .and(header("authorization", format!("Bearer {credential}").as_str()))
        .and(body_partial_json(json!({
            "repository_id": fixture_uuid(LINKED_RESPONSE, "/repository_id"),
            "project_id": fixture_uuid(LINKED_RESPONSE, "/project/id"),
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "connection_id": connection_id })))
        .expect(1)
        .mount(server)
        .await;
}

fn confirm_url() -> Url {
    Url::parse(&fixture_text(LOGIN_RESPONSE, "/confirm_url")).expect("URL")
}

fn assert_linked(report: &LinkReport) {
    assert_eq!(report.lines(), expected_lines());
}

#[tokio::test]
async fn login_opens_the_confirm_page_polls_and_saves_the_session() {
    let server = MockServer::start().await;
    mount_login(&server).await;
    let home = home_with(&fake_provider_config());
    let console = ScriptedConsole::default();

    login::run(home.path(), &login_options(&server), &console)
        .await
        .expect("signs in");

    assert_eq!(console.links(), [confirm_url()]);
    assert_eq!(console.shown(), [format!("Signed in as {SIGNED_IN_EMAIL}.")]);
    assert_eq!(
        saved(home.path()).session,
        Some(CliSession {
            mootbox_url: mootbox_url(&server).to_string(),
            token: signed_in_token(),
            email: SIGNED_IN_EMAIL.to_string(),
        })
    );
}

#[tokio::test]
async fn a_signed_out_mootbox_signs_in_first_then_links_the_checkout() {
    let server = MockServer::start().await;
    mount_login(&server).await;
    mount_link_start(&server, &signed_in_token(), LINKED_WITH_RUNNER_RESPONSE).await;
    let connection_id = Uuid::new_v4();
    mount_minted_connection(&server, connection_id).await;
    let home = home_with(&fake_provider_config());
    let repository = checkout_repository("checkout", ORIGIN);
    let console = ScriptedConsole::default();

    let report = link::link(home.path(), &link_options(&server, &repository.path), &console)
        .await
        .expect("links");

    assert_linked(&report);
    assert_eq!(console.links(), [confirm_url()]);
    let after = saved(home.path());
    assert_eq!(after.session.map(|session| session.token), Some(signed_in_token()));
    assert_eq!(after.workspaces, [minted_profile(&server)]);
    let workspace_id = minted_profile(&server).workspace_id;
    assert_eq!(
        connection_of(&saved(home.path()), connection_id),
        &linked_connection(&repository, workspace_id)
    );
}

#[tokio::test]
async fn a_repository_already_linked_to_a_paired_workspace_connects_without_a_browser() {
    let server = MockServer::start().await;
    mount_link_start(&server, SAVED_TOKEN, LINKED_RESPONSE).await;
    let connection_id = Uuid::new_v4();
    authenticated_route("PUT", CONNECTIONS_ROUTE)
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "connection_id": connection_id })))
        .expect(1)
        .mount(&server)
        .await;
    let mut before = signed_in_config(&server);
    let paired = WorkspaceProfile {
        credential: CREDENTIAL.to_string(),
        ..minted_profile(&server)
    };
    before.upsert_workspace(paired.clone());
    let home = home_with(&before);
    let repository = checkout_repository("checkout", ORIGIN);
    let console = ScriptedConsole::default();

    let report = link::link(home.path(), &link_options(&server, &repository.path), &console)
        .await
        .expect("links");

    assert_linked(&report);
    assert_eq!(console.links(), Vec::<Url>::new());
    let after = saved(home.path());
    assert_eq!(after.workspaces, std::slice::from_ref(&paired));
    assert_eq!(
        connection_of(&after, connection_id),
        &linked_connection(&repository, paired.workspace_id)
    );
    let started = requests_to(&server, LINKS_ROUTE).await;
    assert_eq!(started[0]["paired_workspace_ids"], json!([paired.workspace_id]));
}

#[tokio::test]
async fn an_unlinked_repository_opens_the_choose_page_and_waits_for_a_project() {
    let server = MockServer::start().await;
    mount_link_start(&server, SAVED_TOKEN, CHOOSE_RESPONSE).await;
    let exchange = format!("{LINKS_ROUTE}/{}/exchange", fixture_uuid(CHOOSE_RESPONSE, "/link_id"));
    runner_route("POST", &exchange)
        .and(header("authorization", format!("Bearer {SAVED_TOKEN}").as_str()))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(fixture("exchange-repository-link-response.pending.json")),
        )
        .up_to_n_times(1)
        .mount(&server)
        .await;
    runner_route("POST", &exchange)
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture(EXCHANGED_LINK_RESPONSE)))
        .mount(&server)
        .await;
    let connection_id = Uuid::new_v4();
    mount_minted_connection(&server, connection_id).await;
    let home = home_with(&signed_in_config(&server));
    let repository = checkout_repository("checkout", ORIGIN);
    git(
        &repository.path,
        &[
            "remote",
            "add",
            "fork",
            "https://user:token@github.com/maya/checkout.git",
        ],
    );
    let console = ScriptedConsole::default();

    let report = link::link(home.path(), &link_options(&server, &repository.path), &console)
        .await
        .expect("links");

    assert_linked(&report);
    let choose_url = Url::parse(&fixture_text(CHOOSE_RESPONSE, "/choose_url")).expect("URL");
    assert_eq!(console.links(), [choose_url]);
    let started = requests_to(&server, LINKS_ROUTE).await;
    assert_eq!(
        started[0]["remotes"],
        json!([
            { "name": "origin", "url": ORIGIN },
            { "name": "fork", "url": "https://github.com/maya/checkout.git" },
        ])
    );
    assert_eq!(started[0]["name"], "checkout");
    assert_eq!(started[0]["device"]["name"], LOGIN_DEVICE_NAME);
    let polls: Vec<_> = server
        .received_requests()
        .await
        .expect("recording")
        .into_iter()
        .filter(|request| request.url.path() == exchange)
        .collect();
    assert_eq!(polls.len(), 2);
    assert!(polls.iter().all(|poll| poll.body.is_empty()));
    assert!(polls.iter().all(|poll| !poll.headers.contains_key("content-type")));
    assert_eq!(
        connection_of(&saved(home.path()), connection_id),
        &linked_connection(&repository, minted_profile(&server).workspace_id)
    );
}

#[tokio::test]
async fn a_revoked_session_is_forgotten_and_mootbox_signs_in_again() {
    let server = MockServer::start().await;
    runner_route("POST", LINKS_ROUTE)
        .and(header("authorization", format!("Bearer {SAVED_TOKEN}").as_str()))
        .respond_with(ResponseTemplate::new(HTTP_UNAUTHORIZED).set_body_json(fixture("error.cli-session-invalid.json")))
        .expect(1)
        .mount(&server)
        .await;
    mount_login(&server).await;
    mount_link_start(&server, &signed_in_token(), LINKED_WITH_RUNNER_RESPONSE).await;
    mount_minted_connection(&server, Uuid::new_v4()).await;
    let home = home_with(&signed_in_config(&server));
    let repository = checkout_repository("checkout", ORIGIN);
    let console = ScriptedConsole::default();

    let report = link::link(home.path(), &link_options(&server, &repository.path), &console)
        .await
        .expect("links");

    assert_linked(&report);
    assert_eq!(console.links(), [confirm_url()]);
    assert_eq!(
        saved(home.path()).session.map(|session| session.token),
        Some(signed_in_token())
    );
}

#[tokio::test]
async fn a_session_refused_twice_tells_the_user_to_sign_in() {
    let server = MockServer::start().await;
    runner_route("POST", LINKS_ROUTE)
        .respond_with(ResponseTemplate::new(HTTP_UNAUTHORIZED).set_body_json(fixture("error.cli-session-invalid.json")))
        .expect(2)
        .mount(&server)
        .await;
    mount_login(&server).await;
    let home = home_with(&signed_in_config(&server));
    let repository = checkout_repository("checkout", ORIGIN);

    let error = link::link(
        home.path(),
        &link_options(&server, &repository.path),
        &ScriptedConsole::default(),
    )
    .await
    .expect_err("refused");

    assert_eq!(
        error.to_string(),
        "Mootbox CLI session is invalid or revoked; run mootbox login"
    );
}

#[tokio::test]
async fn bare_mootbox_prints_the_link_and_brings_the_computer_online_in_the_background() {
    let server = MockServer::start().await;
    mount_online_routes(&server, Uuid::new_v4()).await;
    mount_link_start(&server, SAVED_TOKEN, LINKED_WITH_RUNNER_RESPONSE).await;
    let home = home_with(&signed_in_config(&server));
    let repository = checkout_repository("checkout", ORIGIN);
    let mut command = mootbox_command(home.path(), &repository.path, &[]);
    command.env(SERVICE_MANAGER_VARIABLE, ACCEPTING_MANAGER);

    let output = output_of(command).await;

    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!(
            "{}\n● This computer is online in the background\n{BACKGROUND_HINT}\n",
            expected_lines().join("\n")
        )
    );
    assert!(String::from_utf8_lossy(&output.stderr).is_empty(), "{output:?}");
    let after = saved(home.path());
    assert_eq!(after.workspaces, [minted_profile(&server)]);
    assert_eq!(after.connections.len(), 1);
}

#[tokio::test]
async fn bare_mootbox_outside_a_repository_says_where_to_run_it() {
    let server = MockServer::start().await;
    let home = home_with(&signed_in_config(&server));
    let folder = plain_folder("notes");

    let output = output_of(mootbox_command(home.path(), &folder.path, &[])).await;

    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        format!("mootbox: {NOT_IN_REPOSITORY}\n")
    );
    assert!(server.received_requests().await.expect("recording").is_empty());
}

#[tokio::test]
async fn a_minted_runner_is_saved_even_when_connecting_the_checkout_fails() {
    let server = MockServer::start().await;
    mount_link_start(&server, SAVED_TOKEN, LINKED_WITH_RUNNER_RESPONSE).await;
    runner_route("PUT", CONNECTIONS_ROUTE)
        .respond_with(ResponseTemplate::new(HTTP_FORBIDDEN).set_body_json(fixture("error.repository-not-found.json")))
        .expect(1)
        .mount(&server)
        .await;
    let home = home_with(&signed_in_config(&server));
    let repository = checkout_repository("checkout", ORIGIN);

    link::link(
        home.path(),
        &link_options(&server, &repository.path),
        &ScriptedConsole::default(),
    )
    .await
    .expect_err("connecting fails");

    let after = saved(home.path());
    assert_eq!(after.workspaces, [minted_profile(&server)]);
    assert!(after.connections.is_empty());
}

#[tokio::test]
async fn login_without_a_session_signs_in_to_the_server_of_the_saved_workspaces() {
    let server = MockServer::start().await;
    mount_login(&server).await;
    let mut before = fake_provider_config();
    before.upsert_workspace(minted_profile(&server));
    let home = home_with(&before);
    let options = LoginOptions {
        mootbox_url: None,
        ..login_options(&server)
    };

    login::run(home.path(), &options, &ScriptedConsole::default())
        .await
        .expect("signs in");

    assert_eq!(
        saved(home.path()).session.map(|session| session.mootbox_url),
        Some(mootbox_url(&server).to_string())
    );
}

#[tokio::test]
async fn a_revoked_saved_runner_is_forgotten_and_the_link_mints_a_fresh_one() {
    let server = MockServer::start().await;
    runner_route("POST", LINKS_ROUTE)
        .and(header("authorization", format!("Bearer {SAVED_TOKEN}").as_str()))
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture(LINKED_RESPONSE)))
        .up_to_n_times(1)
        .expect(1)
        .mount(&server)
        .await;
    mount_link_start(&server, SAVED_TOKEN, LINKED_WITH_RUNNER_RESPONSE).await;
    authenticated_route("PUT", CONNECTIONS_ROUTE)
        .respond_with(ResponseTemplate::new(HTTP_UNAUTHORIZED).set_body_json(json!({
            "error": { "category": "permission_denied", "message": "runner credential is invalid or revoked" }
        })))
        .expect(1)
        .mount(&server)
        .await;
    let connection_id = Uuid::new_v4();
    mount_minted_connection(&server, connection_id).await;
    let mut before = signed_in_config(&server);
    let revoked = WorkspaceProfile {
        credential: CREDENTIAL.to_string(),
        ..minted_profile(&server)
    };
    before.upsert_workspace(revoked.clone());
    let home = home_with(&before);
    let repository = checkout_repository("checkout", ORIGIN);
    let console = ScriptedConsole::default();

    let report = link::link(home.path(), &link_options(&server, &repository.path), &console)
        .await
        .expect("links");

    assert_linked(&report);
    assert_eq!(
        console.warned(),
        [format!(
            "This computer's access to {} was removed; reconnecting.",
            revoked.display_name()
        )]
    );
    let started = requests_to(&server, LINKS_ROUTE).await;
    assert_eq!(started.len(), 2);
    assert_eq!(started[0]["paired_workspace_ids"], json!([revoked.workspace_id]));
    assert!(started[1].get("paired_workspace_ids").is_none());
    let after = saved(home.path());
    assert_eq!(after.workspaces, [minted_profile(&server)]);
    assert_eq!(
        connection_of(&after, connection_id),
        &linked_connection(&repository, revoked.workspace_id)
    );
}
