use super::*;

#[tokio::test]
async fn add_second_workspace_keeps_first_profile() {
    let first_server = MockServer::start().await;
    let second_server = MockServer::start().await;
    let (home, before) = paired_home(&first_server.uri());
    let repository = checkout_repository("checkout", ORIGIN);
    mount_pairing(&second_server).await;
    mount_match(
        &second_server,
        json!({ "kind": "one", "match": repo_match(Uuid::new_v4(), Uuid::new_v4(), "Web App") }),
    )
    .await;
    mount_connection(&second_server, Uuid::new_v4()).await;

    add::connect(
        home.path(),
        options(&second_server, &repository.path),
        &ScriptedConsole::default(),
    )
    .await
    .expect("adds");

    assert_eq!(
        saved(home.path()).workspaces,
        [before.workspaces[0].clone(), paired_profile(&second_server)]
    );
}

#[tokio::test]
async fn add_new_workspace_on_same_server_pairs_and_keeps_first() {
    let server = MockServer::start().await;
    let (home, before) = paired_home(&server.uri());
    let repository = checkout_repository("checkout", ORIGIN);
    mount_pairing(&server).await;
    mount_match(
        &server,
        json!({ "kind": "one", "match": repo_match(Uuid::new_v4(), Uuid::new_v4(), "Web App") }),
    )
    .await;
    mount_connection(&server, Uuid::new_v4()).await;
    let console = ScriptedConsole::default();
    let mut new_workspace = options(&server, &repository.path);
    new_workspace.new_workspace = true;

    add::connect(home.path(), new_workspace, &console).await.expect("adds");

    let after = saved(home.path());
    assert_eq!(after.workspaces.len(), 2);
    assert_eq!(after.workspaces[0], before.workspaces[0]);
    assert_eq!(after.workspaces[0].credential, CREDENTIAL);
    assert_eq!(after.workspaces[1], paired_profile(&server));
    let link = &console.links()[0];
    let query: Vec<(String, String)> = link.query_pairs().into_owned().collect();
    assert_eq!(link.path(), "/devices/confirm");
    assert_eq!(
        query,
        [
            ("code".to_string(), "KQ7M-4TXR".to_string()),
            ("paired".to_string(), before.workspaces[0].workspace_id.to_string()),
        ]
    );
}

#[tokio::test]
async fn add_prompt_offers_connect_another_workspace() {
    let server = MockServer::start().await;
    let (home, _) = paired_home(&server.uri());
    let repository = checkout_repository("checkout", ORIGIN);
    mount_pairing(&server).await;
    mount_match(
        &server,
        json!({ "kind": "one", "match": repo_match(Uuid::new_v4(), Uuid::new_v4(), "Web App") }),
    )
    .await;
    mount_connection(&server, Uuid::new_v4()).await;
    let console = ScriptedConsole::choosing(&[1]);

    let report = add::connect(home.path(), options(&server, &repository.path), &console)
        .await
        .expect("adds");

    assert_eq!(
        console.questions()[0],
        (
            WHICH_WORKSPACE.to_string(),
            vec![PAIRED_WORKSPACE_NAME.to_string(), CONNECT_ANOTHER_WORKSPACE.to_string()]
        )
    );
    assert_eq!(console.links().len(), 1);
    assert_eq!(report.workspace_id, paired_profile(&server).workspace_id);
    assert_eq!(Some(report.workspace_name), paired_profile(&server).workspace_name);
}

#[tokio::test]
async fn the_first_add_pairs_without_asking_for_a_workspace() {
    let server = MockServer::start().await;
    let home = tempfile::tempdir().expect("temp dir");
    let repository = checkout_repository("checkout", ORIGIN);
    mount_pairing(&server).await;
    mount_match(
        &server,
        json!({ "kind": "one", "match": repo_match(Uuid::new_v4(), Uuid::new_v4(), "Web App") }),
    )
    .await;
    mount_connection(&server, Uuid::new_v4()).await;
    let console = ScriptedConsole::default();

    add::connect(home.path(), options(&server, &repository.path), &console)
        .await
        .expect("adds");

    assert_eq!(saved(home.path()).workspaces, [paired_profile(&server)]);
    let paired = console.links()[0]
        .query_pairs()
        .find(|(name, _)| name == "paired")
        .map(|(_, value)| value.into_owned());
    assert_eq!(paired.as_deref(), Some(""));
}

#[tokio::test]
async fn an_expired_pairing_shows_the_server_message() {
    let server = MockServer::start().await;
    let home = tempfile::tempdir().expect("temp dir");
    let repository = checkout_repository("checkout", ORIGIN);
    runner_route("POST", "/runner/pairings")
        .respond_with(ResponseTemplate::new(200).set_body_json(fixture(PAIRING_RESPONSE)))
        .mount(&server)
        .await;
    let expired = json!({ "error": { "category": "permission_denied", "message": "pairing expired" } });
    runner_route(
        "POST",
        &format!(
            "/runner/pairings/{}/exchange",
            fixture_uuid(PAIRING_RESPONSE, "/pairing_id")
        ),
    )
    .respond_with(ResponseTemplate::new(HTTP_FORBIDDEN).set_body_json(expired))
    .mount(&server)
    .await;

    let error = add::connect(
        home.path(),
        options(&server, &repository.path),
        &ScriptedConsole::default(),
    )
    .await
    .expect_err("expired");

    assert_eq!(error.to_string(), "pairing expired");
    assert!(!config::config_path(home.path()).exists());
}

#[tokio::test]
async fn explicit_project_pairs_its_workspace_instead_of_reusing_the_old_one() {
    let server = MockServer::start().await;
    let (home, before) = paired_home(&server.uri());
    let repository = checkout_repository("checkout", ORIGIN);
    let project_id = Uuid::new_v4();
    authenticated_route("POST", MATCH_ROUTE)
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "kind": "none", "registrable": [] })))
        .mount(&server)
        .await;
    let paired = paired_profile(&server);
    wiremock::Mock::given(wiremock::matchers::method("POST"))
        .and(wiremock::matchers::path(MATCH_ROUTE))
        .and(wiremock::matchers::header(
            "authorization",
            format!("Bearer {}", paired.credential),
        ))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(
                json!({ "kind": "one", "match": repo_match(Uuid::new_v4(), project_id, "New project") }),
            ),
        )
        .with_priority(1)
        .mount(&server)
        .await;
    mount_pairing(&server).await;
    mount_connection(&server, Uuid::new_v4()).await;
    let console = ScriptedConsole::default();
    let mut request = options(&server, &repository.path);
    request.project = Some(project_id);
    let report = add::connect(home.path(), request, &console)
        .await
        .expect("connects the requested workspace");
    assert_eq!(report.workspace_id, paired.workspace_id);
    assert_eq!(report.project_name, "New project");
    assert_eq!(saved(home.path()).workspaces[0], before.workspaces[0]);
    assert_eq!(console.links().len(), 1);
}

#[tokio::test]
async fn explicit_workspace_pairs_only_the_requested_workspace() {
    let server = MockServer::start().await;
    let (home, before) = paired_home(&server.uri());
    let repository = checkout_repository("checkout", ORIGIN);
    let paired = paired_profile(&server);
    let project_id = Uuid::new_v4();
    mount_pairing(&server).await;
    mount_match(
        &server,
        json!({ "kind": "one", "match": repo_match(Uuid::new_v4(), project_id, "New project") }),
    )
    .await;
    mount_connection(&server, Uuid::new_v4()).await;
    let console = ScriptedConsole::default();
    let mut request = options(&server, &repository.path);
    request.project = Some(project_id);
    request.workspace = Some(paired.workspace_id);
    let report = add::connect(home.path(), request, &console)
        .await
        .expect("pairs the requested workspace");
    assert_eq!(report.workspace_id, paired.workspace_id);
    assert_eq!(saved(home.path()).workspaces[0], before.workspaces[0]);
    assert!(
        console.links()[0]
            .query_pairs()
            .any(|(key, value)| key == "workspace" && value == paired.workspace_id.to_string())
    );
}

#[tokio::test]
async fn explicit_workspace_wins_over_an_existing_checkout_in_another_workspace() {
    let server = MockServer::start().await;
    let (home, mut before) = paired_home(&server.uri());
    let repository = checkout_repository("checkout", ORIGIN);
    let target = paired_profile(&server);
    crate::common::connect_checkout(&mut before, Uuid::new_v4(), repository.path.clone());
    before.upsert_workspace(target.clone());
    config::save(home.path(), &before).expect("saves both workspaces");
    let project_id = Uuid::new_v4();
    mount_match(
        &server,
        json!({ "kind": "one", "match": repo_match(Uuid::new_v4(), project_id, "Target project") }),
    )
    .await;
    mount_connection(&server, Uuid::new_v4()).await;
    let console = ScriptedConsole::default();
    let mut request = options(&server, &repository.path);
    request.project = Some(project_id);
    request.workspace = Some(target.workspace_id);
    let report = add::connect(home.path(), request, &console)
        .await
        .expect("uses explicit workspace");
    assert_eq!(report.workspace_id, target.workspace_id);
    assert_eq!(console.questions(), Vec::<(String, Vec<String>)>::new());
    assert_eq!(console.links(), Vec::<Url>::new());
}
