use anyhow::{Context, Result};
use serde_json::json;

use super::{mcp_config, tools};
use crate::providers::{LaunchPlan, LaunchRequest, ProviderOptions};

const MANAGED_SETTING_SOURCES: &str = "project,local";
const TOOL_LIST_SEPARATOR: &str = ",";
const SPACE: char = ' ';

struct LaunchSettings<'a> {
    allowed_tools: &'a [String],
    mcp_config: &'a str,
    resume_session_id: Option<&'a str>,
    options: &'a ProviderOptions,
}

pub fn plan(request: &LaunchRequest<'_>) -> Result<LaunchPlan> {
    let (directory, path) = mcp_config::write(request.mootbox_mcp_url, request.run_token)?;
    let mcp_config = path.to_str().context("the Claude Code MCP config path is not UTF-8")?;
    let allowed_tools = tools::managed_allowed_tools(request.allowed_commands);
    let settings = LaunchSettings {
        allowed_tools: &allowed_tools,
        mcp_config,
        resume_session_id: request.resume_session_id,
        options: request.options,
    };
    Ok(LaunchPlan {
        arguments: arguments(&settings),
        environment: Vec::new(),
        owned_directory: Some(directory),
    })
}

fn arguments(settings: &LaunchSettings<'_>) -> Vec<String> {
    let allowed_tools = settings.allowed_tools;
    let command_line_tools = allowed_tools
        .iter()
        .filter(|tool| !tool.contains(SPACE))
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(TOOL_LIST_SEPARATOR);
    let permissions = json!({ "permissions": { "allow": allowed_tools } }).to_string();
    let mut arguments = [
        "--print",
        "--output-format",
        "stream-json",
        "--verbose",
        "--mcp-config",
        settings.mcp_config,
        "--strict-mcp-config",
        "--setting-sources",
        MANAGED_SETTING_SOURCES,
        "--permission-mode",
        "dontAsk",
        "--allowedTools",
        &command_line_tools,
        "--settings",
        &permissions,
    ]
    .map(String::from)
    .to_vec();
    if let Some(model) = settings.options.model.as_deref().filter(|model| !model.is_empty()) {
        arguments.extend(["--model".to_string(), model.to_string()]);
    }
    if let Some(budget) = settings.options.max_budget_usd.filter(|budget| *budget != 0.0) {
        arguments.extend(["--max-budget-usd".to_string(), budget.to_string()]);
    }
    if let Some(session_id) = settings.resume_session_id.filter(|session_id| !session_id.is_empty()) {
        arguments.extend(["--resume".to_string(), session_id.to_string()]);
    }
    arguments
}
