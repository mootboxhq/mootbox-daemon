use std::path::PathBuf;
use std::process::Output;

use anyhow::{Context, Result};
use reqwest::Url;
use tempfile::TempDir;

pub mod claude_code;
pub mod codex;
mod kind;
pub mod probe;

pub use kind::{ProviderKind, UnknownProvider};
pub use probe::Version;

pub const ALL: [ProviderKind; 2] = [
    ProviderKind::new(&claude_code::ClaudeCode),
    ProviderKind::new(&codex::Codex),
];

pub const DEFAULT_PROVIDER: ProviderKind = ProviderKind::new(&claude_code::ClaudeCode);

pub const MOOTBOX_MCP_SERVER_NAME: &str = "mootbox";

const MOOTBOX_MCP_PATH: &str = "/mcp";

pub trait Provider: Sync {
    fn name(&self) -> &'static str;
    fn label(&self) -> &'static str;
    fn default_command(&self) -> &'static str;
    fn minimum_version(&self) -> Version;
    fn sign_in_arguments(&self) -> &'static [&'static str];
    fn read_sign_in(&self, probe: &Output) -> Option<bool>;
    fn launch(&self, request: &LaunchRequest<'_>) -> Result<LaunchPlan>;
    fn fold_output(&self, result: &mut ProviderResult, line: &[u8]) -> bool;
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProviderOptions {
    pub model: Option<String>,
    pub max_budget_usd: Option<f64>,
}

pub struct LaunchRequest<'a> {
    pub mootbox_mcp_url: &'a Url,
    pub run_token: &'a str,
    pub allowed_commands: &'a [String],
    pub resume_session_id: Option<&'a str>,
    pub options: &'a ProviderOptions,
}

#[derive(Debug)]
pub struct LaunchPlan {
    pub arguments: Vec<String>,
    pub environment: Vec<(String, String)>,
    pub owned_directory: Option<TempDir>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProviderResult {
    pub session_id: Option<String>,
    pub effective_model: Option<String>,
    pub text: Option<String>,
    pub cost_usd: Option<f64>,
    pub turns: Option<u64>,
    pub is_error: bool,
    pub api_error_status: Option<u64>,
    pub has_permission_denials: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectedProvider {
    pub kind: ProviderKind,
    pub program: PathBuf,
}

impl SelectedProvider {
    pub fn program_for(&self, kind: ProviderKind) -> PathBuf {
        if kind == self.kind {
            return self.program.clone();
        }
        PathBuf::from(kind.default_command())
    }
}

pub fn mootbox_mcp_url(mootbox_url: &str) -> Result<Url> {
    Url::parse(mootbox_url)
        .and_then(|url| url.join(MOOTBOX_MCP_PATH))
        .with_context(|| format!("mootbox_url {mootbox_url:?} in the daemon config is not a URL"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_mootbox_mcp_path_is_resolved_against_the_configured_mootbox_url() {
        let url = mootbox_mcp_url("http://127.0.0.1:3000/").expect("a URL");

        assert_eq!(url.as_str(), "http://127.0.0.1:3000/mcp");
    }

    #[test]
    fn a_mootbox_url_that_is_not_a_url_is_refused() {
        assert!(mootbox_mcp_url("not a url").is_err());
    }

    #[test]
    fn the_selected_provider_uses_its_configured_program_and_others_their_default_command() {
        let codex = ProviderKind::new(&codex::Codex);
        let selected = SelectedProvider {
            kind: DEFAULT_PROVIDER,
            program: PathBuf::from("/opt/claude"),
        };

        assert_eq!(selected.program_for(DEFAULT_PROVIDER), PathBuf::from("/opt/claude"));
        assert_eq!(selected.program_for(codex), PathBuf::from("codex"));
    }
}
