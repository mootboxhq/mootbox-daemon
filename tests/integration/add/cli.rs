use super::*;

#[tokio::test]
async fn mootbox_add_prints_the_report_then_serves_in_the_foreground() {
    let server = MockServer::start().await;
    let project_id = Uuid::new_v4();
    mount_online_routes(&server, project_id).await;
    let (home, _) = paired_home(&server.uri());
    let repository = checkout_repository("checkout", ORIGIN);
    let project = project_id.to_string();
    let command = mootbox_command(
        home.path(),
        &repository.path,
        &["add", "--project", &project, "--foreground"],
    );

    let (output, _) = run_until_logged(command, home.path(), CLAIMING_PAUSED).await;

    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!(
            "{}● This computer is online — Ctrl+C to stop\n",
            expected_report(&server)
        )
    );
}

#[tokio::test]
async fn mootbox_add_installs_the_background_service_and_exits() {
    let server = MockServer::start().await;
    let project_id = Uuid::new_v4();
    mount_online_routes(&server, project_id).await;
    let (home, _) = paired_home(&server.uri());
    let repository = checkout_repository("checkout", ORIGIN);
    let project = project_id.to_string();
    let mut command = mootbox_command(
        home.path(),
        &repository.path,
        &["add", "--project", &project, "--slots", "3"],
    );
    command.env(SERVICE_MANAGER_VARIABLE, ACCEPTING_MANAGER);

    let output = output_of(command).await;

    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!(
            "{}● This computer is online in the background\n{BACKGROUND_HINT}\n",
            expected_report(&server)
        )
    );
    let installed = std::fs::read_dir(home.path().join(SERVICE_FOLDER))
        .expect("service folder")
        .count();
    assert_eq!(installed, 1);
    assert_eq!(saved(home.path()).slots, 3);
    assert_eq!(requests_to(&server, "/runner/claim").await, Vec::<Value>::new());
}

#[tokio::test]
async fn mootbox_add_stops_on_a_failing_check_and_prints_it() {
    let server = MockServer::start().await;
    let project_id = Uuid::new_v4();
    mount_online_routes(&server, project_id).await;
    let (home, _) = paired_home(&server.uri());
    let repository = checkout_repository("checkout", ORIGIN);
    let project = project_id.to_string();
    let mut command = mootbox_command(home.path(), &repository.path, &["add", "--project", &project]);
    command.env("FAKE_CLAUDE_LOGGED_IN", "false");

    let output = output_of(command).await;

    let shown = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(shown.contains("fail claude-code sign-in: not signed in"), "{shown}");
    assert!(String::from_utf8_lossy(&output.stdout).is_empty());
}

#[tokio::test]
async fn mootbox_add_without_a_terminal_lists_the_choices_and_the_project_hint() {
    let server = MockServer::start().await;
    let (home, _) = paired_home(&server.uri());
    let repository = checkout_repository("checkout", ORIGIN);

    let output = output_of(mootbox_command(home.path(), &repository.path, &["add"])).await;

    let shown = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert_eq!(
        shown,
        format!(
            "mootbox: {WHICH_WORKSPACE}\n  1. {PAIRED_WORKSPACE_NAME}\n  2. {CONNECT_ANOTHER_WORKSPACE}\n{PROJECT_HINT}\n"
        )
    );
}

#[tokio::test]
async fn mootbox_connect_is_a_hidden_alias_of_add() {
    let home = tempfile::tempdir().expect("temp dir");
    let folder = plain_folder("notes");

    let output = output_of(mootbox_command(home.path(), &folder.path, &["connect"])).await;

    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        format!("mootbox: {NOT_IN_REPOSITORY}\n")
    );
}

#[test]
fn help_leads_with_add_lists_pair_and_hides_the_old_commands() {
    let home = tempfile::tempdir().expect("temp dir");

    let output = mootbox_command(home.path(), home.path(), &["--help"])
        .output()
        .expect("mootbox runs");

    let help = String::from_utf8_lossy(&output.stdout);
    let first_command = help
        .lines()
        .skip_while(|line| *line != "Commands:")
        .nth(1)
        .expect("a command is listed");
    assert!(first_command.trim_start().starts_with("add "), "{help}");
    assert!(help.contains("  pair "), "{help}");
    for hidden in ["  connect ", "  bind ", "  agents "] {
        assert!(!help.contains(hidden), "{help}");
    }
}

#[test]
fn removed_commands_and_flags_are_refused() {
    let home = tempfile::tempdir().expect("temp dir");
    for args in [&["bind"][..], &["agents"], &["add", "--isolated"]] {
        let output = mootbox_command(home.path(), home.path(), args)
            .output()
            .expect("mootbox runs");

        assert!(!output.status.success(), "{args:?}");
    }
}
