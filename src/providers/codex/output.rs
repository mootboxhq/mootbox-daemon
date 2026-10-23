use serde_json::Value;

use crate::providers::ProviderResult;

const FAILED_TURN: &str = "the turn failed";
const FAILED_STATUS: &str = "failed";
const APPROVAL_REQUIRED: &str = "requires approval";

enum Event {
    ThreadStarted { thread_id: String },
    AgentMessage { text: String },
    McpToolCallBlocked,
    TurnCompleted,
    TurnFailed { message: String },
}

pub fn fold(state: &mut ProviderResult, line: &[u8]) -> bool {
    let Some(event) = parse(line) else {
        return false;
    };
    match event {
        Event::ThreadStarted { thread_id } => state.session_id = Some(thread_id),
        Event::AgentMessage { text } => state.text = Some(text),
        Event::McpToolCallBlocked => state.has_permission_denials = true,
        Event::TurnCompleted => state.is_error = false,
        Event::TurnFailed { message } => {
            state.is_error = true;
            state.text = Some(message);
        }
    }
    true
}

fn parse(line: &[u8]) -> Option<Event> {
    let event: Value = serde_json::from_slice(line).ok()?;
    match event.get("type")?.as_str()? {
        "thread.started" => Some(Event::ThreadStarted {
            thread_id: string_at(&event, "thread_id")?,
        }),
        "item.completed" => {
            let item = event.get("item")?;
            match item.get("type")?.as_str()? {
                "agent_message" => Some(Event::AgentMessage {
                    text: string_at(item, "text")?,
                }),
                "mcp_tool_call" => is_blocked_for_approval(item).then_some(Event::McpToolCallBlocked),
                _ => None,
            }
        }
        "turn.completed" => Some(Event::TurnCompleted),
        "turn.failed" => Some(Event::TurnFailed {
            message: event
                .get("error")
                .and_then(|error| error.get("message"))
                .and_then(Value::as_str)
                .unwrap_or(FAILED_TURN)
                .to_string(),
        }),
        _ => None,
    }
}

fn is_blocked_for_approval(item: &Value) -> bool {
    let is_failed = item.get("status").and_then(Value::as_str) == Some(FAILED_STATUS);
    let message = item
        .get("error")
        .and_then(|error| error.get("message"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    is_failed && message.contains(APPROVAL_REQUIRED)
}

fn string_at(value: &Value, key: &str) -> Option<String> {
    Some(value.get(key)?.as_str()?.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_run_reports_the_thread_as_its_session_id() {
        let mut state = ProviderResult::default();

        assert!(fold(&mut state, br#"{"type":"thread.started","thread_id":"01a14bf0"}"#));
        assert_eq!(state.session_id.as_deref(), Some("01a14bf0"));
    }

    #[test]
    fn the_run_reports_the_final_agent_message_as_its_text() {
        let mut state = ProviderResult::default();

        fold(
            &mut state,
            br#"{"type":"item.completed","item":{"type":"reasoning","text":"thinking"}}"#,
        );
        fold(
            &mut state,
            br#"{"type":"item.completed","item":{"type":"agent_message","text":"OK"}}"#,
        );

        assert_eq!(state.text.as_deref(), Some("OK"));
    }

    #[test]
    fn a_failed_turn_becomes_an_error_result() {
        let mut state = ProviderResult::default();

        fold(
            &mut state,
            br#"{"type":"turn.failed","error":{"message":"usage limit reached"}}"#,
        );

        assert!(state.is_error);
        assert_eq!(state.text.as_deref(), Some("usage limit reached"));
    }

    #[test]
    fn a_completed_turn_clears_any_earlier_failure() {
        let mut state = ProviderResult::default();

        fold(&mut state, br#"{"type":"turn.failed","error":{"message":"transient"}}"#);
        fold(&mut state, br#"{"type":"turn.completed","usage":{"output_tokens":5}}"#);

        assert!(!state.is_error);
    }

    #[test]
    fn an_mcp_tool_call_blocked_for_approval_is_a_permission_denial() {
        let mut state = ProviderResult::default();

        fold(
            &mut state,
            br#"{"type":"item.completed","item":{"id":"item_0","type":"mcp_tool_call","server":"mootbox","tool":"mootbox_submit_result","arguments":{},"result":null,"error":{"message":"MCP tool call requires approval, but approval policy is never"},"status":"failed"}}"#,
        );
        fold(
            &mut state,
            br#"{"type":"item.completed","item":{"id":"item_1","type":"agent_message","text":"MCP tool call requires approval, but approval policy is never"}}"#,
        );
        fold(&mut state, br#"{"type":"turn.completed","usage":{"output_tokens":5}}"#);

        assert!(state.has_permission_denials);
        assert!(!state.is_error);
    }

    #[test]
    fn an_mcp_tool_call_that_fails_for_another_reason_is_not_a_permission_denial() {
        let mut state = ProviderResult::default();

        fold(
            &mut state,
            br#"{"type":"item.completed","item":{"id":"item_0","type":"mcp_tool_call","server":"mootbox","tool":"mootbox_read_chat","arguments":{},"result":null,"error":{"message":"chat not found"},"status":"failed"}}"#,
        );
        fold(
            &mut state,
            br#"{"type":"item.completed","item":{"id":"item_1","type":"mcp_tool_call","server":"mootbox","tool":"mootbox_whoami","arguments":{},"result":{"content":[]},"error":null,"status":"completed"}}"#,
        );

        assert!(!state.has_permission_denials);
    }

    #[test]
    fn lines_that_are_not_codex_events_are_ignored() {
        let mut state = ProviderResult::default();

        assert!(!fold(&mut state, b"not json at all"));
        assert!(!fold(
            &mut state,
            br#"{"type":"item.completed","item":{"type":"command_execution"}}"#
        ));

        assert_eq!(state, ProviderResult::default());
    }
}
