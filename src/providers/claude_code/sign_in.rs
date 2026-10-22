use serde_json::Value;

const LOGGED_IN_FIELD: &str = "loggedIn";

pub fn signed_in_from_status(text: &str) -> Option<bool> {
    let status: Value = serde_json::from_str(text).ok()?;
    let logged_in = status.as_object()?.get(LOGGED_IN_FIELD)?;
    Some(logged_in.as_bool() == Some(true))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_signed_in_state_is_read_from_the_logged_in_field() {
        assert_eq!(
            signed_in_from_status(r#"{"loggedIn":true,"authMethod":"claude.ai"}"#),
            Some(true)
        );
        assert_eq!(signed_in_from_status(r#"{"loggedIn":false}"#), Some(false));
        assert_eq!(signed_in_from_status(r#"{"loggedIn":"yes"}"#), Some(false));
        assert_eq!(signed_in_from_status(r#"{"authMethod":"none"}"#), None);
        assert_eq!(signed_in_from_status("[]"), None);
        assert_eq!(signed_in_from_status("not json"), None);
    }
}
