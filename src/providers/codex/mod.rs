use std::process::Output;

use anyhow::Result;

use super::{LaunchPlan, LaunchRequest, Provider, ProviderResult, Version};

mod launch;
mod output;
mod sign_in;

pub use launch::RUN_TOKEN_VARIABLE;

const MINIMUM_VERSION: Version = Version(0, 100, 0);
const SIGN_IN_ARGUMENTS: [&str; 2] = ["login", "status"];

pub struct Codex;

impl Provider for Codex {
    fn name(&self) -> &'static str {
        "codex"
    }

    fn label(&self) -> &'static str {
        "Codex"
    }

    fn default_command(&self) -> &'static str {
        "codex"
    }

    fn minimum_version(&self) -> Version {
        MINIMUM_VERSION
    }

    fn sign_in_arguments(&self) -> &'static [&'static str] {
        &SIGN_IN_ARGUMENTS
    }

    fn read_sign_in(&self, probe: &Output) -> Option<bool> {
        if !probe.status.success() {
            return Some(false);
        }
        sign_in::signed_in_from_status(&format!(
            "{}\n{}",
            String::from_utf8_lossy(&probe.stdout),
            String::from_utf8_lossy(&probe.stderr)
        ))
    }

    fn launch(&self, request: &LaunchRequest<'_>) -> Result<LaunchPlan> {
        Ok(launch::plan(request))
    }

    fn fold_output(&self, result: &mut ProviderResult, line: &[u8]) -> bool {
        output::fold(result, line)
    }
}
