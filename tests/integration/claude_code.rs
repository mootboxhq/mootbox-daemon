use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use mootbox_daemon::config::{self, HOME_OVERRIDE_VARIABLE, PROVIDER_COMMAND_VARIABLE};
use mootbox_daemon::contracts::ClaimResponse;
use mootbox_daemon::private_files;
use mootbox_daemon::providers::ProviderOptions;
use mootbox_daemon::providers::claude_code::tools;
use mootbox_daemon::run::prompt;
use serde_json::{Value, json};
use uuid::Uuid;
use wiremock::{MockServer, ResponseTemplate};

use crate::common::{authenticated_route, fixture};
use crate::harness::{
    Bound, CLAIMING_PAUSED, FakeProvider, Invocation, TestDaemon, answer_later_heartbeats, bound_home, claimed,
    completion_of, daemon_options, execute_claim, healthy_heartbeats, heartbeat_answer, mount_claim_once,
    mount_mootbox, run_until_logged,
};

const OWNER_ONLY_MODE: &str = "-rw-------";
const MODEL: &str = "claude-opus-4-1";
const MAX_BUDGET_USD: f64 = 2.5;
const HTTP_UPGRADE_REQUIRED: u16 = 426;
const AFTER_CLAIMED_RUNS_PRIORITY: u8 = 5;
const BASE_ARGUMENTS: [&str; 4] = ["--print", "--output-format", "stream-json", "--verbose"];

fn run_token_of(claim: &Value) -> String {
    claim["run"]["run_token"].as_str().expect("run token").to_string()
}

fn expected_mcp_config(server: &MockServer, claim: &Value) -> Value {
    json!({
        "mcpServers": {
            "mootbox": {
                "type": "http",
                "url": format!("{}/mcp", server.uri()),
                "headers": { "Authorization": format!("Bearer {}", run_token_of(claim)) },
            }
        }
    })
}

fn typed(claim: &Value) -> ClaimResponse {
    serde_json::from_value(claim.clone()).expect("claim parses")
}

fn assert_private_config_removed(invocation: &Invocation) {
    assert_eq!(invocation.mcp_config_mode, OWNER_ONLY_MODE);
    assert!(invocation.mcp_config_path.is_absolute(), "{invocation:?}");
    assert!(!invocation.mcp_config_path.exists(), "MCP config left behind");
    let directory = invocation.mcp_config_path.parent().expect("directory");
    assert!(!directory.exists(), "MCP config directory left behind");
}

async fn run_one(server: &MockServer, bound: &Bound, claim: &Value, provider: &FakeProvider) -> Value {
    let run_id: Uuid = claim["run"]["run_id"].as_str().expect("run id").parse().expect("UUID");
    mount_claim_once(server, claimed(claim.clone())).await;
    mount_mootbox(server, healthy_heartbeats()).await;
    let mut options = daemon_options(bound.config.clone(), &bound.home, provider);
    options.provider_options = ProviderOptions {
        model: Some(MODEL.to_string()),
        max_budget_usd: Some(MAX_BUDGET_USD),
    };
    let daemon = TestDaemon::start(options);
    let completion = completion_of(server, run_id).await;
    daemon.stop().await;
    completion
}

#[tokio::test]
async fn execute_runs_get_the_claude_flags_owner_commands_and_chat_prompt() {
    let server = MockServer::start().await;
    let mut bound = bound_home(&server);
    bound.config.allowed_commands = vec!["Bash(pnpm test:*)".to_string()];
    let run_id = Uuid::new_v4();
    let claim = execute_claim(run_id, bound.repository_id);
    let provider = FakeProvider::claude("success");

    let completion = run_one(&server, &bound, &claim, &provider).await;

    assert_eq!(completion["outcome"], "succeeded", "{completion}");
    assert_eq!(completion["exit"]["error_category"], Value::Null);
    let invocation = provider.only_invocation();
    let expected_tools = tools::managed_allowed_tools(&bound.config.allowed_commands);
    assert_eq!(invocation.arguments[..BASE_ARGUMENTS.len()], BASE_ARGUMENTS);
    assert_eq!(invocation.value_after("--setting-sources"), Some("project,local"));
    assert_eq!(invocation.value_after("--permission-mode"), Some("dontAsk"));
    assert!(invocation.arguments.contains(&"--strict-mcp-config".to_string()));
    assert_eq!(invocation.value_after("--model"), Some(MODEL));
    assert_eq!(invocation.value_after("--max-budget-usd"), Some("2.5"));
    assert_eq!(invocation.value_after("--resume"), None);
    assert_eq!(invocation.allowed_tools(), expected_tools);
    let ClaimResponse::Claimed { run } = typed(&claim) else {
        panic!("execute claim")
    };
    assert_eq!(
        invocation.prompt,
        prompt::execute_prompt(&run.manifest).expect("prompt renders")
    );
    assert_eq!(invocation.mcp_config, expected_mcp_config(&server, &claim));
    assert_eq!(
        invocation.working_directory,
        bound
            .home
            .join("worktrees")
            .join(format!("mootbox-{run_id}"))
            .canonicalize()
            .expect("worktree")
    );
    assert_private_config_removed(&invocation);
}

#[tokio::test]
async fn a_crash_removes_the_mcp_config_and_reports_the_error_category() {
    let server = MockServer::start().await;
    let bound = bound_home(&server);
    let claim = execute_claim(Uuid::new_v4(), bound.repository_id);
    let provider = FakeProvider::claude("crash");

    let completion = run_one(&server, &bound, &claim, &provider).await;

    assert_eq!(completion["outcome"], "failed", "{completion}");
    assert_eq!(completion["error"], "process_failed");
    assert_eq!(completion["exit"]["error_category"], "process_failed");
    assert_private_config_removed(&provider.only_invocation());
}

#[tokio::test]
async fn a_cancel_removes_the_mcp_config() {
    let server = MockServer::start().await;
    let bound = bound_home(&server);
    let run_id = Uuid::new_v4();
    mount_claim_once(&server, claimed(execute_claim(run_id, bound.repository_id))).await;
    mount_mootbox(&server, healthy_heartbeats()).await;
    let provider = FakeProvider::claude("hang");
    let daemon = TestDaemon::start(daemon_options(bound.config.clone(), &bound.home, &provider));
    provider.grandchild().await;

    answer_later_heartbeats(
        &server,
        ResponseTemplate::new(200).set_body_json(heartbeat_answer(true)),
    )
    .await;
    let completion = completion_of(&server, run_id).await;
    daemon.stop().await;

    assert_eq!(completion["outcome"], "cancelled", "{completion}");
    assert_eq!(completion["exit"]["error_category"], "interrupted");
    assert_private_config_removed(&provider.only_invocation());
}

fn files_containing(directory: &Path, needle: &str) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(directory) else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .flat_map(|path| {
            if path.is_dir() {
                return files_containing(&path, needle);
            }
            let contents = fs::read(&path).unwrap_or_default();
            let found = String::from_utf8_lossy(&contents).contains(needle);
            if found { vec![path] } else { Vec::new() }
        })
        .collect()
}

async fn start_binary_until_runs_complete(home: &Path, provider: &FakeProvider) -> (Output, String) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mootbox"));
    command
        .env(HOME_OVERRIDE_VARIABLE, home)
        .env(PROVIDER_COMMAND_VARIABLE, &provider.program)
        .arg("start");
    run_until_logged(command, home, CLAIMING_PAUSED).await
}

async fn mount_start_routes(server: &MockServer) {
    authenticated_route("POST", "/runner/claim")
        .respond_with(
            ResponseTemplate::new(HTTP_UPGRADE_REQUIRED).set_body_json(fixture("error.upgrade-required.json")),
        )
        .with_priority(AFTER_CLAIMED_RUNS_PRIORITY)
        .mount(server)
        .await;
}

#[tokio::test]
async fn run_tokens_never_reach_argv_stdin_logs_or_files_in_the_daemon_home() {
    let server = MockServer::start().await;
    let bound = bound_home(&server);
    config::save(&bound.home, &bound.config).expect("saves config");
    let (first_run_id, second_run_id) = (Uuid::new_v4(), Uuid::new_v4());
    let claims = [
        execute_claim(first_run_id, bound.repository_id),
        execute_claim(second_run_id, bound.repository_id),
    ];
    for claim in &claims {
        mount_claim_once(&server, claimed(claim.clone())).await;
    }
    mount_start_routes(&server).await;
    mount_mootbox(&server, healthy_heartbeats()).await;
    let provider = FakeProvider::claude("success");

    let (output, logs) = start_binary_until_runs_complete(&bound.home, &provider).await;

    assert!(logs.contains("started provider"), "{logs}\n{output:?}");
    for run_id in [first_run_id, second_run_id] {
        assert_eq!(completion_of(&server, run_id).await["outcome"], "succeeded");
    }
    let invocations = provider.invocations();
    assert_eq!(invocations.len(), claims.len());
    for (claim, invocation) in claims.iter().zip(&invocations) {
        let token = run_token_of(claim);
        assert!(
            invocation.mcp_config.to_string().contains(&token),
            "token not handed to the provider"
        );
        assert!(!invocation.arguments.iter().any(|argument| argument.contains(&token)));
        assert!(!invocation.prompt.contains(&token));
        assert!(!logs.contains(&token), "token logged");
        assert_eq!(files_containing(&bound.home, &token), Vec::<PathBuf>::new());
    }
    assert!(
        bound
            .home
            .join("runs")
            .join(first_run_id.to_string())
            .join("manifest.json")
            .exists()
    );
}

#[tokio::test]
async fn an_opaque_session_mapping_survives_a_daemon_restart_and_resumes_the_same_provider() {
    let server = MockServer::start().await;
    let bound = bound_home(&server);
    let first_id = Uuid::new_v4();
    let first = execute_claim(first_id, bound.repository_id);
    let provider = FakeProvider::claude("success");
    assert_eq!(
        run_one(&server, &bound, &first, &provider).await["outcome"],
        "succeeded"
    );
    let session_file = bound.home.join("sessions").join(format!("{first_id}.json"));
    assert_eq!(fs::metadata(&session_file).unwrap().permissions().mode() & 0o777, 0o600);
    let mut next = execute_claim(Uuid::new_v4(), bound.repository_id);
    next["run"]["manifest"]["session_key"] = json!(first_id);
    next["run"]["manifest"]["resume"] = json!(true);
    let provider = FakeProvider::claude("success");
    assert_eq!(run_one(&server, &bound, &next, &provider).await["outcome"], "succeeded");
    assert_eq!(provider.only_invocation().value_after("--resume"), Some("session-1"));
}

#[tokio::test]
async fn a_missing_local_session_fails_visibly_without_starting_a_fresh_provider() {
    let server = MockServer::start().await;
    let bound = bound_home(&server);
    let mut claim = execute_claim(Uuid::new_v4(), bound.repository_id);
    claim["run"]["manifest"]["resume"] = json!(true);
    let provider = FakeProvider::claude("success");
    let completion = run_one(&server, &bound, &claim, &provider).await;
    assert_eq!(completion["outcome"], "failed");
    assert!(completion["error"].as_str().unwrap().contains("explicitly start fresh"));
    assert!(provider.invocations().is_empty());
}

#[tokio::test]
async fn a_failed_resume_never_retries_with_a_fresh_session() {
    let server = MockServer::start().await;
    let bound = bound_home(&server);
    let id = Uuid::new_v4();
    let mut claim = execute_claim(id, bound.repository_id);
    claim["run"]["manifest"]["resume"] = json!(true);
    private_files::create_private_directory(&bound.home.join("sessions")).unwrap();
    private_files::write_owner_only(
        &bound.home.join("sessions").join(format!("{id}.json")),
        b"\"missing-session\"",
    )
    .unwrap();
    let provider = FakeProvider::claude("resume-missing");
    let completion = run_one(&server, &bound, &claim, &provider).await;
    assert_eq!(completion["outcome"], "failed");
    assert_eq!(provider.invocations().len(), 1);
}
