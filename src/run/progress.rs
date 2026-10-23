use serde_json::{Value, json};
use tokio::sync::mpsc;
use tracing::warn;

use super::RunContext;
use super::lease::Lease;
use crate::contracts::{RunEventRequest, RunEventRequestType};
use crate::providers::ProviderKind;

const MESSAGE_LIMIT: usize = 4000;

pub fn messages(provider: ProviderKind, line: &[u8]) -> Vec<String> {
    let Ok(event) = serde_json::from_slice::<Value>(line) else {
        return Vec::new();
    };
    let messages = match provider.name() {
        "codex" => codex(&event),
        "claude-code" => claude(&event),
        _ => Vec::new(),
    };
    messages
        .into_iter()
        .filter(|text| !text.trim().is_empty())
        .map(|text| {
            if text.chars().count() <= MESSAGE_LIMIT {
                text
            } else {
                format!(
                    "{}\n[output truncated]",
                    text.chars().take(MESSAGE_LIMIT - 20).collect::<String>()
                )
            }
        })
        .collect()
}

fn codex(event: &Value) -> Vec<String> {
    let phase = event["type"].as_str().unwrap_or_default();
    if phase != "item.started" && phase != "item.completed" {
        return Vec::new();
    }
    let item = &event["item"];
    let completed = phase == "item.completed";
    let message = match item["type"].as_str().unwrap_or_default() {
        "command_execution" => {
            let command = item["command"].as_str().unwrap_or_default();
            let output = item["aggregated_output"].as_str().unwrap_or_default();
            let status = if completed { "Finished" } else { "Running" };
            let exit = item["exit_code"]
                .as_i64()
                .map(|code| format!(" (exit {code})"))
                .unwrap_or_default();
            format!("{status}{exit}\n$ {command}\n{output}")
        }
        "mcp_tool_call" => format!(
            "{} {} · {}",
            if completed { "Finished" } else { "Calling" },
            item["server"].as_str().unwrap_or("tool"),
            item["tool"].as_str().unwrap_or_default()
        ),
        "file_change" if completed => {
            let paths = item["changes"]
                .as_array()
                .map(|changes| {
                    changes
                        .iter()
                        .filter_map(|change| change["path"].as_str())
                        .collect::<Vec<_>>()
                        .join("\n")
                })
                .unwrap_or_default();
            format!("Updated files\n{paths}")
        }
        "agent_message" if completed => item["text"].as_str().unwrap_or_default().to_string(),
        _ => return Vec::new(),
    };
    vec![message]
}

fn claude(event: &Value) -> Vec<String> {
    if !matches!(event["type"].as_str(), Some("assistant" | "user")) {
        return Vec::new();
    }
    let Some(content) = event["message"]["content"].as_array() else {
        return Vec::new();
    };
    content
        .iter()
        .filter_map(|block| match block["type"].as_str()? {
            "text" if event["type"] == "assistant" => block["text"].as_str().map(str::to_string),
            "tool_use" => {
                let name = block["name"].as_str().unwrap_or("tool");
                let input = &block["input"];
                let detail = input["command"]
                    .as_str()
                    .or_else(|| input["file_path"].as_str())
                    .unwrap_or_default();
                Some(format!("Calling {name}\n{detail}"))
            }
            "tool_result" => {
                let value = &block["content"];
                let text = value.as_str().map(str::to_string).or_else(|| {
                    value.as_array().map(|parts| {
                        parts
                            .iter()
                            .filter(|part| part["type"] == "text")
                            .filter_map(|part| part["text"].as_str())
                            .collect::<Vec<_>>()
                            .join("\n")
                    })
                })?;
                Some(format!("Tool output\n{text}"))
            }
            _ => None,
        })
        .collect()
}

pub async fn upload(context: &RunContext, lease: &Lease, mut receiver: mpsc::Receiver<String>) {
    while let Some(message) = receiver.recv().await {
        let request = RunEventRequest {
            generation: lease.generation,
            payload: json!({ "message": message }).as_object().cloned().unwrap_or_default(),
            type_: RunEventRequestType::Progress,
        };
        match tokio::time::timeout(
            std::time::Duration::from_secs(2),
            context.client.record_event(lease.run_id, &request),
        )
        .await
        {
            Ok(Ok(_)) => {}
            Ok(Err(error)) => warn!(run_id = %lease.run_id, %error, "activity upload failed"),
            Err(_) => warn!(run_id = %lease.run_id, "activity upload timed out"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_output_is_visible_but_private_reasoning_is_not() {
        let provider = "codex".parse().expect("provider");
        let output = messages(provider, br#"{"type":"item.completed","item":{"type":"command_execution","command":"cargo test","aggregated_output":"12 tests passed","exit_code":0}}"#);
        assert!(output[0].contains("$ cargo test"));
        assert!(output[0].contains("12 tests passed"));
        assert_eq!(
            messages(
                provider,
                br#"{"type":"item.completed","item":{"type":"reasoning","text":"private"}}"#
            ),
            Vec::<String>::new()
        );
    }

    #[test]
    fn claude_reports_tools_and_output_without_thinking_blocks() {
        let provider = "claude-code".parse().expect("provider");
        let output = messages(provider, br#"{"type":"assistant","message":{"content":[{"type":"thinking","thinking":"private"},{"type":"tool_use","name":"Bash","input":{"command":"pwd"}}]}}"#);
        assert_eq!(output, vec!["Calling Bash\npwd"]);
        assert_eq!(
            messages(
                provider,
                br#"{"type":"user","message":{"content":[{"type":"tool_result","content":"/work/project"}]}}"#
            ),
            vec!["Tool output\n/work/project"]
        );
    }

    #[test]
    fn malformed_events_are_ignored_and_large_output_is_bounded() {
        let provider = "codex".parse().expect("provider");
        assert_eq!(messages(provider, b"not json"), Vec::<String>::new());
        let event = json!({"type":"item.completed","item":{"type":"agent_message","text":"x".repeat(8000)}});
        let output = messages(provider, event.to_string().as_bytes());
        assert!(output[0].len() <= MESSAGE_LIMIT);
        assert!(output[0].ends_with("[output truncated]"));
    }
}
