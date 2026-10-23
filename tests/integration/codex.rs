use mootbox_daemon::contracts::ClaimResponse;
use mootbox_daemon::providers::ProviderOptions;
use mootbox_daemon::providers::codex::RUN_TOKEN_VARIABLE;
use mootbox_daemon::run::prompt;
use serde_json::{Value, json};
use uuid::Uuid;
use wiremock::MockServer;

use crate::harness::{
    Bound, FakeProvider, TestDaemon, bound_home, claimed, completion_of, daemon_options, execute_claim,
    healthy_heartbeats, mount_claim_once, mount_mootbox, requests_to,
};

const MODEL: &str = "gpt-5-codex";

fn codex_bound(server: &MockServer) -> Bound {
    let mut bound = bound_home(server);
    bound.config.provider = "codex".parse().expect("known provider");
    bound
}

fn codex_claim(run_id: Uuid, bound: &Bound) -> Value {
    let mut claim = execute_claim(run_id, bound.repository_id);
    claim["run"]["provider"] = json!("codex");
    claim["run"]["manifest"]["provider"] = json!("codex");
    claim
}

async fn run_one(server: &MockServer, bound: &Bound, claim: &Value, provider: &FakeProvider) -> Value {
    let run_id: Uuid = claim["run"]["run_id"].as_str().expect("run id").parse().expect("UUID");
    mount_claim_once(server, claimed(claim.clone())).await;
    mount_mootbox(server, healthy_heartbeats()).await;
    let mut options = daemon_options(bound.config.clone(), &bound.home, provider);
    options.provider_options = ProviderOptions {
        model: Some(MODEL.to_string()),
        max_budget_usd: None,
    };
    let daemon = TestDaemon::start(options);
    let completion = completion_of(server, run_id).await;
    daemon.stop().await;
    completion
}

#[tokio::test]
async fn a_codex_run_is_claimed_executed_and_completed_with_its_final_message() {
    let server = MockServer::start().await;
    let bound = codex_bound(&server);
    let run_id = Uuid::new_v4();
    let claim = codex_claim(run_id, &bound);
    let provider = FakeProvider::codex("success");

    let completion = run_one(&server, &bound, &claim, &provider).await;

    assert_eq!(completion["outcome"], "succeeded", "{completion}");
    assert_eq!(completion["final_answer"], "done in codex");
    let events = requests_to(&server, &format!("/runner/runs/{run_id}/events")).await;
    assert!(
        events
            .iter()
            .any(|event| event["type"] == "progress" && event["payload"]["message"] == "done in codex"),
        "{events:?}"
    );
    assert!(
        !events.iter().any(|event| event["payload"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("thinking"))),
        "{events:?}"
    );
    assert_eq!(completion["exit"]["code"], 0);
    assert_eq!(completion["exit"]["error_category"], Value::Null);
    assert_eq!(completion["exit"]["checkpoint"]["branch"], format!("mootbox/{run_id}"));
    let claims = requests_to(&server, "/runner/claim").await;
    assert!(
        claims.iter().any(|request| request["providers"]
            .as_array()
            .is_some_and(|providers| providers.contains(&json!("codex")))),
        "{claims:?}"
    );
    let invocation = provider.only_invocation();
    assert_eq!(invocation.arguments[..3], ["exec", "--json", "--skip-git-repo-check"]);
    assert_eq!(invocation.value_after("--model"), Some(MODEL));
    let mcp_url = format!("mcp_servers.mootbox.url=\"{}/mcp\"", server.uri());
    assert!(invocation.arguments.contains(&mcp_url), "{:?}", invocation.arguments);
    assert!(invocation.arguments.contains(&format!(
        "mcp_servers.mootbox.bearer_token_env_var=\"{RUN_TOKEN_VARIABLE}\""
    )));
    assert!(
        invocation
            .arguments
            .contains(&"mcp_servers.mootbox.default_tools_approval_mode=\"approve\"".to_string())
    );
    let token = claim["run"]["run_token"].as_str().expect("run token");
    assert_eq!(invocation.run_token, token);
    assert!(!invocation.arguments.iter().any(|argument| argument.contains(token)));
    let ClaimResponse::Claimed { run } = serde_json::from_value(claim.clone()).expect("claim parses") else {
        panic!("execute claim")
    };
    assert_eq!(
        invocation.prompt,
        prompt::execute_prompt(&run.manifest).expect("prompt renders")
    );
    assert_eq!(
        invocation.working_directory,
        bound
            .home
            .join("worktrees")
            .join(format!("mootbox-{run_id}"))
            .canonicalize()
            .expect("worktree")
    );
}

#[tokio::test]
async fn a_codex_thread_is_resumed_behind_the_resume_subcommand() {
    let server = MockServer::start().await;
    let bound = codex_bound(&server);
    let first_id = Uuid::new_v4();
    let first = codex_claim(first_id, &bound);
    let provider = FakeProvider::codex("success");
    assert_eq!(
        run_one(&server, &bound, &first, &provider).await["outcome"],
        "succeeded"
    );
    let mut next = codex_claim(Uuid::new_v4(), &bound);
    next["run"]["manifest"]["session_key"] = json!(first_id);
    next["run"]["manifest"]["resume"] = json!(true);
    let provider = FakeProvider::codex("success");

    let completion = run_one(&server, &bound, &next, &provider).await;

    assert_eq!(completion["outcome"], "succeeded", "{completion}");
    assert_eq!(
        provider.only_invocation().arguments[..3],
        ["exec", "resume", "thread-1"]
    );
}

#[tokio::test]
async fn a_failed_codex_turn_fails_the_run_with_the_turn_message() {
    let server = MockServer::start().await;
    let bound = codex_bound(&server);
    let claim = codex_claim(Uuid::new_v4(), &bound);
    let provider = FakeProvider::codex("fail");

    let completion = run_one(&server, &bound, &claim, &provider).await;

    assert_eq!(completion["outcome"], "failed", "{completion}");
    assert_eq!(completion["error"], "rate_limited");
    assert_eq!(completion["final_answer"], "usage limit reached");
}

#[tokio::test]
async fn a_codex_run_whose_mootbox_tool_call_is_blocked_for_approval_fails_as_permission_required() {
    let server = MockServer::start().await;
    let bound = codex_bound(&server);
    let claim = codex_claim(Uuid::new_v4(), &bound);
    let provider = FakeProvider::codex("mcp_blocked");

    let completion = run_one(&server, &bound, &claim, &provider).await;

    assert_eq!(completion["outcome"], "failed", "{completion}");
    assert_eq!(completion["error"], "permission_required");
}
