use std::sync::LazyLock;

use regress::Regex;

use super::process::ProviderReport;

const HTTP_UNAUTHORIZED: u64 = 401;
const HTTP_TOO_MANY_REQUESTS: u64 = 429;
const CASE_INSENSITIVE: &str = "i";
const DIAGNOSTICS_SEPARATOR: &str = "\n";

static AUTHENTICATION_PATTERN: LazyLock<Option<Regex>> =
    LazyLock::new(|| ignoring_case(r"not logged in|log ?in|authenticat|invalid api key|oauth token"));
static RATE_LIMIT_PATTERN: LazyLock<Option<Regex>> =
    LazyLock::new(|| ignoring_case(r"rate.?limit|too many requests|usage limit|overloaded"));
static RESUME_FAILURE_PATTERN: LazyLock<Option<Regex>> =
    LazyLock::new(|| ignoring_case(r"no conversation found|session"));

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorCategory {
    AuthenticationRequired,
    PermissionRequired,
    RateLimited,
    ProcessFailed,
    Interrupted,
    ResumeFailed,
}

impl ErrorCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AuthenticationRequired => "authentication_required",
            Self::PermissionRequired => "permission_required",
            Self::RateLimited => "rate_limited",
            Self::ProcessFailed => "process_failed",
            Self::Interrupted => "interrupted",
            Self::ResumeFailed => "resume_failed",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Attempt {
    pub is_resumed: bool,
    pub is_stop_requested: bool,
}

pub fn classify(report: &ProviderReport, attempt: Attempt) -> Option<ErrorCategory> {
    if attempt.is_stop_requested || report.signal_name().is_some() {
        return Some(ErrorCategory::Interrupted);
    }
    let result = report.result.as_ref();
    match result.and_then(|result| result.api_error_status) {
        Some(HTTP_UNAUTHORIZED) => return Some(ErrorCategory::AuthenticationRequired),
        Some(HTTP_TOO_MANY_REQUESTS) => return Some(ErrorCategory::RateLimited),
        _ => {}
    }
    let is_failed = result.is_none_or(|result| result.is_error) || !report.status.success();
    let has_permission_denials = result.is_some_and(|result| result.has_permission_denials);
    if !is_failed {
        return has_permission_denials.then_some(ErrorCategory::PermissionRequired);
    }
    let diagnostics = diagnostics(report);
    if is_match(&AUTHENTICATION_PATTERN, &diagnostics) {
        return Some(ErrorCategory::AuthenticationRequired);
    }
    if is_match(&RATE_LIMIT_PATTERN, &diagnostics) {
        return Some(ErrorCategory::RateLimited);
    }
    let turns = result.and_then(|result| result.turns).unwrap_or(0);
    if attempt.is_resumed && turns == 0 && is_match(&RESUME_FAILURE_PATTERN, &diagnostics) {
        return Some(ErrorCategory::ResumeFailed);
    }
    if has_permission_denials {
        return Some(ErrorCategory::PermissionRequired);
    }
    Some(ErrorCategory::ProcessFailed)
}

fn diagnostics(report: &ProviderReport) -> String {
    let text = report.result.as_ref().and_then(|result| result.text.as_deref());
    std::iter::once(text.unwrap_or_default())
        .chain(report.stderr_tail.iter().map(String::as_str))
        .collect::<Vec<_>>()
        .join(DIAGNOSTICS_SEPARATOR)
        .trim()
        .to_string()
}

fn ignoring_case(pattern: &str) -> Option<Regex> {
    Regex::with_flags(pattern, CASE_INSENSITIVE).ok()
}

fn is_match(pattern: &LazyLock<Option<Regex>>, text: &str) -> bool {
    pattern.as_ref().is_some_and(|regex| regex.find(text).is_some())
}

#[cfg(test)]
mod tests {
    use std::os::unix::process::ExitStatusExt;
    use std::process::ExitStatus;

    use super::*;
    use crate::providers::ProviderResult;

    const EXIT_CODE_SHIFT: i32 = 8;
    const SIGINT: i32 = 2;
    const FRESH: Attempt = Attempt {
        is_resumed: false,
        is_stop_requested: false,
    };
    const RESUMED: Attempt = Attempt {
        is_resumed: true,
        is_stop_requested: false,
    };

    fn exited(code: i32) -> ExitStatus {
        ExitStatus::from_raw(code << EXIT_CODE_SHIFT)
    }

    fn report(status: ExitStatus, result: Option<ProviderResult>, stderr: &[&str]) -> ProviderReport {
        ProviderReport {
            status,
            result,
            stderr_tail: stderr.iter().map(ToString::to_string).collect(),
        }
    }

    fn succeeded() -> ProviderResult {
        ProviderResult {
            session_id: Some("session-1".to_string()),
            text: Some("done".to_string()),
            turns: Some(3),
            ..ProviderResult::default()
        }
    }

    #[test]
    fn a_clean_exit_with_a_result_has_no_error() {
        assert_eq!(classify(&report(exited(0), Some(succeeded()), &[]), FRESH), None);
    }

    #[test]
    fn stops_and_signals_are_interrupted() {
        let stopped = Attempt {
            is_stop_requested: true,
            ..FRESH
        };
        assert_eq!(
            classify(&report(exited(0), Some(succeeded()), &[]), stopped),
            Some(ErrorCategory::Interrupted)
        );
        assert_eq!(
            classify(&report(ExitStatus::from_raw(SIGINT), None, &[]), FRESH),
            Some(ErrorCategory::Interrupted)
        );
    }

    #[test]
    fn api_error_statuses_win_over_diagnostics() {
        for (status, category) in [
            (401, ErrorCategory::AuthenticationRequired),
            (429, ErrorCategory::RateLimited),
        ] {
            let result = ProviderResult {
                api_error_status: Some(status),
                ..succeeded()
            };
            assert_eq!(classify(&report(exited(0), Some(result), &[]), FRESH), Some(category));
        }
    }

    #[test]
    fn a_logged_out_provider_needs_authentication() {
        let result = ProviderResult {
            is_error: true,
            text: Some("Not logged in · Please run /login".to_string()),
            ..ProviderResult::default()
        };

        assert_eq!(
            classify(&report(exited(1), Some(result), &[]), FRESH),
            Some(ErrorCategory::AuthenticationRequired)
        );
    }

    #[test]
    fn usage_limits_in_stderr_are_rate_limits() {
        assert_eq!(
            classify(&report(exited(1), None, &["Claude AI usage limit reached"]), FRESH),
            Some(ErrorCategory::RateLimited)
        );
    }

    #[test]
    fn a_missing_session_fails_the_resume_only_when_resuming_without_turns() {
        let stderr = ["No conversation found with session ID: gone-session"];

        assert_eq!(
            classify(&report(exited(1), None, &stderr), RESUMED),
            Some(ErrorCategory::ResumeFailed)
        );
        assert_eq!(
            classify(&report(exited(1), None, &stderr), FRESH),
            Some(ErrorCategory::ProcessFailed)
        );
        let worked = ProviderResult {
            is_error: true,
            turns: Some(2),
            ..ProviderResult::default()
        };
        assert_eq!(
            classify(&report(exited(1), Some(worked), &stderr), RESUMED),
            Some(ErrorCategory::ProcessFailed)
        );
    }

    #[test]
    fn denied_tools_need_permission_even_after_a_clean_exit() {
        let result = ProviderResult {
            has_permission_denials: true,
            ..succeeded()
        };

        assert_eq!(
            classify(&report(exited(0), Some(result), &[]), FRESH),
            Some(ErrorCategory::PermissionRequired)
        );
    }

    #[test]
    fn a_crash_without_a_known_cause_is_a_process_failure() {
        assert_eq!(
            classify(&report(exited(3), None, &["fatal: the fake provider crashed"]), FRESH),
            Some(ErrorCategory::ProcessFailed)
        );
    }
}
