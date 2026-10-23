use crate::providers::{LaunchPlan, LaunchRequest, MOOTBOX_MCP_SERVER_NAME};

pub const RUN_TOKEN_VARIABLE: &str = "MOOTBOX_RUN_TOKEN";

const WRITABLE_SANDBOX: &str = "sandbox_mode=\"workspace-write\"";
const NEVER_ASK_FOR_APPROVAL: &str = "approval_policy=\"never\"";
const PRE_APPROVED_TOOLS: &str = "default_tools_approval_mode=\"approve\"";

pub fn plan(request: &LaunchRequest<'_>) -> LaunchPlan {
    LaunchPlan {
        arguments: arguments(request),
        environment: vec![(RUN_TOKEN_VARIABLE.to_string(), request.run_token.to_string())],
        owned_directory: None,
    }
}

fn arguments(request: &LaunchRequest<'_>) -> Vec<String> {
    let mut arguments = vec![String::from("exec")];
    if let Some(session_id) = request.resume_session_id {
        arguments.extend([String::from("resume"), session_id.to_string()]);
    }
    arguments.extend([String::from("--json"), String::from("--skip-git-repo-check")]);
    arguments.extend([
        String::from("-c"),
        String::from(NEVER_ASK_FOR_APPROVAL),
        String::from("-c"),
        String::from(WRITABLE_SANDBOX),
        String::from("-c"),
        format!(
            "mcp_servers.{MOOTBOX_MCP_SERVER_NAME}.url=\"{}\"",
            request.mootbox_mcp_url
        ),
        String::from("-c"),
        format!("mcp_servers.{MOOTBOX_MCP_SERVER_NAME}.bearer_token_env_var=\"{RUN_TOKEN_VARIABLE}\""),
        String::from("-c"),
        format!("mcp_servers.{MOOTBOX_MCP_SERVER_NAME}.{PRE_APPROVED_TOOLS}"),
    ]);
    if let Some(model) = &request.options.model {
        arguments.extend([String::from("--model"), model.clone()]);
    }
    arguments
}

#[cfg(test)]
mod tests {
    use reqwest::Url;

    use super::*;
    use crate::providers::ProviderOptions;

    const TOKEN: &str = "mootbox_run_secret";

    fn plan_for(options: &ProviderOptions, resume_session_id: Option<&str>) -> LaunchPlan {
        let mootbox_mcp_url = Url::parse("http://mootbox/mcp").unwrap();
        plan(&LaunchRequest {
            mootbox_mcp_url: &mootbox_mcp_url,
            run_token: TOKEN,
            allowed_commands: &[],
            resume_session_id,
            options,
        })
    }

    #[test]
    fn a_fresh_run_asks_for_jsonl_and_tolerates_a_directory_without_git() {
        let arguments = plan_for(&ProviderOptions::default(), None).arguments;

        assert_eq!(arguments.first().map(String::as_str), Some("exec"));
        assert!(!arguments.iter().any(|argument| argument == "resume"));
        assert!(arguments.contains(&"--json".to_string()));
        assert!(arguments.contains(&"--skip-git-repo-check".to_string()));
        assert!(arguments.contains(&WRITABLE_SANDBOX.to_string()));
    }

    #[test]
    fn the_mootbox_mcp_server_is_passed_as_config_overrides() {
        let arguments = plan_for(&ProviderOptions::default(), None).arguments;

        assert!(arguments.contains(&"mcp_servers.mootbox.url=\"http://mootbox/mcp\"".to_string()));
        assert!(arguments.contains(&"mcp_servers.mootbox.bearer_token_env_var=\"MOOTBOX_RUN_TOKEN\"".to_string()));
        assert!(arguments.contains(&"approval_policy=\"never\"".to_string()));
    }

    #[test]
    fn only_the_mootbox_mcp_server_tools_are_pre_approved() {
        let arguments = plan_for(&ProviderOptions::default(), None).arguments;
        let approvals: Vec<&String> = arguments
            .iter()
            .filter(|argument| argument.contains("approval_mode"))
            .collect();

        assert_eq!(
            approvals,
            ["mcp_servers.mootbox.default_tools_approval_mode=\"approve\""]
        );
        assert!(!arguments.iter().any(|argument| argument.starts_with("--dangerously")));
        assert!(!arguments.contains(&"--approve-for-me".to_string()));
    }

    #[test]
    fn the_run_token_travels_only_in_the_environment_and_never_on_disk() {
        let plan = plan_for(&ProviderOptions::default(), None);

        assert_eq!(plan.environment, [(RUN_TOKEN_VARIABLE.to_string(), TOKEN.to_string())]);
        assert!(!plan.arguments.iter().any(|argument| argument.contains(TOKEN)));
        assert!(plan.owned_directory.is_none());
    }

    #[test]
    fn a_resumed_run_reuses_the_same_overrides_behind_the_resume_subcommand() {
        let arguments = plan_for(&ProviderOptions::default(), Some("thread-1")).arguments;

        assert_eq!(arguments[..3], ["exec", "resume", "thread-1"]);
        assert!(arguments.contains(&"--json".to_string()));
        assert!(arguments.contains(&WRITABLE_SANDBOX.to_string()));
        assert!(arguments.contains(&format!("mcp_servers.mootbox.{PRE_APPROVED_TOOLS}")));
    }

    #[test]
    fn a_chosen_model_is_forwarded_but_left_out_when_unset() {
        let chosen = ProviderOptions {
            model: Some("gpt-5-codex".to_string()),
            max_budget_usd: None,
        };

        let with_model = plan_for(&chosen, None).arguments;
        let without_model = plan_for(&ProviderOptions::default(), None).arguments;

        assert!(with_model.contains(&"gpt-5-codex".to_string()));
        assert!(!without_model.contains(&"--model".to_string()));
    }
}
