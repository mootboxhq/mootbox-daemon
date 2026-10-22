use std::process::Output;

use anyhow::Result;

use super::{LaunchPlan, LaunchRequest, Provider, ProviderResult, Version};

mod launch;
mod mcp_config;
mod output;
mod sign_in;
pub mod tools;

const MINIMUM_VERSION: Version = Version(2, 1, 295);
const SIGN_IN_ARGUMENTS: [&str; 2] = ["auth", "status"];

pub struct ClaudeCode;

impl Provider for ClaudeCode {
    fn name(&self) -> &'static str {
        "claude-code"
    }

    fn label(&self) -> &'static str {
        "Claude Code"
    }

    fn default_command(&self) -> &'static str {
        "claude"
    }

    fn minimum_version(&self) -> Version {
        MINIMUM_VERSION
    }

    fn sign_in_arguments(&self) -> &'static [&'static str] {
        &SIGN_IN_ARGUMENTS
    }

    fn read_sign_in(&self, probe: &Output) -> Option<bool> {
        if !probe.status.success() {
            return None;
        }
        sign_in::signed_in_from_status(&String::from_utf8_lossy(&probe.stdout))
    }

    fn launch(&self, request: &LaunchRequest<'_>) -> Result<LaunchPlan> {
        launch::plan(request)
    }

    fn fold_output(&self, result: &mut ProviderResult, line: &[u8]) -> bool {
        let Some(found) = output::parse_result_event(line) else {
            return false;
        };
        *result = found;
        true
    }
}
