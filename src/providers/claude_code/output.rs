use serde_json::Value;

use crate::providers::ProviderResult;

const RESULT_EVENT_TYPE: &str = "result";

pub fn parse_result_event(line: &[u8]) -> Option<ProviderResult> {
    let event: Value = serde_json::from_slice(line).ok()?;
    if event.get("type").and_then(Value::as_str) != Some(RESULT_EVENT_TYPE) {
        return None;
    }
    let text_of = |key: &str| event.get(key).and_then(Value::as_str).map(str::to_string);
    Some(ProviderResult {
        session_id: text_of("session_id"),
        effective_model: text_of("model").or_else(|| {
            event
                .get("modelUsage")
                .and_then(Value::as_object)
                .filter(|models| models.len() == 1)
                .and_then(|models| models.keys().next().cloned())
        }),
        text: text_of("result"),
        cost_usd: event.get("total_cost_usd").and_then(Value::as_f64),
        turns: event.get("num_turns").and_then(Value::as_u64),
        is_error: event.get("is_error").and_then(Value::as_bool).unwrap_or(false),
        api_error_status: event.get("api_error_status").and_then(Value::as_u64),
        has_permission_denials: event
            .get("permission_denials")
            .and_then(Value::as_array)
            .is_some_and(|denials| !denials.is_empty()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn result_events_yield_session_cost_turns_and_text() {
        let line = br#"{"type":"result","is_error":false,"result":"done","session_id":"s-1","num_turns":3,"total_cost_usd":0.01}"#;

        let result = parse_result_event(line).expect("result event");

        assert_eq!(
            result,
            ProviderResult {
                session_id: Some("s-1".to_string()),
                effective_model: None,
                text: Some("done".to_string()),
                cost_usd: Some(0.01),
                turns: Some(3),
                is_error: false,
                api_error_status: None,
                has_permission_denials: false,
            }
        );
    }

    #[test]
    fn other_events_and_broken_lines_are_skipped() {
        for line in [
            br#"{"type":"system","subtype":"init","session_id":"s-1"}"#.as_slice(),
            b"not json",
            b"[1,2]",
            b"",
        ] {
            assert_eq!(parse_result_event(line), None);
        }
    }

    #[test]
    fn api_error_status_and_permission_denials_are_read() {
        let line =
            br#"{"type":"result","is_error":true,"api_error_status":429,"permission_denials":[{"tool_name":"Bash"}]}"#;

        let result = parse_result_event(line).expect("result event");

        assert_eq!(result.api_error_status, Some(429));
        assert!(result.has_permission_denials);
    }

    #[test]
    fn error_results_are_flagged() {
        let result = parse_result_event(br#"{"type":"result","is_error":true,"result":"Not logged in"}"#);

        assert!(result.is_some_and(|result| result.is_error));
    }
}
