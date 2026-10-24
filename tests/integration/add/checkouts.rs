use super::*;

#[tokio::test]
async fn no_remote_explains() {
    let server = MockServer::start().await;
    let (home, before) = paired_home(&server.uri());
    let repository = repository_without_remote("checkout");

    let error = add::connect(
        home.path(),
        options(&server, &repository.path),
        &ScriptedConsole::default(),
    )
    .await
    .expect_err("no remote");

    assert_eq!(error.to_string(), NO_REMOTE);
    assert_eq!(saved(home.path()), before);
    assert!(server.received_requests().await.expect("recording").is_empty());
}

#[tokio::test]
async fn no_commits_explains() {
    let server = MockServer::start().await;
    let (home, before) = paired_home(&server.uri());
    let repository = repository_without_commits("checkout");
    git(&repository.path, &["remote", "add", "origin", ORIGIN]);

    let error = add::connect(
        home.path(),
        options(&server, &repository.path),
        &ScriptedConsole::default(),
    )
    .await
    .expect_err("no commits");

    assert_eq!(error.to_string(), NO_COMMITS);
    assert_eq!(saved(home.path()), before);
    assert!(server.received_requests().await.expect("recording").is_empty());
}

#[tokio::test]
async fn detached_head_explains() {
    let server = MockServer::start().await;
    let (home, before) = paired_home(&server.uri());
    let repository = checkout_repository("checkout", ORIGIN);
    git(&repository.path, &["checkout", "--quiet", "--detach"]);

    let error = add::connect(
        home.path(),
        options(&server, &repository.path),
        &ScriptedConsole::default(),
    )
    .await
    .expect_err("detached head");

    assert_eq!(error.to_string(), DETACHED_HEAD);
    assert_eq!(saved(home.path()), before);
    assert!(server.received_requests().await.expect("recording").is_empty());
}

#[tokio::test]
async fn several_remotes_without_origin_ask_which_one_identifies_the_repository() {
    let server = MockServer::start().await;
    let (home, _) = paired_home(&server.uri());
    let repository = repository_without_remote("checkout");
    git(
        &repository.path,
        &["remote", "add", "fork", "git@github.com:maya/checkout.git"],
    );
    git(&repository.path, &["remote", "add", "upstream", ORIGIN]);
    mount_match(
        &server,
        json!({ "kind": "one", "match": repo_match(Uuid::new_v4(), Uuid::new_v4(), "Web App") }),
    )
    .await;
    let connection_id = Uuid::new_v4();
    mount_connection(&server, connection_id).await;
    let console = ScriptedConsole::choosing(&[1, 0]);

    add::connect(home.path(), options(&server, &repository.path), &console)
        .await
        .expect("adds");

    assert_eq!(
        console.questions()[0].1,
        [
            "fork  git@github.com:maya/checkout.git".to_string(),
            format!("upstream  {ORIGIN}")
        ]
    );
    assert_eq!(
        connection_of(&saved(home.path()), connection_id).remote.as_deref(),
        Some("upstream")
    );
}

#[tokio::test]
async fn a_folder_outside_git_is_refused_before_calling_mootbox() {
    let server = MockServer::start().await;
    let home = tempfile::tempdir().expect("temp dir");
    let folder = plain_folder("notes");

    let error = add::connect(home.path(), options(&server, &folder.path), &ScriptedConsole::default())
        .await
        .expect_err("not a repo");

    assert_eq!(error.to_string(), NOT_IN_REPOSITORY);
    assert!(server.received_requests().await.expect("recording").is_empty());
}

#[tokio::test]
async fn without_a_mootbox_url_or_a_paired_server_asks_for_the_url() {
    let home = tempfile::tempdir().expect("temp dir");
    let repository = checkout_repository("checkout", ORIGIN);
    let options = AddOptions {
        workspace: None,
        mootbox_url: None,
        project: None,
        new_workspace: false,
        device_name: DEVICE_NAME.to_string(),
        directory: repository.path.clone(),
        poll_interval: FAST_POLL,
    };

    let error = add::connect(home.path(), options, &ScriptedConsole::default())
        .await
        .expect_err("no URL");

    assert_eq!(error.to_string(), NO_MOOTBOX_URL);
}

#[tokio::test]
async fn without_a_mootbox_url_the_paired_server_is_used() {
    let server = MockServer::start().await;
    let (home, _) = paired_home(&server.uri());
    let repository = checkout_repository("checkout", ORIGIN);
    mount_match(
        &server,
        json!({ "kind": "one", "match": repo_match(Uuid::new_v4(), Uuid::new_v4(), "Web App") }),
    )
    .await;
    mount_connection(&server, Uuid::new_v4()).await;
    let mut saved_url = options(&server, &repository.path);
    saved_url.mootbox_url = None;

    add::connect(home.path(), saved_url, &ScriptedConsole::choosing(&[0]))
        .await
        .expect("adds");

    assert_eq!(requests_to(&server, CONNECTIONS_ROUTE).await.len(), 1);
}
