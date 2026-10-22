use mootbox_daemon::commands::add::ALREADY_RUNNING;
use mootbox_daemon::config::{self, WorkspaceProfile};
use mootbox_daemon::lock;
use serde_json::json;
use uuid::Uuid;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer};

use crate::add::{
    ACCEPTING_MANAGER, SERVICE_FOLDER, checkout_repository, mootbox_command, mount_connection, mount_match, output_of,
    repo_match,
};
use crate::common::{CREDENTIAL, checkout_of, connect_checkout};
use crate::harness::{
    FakeProvider, TestDaemon, bound_home, claimed, committed_repository, completion_of, daemon_options, execute_claim,
    git, healthy_heartbeats, mount_claim_once, mount_mootbox, requests_to, wait_for_claims, wait_until,
};
use mootbox_daemon::service;

const SECOND_CREDENTIAL: &str = "mootbox_runner_second_workspace";
const SAVED_CONFIG_CLAIMS: usize = 2;
const CLAIMS_AFTER_REMOVAL: usize = 4;

async fn claims_by(server: &MockServer, credential: &str) -> usize {
    let bearer = format!("Bearer {credential}");
    server
        .received_requests()
        .await
        .expect("recording")
        .iter()
        .filter(|request| request.url.path() == "/runner/claim")
        .filter(|request| {
            request
                .headers
                .get("authorization")
                .is_some_and(|value| value.as_bytes() == bearer.as_bytes())
        })
        .count()
}

async fn mount_claim_for(server: &MockServer, credential: &str, body: serde_json::Value) {
    Mock::given(method("POST"))
        .and(path("/runner/claim"))
        .and(header("authorization", format!("Bearer {credential}").as_str()))
        .respond_with(claimed(body))
        .up_to_n_times(1)
        .with_priority(1)
        .mount(server)
        .await;
}

fn second_workspace(server: &MockServer) -> WorkspaceProfile {
    WorkspaceProfile {
        mootbox_url: server.uri(),
        workspace_id: Uuid::new_v4(),
        runner_id: Uuid::new_v4(),
        credential: SECOND_CREDENTIAL.to_string(),
        workspace_name: None,
    }
}

#[tokio::test]
async fn add_while_running_keeps_active_run() {
    let server = MockServer::start().await;
    let mut bound = bound_home(&server);
    bound.config.slots = 2;
    config::save(&bound.home, &bound.config).expect("saves config");
    let running_daemon = lock::acquire(&bound.home).expect("the daemon holds the lock");
    let (first, second) = (Uuid::new_v4(), Uuid::new_v4());
    mount_claim_once(&server, claimed(execute_claim(first, bound.repository_id))).await;
    mount_mootbox(&server, healthy_heartbeats()).await;
    let provider = FakeProvider::claude("gated");
    let daemon = TestDaemon::start(daemon_options(bound.config.clone(), &bound.home, &provider));
    provider.grandchild().await;
    let payments = checkout_repository("payments", "git@github.com:acme/payments.git");
    git(&payments.path, &["commit", "--quiet", "--allow-empty", "-m", "init"]);
    let (payments_id, project_id) = (Uuid::new_v4(), Uuid::new_v4());
    mount_match(
        &server,
        json!({ "kind": "one", "match": repo_match(payments_id, project_id, "Payments") }),
    )
    .await;
    mount_connection(&server, payments_id).await;
    let project = project_id.to_string();
    let mut add = mootbox_command(&bound.home, &payments.path, &["add", "--project", &project]);
    add.env(service::SERVICE_MANAGER_VARIABLE, ACCEPTING_MANAGER);

    let added = output_of(add).await;
    let first_finished_during_add = !requests_to(&server, &format!("/runner/runs/{first}/complete"))
        .await
        .is_empty();
    provider.release();
    mount_claim_once(&server, claimed(execute_claim(second, payments_id))).await;
    let second_completion = completion_of(&server, second).await;
    let first_completion = completion_of(&server, first).await;
    daemon.stop().await;

    assert!(added.status.success(), "{added:?}");
    assert!(
        String::from_utf8_lossy(&added.stdout).ends_with(&format!("{ALREADY_RUNNING}\n")),
        "{added:?}"
    );
    assert!(
        !bound.home.join(SERVICE_FOLDER).exists(),
        "a second daemon was installed"
    );
    assert!(lock::acquire(&bound.home).is_err(), "the running daemon lost its lock");
    drop(running_daemon);
    assert!(!first_finished_during_add, "the first run did not outlast mootbox add");
    assert_eq!(first_completion["outcome"], "succeeded", "{first_completion}");
    assert_eq!(second_completion["outcome"], "succeeded", "{second_completion}");
    let worktree = bound.home.join("worktrees").join(format!("mootbox-{second}"));
    assert_eq!(
        git(&worktree, &["remote", "get-url", "origin"]),
        "git@github.com:acme/payments.git"
    );
}

#[tokio::test]
async fn each_workspace_claims_with_its_own_credential_and_routes_to_its_checkouts() {
    let server = MockServer::start().await;
    let mut bound = bound_home(&server);
    let second = second_workspace(&server);
    bound.config.upsert_workspace(second.clone());
    let (other_directory, other_repository) = committed_repository();
    let other_repository_id = Uuid::new_v4();
    connect_checkout(&mut bound.config, other_repository_id, other_repository.clone());
    checkout_of(&mut bound.config, other_repository_id).workspace_id = second.workspace_id;
    let run_id = Uuid::new_v4();
    mount_claim_for(&server, SECOND_CREDENTIAL, execute_claim(run_id, other_repository_id)).await;
    mount_mootbox(&server, healthy_heartbeats()).await;
    let provider = FakeProvider::claude("success");
    let daemon = TestDaemon::start(daemon_options(bound.config.clone(), &bound.home, &provider));

    let completion = completion_of(&server, run_id).await;
    wait_for_claims(&server, 4).await;
    daemon.stop().await;

    assert_eq!(completion["outcome"], "succeeded", "{completion}");
    assert!(claims_by(&server, CREDENTIAL).await > 0);
    assert!(claims_by(&server, SECOND_CREDENTIAL).await > 0);
    let worktree = bound.home.join("worktrees").join(format!("mootbox-{run_id}"));
    assert_eq!(
        provider.only_invocation().working_directory,
        worktree.canonicalize().expect("worktree")
    );
    assert_eq!(
        git(&worktree, &["rev-parse", "HEAD"]),
        git(&other_repository, &["rev-parse", "HEAD"])
    );
    drop(other_directory);
}

#[tokio::test]
async fn a_run_of_one_workspace_is_not_routed_to_another_workspaces_checkout() {
    let server = MockServer::start().await;
    let mut bound = bound_home(&server);
    let second = second_workspace(&server);
    bound.config.upsert_workspace(second);
    let run_id = Uuid::new_v4();
    mount_claim_for(&server, SECOND_CREDENTIAL, execute_claim(run_id, bound.repository_id)).await;
    mount_mootbox(&server, healthy_heartbeats()).await;
    let provider = FakeProvider::claude("success");
    let daemon = TestDaemon::start(daemon_options(bound.config.clone(), &bound.home, &provider));

    let completion = completion_of(&server, run_id).await;
    daemon.stop().await;

    assert_eq!(completion["error"], "repository_not_bound_locally");
    assert!(provider.invocations().is_empty());
}

#[tokio::test]
async fn a_removed_workspace_stops_claiming_but_finishes_its_active_run() {
    let server = MockServer::start().await;
    let mut bound = bound_home(&server);
    bound.config.slots = 2;
    let second = second_workspace(&server);
    bound.config.upsert_workspace(second.clone());
    config::save(&bound.home, &bound.config).expect("saves config");
    let run_id = Uuid::new_v4();
    mount_claim_for(&server, CREDENTIAL, execute_claim(run_id, bound.repository_id)).await;
    mount_mootbox(&server, healthy_heartbeats()).await;
    let provider = FakeProvider::claude("gated");
    let daemon = TestDaemon::start(daemon_options(bound.config.clone(), &bound.home, &provider));
    provider.grandchild().await;

    let mut without_first = bound.config.clone();
    without_first
        .workspaces
        .retain(|profile| profile.credential != CREDENTIAL);
    let second_claims_at_save = claims_by(&server, SECOND_CREDENTIAL).await;
    config::save(&bound.home, &without_first).expect("saves config");
    wait_until(|| async {
        (claims_by(&server, SECOND_CREDENTIAL).await >= second_claims_at_save + SAVED_CONFIG_CLAIMS).then_some(())
    })
    .await;
    provider.release();
    let completion = completion_of(&server, run_id).await;
    let first_claims = claims_by(&server, CREDENTIAL).await;
    let second_claims = claims_by(&server, SECOND_CREDENTIAL).await;
    wait_until(|| async {
        (claims_by(&server, SECOND_CREDENTIAL).await >= second_claims + CLAIMS_AFTER_REMOVAL).then_some(())
    })
    .await;
    daemon.stop().await;

    assert_eq!(completion["outcome"], "succeeded", "{completion}");
    assert_eq!(claims_by(&server, CREDENTIAL).await, first_claims);
}
