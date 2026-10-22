use super::*;

#[tokio::test]
async fn add_one_match_creates_connection() {
    let server = MockServer::start().await;
    let (home, before) = paired_home(&server.uri());
    let repository = checkout_repository("checkout", ORIGIN);
    let nested = repository.path.join("src");
    std::fs::create_dir(&nested).expect("creates nested folder");
    let (repository_id, project_id, connection_id) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    authenticated_route("POST", MATCH_ROUTE)
        .and(body_json(json!({ "remotes": [{ "name": "origin", "url": ORIGIN }] })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "kind": "one", "match": repo_match(repository_id, project_id, "Web App") })),
        )
        .expect(1)
        .mount(&server)
        .await;
    authenticated_route("PUT", CONNECTIONS_ROUTE)
        .and(body_json(json!({
            "repository_id": repository_id,
            "project_id": project_id,
            "local_key": sha256_hex(&repository.path),
            "branch": "main",
            "head_sha": harness::git(&repository.path, &["rev-parse", "HEAD"]),
            "dirty": false,
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "connection_id": connection_id })))
        .expect(1)
        .mount(&server)
        .await;
    let console = ScriptedConsole::choosing(&[0]);

    let report = add::connect(home.path(), options(&server, &nested), &console)
        .await
        .expect("adds");

    let after = saved(home.path());
    let workspace_id = before.workspaces[0].workspace_id;
    assert_eq!(after.workspaces, before.workspaces);
    assert_eq!(
        connection_of(&after, connection_id),
        &Connection {
            workspace_id,
            repository_id,
            project_ids: BTreeSet::from([project_id]),
            path: repository.path.clone(),
            remote: Some("origin".to_string()),
        }
    );
    assert_eq!(report.project_name, "Web App");
    assert_eq!(report.team_name, "Frontend");
    assert!(!report.is_daemon_running);
    assert_eq!(
        report.lines()[..4],
        [
            format!(
                "✓ Signed in to {} (workspace {PAIRED_WORKSPACE_NAME})",
                server.address()
            ),
            "✓ Project Frontend / Web App".to_string(),
            "✓ Repo checkout".to_string(),
            format!("✓ Device {DEVICE_NAME}"),
        ]
    );
}

#[tokio::test]
async fn add_many_prompts_and_remembers_choice() {
    let server = MockServer::start().await;
    let (home, _) = paired_home(&server.uri());
    let repository = checkout_repository("checkout", ORIGIN);
    let repository_id = Uuid::new_v4();
    let (web, admin) = (Uuid::new_v4(), Uuid::new_v4());
    let connection_id = Uuid::new_v4();
    mount_match(
        &server,
        json!({ "kind": "many", "matches": [
            repo_match(repository_id, web, "Web App"),
            repo_match(repository_id, admin, "Admin"),
        ] }),
    )
    .await;
    mount_connection(&server, connection_id).await;
    let first = ScriptedConsole::choosing(&[0, 1]);

    add::connect(home.path(), options(&server, &repository.path), &first)
        .await
        .expect("adds");
    let again = ScriptedConsole::default();
    add::connect(home.path(), options(&server, &repository.path), &again)
        .await
        .expect("adds again");

    let asked = first.questions();
    assert_eq!(asked[1].0, WHICH_PROJECT);
    assert_eq!(asked[1].1, ["Frontend / Web App", "Frontend / Admin"]);
    assert!(again.questions().is_empty(), "{:?}", again.questions());
    let puts = requests_to(&server, CONNECTIONS_ROUTE).await;
    assert_eq!(puts.len(), 2);
    assert!(puts.iter().all(|put| put["project_id"] == json!(admin)), "{puts:?}");
    assert_eq!(
        connection_of(&saved(home.path()), connection_id).project_ids,
        BTreeSet::from([admin])
    );
}

#[tokio::test]
async fn add_none_without_authority_explains() {
    let server = MockServer::start().await;
    let (home, before) = paired_home(&server.uri());
    let repository = checkout_repository("checkout", ORIGIN);
    mount_match(&server, json!({ "kind": "none", "registrable": [] })).await;
    let console = ScriptedConsole::choosing(&[0]);

    let error = add::connect(home.path(), options(&server, &repository.path), &console)
        .await
        .expect_err("no authority");

    assert_eq!(error.to_string(), NO_AUTHORITY);
    assert_eq!(saved(home.path()), before);
    assert_eq!(requests_to(&server, CONNECTIONS_ROUTE).await, Vec::<Value>::new());
}

#[tokio::test]
async fn add_registers_the_repository_into_the_chosen_new_project() {
    let server = MockServer::start().await;
    let (home, _) = paired_home(&server.uri());
    let repository = checkout_repository("storefront", "https://lead:secret@github.com/acme/storefront.git");
    let (team_id, existing, created) = (Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4());
    let repository_id = Uuid::new_v4();
    mount_match(
        &server,
        json!({ "kind": "none", "registrable": [
            { "team_id": team_id, "team_name": "Frontend", "projects": [named(existing, "Web App")] },
        ] }),
    )
    .await;
    authenticated_route("POST", REGISTER_ROUTE)
        .and(body_json(json!({
            "remote_url": "https://github.com/acme/storefront.git",
            "name": "storefront",
            "default_branch": "main",
            "new_project": { "team_id": team_id, "name": "Storefront" },
        })))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({ "repository_id": repository_id, "project_id": created })),
        )
        .expect(1)
        .mount(&server)
        .await;
    mount_connection(&server, Uuid::new_v4()).await;
    let console = ScriptedConsole::choosing(&[0, 1]);
    console
        .answers
        .lock()
        .expect("answers")
        .push_back("Storefront".to_string());

    let report = add::connect(home.path(), options(&server, &repository.path), &console)
        .await
        .expect("registers");

    let asked = console.questions();
    assert_eq!(asked[1].1, ["Web App", NEW_PROJECT]);
    assert_eq!(asked[2].1, ["storefront"]);
    assert_eq!(report.project_name, "Storefront");
    let matched = requests_to(&server, MATCH_ROUTE).await;
    assert_eq!(
        matched[0]["remotes"][0]["url"],
        "https://github.com/acme/storefront.git"
    );
    let puts = requests_to(&server, CONNECTIONS_ROUTE).await;
    assert_eq!(puts[0]["repository_id"], json!(repository_id));
    assert_eq!(puts[0]["project_id"], json!(created));
}

#[tokio::test]
async fn add_with_a_project_flag_registers_into_that_project_without_asking() {
    let server = MockServer::start().await;
    let (home, _) = paired_home(&server.uri());
    let repository = checkout_repository("checkout", ORIGIN);
    let (team_id, project_id) = (Uuid::new_v4(), Uuid::new_v4());
    mount_match(
        &server,
        json!({ "kind": "none", "registrable": [
            { "team_id": team_id, "team_name": "Frontend", "projects": [named(project_id, "Web App")] },
        ] }),
    )
    .await;
    authenticated_route("POST", REGISTER_ROUTE)
        .and(body_partial_json(json!({ "project_id": project_id })))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "repository_id": Uuid::new_v4(), "project_id": project_id })),
        )
        .expect(1)
        .mount(&server)
        .await;
    mount_connection(&server, Uuid::new_v4()).await;
    let console = ScriptedConsole::default();
    let mut flagged = options(&server, &repository.path);
    flagged.project = Some(project_id);

    add::connect(home.path(), flagged, &console).await.expect("registers");

    assert!(console.questions().is_empty(), "{:?}", console.questions());
}

#[tokio::test]
async fn add_twice_is_idempotent() {
    let server = MockServer::start().await;
    let (home, _) = paired_home(&server.uri());
    let repository = checkout_repository("checkout", ORIGIN);
    mount_match(
        &server,
        json!({ "kind": "one", "match": repo_match(Uuid::new_v4(), Uuid::new_v4(), "Web App") }),
    )
    .await;
    mount_connection(&server, Uuid::new_v4()).await;
    add::connect(
        home.path(),
        options(&server, &repository.path),
        &ScriptedConsole::choosing(&[0]),
    )
    .await
    .expect("adds");
    let first_file = std::fs::read_to_string(config::config_path(home.path())).expect("config");

    add::connect(
        home.path(),
        options(&server, &repository.path),
        &ScriptedConsole::default(),
    )
    .await
    .expect("adds again");

    let puts = requests_to(&server, CONNECTIONS_ROUTE).await;
    assert_eq!(puts.len(), 2);
    assert_eq!(puts[0], puts[1]);
    assert_eq!(
        std::fs::read_to_string(config::config_path(home.path())).expect("config"),
        first_file
    );
    assert_eq!(saved(home.path()).connections.len(), 1);
}
