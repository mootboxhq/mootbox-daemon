pub fn signed_in_from_status(text: &str) -> Option<bool> {
    let text = text.to_lowercase();
    if text.contains("not logged in") || text.contains("not login") {
        return Some(false);
    }
    if text.contains("logged in") {
        return Some(true);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_signed_in_state_is_read_from_the_login_status_line() {
        assert_eq!(signed_in_from_status("Logged in using ChatGPT"), Some(true));
        assert_eq!(signed_in_from_status("Logged in using an API key"), Some(true));
        assert_eq!(signed_in_from_status("Not logged in"), Some(false));
        assert_eq!(signed_in_from_status("not login attempted"), Some(false));
    }

    #[test]
    fn output_that_names_neither_state_stays_inconclusive() {
        assert_eq!(signed_in_from_status("Some future wording"), None);
        assert_eq!(signed_in_from_status(""), None);
    }
}
