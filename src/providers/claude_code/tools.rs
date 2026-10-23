use crate::providers::MOOTBOX_MCP_SERVER_NAME;

const MOOTBOX_TOOLS: [&str; 7] = [
    "mootbox_whoami",
    "mootbox_get_context",
    "mootbox_read_chat",
    "mootbox_post_chat_message",
    "mootbox_ask_question",
    "mootbox_request_approval",
    "mootbox_submit_result",
];
const LOCAL_TOOLS: [&str; 7] = [
    "Read",
    "Glob",
    "Grep",
    "Edit",
    "Write",
    "Bash(git status:*)",
    "Bash(git diff:*)",
];

pub fn managed_allowed_tools(allowed_commands: &[String]) -> Vec<String> {
    MOOTBOX_TOOLS
        .iter()
        .map(|tool| format!("mcp__{MOOTBOX_MCP_SERVER_NAME}__{tool}"))
        .chain(LOCAL_TOOLS.iter().map(|tool| (*tool).to_string()))
        .chain(allowed_commands.iter().cloned())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_mootbox_tool_is_allowed_through_the_mootbox_server_and_owner_commands_come_last() {
        let tools = managed_allowed_tools(&["Bash(pnpm test:*)".to_string()]);

        assert_eq!(
            tools.iter().filter(|tool| tool.starts_with("mcp__mootbox__")).count(),
            7
        );
        assert!(tools.contains(&"mcp__mootbox__mootbox_ask_question".to_string()));
        assert!(tools.contains(&"mcp__mootbox__mootbox_request_approval".to_string()));
        assert!(!tools.iter().any(|tool| tool.contains("mootbox_create_handoff")));
        assert_eq!(tools.last().map(String::as_str), Some("Bash(pnpm test:*)"));
    }
}
